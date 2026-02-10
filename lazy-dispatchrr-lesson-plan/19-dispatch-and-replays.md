# Module 19 — Workflow Dispatch & Replays

## Learning Objectives

- Build command arguments dynamically and preview them
- Implement dispatch with confirmation
- Save and load replay presets (input configurations)
- Handle post-dispatch actions (logs, browser)

---

## 1. Building the Dispatch Command

The dispatch command is built from three selections + input values:

```rust
pub fn build_dispatch_command(&self) -> Result<(Vec<String>, String), Box<dyn Error>> {
    let repo = &self.data.repos[self.selected_repo_real_index().ok_or("...")?].name;
    let branch = &self.data.branches[self.selected_branch_real_index().ok_or("...")?];
    let workflow = &self.data.workflows[self.selected_workflow_real_index().ok_or("...")?].name;

    let mut args = vec![
        "workflow".to_string(),
        "run".to_string(),
        workflow.to_string(),
        "--repo".to_string(),
        repo.to_string(),
        "--ref".to_string(),
        branch.to_string(),
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
```

This produces something like:
```
gh workflow run deploy.yml --repo owner/app --ref main -f env=prod -f version=1.0
```

## 2. Confirmation Before Dispatch

Never dispatch without showing the user exactly what will run:

```rust
// In event.rs — when user presses 'D' in inputs popup
KeyCode::Char('D') if !state.ui.input_fields_editing => {
    match state.build_dispatch_command() {
        Ok((_args, preview)) => {
            state.ui.dispatch_command_preview = preview;
            state.ui.show_inputs_popup = false;
            state.ui.show_confirm_dispatch = true;  // Show confirmation
        }
        Err(e) => {
            state.ui.output = Some(format!("Error: {}", e));
            state.ui.output_is_error = true;
        }
    }
}

// Handling confirmation popup
if state.ui.show_confirm_dispatch {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            state.ui.show_confirm_dispatch = false;
            state.run_workflow()?;  // Actually dispatch
        }
        _ => {
            state.ui.show_confirm_dispatch = false;
            state.ui.output = Some("Dispatch cancelled.".to_string());
        }
    }
    continue;
}
```

## 3. Executing the Dispatch

```rust
pub fn run_workflow(&mut self) -> Result<(), Box<dyn Error>> {
    // Get all selections
    let repo = /* ... */;
    let branch = /* ... */;
    let workflow = /* ... */;

    // Call the service
    let (_, preview) = self.github.dispatch_workflow(
        repo, &branch, workflow, &self.data.input_fields
    )?;

    // Build styled success output
    self.ui.output_is_success = true;
    self.ui.dispatch_output_lines = vec![
        ("✓ Workflow dispatched!".to_string(), DispatchOutputColor::Green),
        (String::new(), DispatchOutputColor::White),
        ("Command:".to_string(), DispatchOutputColor::Yellow),
        (format!("  {}", preview), DispatchOutputColor::Yellow),
        (String::new(), DispatchOutputColor::White),
        ("Inputs:".to_string(), DispatchOutputColor::White),
    ];
    
    // Add each input
    for field in &self.data.input_fields {
        self.ui.dispatch_output_lines.push((
            format!("  {} = {}", field.name, field.value),
            DispatchOutputColor::White,
        ));
    }
    
    // Prompt for next action
    self.ui.dispatch_output_lines.push((
        "Press 'l' for logs, 'v' for browser, any key to continue.".to_string(),
        DispatchOutputColor::Blue,
    ));

    self.ui.awaiting_log_prompt = true;
    Ok(())
}
```

## 4. Post-Dispatch Actions

After a successful dispatch, the user can:

```rust
if state.ui.awaiting_log_prompt {
    match key.code {
        KeyCode::Char('l') => {
            // Fetch and display run logs
            state.watch_workflow_logs()?;
            // Keep prompt active — user can refresh with 'l' again
        }
        KeyCode::Char('v') => {
            // Open the run in browser
            state.open_run_in_browser()?;
            state.ui.awaiting_log_prompt = false;
        }
        _ => {
            // Dismiss
            state.ui.awaiting_log_prompt = false;
        }
    }
    continue;
}
```

### Fetching Logs

```rust
pub fn watch_workflow_logs(&mut self) -> Result<(), Box<dyn Error>> {
    let (run_id, status, conclusion, logs) = 
        self.github.get_latest_run_logs(repo_name, workflow)?;
    
    self.ui.last_run_id = Some(run_id);
    self.ui.output = Some(format!(
        "Run #{} | status: {} | conclusion: {}\n{}\n\n{}\n\nPress 'l' to refresh, 'v' for browser.",
        run_id, status, conclusion,
        "─".repeat(60),
        logs
    ));
    Ok(())
}
```

## 5. Replays — Saved Input Presets

Replays let users save frequently-used input combinations:

### Saving a Replay

