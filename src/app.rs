use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::config::{load_config, save_config, Config, ReplayConfig, ReplayInput, RepoConfig};
use crate::domain::{InputField, Repo, Workflow};
use crate::service::github::GitHubService;

#[derive(Debug, Default)]
pub enum Focus {
    #[default]
    Repo,
    Branches,
    Workflows,
    Inputs,
    Output,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DispatchOutputColor {
    Green,
    Yellow,
    White,
    Blue,
}

#[derive(Debug, Default)]
pub struct AppData {
    pub repos: Vec<Repo>,
    pub branches: Vec<String>, // branches for currently selected repo
    pub workflows: Vec<Workflow>,
    pub inputs: Vec<String>,
    pub input_fields: Vec<InputField>,
    pub replays_list: Vec<ReplayConfig>,
}

#[derive(Debug, Default)]
pub struct UiState {
    // List States
    pub repos_state: ratatui::widgets::ListState,
    pub branches_state: ratatui::widgets::ListState,
    pub workflows_state: ratatui::widgets::ListState,
    pub inputs_state: ratatui::widgets::ListState,
    pub replays_state: ratatui::widgets::ListState,

    pub focus: Focus,

    // Right/output area
    pub output: Option<String>,
    pub output_is_error: bool,
    pub output_is_success: bool,
    pub dispatch_output_lines: Vec<(String, DispatchOutputColor)>,

    // Popups
    pub show_add_repo_popup: bool,
    pub show_inputs_popup: bool,
    pub show_confirm_dispatch: bool,
    pub show_help_popup: bool,
    pub show_replays_popup: bool,
    
    // Popup state
    pub add_repo_owner: String,
    pub add_repo_name: String,
    pub add_repo_focus_owner: bool, // true = owner field, false = repo field
    
    pub input_fields_selected: usize, // which input row is focused
    pub input_fields_editing: bool,   // whether we're typing into the value
    
    pub dispatch_command_preview: String,
    
    // Logic/Flow state
    pub awaiting_log_prompt: bool,
    pub last_run_id: Option<u64>,

    // Search
    pub search_active: bool,
    pub search_query: String,
    pub filtered_repo_indices: Vec<usize>,
    pub filtered_branch_indices: Vec<usize>,
    pub filtered_workflow_indices: Vec<usize>,

    pub repos_hscroll: u16,
    pub output_scroll: u16,
}

#[derive(Debug, Default)]
pub struct AppState {
    pub config: Config,
    pub data: AppData,
    pub ui: UiState,
    pub github: GitHubService,
}

impl AppState {
    pub fn new() -> Self {
        // Load persisted repos from config
        let config = load_config();
        let repos: Vec<Repo> = config
            .repos
            .iter()
            .map(|rc| Repo {
                name: rc.name.clone(),
                branches: vec![],
                workflows: vec![],
            })
            .collect();

        let branches: Vec<String> = vec![];
        let workflows: Vec<Workflow> = vec![];
        let inputs: Vec<String> = vec![];

        // Initialize ListStates with first item selected
        let mut repos_state = ratatui::widgets::ListState::default();
        repos_state.select(Some(0));

        let mut branches_state = ratatui::widgets::ListState::default();
        branches_state.select(Some(0));

        let workflows_state = ratatui::widgets::ListState::default();

        let mut inputs_state = ratatui::widgets::ListState::default();
        inputs_state.select(Some(0));

        let filtered_repo_indices: Vec<usize> = (0..repos.len()).collect();
        let has_repos = !repos.is_empty();

        Self {
            config,
            github: GitHubService::new(),
            data: AppData {
                repos,
                branches,
                workflows,
                inputs,
                input_fields: vec![],
                replays_list: vec![],
            },
            ui: UiState {
                repos_state,
                branches_state,
                workflows_state,
                inputs_state,
                replays_state: ratatui::widgets::ListState::default(),
                focus: Focus::Repo,
                output: Some(if has_repos {
                    "Ready to dispatch workflows...\n\nSelect a repo and press Enter to load branches.\nPress 'a' to add a new repo, '?' for all keybindings.".to_string()
                } else {
                    "Welcome to Lazy-Dispatchrr!\n\nPress 'a' to add a repo, '?' for all keybindings.".to_string()
                }),
                output_is_error: false,
                output_is_success: false,
                dispatch_output_lines: vec![],
                show_add_repo_popup: false,
                add_repo_owner: String::new(),
                add_repo_name: String::new(),
                add_repo_focus_owner: true,
                show_inputs_popup: false,
                input_fields_selected: 0,
                input_fields_editing: false,
                show_confirm_dispatch: false,
                dispatch_command_preview: String::new(),
                show_help_popup: false,
                awaiting_log_prompt: false,
                last_run_id: None,
                show_replays_popup: false,
                search_active: false,
                search_query: String::new(),
                filtered_repo_indices,
                filtered_branch_indices: vec![],
                filtered_workflow_indices: vec![],
                repos_hscroll: 0,
                output_scroll: 0,
            },
        }
    }

