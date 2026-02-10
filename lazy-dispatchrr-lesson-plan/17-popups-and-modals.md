# Module 17 — Popups & Modal Dialogs

## Learning Objectives

- Render popup overlays on top of the main UI
- Manage popup state and keyboard input capture
- Build text input forms in popups
- Implement scrollable content in popups

---

## 1. Popup Architecture

Popups are rendered **after** the main UI, on top of it:

```rust
pub fn render(frame: &mut Frame, state: &mut AppState) {
    // 1. Render main UI (always)
    render_main_layout(frame, state);
    
    // 2. Render popups (conditionally, on top)
    if state.ui.show_add_repo_popup {
        render_add_repo_popup(frame, state);
    }
    if state.ui.show_inputs_popup {
        render_inputs_popup(frame, state);
    }
    if state.ui.show_confirm_dispatch {
        render_confirm_popup(frame, state);
    }
    if state.ui.show_help_popup {
        render_help_popup(frame, state);
    }
}
```

## 2. Centering a Popup

To center a popup, create a layout that splits the screen into thirds:

```rust
fn centered_popup(area: Rect, width: u16, height: u16) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Min(0),           // Space above
        Constraint::Length(height),    // Popup
        Constraint::Min(0),           // Space below
    ])
    .split(area);

    let horizontal = Layout::horizontal([
        Constraint::Min(0),           // Space left
        Constraint::Length(width),    // Popup
        Constraint::Min(0),           // Space right
    ])
    .split(vertical[1]);

    horizontal[1]  // The centered rectangle
}
```

Or using percentages:

```rust
let popup_v = Layout::vertical([
    Constraint::Percentage(35),
    Constraint::Length(8),       // Popup height
    Constraint::Percentage(35),
])
.split(area);

let popup_h = Layout::horizontal([
    Constraint::Percentage(25),
    Constraint::Min(40),         // Popup width
    Constraint::Percentage(25),
])
.split(popup_v[1]);

let popup_area = popup_h[1];
```

## 3. The `Clear` Widget

Before drawing a popup, clear its area to remove the background content:

```rust
use ratatui::widgets::Clear;

frame.render_widget(Clear, popup_area);   // Erase background
frame.render_widget(popup_block, popup_area);  // Draw popup
```

Without `Clear`, the popup's content would render on top of whatever was behind it,
creating visual artifacts.

## 4. Add Repo Popup — Text Input Form

```rust
if state.ui.show_add_repo_popup {
    let area = frame.area();
    
    // Center the popup
    let popup_area = centered_popup(area, 50, 8);
    frame.render_widget(Clear, popup_area);
    
    // Draw the border/title
    let block = Block::default()
        .title(" Add Repo (Tab: switch, Enter: submit, Esc: cancel) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::LightCyan));
    
    let inner = block.inner(popup_area);  // Area inside the border
    frame.render_widget(block, popup_area);
    
    // Layout for two fields
    let fields = Layout::vertical([
        Constraint::Length(1),  // Owner field
        Constraint::Length(1),  // Spacer
        Constraint::Length(1),  // Repo field
    ])
    .split(inner);
    
    // Render fields with cursor indicator
    let cursor = "█";
    let owner_text = if state.ui.add_repo_focus_owner {
        format!("Owner: {}{}", state.ui.add_repo_owner, cursor)
    } else {
        format!("Owner: {}", state.ui.add_repo_owner)
    };
    
    let owner_style = if state.ui.add_repo_focus_owner {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    
    frame.render_widget(Paragraph::new(owner_text).style(owner_style), fields[0]);
    // ... similar for repo field
}
```

### Key Concepts:
- **Block cursor** (`█`) as a text cursor indicator
- **Active field highlighting** — green+bold for focused, gray for unfocused
- **`block.inner(area)`** — gets the usable area inside the border

## 5. Inputs Popup — Scrollable Content

For many inputs, a scrollable paragraph works better than fixed-size rows:

