# Module 14 — Event Loop & Keyboard Input

## Learning Objectives

- Implement a blocking event loop with crossterm
- Handle keyboard events with `KeyCode` and `KeyModifiers`
- Filter key event kinds (press, release, repeat)
- Structure event handling for complex UIs

---

## 1. The Event Loop Pattern

Every TUI app follows this loop:

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│  Render  │ ──→ │  Wait    │ ──→ │  Handle  │ ──→ (back to Render)
│  Screen  │     │  Input   │     │  Event   │
└──────────┘     └──────────┘     └──────────┘
```

```rust
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;

pub fn run(mut terminal: DefaultTerminal, state: &mut AppState) -> Result<()> {
    loop {
        // 1. Render
        terminal.draw(|frame| render(frame, state))?;
        
        // 2. Wait for input (blocking)
        if let Event::Key(key) = event::read()? {
            // 3. Only handle key press events (not release/repeat)
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    // ... handle other keys
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
```

### Why `KeyEventKind::Press`?

On some platforms, `event::read()` returns press, release, AND repeat events.
Without this check, every keypress would trigger 2-3 events.

## 2. KeyCode Variants

```rust
use crossterm::event::KeyCode;

match key.code {
    KeyCode::Char('q') => { /* specific character */ },
    KeyCode::Char(c) => { /* any character: c */ },
    KeyCode::Enter => { /* enter/return */ },
    KeyCode::Esc => { /* escape */ },
    KeyCode::Tab => { /* tab */ },
    KeyCode::BackTab => { /* shift+tab */ },
    KeyCode::Backspace => { /* backspace */ },
    KeyCode::Up => { /* arrow up */ },
    KeyCode::Down => { /* arrow down */ },
    KeyCode::Left => { /* arrow left */ },
    KeyCode::Right => { /* arrow right */ },
    _ => { /* everything else */ },
}
```

### Key Modifiers

```rust
use crossterm::event::KeyModifiers;

if key.modifiers.contains(KeyModifiers::CONTROL) {
    // Ctrl+something
}
if key.modifiers.contains(KeyModifiers::SHIFT) {
    // Shift+something (also: uppercase chars come as Char('A'))
}
```

> **In our app:** Shift+D dispatches, Shift+S saves a replay. Uppercase chars are 
> distinguished from lowercase: `KeyCode::Char('D')` vs `KeyCode::Char('d')`.

## 3. Structuring Event Handling

Our app handles events in a specific priority order. This is critical for 
complex UIs with popups and modal states:

```
1. Help popup    → any key dismisses it
2. Add repo popup → captures all input
3. Confirm popup  → Y/N only
4. Log prompt    → L/V/any key
5. Inputs popup  → navigation + editing
6. Replays popup → navigation + actions
7. Search mode   → typing + navigation
8. Normal mode   → global keybindings
```

Each level "eats" the input and `continue`s the loop:

```rust
// Handle help popup — any key dismisses
if state.ui.show_help_popup {
    state.ui.show_help_popup = false;
    continue;  // Don't process this key further
}

// Handle add-repo popup
if state.ui.show_add_repo_popup {
    match key.code {
        KeyCode::Esc => { /* close */ },
        KeyCode::Enter => { /* submit */ },
        KeyCode::Char(c) => { /* type into field */ },
        _ => {}
    }
    continue;  // Don't fall through to normal handling
}

// ... more popup handlers ...

// Normal mode — only reached if no popup is active
match key.code {
    KeyCode::Char('q') => break,
    KeyCode::Tab => { /* cycle focus */ },
    // ...
}
```

> **In our app:** This `if popup { handle; continue; }` pattern prevents key 
> events from "leaking" through popups. Without `continue`, pressing 'q' in a 
> popup would quit the app.

## 4. Text Input Handling

For popup text fields:

```rust
if state.ui.show_add_repo_popup {
    match key.code {
        KeyCode::Esc => {
            state.ui.show_add_repo_popup = false;
            state.ui.add_repo_owner.clear();
            state.ui.add_repo_name.clear();
        }
        KeyCode::Tab => {
            // Toggle between owner and repo fields
            state.ui.add_repo_focus_owner = !state.ui.add_repo_focus_owner;
        }
        KeyCode::Enter => {
            // Submit the form
            let owner = state.ui.add_repo_owner.clone();
            let name = state.ui.add_repo_name.clone();
            if owner.is_empty() || name.is_empty() {
                state.ui.output = Some("Both fields required.".to_string());
            } else {
                state.add_repo(&owner, &name);
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
```

## 5. List Navigation Helpers

Reusable functions for moving up/down in lists:

```rust
fn select_next(state: &mut ListState, len: usize) {
    if len == 0 { return; }
    let i = match state.selected() {
        Some(i) => (i + 1) % len,  // Wrap around
        None => 0,
    };
    state.select(Some(i));
}

fn select_previous(state: &mut ListState, len: usize) {
    if len == 0 { return; }
    let i = match state.selected() {
        Some(0) => len - 1,         // Wrap to end
        Some(i) => i - 1,
        None => 0,
    };
    state.select(Some(i));
}
```

Usage:

```rust
KeyCode::Char('j') | KeyCode::Down => {
    match state.ui.focus {
        Focus::Repo => select_next(&mut state.ui.repos_state, state.ui.filtered_repo_indices.len()),
        Focus::Branches => select_next(&mut state.ui.branches_state, state.ui.filtered_branch_indices.len()),
        Focus::Workflows => select_next(&mut state.ui.workflows_state, state.ui.filtered_workflow_indices.len()),
        Focus::Output => state.ui.output_scroll = state.ui.output_scroll.saturating_add(1),
        _ => {}
    }
}
```

## 6. Focus Cycling

Tab cycles through panels:

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

## 7. Enter — Context-Sensitive Actions

Enter does different things based on focus:

```rust
KeyCode::Enter => {
    match state.ui.focus {
        Focus::Repo => {
            state.load_branches()?;       // Fetch branches for selected repo
            state.ui.focus = Focus::Branches;  // Auto-advance focus
        }
        Focus::Branches => {
            state.load_workflows()?;
            state.ui.focus = Focus::Workflows;
        }
        Focus::Workflows => {
            state.load_inputs()?;
            state.ui.focus = Focus::Inputs;
        }
        Focus::Inputs => {
            state.show_dispatch_confirmation()?;
        }
        Focus::Output => {}
    }
}
```

> **In our app:** Enter progressively drills down: Repo → Branches → Workflows → Inputs → Dispatch.

---

## Exercises

1. **Basic event loop:** Create an event loop that:
   - Displays a counter
   - Increments on 'j'/Down
   - Decrements on 'k'/Up
   - Quits on 'q'

2. **Text input:** Add a "popup" mode where typing characters builds a string.
   Backspace removes the last char. Esc closes the popup. Enter confirms.

3. **List navigation:** Create a list of 10 items with `ListState`. Implement 
   j/k navigation that wraps around. Highlight the selected item.

4. **Focus cycling:** Create 3 panels. Implement Tab/Shift+Tab cycling between them.
   Change the border color of the focused panel.

5. **Layered handling:** Implement the popup priority pattern:
   - A help popup that dismisses on any key
   - A text input popup that captures chars
   - Normal mode with navigation
   Verify that typing in the popup doesn't trigger normal-mode actions.

6. **Connect to the app:** Read all of `src/event.rs`. Map the execution flow for:
   - Pressing 'a' (add repo)
   - Typing a repo name and pressing Enter
   - Pressing Tab three times
   - Pressing Enter on a repo (loads branches)

---

## Key Takeaways

- The event loop: render → read → handle → repeat
- Always check `KeyEventKind::Press` to avoid duplicate events
- Use `if popup { handle; continue; }` to prevent key leaking
- Reusable `select_next`/`select_previous` helpers for list navigation
- Output panel can scroll when it has focus
- Focus cycling with `Tab`/`BackTab` using `match` on the focus enum
- Context-sensitive Enter that advances through the app's workflow

---

**Next:** [Module 15 — Application State Management →](./15-app-state.md)