    // --- Fuzzy search helpers ---

    /// Get the real index into `self.data.repos` for the currently selected filtered item.
    pub fn selected_repo_real_index(&self) -> Option<usize> {
        let sel = self.ui.repos_state.selected()?;
        self.ui.filtered_repo_indices.get(sel).copied()
    }

    /// Get the real index into `self.data.branches` for the currently selected filtered item.
    pub fn selected_branch_real_index(&self) -> Option<usize> {
        let sel = self.ui.branches_state.selected()?;
        self.ui.filtered_branch_indices.get(sel).copied()
    }

    /// Get the real index into `self.data.workflows` for the currently selected filtered item.
    pub fn selected_workflow_real_index(&self) -> Option<usize> {
        let sel = self.ui.workflows_state.selected()?;
        self.ui.filtered_workflow_indices.get(sel).copied()
    }

    /// Re-filter the currently focused list based on `self.ui.search_query`.
    pub fn update_search_filter(&mut self) {
        let matcher = SkimMatcherV2::default();
        let query = &self.ui.search_query;

        match self.ui.focus {
            Focus::Repo => {
                if query.is_empty() {
                    self.ui.filtered_repo_indices = (0..self.data.repos.len()).collect();
                } else {
                    let mut scored: Vec<(usize, i64)> = self.data
                        .repos
                        .iter()
                        .enumerate()
                        .filter_map(|(i, r)| {
                            matcher.fuzzy_match(&r.name, query).map(|score| (i, score))
                        })
                        .collect();
                    scored.sort_by(|a, b| b.1.cmp(&a.1));
                    self.ui.filtered_repo_indices = scored.into_iter().map(|(i, _)| i).collect();
                }
                self.ui.repos_state.select(if self.ui.filtered_repo_indices.is_empty() {
                    None
                } else {
                    Some(0)
                });
            }
            Focus::Branches => {
                if query.is_empty() {
                    self.ui.filtered_branch_indices = (0..self.data.branches.len()).collect();
                } else {
                    let mut scored: Vec<(usize, i64)> = self.data
                        .branches
                        .iter()
                        .enumerate()
                        .filter_map(|(i, b)| {
                            matcher.fuzzy_match(b, query).map(|score| (i, score))
                        })
                        .collect();
                    scored.sort_by(|a, b| b.1.cmp(&a.1));
                    self.ui.filtered_branch_indices = scored.into_iter().map(|(i, _)| i).collect();
                }
                self.ui.branches_state.select(if self.ui.filtered_branch_indices.is_empty() {
                    None
                } else {
                    Some(0)
                });
            }
            Focus::Workflows => {
                if query.is_empty() {
                    self.ui.filtered_workflow_indices = (0..self.data.workflows.len()).collect();
                } else {
                    let mut scored: Vec<(usize, i64)> = self.data
                        .workflows
                        .iter()
                        .enumerate()
                        .filter_map(|(i, w)| {
                            matcher.fuzzy_match(&w.name, query).map(|score| (i, score))
                        })
                        .collect();
                    scored.sort_by(|a, b| b.1.cmp(&a.1));
                    self.ui.filtered_workflow_indices =
                        scored.into_iter().map(|(i, _)| i).collect();
                }
                self.ui.workflows_state.select(if self.ui.filtered_workflow_indices.is_empty() {
                    None
                } else {
                    Some(0)
                });
            }
            _ => {}
        }
    }

    /// Cancel search and restore all items in every list.
    pub fn reset_search(&mut self) {
        self.ui.search_active = false;
        self.ui.search_query.clear();
        self.ui.filtered_repo_indices = (0..self.data.repos.len()).collect();
        self.ui.filtered_branch_indices = (0..self.data.branches.len()).collect();
        self.ui.filtered_workflow_indices = (0..self.data.workflows.len()).collect();
    }

