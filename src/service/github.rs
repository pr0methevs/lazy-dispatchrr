use crate::domain::InputField;

#[derive(Debug, Default)]
pub struct GitHubService;

impl GitHubService {
    pub fn new() -> Self {
        Self
    }

    /// Fetch a repo's branches and workflow file names via `gh api graphql`
    pub fn fetch_repo_details(&self, owner: &str, name: &str) -> Result<(Vec<String>, Vec<String>), Box<dyn std::error::Error>> {
        let query = "query($owner: String!, $name: String!) {
            repository(owner: $owner, name: $name) {
                refs(refPrefix: \"refs/heads/\", first: 100) {
                    nodes {
                        name
                    }
                }
                object(expression: \"HEAD:.github/workflows/\") {
                    ... on Tree {
                        entries {
                            name
                        }
                    }
                }
            }
        }";

        let output = std::process::Command::new("gh")
            .args([
                "api", "graphql",
                "-f", &format!("query={}", query),
                "-F", &format!("owner={}", owner),
                "-F", &format!("name={}", name),
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("gh cli error: {}", stderr.trim()).into());
        }

        let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let repository = &json["data"]["repository"];

        if repository.is_null() {
            // GraphQL returned data but repository was not found
             let errors = json["errors"]
                .as_array()
                .map(|errs| {
                    errs.iter()
                        .filter_map(|e| e["message"].as_str())
                        .collect::<Vec<_>>()
                        .join("; ")
                })
                .unwrap_or_else(|| "Repository not found".to_string());
            return Err(format!("GitHub API error: {}", errors).into());
        }

