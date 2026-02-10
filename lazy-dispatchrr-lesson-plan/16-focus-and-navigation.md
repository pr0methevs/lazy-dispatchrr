# Module 16 — Focus System & Navigation

## Learning Objectives

- Implement a focus system for multi-panel TUIs
- Navigate lists with vim-style keybindings
- Handle Enter for context-sensitive actions
- Build the progressive drill-down workflow

---

## 1. The Focus Enum

Focus tracks which panel is currently active:

```rust
#[derive(Debug, Default)]
pub enum Focus {
    #[default]
    Repo,
    Branches,
    Workflows,
    Inputs,
    Output,
}
```

Focus affects:
- **Visual styling** — active panel has blue borders, inactive has gray
- **Key handling** — j/k navigate the focused list or scroll output
- **Enter behavior** — loads data for the focused concept
- **Search** — / searches in the focused panel

## 2. Tab Cycling

```rust
KeyCode::Tab => {
    state.ui.focus = match state.ui.focus {
        Focus::Repo => Focus::Branches,
        Focus::Branches => Focus::Workflows,
        Focus::Workflows => Focus::Inputs,
        Focus::Inputs => Focus::Output,
        Focus::Output => Focus::Repo,
    };
}
KeyCode::BackTab => {
    state.ui.focus = match state.ui.focus {
        Focus::Repo => Focus::Output,
        Focus::Branches => Focus::Repo,
        Focus::Workflows => Focus::Branches,
        Focus::Inputs => Focus::Workflows,
        Focus::Output => Focus::Inputs,
    };
}
```

## 3. Vim-Style List Navigation

j/k (and arrow keys) move up/down in the focused list:

```rust
KeyCode::Char('j') | KeyCode::Down => {
    match state.ui.focus {
        Focus::Repo => select_next(
            &mut state.ui.repos_state,
            state.ui.filtered_repo_indices.len()
        ),
        Focus::Branches => select_next(
            &mut state.ui.branches_state,
            state.ui.filtered_branch_indices.len()
        ),
        Focus::Workflows => select_next(
            &mut state.ui.workflows_state,
            state.ui.filtered_workflow_indices.len()
        ),
        Focus::Inputs => select_next(
            &mut state.ui.inputs_state,
            state.data.inputs.len()
        ),
        Focus::Output => {
            state.ui.output_scroll = state.ui.output_scroll.saturating_add(1);
        }
    }
}
```

### The Navigation Helpers

```rust
fn select_next(state: &mut ListState, len: usize) {
    if len == 0 { return; }
    let i = match state.selected() {
        Some(i) => (i + 1) % len,  // Wrap around to start
        None => 0,
    };
    state.select(Some(i));
}

fn select_previous(state: &mut ListState, len: usize) {
    if len == 0 { return; }
    let i = match state.selected() {
        Some(i) => if i == 0 { len - 1 } else { i - 1 },  // Wrap to end
        None => 0,
    };
    state.select(Some(i));
}
```

**Key details:**
- Uses **filtered** length, not raw data length — navigation respects search filters
- Wraps around — going past the last item goes to the first
- Handles empty lists — does nothing if `len == 0`

## 4. Enter — The Progressive Drill-Down

Enter does different things based on focus, creating a natural workflow:

```
Repo → Enter → Load branches → Focus Branches
Branches → Enter → Show workflows → Focus Workflows  
Workflows → Enter → Load inputs → Focus Inputs
Inputs → Enter → Dispatch confirmation popup
```

```rust
KeyCode::Enter => {
    match state.ui.focus {
        Focus::Repo => {
            if let Err(e) = state.load_branches() {
                state.ui.output = Some(format!("Error: {}", e));
                state.ui.output_is_error = true;
            } else {
                state.ui.focus = Focus::Branches;  // Auto-advance
            }
        }
        Focus::Branches => {
            if let Err(e) = state.load_workflows() {
                state.ui.output = Some(format!("Error: {}", e));
                state.ui.output_is_error = true;
            } else {
                state.ui.focus = Focus::Workflows;
            }
        }
        Focus::Workflows => {
            if let Err(e) = state.load_inputs() {
                state.ui.output = Some(format!("Error: {}", e));
                state.ui.output_is_error = true;
            } else {
                state.ui.focus = Focus::Inputs;
            }
        }
        Focus::Inputs => {
            // Show dispatch confirmation
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
        Focus::Output => {}
    }
}
```

> **UX principle:** Enter always moves forward. Tab lets you move freely between panels.

## 5. Visual Focus Feedback

The renderer uses the focus to style panels:

```rust
let is_focused = matches!(state.ui.focus, Focus::Repo);

let highlight_style = if is_focused {
    Style::default().add_modifier(Modifier::BOLD).fg(Color::Blue)
} else {
    Style::default().fg(Color::Gray)
};

let border_style = if is_focused {
    Style::default().fg(Color::Blue)
} else {
    Style::default().fg(Color::Gray)
};
```

This gives the user clear visual feedback about which panel will respond to j/k and Enter.

## 6. The Selected Item Chain

To dispatch a workflow, the app needs all three selections:

```rust
pub fn run_workflow(&mut self) -> Result<(), Box<dyn Error>> {
    // Need all three selections
    let repo_idx = self.selected_repo_real_index()
        .ok_or("No repo selected.")?;
    let repo_name = &self.data.repos[repo_idx].name;

    let branch_idx = self.selected_branch_real_index()
        .ok_or("No branch selected.")?;
    let branch = &self.data.branches[branch_idx];

    let wf_idx = self.selected_workflow_real_index()
        .ok_or("No workflow selected.")?;
    let workflow = &self.data.workflows[wf_idx].name;

    // Now dispatch with all three
    self.github.dispatch_workflow(repo_name, branch, workflow, &self.data.input_fields)?;
    Ok(())
}
```

Each selection maps through the filtered index pattern:
```
UI selection → filtered index → real data index → actual data
```

---

## Exercises

1. **Focus cycling:** Implement Tab and Shift+Tab cycling. Add visual feedback — print 
   the current focus name in the title bar.

2. **List navigation:** Add j/k navigation that operates on the focused list. Test with 
   3 lists of dummy data.

3. **Progressive Enter:** Implement the drill-down pattern where Enter on repos loads 
   branches, Enter on branches shows workflows, etc.

4. **Auto-advance:** After Enter loads data, automatically move focus to the next panel.

5. **Error handling:** If loading branches fails, show the error in the output panel 
   and DON'T advance focus.

6. **Connect to the app:** Trace the complete flow of pressing Enter on a repo:
   - Which match arm fires?
   - What does `load_branches()` do?
   - What state changes after?
   - What does the UI look like after?

---

## Key Takeaways

- Focus enum tracks which panel is active
- Tab cycles forward, Shift+Tab cycles backward
- j/k navigate the focused list, or scroll output when Output is focused
- Enter is context-sensitive — each panel loads data for the next
- Auto-advance focus after successful data loading
- Visual feedback (color) tells the user which panel is active
- The selected-item chain goes: UI → filtered index → real index → data

---

**Next:** [Module 17 — Popups & Modal Dialogs →](./17-popups-and-modals.md)