    /// Fetch a repo's branches and workflow file names via `gh api graphql`
    /// and add it to the repos list.
    pub fn add_repo(&mut self, owner: &str, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let (branches, workflows) = self.github.fetch_repo_details(owner, name)?;

        let repo = Repo {
            name: format!("{}/{}", owner, name),
            branches,
            workflows,
        };

        self.ui.output = Some(format!("Added repo '{}'", repo.name));
        self.data.repos.push(repo);
        self.ui.filtered_repo_indices = (0..self.data.repos.len()).collect();

        // Persist to config file
        self.save_repos_to_config()?;

        Ok(())
    }

    /// Save current repos list to the config file, preserving replays.
    fn save_repos_to_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Load existing config to preserve replays for repos we didn't touch
        let mut existing = load_config();
        let mut repo_configs: Vec<RepoConfig> = Vec::new();
        for repo in &self.data.repos {
            // Find existing replays for this repo
            let replays = existing
                .repos
                .iter()
                .find(|rc| rc.name == repo.name)
                .map(|rc| rc.replays.clone())
                .unwrap_or_default();
            repo_configs.push(RepoConfig {
                name: repo.name.clone(),
                replays,
            });
        }
        existing.repos = repo_configs;
        save_config(&existing)?;
        Ok(())
    }

    pub fn load_branches(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let selected_repo_idx = self.selected_repo_real_index()
            .ok_or("No repo selected.")?;
        let repo_name = self.data.repos[selected_repo_idx].name.clone();

        // Split "owner/name" to query GitHub
        let parts: Vec<&str> = repo_name.splitn(2, '/').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid repo format: '{}'. Expected 'owner/name'.", repo_name).into());
        }
        let (owner, name) = (parts[0], parts[1]);

        self.ui.output = Some(format!("Fetching branches for '{}'...", repo_name));
        self.ui.output_is_error = false;

        let (branches, workflows): (Vec<String>, Vec<String>) = self.github.fetch_repo_details(owner, name)?;

        // Update the cached repo data
        self.data.repos[selected_repo_idx].branches = branches.clone();
        self.data.repos[selected_repo_idx].workflows = workflows.clone();

        // Populate the UI lists
        self.data.branches = branches;
        self.ui.branches_state.select(if self.data.branches.is_empty() { None } else { Some(0) });

        self.data.workflows = workflows.iter().enumerate()
            .map(|(i, name): (usize, &String)| Workflow {
                id: format!("wf-{}", i),
                name: name.clone(),
                inputs: vec![],
            })
            .collect();
        self.ui.workflows_state.select(if self.data.workflows.is_empty() { None } else { Some(0) });

        // Reset search filters for the newly loaded data
        self.ui.filtered_branch_indices = (0..self.data.branches.len()).collect();
        self.ui.filtered_workflow_indices = (0..self.data.workflows.len()).collect();
        self.ui.search_active = false;
        self.ui.search_query.clear();

