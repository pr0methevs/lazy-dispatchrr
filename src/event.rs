use crate::{
    app::{AppState, Focus},
    ui::render,
};
use color_eyre::eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::DefaultTerminal;

pub fn run(mut terminal: DefaultTerminal, state: &mut AppState) -> Result<()> {
    loop {
        terminal.draw(|frame| render(frame, state))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                // Handle help popup — any key dismisses it
                if state.ui.show_help_popup {
                    state.ui.show_help_popup = false;
                    continue;
                }

                // Handle add-repo popup input first
                if state.ui.show_add_repo_popup {
                    match key.code {
                        KeyCode::Esc => {
                            state.ui.show_add_repo_popup = false;
                            state.ui.add_repo_owner.clear();
                            state.ui.add_repo_name.clear();
                            state.ui.add_repo_focus_owner = true;
                        }
                        KeyCode::Tab | KeyCode::BackTab => {
                            state.ui.add_repo_focus_owner = !state.ui.add_repo_focus_owner;
                        }
                        KeyCode::Enter => {
                            let owner = state.ui.add_repo_owner.clone();
                            let name = state.ui.add_repo_name.clone();
                            if owner.is_empty() || name.is_empty() {
                                state.ui.output = Some("Both owner and repo fields are required.".to_string());
                                state.ui.output_is_error = true;
                            } else {
                                state.ui.show_add_repo_popup = false;
                                if let Err(e) = state.add_repo(&owner, &name) {
                                    state.ui.output = Some(format!("Error adding repo: {}", e));
                                    state.ui.output_is_error = true;
                                } else {
                                    state.ui.output_is_error = false;
                                }
                                state.ui.add_repo_owner.clear();
                                state.ui.add_repo_name.clear();
                                state.ui.add_repo_focus_owner = true;
                            }
                        }
                        KeyCode::Backspace => {
                            if state.ui.add_repo_focus_owner {
                                state.ui.add_repo_owner.pop();
                            } else {
                                state.ui.add_repo_name.pop();
                            }
                        }
                        KeyCode::Char(c) => {
                            if state.ui.add_repo_focus_owner {
                                state.ui.add_repo_owner.push(c);
                            } else {
                                state.ui.add_repo_name.push(c);
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                // Handle dispatch confirmation popup
                if state.ui.show_confirm_dispatch {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            state.ui.show_confirm_dispatch = false;
                            if let Err(e) = state.run_workflow() {
                                state.ui.output = Some(format!("Error dispatching workflow: {}", e));
                                state.ui.output_is_error = true;
                            } else {
                                state.ui.output_is_error = false;
                            }
                        }
                        _ => {
                            state.ui.show_confirm_dispatch = false;
                            state.ui.output = Some("Dispatch cancelled.".to_string());
                            state.ui.output_is_error = false;
                        }
                    }
                    continue;
                }

                // Handle post-dispatch log prompt
                if state.ui.awaiting_log_prompt {
                    match key.code {
                        KeyCode::Char('l') | KeyCode::Char('L') => {
                            // Fetch logs but keep prompt active for retry
                            if let Err(e) = state.watch_workflow_logs() {
                                state.ui.output = Some(format!("Error fetching logs: {}\n\nPress 'l' to retry, 'v' to open in browser, or any other key to dismiss.", e));
                                state.ui.output_is_error = true;
                            } else {
                                state.ui.output_is_error = false;
                            }
                        }
                        KeyCode::Char('v') => {
                            if let Err(e) = state.open_run_in_browser() {
                                state.ui.output = Some(format!("Error opening browser: {}", e));
                                state.ui.output_is_error = true;
                            }
                            state.ui.awaiting_log_prompt = false;
                        }
                        _ => {
                            state.ui.awaiting_log_prompt = false;
                        }
                    }
                    continue;
                }

                // Handle inputs popup
                if state.ui.show_inputs_popup {
                    // Tab cycles choice options regardless of editing state
                    if matches!(key.code, KeyCode::Tab) {
                        if let Some(field) = state.data.input_fields.get_mut(state.ui.input_fields_selected) {
                            if field.input_type == "choice" && !field.options.is_empty() {
                                let current_idx = field.options.iter().position(|o| o == &field.value);
                                let next_idx = match current_idx {
                                    Some(i) => (i + 1) % field.options.len(),
                                    None => 0,
                                };
                                field.value = field.options[next_idx].clone();
                            }
                        }
                        continue;
                    }
                    // BackTab cycles choice options backwards
                    if matches!(key.code, KeyCode::BackTab) {
                        if let Some(field) = state.data.input_fields.get_mut(state.ui.input_fields_selected) {
                            if field.input_type == "choice" && !field.options.is_empty() {
                                let current_idx = field.options.iter().position(|o| o == &field.value);
                                let next_idx = match current_idx {
                                    Some(0) | None => field.options.len() - 1,
                                    Some(i) => i - 1,
                                };
                                field.value = field.options[next_idx].clone();
                            }
                        }
                        continue;
                    }

                    match key.code {
                        KeyCode::Esc => {
                            if state.ui.input_fields_editing {
                                state.ui.input_fields_editing = false;
                            } else {
                                state.ui.show_inputs_popup = false;
                            }
                        }
                        KeyCode::Char('j') | KeyCode::Down if !state.ui.input_fields_editing => {
                            if !state.data.input_fields.is_empty() {
                                state.ui.input_fields_selected =
                                    (state.ui.input_fields_selected + 1) % state.data.input_fields.len();
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up if !state.ui.input_fields_editing => {
                            if !state.data.input_fields.is_empty() {
                                if state.ui.input_fields_selected == 0 {
                                    state.ui.input_fields_selected = state.data.input_fields.len() - 1;
                                } else {
                                    state.ui.input_fields_selected -= 1;
                                }
                            }
                        }
                        KeyCode::Enter if !state.ui.input_fields_editing => {
                            state.ui.input_fields_editing = true;
                        }
                        KeyCode::Enter if state.ui.input_fields_editing => {
                            state.ui.input_fields_editing = false;
                        }
                        KeyCode::Char('D') if !state.ui.input_fields_editing => {
                            // Shift+D: show confirmation before dispatch
                            state.ui.input_fields_editing = false;
                            match state.build_dispatch_command() {
                                Ok((_args, preview)) => {
                                    state.ui.dispatch_command_preview = preview;
                                    state.ui.show_inputs_popup = false;
                                    state.ui.show_confirm_dispatch = true;
                                }
                                Err(e) => {
                                    state.ui.output = Some(format!("Error: {}", e));
                                    state.ui.output_is_error = true;
                                    state.ui.show_inputs_popup = false;
                                }
                            }
                        }
                        KeyCode::Char('S') if !state.ui.input_fields_editing => {
                            // Shift+S: save current inputs as a replay
                            match state.save_replay() {
                                Ok(()) => {
                                    state.ui.output_is_error = false;
                                }
                                Err(e) => {
                                    state.ui.output = Some(format!("Error saving replay: {}", e));
                                    state.ui.output_is_error = true;
                                }
                            }
                        }
                        KeyCode::Backspace if state.ui.input_fields_editing => {
                            if let Some(field) = state.data.input_fields.get_mut(state.ui.input_fields_selected) {
                                if field.input_type != "choice" {
                                    field.value.pop();
                                }
                            }
                        }
                        KeyCode::Char(c) if state.ui.input_fields_editing => {
                            if let Some(field) = state.data.input_fields.get_mut(state.ui.input_fields_selected) {
                                if field.input_type == "boolean" {
                                    field.value = if field.value == "true" {
                                        "false".to_string()
                                    } else {
                                        "true".to_string()
                                    };
                                } else if field.input_type != "choice" {
                                    field.value.push(c);
                                }
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                // Handle replays popup
                if state.ui.show_replays_popup {
                    match key.code {
                        KeyCode::Esc => {
                            state.ui.show_replays_popup = false;
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            if !state.data.replays_list.is_empty() {
                                let sel = state.ui.replays_state.selected().unwrap_or(0);
                                state.ui.replays_state.select(Some((sel + 1) % state.data.replays_list.len()));
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            if !state.data.replays_list.is_empty() {
                                let sel = state.ui.replays_state.selected().unwrap_or(0);
                                if sel == 0 {
                                    state.ui.replays_state.select(Some(state.data.replays_list.len() - 1));
                                } else {
                                    state.ui.replays_state.select(Some(sel - 1));
                                }
                            }
                        }
                        KeyCode::Enter => {
                            match state.run_replay() {
                                Ok(()) => {
                                    state.ui.output_is_error = false;
                                }
                                Err(e) => {
                                    state.ui.show_replays_popup = false;
                                    state.ui.output = Some(format!("Error running replay: {}", e));
                                    state.ui.output_is_error = true;
                                }
                            }
                        }
                        KeyCode::Char('d') => {
                            if let Err(e) = state.delete_replay() {
                                state.ui.output = Some(format!("Error deleting replay: {}", e));
                                state.ui.output_is_error = true;
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                // Handle fuzzy search input
                if state.ui.search_active {
                    match key.code {
                        KeyCode::Esc => {
                            // Cancel search, restore full list
                            state.reset_search();
                            match state.ui.focus {
                                Focus::Repo => state.ui.repos_state.select(
                                    if state.data.repos.is_empty() { None } else { Some(0) }
                                ),
                                Focus::Branches => state.ui.branches_state.select(
                                    if state.data.branches.is_empty() { None } else { Some(0) }
                                ),
                                Focus::Workflows => state.ui.workflows_state.select(
                                    if state.data.workflows.is_empty() { None } else { Some(0) }
                                ),
                                _ => {}
                            }
                        }
                        KeyCode::Enter => {
                            // Confirm search, keep filter active
                            state.ui.search_active = false;
                        }
                        KeyCode::Backspace => {
                            state.ui.search_query.pop();
                            state.update_search_filter();
                        }
                        KeyCode::Up | KeyCode::Char('k') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                            match state.ui.focus {
                                Focus::Repo => select_previous(&mut state.ui.repos_state, state.ui.filtered_repo_indices.len()),
                                Focus::Branches => select_previous(&mut state.ui.branches_state, state.ui.filtered_branch_indices.len()),
                                Focus::Workflows => select_previous(&mut state.ui.workflows_state, state.ui.filtered_workflow_indices.len()),
                                _ => {}
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                            match state.ui.focus {
                                Focus::Repo => select_next(&mut state.ui.repos_state, state.ui.filtered_repo_indices.len()),
                                Focus::Branches => select_next(&mut state.ui.branches_state, state.ui.filtered_branch_indices.len()),
                                Focus::Workflows => select_next(&mut state.ui.workflows_state, state.ui.filtered_workflow_indices.len()),
                                _ => {}
                            }
                        }
                        KeyCode::Up => {
                            match state.ui.focus {
                                Focus::Repo => select_previous(&mut state.ui.repos_state, state.ui.filtered_repo_indices.len()),
                                Focus::Branches => select_previous(&mut state.ui.branches_state, state.ui.filtered_branch_indices.len()),
                                Focus::Workflows => select_previous(&mut state.ui.workflows_state, state.ui.filtered_workflow_indices.len()),
                                _ => {}
                            }
                        }
                        KeyCode::Down => {
                            match state.ui.focus {
                                Focus::Repo => select_next(&mut state.ui.repos_state, state.ui.filtered_repo_indices.len()),
                                Focus::Branches => select_next(&mut state.ui.branches_state, state.ui.filtered_branch_indices.len()),
                                Focus::Workflows => select_next(&mut state.ui.workflows_state, state.ui.filtered_workflow_indices.len()),
                                _ => {}
                            }
                        }
                        KeyCode::Char(c) => {
                            state.ui.search_query.push(c);
                            state.update_search_filter();
                        }
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => break,
                    KeyCode::Char('?') => {
                        state.ui.show_help_popup = !state.ui.show_help_popup;
                    }
                    KeyCode::Char('a') => {
                        state.ui.show_add_repo_popup = true;
                        state.ui.add_repo_focus_owner = true;
                    }
                    KeyCode::Char('v') => {
                        if let Err(e) = state.open_repo_in_browser() {
                            state.ui.output = Some(format!("Error opening browser: {}", e));
                            state.ui.output_is_error = true;
                        }
                    }
                    KeyCode::Char('i') => {
                        if !state.data.input_fields.is_empty() {
                            state.ui.show_inputs_popup = true;
                            state.ui.input_fields_selected = 0;
                            state.ui.input_fields_editing = false;
                        } else if !state.data.workflows.is_empty() {
                            // No inputs, but workflow selected — show dispatch confirmation directly
                            match state.build_dispatch_command() {
                                Ok((_args, preview)) => {
                                    state.ui.dispatch_command_preview = preview;
                                    state.ui.show_confirm_dispatch = true;
                                }
                                Err(e) => {
                                    state.ui.output = Some(format!("Error: {}", e));
                                    state.ui.output_is_error = true;
                                }
                            }
                        }
                    }
                    KeyCode::Char('/') => {
                        // Activate fuzzy search for the focused panel
                        if matches!(state.ui.focus, Focus::Repo | Focus::Branches | Focus::Workflows) {
                            state.ui.search_active = true;
                            state.ui.search_query.clear();
                        }
                    }
                    KeyCode::Char('r') => {
                        // Open replays popup for the selected repo
                        state.open_replays();
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        // Move down in the current focused list
                        match state.ui.focus {
                            Focus::Repo => select_next(&mut state.ui.repos_state, state.ui.filtered_repo_indices.len()),
                            Focus::Branches => {
                                select_next(&mut state.ui.branches_state, state.ui.filtered_branch_indices.len())
                            }
                            Focus::Workflows => {
                                select_next(&mut state.ui.workflows_state, state.ui.filtered_workflow_indices.len())
                            }
                            Focus::Inputs => {
                                select_next(&mut state.ui.inputs_state, state.data.inputs.len())
                            }
                            Focus::Output => {}
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        // Move up in the current focused list
                        match state.ui.focus {
                            Focus::Repo => {
                                select_previous(&mut state.ui.repos_state, state.ui.filtered_repo_indices.len())
                            }
                            Focus::Branches => {
                                select_previous(&mut state.ui.branches_state, state.ui.filtered_branch_indices.len())
                            }
                            Focus::Workflows => {
                                select_previous(&mut state.ui.workflows_state, state.ui.filtered_workflow_indices.len())
                            }
                            Focus::Inputs => {
                                select_previous(&mut state.ui.inputs_state, state.data.inputs.len())
                            }
                            Focus::Output => {}
                        }
                    }
                    KeyCode::Tab => {
                        // Cycle through focus areas
                        state.ui.focus = match state.ui.focus {
                            Focus::Repo => Focus::Branches,
                            Focus::Branches => Focus::Workflows,
                            Focus::Workflows => Focus::Inputs,
                            Focus::Inputs => Focus::Output,
                            Focus::Output => Focus::Repo,
                        };
                    }
                    KeyCode::BackTab => {
                        // Cycle backwards through focus areas
                        state.ui.focus = match state.ui.focus {
                            Focus::Repo => Focus::Output,
                            Focus::Branches => Focus::Repo,
                            Focus::Workflows => Focus::Branches,
                            Focus::Inputs => Focus::Workflows,
                            Focus::Output => Focus::Inputs,
                        };
                    }
                    KeyCode::Enter => {
                        // Handle selection based on current focus
                        match state.ui.focus {
                            Focus::Repo => {
                                state.load_branches();
                                state.ui.focus = Focus::Branches;
                            }
                            Focus::Branches => {
                                state.load_workflows();
                                state.ui.focus = Focus::Workflows;
                            }
                            Focus::Workflows => {
                                state.load_inputs();
                                state.ui.focus = Focus::Inputs;
                            }
                            Focus::Inputs => {
                                // Show dispatch confirmation popup
                                match state.build_dispatch_command() {
                                    Ok((_args, preview)) => {
                                        state.ui.dispatch_command_preview = preview;
                                        state.ui.show_confirm_dispatch = true;
                                    }
                                    Err(e) => {
                                        state.ui.output = Some(format!("Error: {}", e));
                                        state.ui.output_is_error = true;
                                        state.ui.focus = Focus::Output;
                                    }
                                }
                            }
                            Focus::Output => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn select_next(state: &mut ratatui::widgets::ListState, len: usize) {
    if len == 0 {
        return;
    }
    let i = match state.selected() {
        Some(i) => (i + 1) % len,
        None => 0,
    };
    state.select(Some(i));
}

fn select_previous(state: &mut ratatui::widgets::ListState, len: usize) {
    if len == 0 {
        return;
    }
    let i = match state.selected() {
        Some(i) => {
            if i == 0 {
                len - 1
            } else {
                i - 1
            }
        }
        None => 0,
    };
    state.select(Some(i));
}