```rust
// Build all fields into a single Vec<Line>
let mut lines: Vec<Line> = Vec::new();

for (i, field) in state.data.input_fields.iter().enumerate() {
    let is_selected = i == state.ui.input_fields_selected;
    let is_editing = is_selected && state.ui.input_fields_editing;

    // Line 1: name + description
    let name_style = if is_selected {
        Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    lines.push(Line::from(Span::styled(
        format!("{}: {}", field.name, field.description),
        name_style,
    )));

    // Line 2: metadata (type, default, options)
    lines.push(Line::from(Span::styled(
        format!("  type: {} | default: {}", field.input_type, field.default_value),
        Style::default().fg(Color::DarkGray),
    )));

    // Line 3: value (with cursor if editing)
    let value_text = if is_editing {
        format!("  > {}█", field.value)
    } else {
        format!("  > {}", field.value)
    };
    lines.push(Line::from(Span::styled(value_text, 
        if is_editing { Style::default().fg(Color::Green) }
        else { Style::default().fg(Color::Yellow) }
    )));

    // Line 4: spacer
    lines.push(Line::from(""));
}

// Calculate scroll to keep selected field visible
let lines_per_field: u16 = 4;
let selected_top = state.ui.input_fields_selected as u16 * lines_per_field;
let visible_height = inner.height;
let scroll_offset = if selected_top + lines_per_field <= visible_height {
    0
} else {
    selected_top.saturating_sub(visible_height / 3)
};

let paragraph = Paragraph::new(lines).scroll((scroll_offset, 0));
frame.render_widget(paragraph, inner);
```

> **Key insight:** Using a scrollable `Paragraph` instead of per-field `Layout` constraints
> prevents the layout solver from collapsing rows to zero height when there are many fields.

## 6. Confirmation Popup

A simple yes/no dialog:

```rust
if state.ui.show_confirm_dispatch {
    let popup_area = centered_popup(frame.area(), 60, 10);
    frame.render_widget(Clear, popup_area);
    
    let block = Block::default()
        .title(" Confirm Dispatch ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::LightYellow));
    
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);
    
    let text = format!(
        "Command to run:\n\n  {}\n\n(Y) to confirm  |  any other key to cancel",
        state.ui.dispatch_command_preview
    );
    
    frame.render_widget(
        Paragraph::new(text).wrap(Wrap { trim: true }),
        inner,
    );
}
```

## 7. Help Popup

Display keybindings in a styled popup:

```rust
let help_lines: Vec<Line> = vec![
    Line::from(Span::styled(
        "── General ──",
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )),
    Line::from(vec![
        Span::styled("  Tab / Shift+Tab  ", Style::default().fg(Color::LightCyan)),
        Span::raw("Cycle focus between panels"),
    ]),
    // ... more keybindings
];
```

## 8. Popup Input Handling (Review)

Remember: popup input handling uses the `continue` pattern from Module 14:

```rust
if state.ui.show_add_repo_popup {
    match key.code {
        KeyCode::Esc => state.ui.show_add_repo_popup = false,
        KeyCode::Char(c) => { /* type into field */ },
        // ...
    }
    continue;  // CRITICAL — prevents keys from reaching normal handlers
}
```

---

## Exercises

1. **Basic popup:** Create a centered popup with a border and title. Render it over the 
   main UI. Toggle it with '?' key.

2. **Text input popup:** Build an add-repo popup with two text fields (owner, repo).
   Implement Tab to switch fields, Enter to submit, Esc to cancel.

3. **Scrollable content:** Create a popup that shows 20+ items. Implement scrolling so 
   the selected item is always visible.

4. **Confirmation dialog:** Build a Y/N confirmation popup. Show a preview of the action,
   Y confirms, any other key cancels.

5. **Help popup:** Create a help popup showing keybindings with styled text (colored 
   keys, descriptions).

6. **Connect to the app:** Read the popup rendering code in `src/ui.rs` (lines 200+).
   For each popup, identify:
   - How it's centered
   - What triggers it to show
   - What dismisses it
   - How input is captured in `event.rs`

---

## Key Takeaways

- Popups render **after** the main UI, on top
- `Clear` widget erases background before drawing popup
- Center popups with percentage-based layouts
- `block.inner(area)` gives the usable area inside borders
- Text cursors: append `█` to the current field value
- Scrollable paragraphs for many items (avoid layout collapse)
- Always `continue` after handling popup input to prevent key leaking

---

**Next:** [Module 18 — Fuzzy Search →](./18-fuzzy-search.md)