        self.ui.output = Some(format!(
            "Loaded {} branches and {} workflows for '{}'",
            self.data.branches.len(),
            self.data.workflows.len(),
            repo_name,
        ));
        Ok(())
    }

    pub fn load_workflows(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Fetch workflows for the selected branch (not just the default branch)
        let selected_repo_idx = self.selected_repo_real_index()
            .ok_or("No repo selected.")?;
        let repo_name = self.data.repos[selected_repo_idx].name.clone();

        let selected_branch_idx = self.selected_branch_real_index()
            .ok_or("No branch selected.")?;
        let selected_branch = self.data.branches[selected_branch_idx].clone();

        let parts: Vec<&str> = repo_name.splitn(2, '/').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid repo format: '{}'. Expected 'owner/name'.", repo_name).into());
        }
        let (owner, name) = (parts[0], parts[1]);

        self.ui.output = Some(format!("Fetching workflows for branch '{}'...", selected_branch));
        self.ui.output_is_error = false;

        let workflows = self.github.fetch_branch_workflows(owner, name, &selected_branch)?;

        self.data.workflows = workflows.iter().enumerate()
            .map(|(i, name): (usize, &String)| Workflow {
                id: format!("wf-{}", i),
                name: name.clone(),
                inputs: vec![],
            })
            .collect();

        // Reset workflow selection and search filters
        self.ui.workflows_state.select(if self.data.workflows.is_empty() { None } else { Some(0) });
        self.ui.filtered_workflow_indices = (0..self.data.workflows.len()).collect();

        // Show the loaded workflows in the output
        let workflow_names: Vec<String> = self.data.workflows.iter().map(|w| format!("- {}", w.name)).collect();
        let display = if workflow_names.is_empty() {
            format!("No workflows found on branch '{}'.", selected_branch)
        } else {
            format!("Loaded {} workflows for branch '{}':\n\n{}", workflow_names.len(), selected_branch, workflow_names.join("\n"))
        };
        self.ui.output = Some(display);
        Ok(())
    }

    pub fn load_inputs(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Fetch the selected workflow's YAML content and parse workflow_dispatch inputs
        let selected_wf_idx = match self.selected_workflow_real_index() {
            Some(idx) => idx,
            None => {
                self.ui.output = Some("No workflow selected.".to_string());
                return Ok(());
            }
        };
        let workflow_filename = &self.data.workflows[selected_wf_idx].name;

        // We need owner/repo from the selected repo
        let selected_repo_idx = match self.selected_repo_real_index() {
            Some(idx) => idx,
            None => {
                self.ui.output = Some("No repo selected.".to_string());
                return Ok(());
            }
        };
        let repo_name = &self.data.repos[selected_repo_idx].name; // "owner/repo"

        // Get the selected branch to fetch the workflow file from that branch
        let branch_ref = self.selected_branch_real_index()
            .map(|idx| self.data.branches[idx].clone());

        let (inputs_list, fields) = self.github.fetch_workflow_inputs(repo_name, workflow_filename, branch_ref.as_deref())?;

        self.data.inputs = inputs_list;
        self.data.input_fields = fields;
        self.ui.input_fields_selected = 0;
        self.ui.input_fields_editing = false;

        if self.data.inputs.is_empty() {
            self.ui.inputs_state.select(None);
            self.ui.output = Some(format!(
                "Workflow '{}' has no dispatch inputs.\n\nPress 'i' or Enter to dispatch.",
                workflow_filename
            ));
        } else {
            self.ui.inputs_state.select(Some(0));
            let display: Vec<String> = self.data.inputs.iter().map(|i| format!("- {}", i)).collect();
            self.ui.output = Some(format!(
                "Inputs for '{}':\n\n{}\n\nPress 'i' to edit inputs and dispatch.",
                workflow_filename,
                display.join("\n")
            ));
        }
        Ok(())
    }

    pub fn run_workflow(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let selected_repo_idx = match self.selected_repo_real_index() {
            Some(idx) => idx,
            None => return Err("No repo selected.".into()),
        };
        let repo_name = &self.data.repos[selected_repo_idx].name;

        let selected_branch = match self.selected_branch_real_index() {
            Some(idx) => self.data.branches[idx].clone(),
            None => return Err("No branch selected.".into()),
        };

        let selected_wf_idx = match self.selected_workflow_real_index() {
            Some(idx) => idx,
            None => return Err("No workflow selected.".into()),
        };
        let workflow_filename = &self.data.workflows[selected_wf_idx].name;

        let (_, preview) = self.github.dispatch_workflow(repo_name, &selected_branch, workflow_filename, &self.data.input_fields)?;

        self.ui.output_is_success = true;
        self.ui.output_is_error = false;

        let inputs_display = self.data.input_fields
            .iter()
            .map(|f| format!("  {} = {}", f.name, f.value))
            .collect::<Vec<_>>()
            .join("\n");

        self.ui.dispatch_output_lines = vec![
            ("✓ Workflow dispatched!".to_string(), DispatchOutputColor::Green),
            (String::new(), DispatchOutputColor::White),
            ("Command:".to_string(), DispatchOutputColor::Yellow),
            (format!("  {}", preview), DispatchOutputColor::Yellow),
            (String::new(), DispatchOutputColor::White),
            ("Inputs:".to_string(), DispatchOutputColor::White),
        ];
        for line in inputs_display.lines() {
            self.ui.dispatch_output_lines.push((line.to_string(), DispatchOutputColor::White));
        }
        self.ui.dispatch_output_lines.push((String::new(), DispatchOutputColor::White));
        self.ui.dispatch_output_lines.push((
            "Press 'l' to watch run logs, 'v' to open in browser, or any other key to continue.".to_string(),
            DispatchOutputColor::Blue,
        ));

        self.ui.output = Some("dispatch_success".to_string());
        self.ui.awaiting_log_prompt = true;
        Ok(())
    }

    /// Fetch the latest workflow run logs for the current repo/workflow.
    pub fn watch_workflow_logs(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let selected_repo_idx = match self.selected_repo_real_index() {
            Some(idx) => idx,
            None => return Err("No repo selected.".into()),
        };
        let repo_name = &self.data.repos[selected_repo_idx].name;

        let selected_wf_idx = match self.selected_workflow_real_index() {
            Some(idx) => idx,
            None => return Err("No workflow selected.".into()),
        };
        let workflow_filename = &self.data.workflows[selected_wf_idx].name;

        self.ui.output = Some(format!("Fetching latest run for '{}'...", workflow_filename));
        self.ui.output_is_error = false;

        let (run_id, status, conclusion, logs) = self.github.get_latest_run_logs(repo_name, workflow_filename)?;
        self.ui.last_run_id = Some(run_id);

        self.ui.output = Some(format!(
            "Run #{} | status: {} | conclusion: {}\n{}\n\n{}\n\nPress 'l' to refresh logs, 'v' to open in browser, or any other key to dismiss.",
            run_id, status, conclusion,
            "─".repeat(60),
            logs
        ));
        Ok(())
    }

    // --- Replay methods ---

    /// Save the current workflow inputs as a replay for the selected repo.
    pub fn save_replay(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let selected_repo_idx = self.selected_repo_real_index()
            .ok_or("No repo selected.")?;
        let repo_name = &self.data.repos[selected_repo_idx].name;

        let selected_wf_idx = self.selected_workflow_real_index()
            .ok_or("No workflow selected.")?;
        let workflow_filename = self.data.workflows[selected_wf_idx].name.clone();

        // Only save if there are inputs with non-default values
        let inputs_with_values: Vec<ReplayInput> = self.data
            .input_fields
            .iter()
            .filter(|f| !f.value.is_empty())
            .map(|f| ReplayInput {
                name: f.name.clone(),
                value: f.value.clone(),
            })
            .collect();

        if inputs_with_values.is_empty() {
            return Err("No inputs to save — workflows without inputs don't need replays.".into());
        }

        // Auto-generate description from inputs
        let description = inputs_with_values
            .iter()
            .map(|i| format!("{}={}", i.name, i.value))
            .collect::<Vec<_>>()
            .join(", ");

        let replay = ReplayConfig {
            workflow: workflow_filename.clone(),
            description,
            inputs: inputs_with_values,
        };

        // Load config, find this repo, add the replay
        let mut config = load_config();
        if let Some(rc) = config.repos.iter_mut().find(|rc| rc.name == *repo_name) {
            rc.replays.push(replay.clone());
        } else {
            // Repo not in config yet (shouldn't happen, but handle gracefully)
            config.repos.push(RepoConfig {
                name: repo_name.clone(),
                replays: vec![replay.clone()],
            });
        }
        save_config(&config)?;

        self.ui.output = Some(format!(
            "✓ Replay saved for '{}' → {}\n  {}",
            repo_name, workflow_filename, replay.description
        ));
        self.ui.output_is_error = false;
        Ok(())
    }

    /// Load replays for the currently selected repo and show the popup.
    pub fn open_replays(&mut self) {
        let repo_name = match self.selected_repo_real_index() {
            Some(idx) => self.data.repos[idx].name.clone(),
            None => {
                self.ui.output = Some("No repo selected.".to_string());
                self.ui.output_is_error = true;
                return;
            }
        };

        let config = load_config();
        self.data.replays_list = config
            .repos
            .iter()
            .find(|rc| rc.name == repo_name)
            .map(|rc| rc.replays.clone())
            .unwrap_or_default();

        if self.data.replays_list.is_empty() {
            self.ui.output = Some(format!("No saved replays for '{}'.", repo_name));
            self.ui.output_is_error = false;
            return;
        }

        self.ui.show_replays_popup = true;
        self.ui.replays_state.select(Some(0));
    }

    /// Run the selected replay with the currently selected branch.
    pub fn run_replay(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let replay_idx = self.ui.replays_state.selected()
            .ok_or("No replay selected.")?;
        let replay = self.data.replays_list[replay_idx].clone();

        let selected_repo_idx = self.selected_repo_real_index()
            .ok_or("No repo selected.")?;
        let repo_name = self.data.repos[selected_repo_idx].name.clone();

        let selected_branch = match self.selected_branch_real_index() {
            Some(idx) => self.data.branches[idx].clone(),
            None => return Err("No branch selected.".into()),
        };

        let mut args = vec![
            "workflow".to_string(),
            "run".to_string(),
            replay.workflow.clone(),
            "--repo".to_string(),
            repo_name.clone(),
            "--ref".to_string(),
            selected_branch,
        ];

        for input in &replay.inputs {
            args.push("-f".to_string());
            args.push(format!("{}={}", input.name, input.value));
        }

        let preview = format!("gh {}", args.join(" "));

        let output = std::process::Command::new("gh")
            .args(&args)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Replay dispatch failed: {}", stderr.trim()).into());
        }

        self.ui.show_replays_popup = false;
        self.ui.output_is_success = true;
        self.ui.output_is_error = false;
        self.ui.output = Some(format!(
            "✓ Replay dispatched!\n\nCommand:\n  {}\n\nInputs:\n{}\n\nPress 'l' to watch run logs, 'v' to open in browser, or any other key to continue.",
            preview,
            replay.inputs
                .iter()
                .map(|i| format!("  {} = {}", i.name, i.value))
                .collect::<Vec<_>>()
                .join("\n")
        ));
        self.ui.awaiting_log_prompt = true;
        Ok(())
    }

    /// Delete the currently selected replay from config.
    pub fn delete_replay(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let replay_idx = self.ui.replays_state.selected()
            .ok_or("No replay selected.")?;

        let selected_repo_idx = self.selected_repo_real_index()
            .ok_or("No repo selected.")?;
        let repo_name = &self.data.repos[selected_repo_idx].name;

        let mut config = load_config();
        if let Some(rc) = config.repos.iter_mut().find(|rc| rc.name == *repo_name) {
            if replay_idx < rc.replays.len() {
                let removed = rc.replays.remove(replay_idx);
                let remaining = rc.replays.clone();
                save_config(&config)?;
                self.data.replays_list = remaining;

                if self.data.replays_list.is_empty() {
                    self.ui.show_replays_popup = false;
                    self.ui.output = Some(format!("Deleted replay '{}'. No replays remaining.", removed.description));
                } else {
                    // Adjust selection
                    let new_sel = if replay_idx >= self.data.replays_list.len() {
                        self.data.replays_list.len() - 1
                    } else {
                        replay_idx
                    };
                    self.ui.replays_state.select(Some(new_sel));
                    self.ui.output = Some(format!("Deleted replay '{}'.", removed.description));
                }
                self.ui.output_is_error = false;
            }
        }
        Ok(())
    }

    /// Open the selected repo's GitHub page in the browser.
    pub fn open_repo_in_browser(&self) -> Result<(), Box<dyn std::error::Error>> {
        let selected_repo_idx = self.selected_repo_real_index()
            .ok_or("No repo selected.")?;
        let repo_name = &self.data.repos[selected_repo_idx].name;
        let url = format!("https://github.com/{}", repo_name);
        std::process::Command::new("open")
            .arg(&url)
            .spawn()?;
        Ok(())
    }

    /// Open the last workflow run's GitHub page in the browser.
    pub fn open_run_in_browser(&self) -> Result<(), Box<dyn std::error::Error>> {
        let selected_repo_idx = self.selected_repo_real_index()
            .ok_or("No repo selected.")?;
        let repo_name = &self.data.repos[selected_repo_idx].name;
        let run_id = self.ui.last_run_id
            .ok_or("No workflow run to view.")?;
        let url = format!("https://github.com/{}/actions/runs/{}", repo_name, run_id);
        std::process::Command::new("open")
            .arg(&url)
            .spawn()?;
        Ok(())
    }

    /// Build the dispatch command preview string without executing it.
    /// Returns (args, preview_string) for display in confirmation popup.
    pub fn build_dispatch_command(&self) -> Result<(Vec<String>, String), Box<dyn std::error::Error>> {
        let selected_repo_idx = self.selected_repo_real_index()
            .ok_or("No repo selected.")?;
        let repo_name = &self.data.repos[selected_repo_idx].name;

        let selected_branch_idx = self.selected_branch_real_index()
            .ok_or("No branch selected.")?;
        let selected_branch = &self.data.branches[selected_branch_idx];

        let selected_wf_idx = self.selected_workflow_real_index()
            .ok_or("No workflow selected.")?;
        let workflow_filename = &self.data.workflows[selected_wf_idx].name;

        let mut args = vec![
            "workflow".to_string(),
            "run".to_string(),
            workflow_filename.to_string(),
            "--repo".to_string(),
            repo_name.to_string(),
            "--ref".to_string(),
            selected_branch.to_string(),
        ];

        for field in &self.data.input_fields {
            if !field.value.is_empty() {
                args.push("-f".to_string());
                args.push(format!("{}={}", field.name, field.value));
            }
        }

        let preview = format!("gh {}", args.join(" "));
        Ok((args, preview))
    }
}