```rust
pub fn save_replay(&mut self) -> Result<(), Box<dyn Error>> {
    // Collect non-empty inputs
    let inputs: Vec<ReplayInput> = self.data.input_fields.iter()
        .filter(|f| !f.value.is_empty())
        .map(|f| ReplayInput { name: f.name.clone(), value: f.value.clone() })
        .collect();

    if inputs.is_empty() {
        return Err("No inputs to save.".into());
    }

    // Auto-generate description
    let description = inputs.iter()
        .map(|i| format!("{}={}", i.name, i.value))
        .collect::<Vec<_>>()
        .join(", ");

    let replay = ReplayConfig {
        workflow: workflow_filename.clone(),
        description,
        inputs,
    };

    // Save to config file
    let mut config = load_config();
    if let Some(rc) = config.repos.iter_mut().find(|rc| rc.name == *repo_name) {
        rc.replays.push(replay);
    }
    save_config(&config)?;
    Ok(())
}
```

### Loading Replays

```rust
pub fn open_replays(&mut self) {
    let repo_name = /* selected repo */;
    
    let config = load_config();
    self.data.replays_list = config.repos.iter()
        .find(|rc| rc.name == repo_name)
        .map(|rc| rc.replays.clone())
        .unwrap_or_default();

    if self.data.replays_list.is_empty() {
        self.ui.output = Some("No saved replays.".to_string());
    } else {
        self.ui.show_replays_popup = true;
        self.ui.replays_state.select(Some(0));
    }
}
```

### Running a Replay

```rust
pub fn run_replay(&mut self) -> Result<(), Box<dyn Error>> {
    let replay = &self.data.replays_list[replay_idx];
    
    let mut args = vec![
        "workflow".to_string(), "run".to_string(),
        replay.workflow.clone(),
        "--repo".to_string(), repo_name.clone(),
        "--ref".to_string(), branch.clone(),
    ];

    for input in &replay.inputs {
        args.push("-f".to_string());
        args.push(format!("{}={}", input.name, input.value));
    }

    let output = Command::new("gh").args(&args).output()?;
    // ... handle result
}
```

### Deleting a Replay

```rust
pub fn delete_replay(&mut self) -> Result<(), Box<dyn Error>> {
    let replay_idx = self.ui.replays_state.selected().ok_or("No replay selected.")?;
    
    let mut config = load_config();
    if let Some(rc) = config.repos.iter_mut().find(|rc| rc.name == *repo_name) {
        rc.replays.remove(replay_idx);
        save_config(&config)?;
        self.data.replays_list = rc.replays.clone();
    }
    
    // Adjust selection if needed
    if self.data.replays_list.is_empty() {
        self.ui.show_replays_popup = false;
    }
    Ok(())
}
```

## 6. Input Types Handling

Different input types have different editing behavior:

```rust
KeyCode::Char(c) if state.ui.input_fields_editing => {
    if let Some(field) = state.data.input_fields.get_mut(idx) {
        match field.input_type.as_str() {
            "boolean" => {
                // Toggle between true/false
                field.value = if field.value == "true" { "false" } else { "true" }.to_string();
            }
            "choice" => {
                // Choice inputs can't be typed — use Tab to cycle
            }
            _ => {
                // String, number — free-form typing
                field.value.push(c);
            }
        }
    }
}

// Tab cycles through choice options
KeyCode::Tab => {
    if field.input_type == "choice" && !field.options.is_empty() {
        let current = field.options.iter().position(|o| o == &field.value);
        let next = match current {
            Some(i) => (i + 1) % field.options.len(),
            None => 0,
        };
        field.value = field.options[next].clone();
    }
}
```

---

## Exercises

1. **Build command preview:** Implement `build_dispatch_command()` that takes a repo, 
   branch, workflow, and inputs, and returns a command string preview.

2. **Confirmation flow:** Implement the confirmation popup cycle:
   - User presses D → show preview → Y confirms → dispatch → show result
   
3. **Save a replay:** Implement `save_replay()` that saves current inputs to the config.
   Verify the YAML file has the replay after saving.

4. **Load and run replays:** Implement `open_replays()` and `run_replay()`. Show replays 
   in a popup and dispatch on Enter.

5. **Input type handling:** Handle different input types in the editing logic:
   - String: free typing
   - Boolean: toggle true/false
   - Choice: Tab to cycle options

6. **Connect to the app:** Trace the complete dispatch flow through the code:
   - From pressing 'D' in the inputs popup
   - Through `build_dispatch_command()`
   - To the confirmation popup
   - To `run_workflow()`
   - To the post-dispatch log prompt
   - To `watch_workflow_logs()`

---

## Key Takeaways

- Build commands as `Vec<String>` args — flexible and previewable
- Always confirm before dispatch — show the exact command
- Post-dispatch: offer logs, browser, or dismiss
- Replays save input configurations to config for reuse
- Different input types need different editing behavior (type, toggle, cycle)
- Replay management: save, load, run, delete — full CRUD on config

---

**Next:** [Module 20 — Build Scripts & Polish →](./20-build-scripts-polish.md)