        // Extract branch names
        let branches: Vec<String> = repository["refs"]["nodes"]
            .as_array()
            .map(|nodes| {
                nodes
                    .iter()
                    .filter_map(|n| n["name"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        // Extract workflow file names from .github/workflows/
        let workflows: Vec<String> = repository["object"]["entries"]
            .as_array()
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(|e| e["name"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok((branches, workflows))
    }

    /// Fetch workflow file names for a specific branch via `gh api graphql`.
    pub fn fetch_branch_workflows(&self, owner: &str, name: &str, branch: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let query = "query($owner: String!, $name: String!, $expr: String!) {
            repository(owner: $owner, name: $name) {
                object(expression: $expr) {
                    ... on Tree {
                        entries {
                            name
                        }
                    }
                }
            }
        }";

        let expression = format!("{}:.github/workflows/", branch);

        let output = std::process::Command::new("gh")
            .args([
                "api", "graphql",
                "-f", &format!("query={}", query),
                "-F", &format!("owner={}", owner),
                "-F", &format!("name={}", name),
                "-F", &format!("expr={}", expression),
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("gh cli error: {}", stderr.trim()).into());
        }

        let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let repository = &json["data"]["repository"];

        let workflows: Vec<String> = repository["object"]["entries"]
            .as_array()
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(|e| e["name"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(workflows)
    }

     pub fn fetch_workflow_inputs(&self, repo_name: &str, workflow_filename: &str, branch: Option<&str>) -> Result<(Vec<String>, Vec<InputField>), Box<dyn std::error::Error>> {
        // Fetch workflow file content via gh api
        let api_path = if let Some(branch_ref) = branch {
            format!(
                "repos/{}/contents/.github/workflows/{}?ref={}",
                repo_name, workflow_filename, branch_ref
            )
        } else {
            format!(
                "repos/{}/contents/.github/workflows/{}",
                repo_name, workflow_filename
            )
        };
        let args = vec!["api".to_string(), api_path.clone(), "--jq".to_string(), ".content".to_string()];
        let output = std::process::Command::new("gh")
            .args(&args)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to fetch workflow file: {}", stderr.trim()).into());
        }

        // Decode base64 content (gh returns it with newlines)
        let b64_content = String::from_utf8_lossy(&output.stdout)
            .replace('\n', "")
            .replace('\r', "");

        use base64::Engine;
        let yaml_bytes = base64::engine::general_purpose::STANDARD
            .decode(&b64_content)
            .map_err(|e| format!("Base64 decode error: {}", e))?;
        let yaml_str = String::from_utf8_lossy(&yaml_bytes);

        // Parse the YAML and extract workflow_dispatch inputs
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&yaml_str)
            .map_err(|e| format!("YAML parse error: {}", e))?;

        let mut inputs_list: Vec<String> = Vec::new();
        let mut fields: Vec<InputField> = Vec::new();

        // Handle `on.workflow_dispatch.inputs`
        let dispatch = &yaml_value["on"]["workflow_dispatch"];
        if let Some(inputs_map) = dispatch["inputs"].as_mapping() {
            for (key, val) in inputs_map {
                let name = key.as_str().unwrap_or("unknown").to_string();
                let desc = val["description"].as_str().unwrap_or("").to_string();
                let required = val["required"].as_bool().unwrap_or(false);
                let default_value = match &val["default"] {
                    serde_yaml::Value::String(s) => s.clone(),
                    serde_yaml::Value::Bool(b) => b.to_string(),
                    serde_yaml::Value::Number(n) => n.to_string(),
                    _ => String::new(),
                };
                let input_type = val["type"].as_str().unwrap_or("string").to_string();
                let options: Vec<String> = val["options"]
                    .as_sequence()
                    .map(|opts| {
                        opts.iter()
                            .filter_map(|o| o.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();

                // Build display string
                let mut parts = vec![format!("{}:", name)];
                if !desc.is_empty() {
                    parts.push(format!(" {}", desc));
                }
                parts.push(format!(" [type: {}]", input_type));
                parts.push(format!(" [required: {}]", required));
                if !default_value.is_empty() {
                    parts.push(format!(" [default: {}]", default_value));
                }
                if !options.is_empty() {
                    parts.push(format!(" [options: {}]", options.join(", ")));
                }
                inputs_list.push(parts.join(""));

                fields.push(InputField {
                    name,
                    description: desc,
                    input_type,
                    required,
                    value: default_value.clone(),
                    default_value,
                    options,
                });
            }
        }
        
        Ok((inputs_list, fields))
    }

    pub fn dispatch_workflow(&self, repo_name: &str, branch: &str, workflow_filename: &str, inputs: &[InputField]) -> Result<(Vec<String>, String), Box<dyn std::error::Error>> {
         let mut args = vec![
            "workflow".to_string(),
            "run".to_string(),
            workflow_filename.to_string(),
            "--repo".to_string(),
            repo_name.to_string(),
            "--ref".to_string(),
            branch.to_string(),
        ];

        for field in inputs {
            if !field.value.is_empty() {
                args.push("-f".to_string());
                args.push(format!("{}={}", field.name, field.value));
            }
        }

        let preview = format!("gh {}", args.join(" "));

        let output = std::process::Command::new("gh")
            .args(&args)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Workflow dispatch failed: {}", stderr.trim()).into());
        }

        Ok((args, preview))
    }

    /// Find the latest run ID for a workflow, with retry/polling for freshly dispatched runs.
    pub fn find_latest_run_id(&self, repo_name: &str, workflow_filename: &str) -> Result<u64, Box<dyn std::error::Error>> {
        // Poll a few times since the run may not appear instantly after dispatch
        for attempt in 0..5 {
            if attempt > 0 {
                std::thread::sleep(std::time::Duration::from_secs(2));
            }

            let list_output = std::process::Command::new("gh")
                .args([
                    "run", "list",
                    "--repo", repo_name,
                    "--workflow", workflow_filename,
                    "--limit", "1",
                    "--json", "databaseId,status,event",
                ])
                .output()?;

            if !list_output.status.success() {
                continue;
            }

            let runs: serde_json::Value = serde_json::from_slice(&list_output.stdout)?;
            if let Some(run_id) = runs[0]["databaseId"].as_u64() {
                return Ok(run_id);
            }
        }
        Err("Could not find workflow run after dispatch. Try pressing 'l' again in a few seconds.".into())
    }

    pub fn get_run_logs(&self, repo_name: &str, run_id: u64) -> Result<(String, String, String), Box<dyn std::error::Error>> {
        // Fetch run status
        let status_output = std::process::Command::new("gh")
            .args([
                "run", "view",
                &run_id.to_string(),
                "--repo", repo_name,
                "--json", "status,conclusion",
            ])
            .output()?;

        let (status, conclusion) = if status_output.status.success() {
            let info: serde_json::Value = serde_json::from_slice(&status_output.stdout)?;
            (
                info["status"].as_str().unwrap_or("unknown").to_string(),
                info["conclusion"].as_str().unwrap_or("pending").to_string(),
            )
        } else {
            ("unknown".to_string(), "pending".to_string())
        };

        // Fetch the logs for that run
        let log_output = std::process::Command::new("gh")
            .args([
                "run", "view",
                &run_id.to_string(),
                "--repo", repo_name,
                "--log",
            ])
            .output()?;

        let logs = if log_output.status.success() {
            let full_log = String::from_utf8_lossy(&log_output.stdout).to_string();
            // Truncate to last 200 lines to fit in the output panel
            let lines: Vec<&str> = full_log.lines().collect();
            let start = if lines.len() > 200 { lines.len() - 200 } else { 0 };
            lines[start..].join("\n")
        } else {
            let stderr = String::from_utf8_lossy(&log_output.stderr);
            format!("(logs not yet available: {})", stderr.trim())
        };

        Ok((status, conclusion, logs))
    }

    pub fn get_latest_run_logs(&self, repo_name: &str, workflow_filename: &str) -> Result<(u64, String, String, String), Box<dyn std::error::Error>> {
        let run_id = self.find_latest_run_id(repo_name, workflow_filename)?;
        let (status, conclusion, logs) = self.get_run_logs(repo_name, run_id)?;
        Ok((run_id, status, conclusion, logs))
    }

}
