# Module 13 — Terminal UI: Widgets & Rendering

## Learning Objectives

- Use `Paragraph`, `List`, and `Block` widgets
- Style text with colors, bold, and other modifiers
- Work with stateful widgets (`ListState`)
- Build reusable styled components

---

## 1. Blocks — Borders and Titles

Almost every widget is wrapped in a `Block`:

```rust
use ratatui::widgets::{Block, Borders};

let block = Block::default()
    .title("My Panel")
    .borders(Borders::ALL)
    .border_style(Style::default().fg(Color::Blue));
```

### Border Styles

```rust
Borders::ALL         // All four sides
Borders::TOP         // Just the top
Borders::LEFT | Borders::RIGHT  // Left and right only
Borders::NONE        // No borders
```

> **In our app:** Every panel has a bordered block with a title:
> ```rust
> Block::default()
>     .title("Repos")
>     .borders(Borders::ALL)
>     .border_style(repos_border)
> ```

## 2. Paragraph — Text Display

```rust
use ratatui::widgets::{Paragraph, Wrap};

// Simple text
let p = Paragraph::new("Hello, world!")
    .style(Style::default().fg(Color::White))
    .block(Block::default().title("Output").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

frame.render_widget(p, area);
```

### Multi-line with Styled Spans

```rust
use ratatui::prelude::*;

let lines = vec![
    Line::from(Span::styled("✓ Success!", Style::default().fg(Color::Green))),
    Line::from(""),
    Line::from(vec![
        Span::styled("Command: ", Style::default().fg(Color::Yellow)),
        Span::raw("gh workflow run deploy.yml"),
    ]),
];

let paragraph = Paragraph::new(lines)
    .block(Block::default().title("Output").borders(Borders::ALL))
    .wrap(Wrap { trim: true });
```

> **In our app:** The output panel uses styled lines for dispatch results:
> ```rust
> let lines: Vec<Line> = state.ui.dispatch_output_lines
>     .iter()
>     .map(|(text, color)| {
>         let fg = match color {
>             DispatchOutputColor::Green => Color::Green,
>             DispatchOutputColor::Yellow => Color::Yellow,
>             // ...
>         };
>         Line::from(Span::styled(text.clone(), Style::default().fg(fg)))
>     })
>     .collect();
> ```

### Scrollable Paragraph

```rust
let paragraph = Paragraph::new(lines)
    .scroll((scroll_offset, 0));  // (vertical_offset, horizontal_offset)
```

> **In our app:** The output panel scrolls vertically with `output_scroll`, and the
> inputs popup uses scroll to keep the selected field visible.

## 3. List — Selectable Lists

Lists are the core navigation widget:

```rust
use ratatui::widgets::{List, ListItem, ListState};

// Create items
let items: Vec<ListItem> = vec![
    ListItem::new("owner/app-1"),
    ListItem::new("owner/app-2"),
    ListItem::new("owner/app-3"),
];

// Create the list widget
let list = List::new(items)
    .block(Block::default().title("Repos").borders(Borders::ALL))
    .highlight_symbol(">> ")
    .highlight_style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD));

// Render with state (for selection tracking)
let mut list_state = ListState::default();
list_state.select(Some(0));  // First item selected

frame.render_stateful_widget(list, area, &mut list_state);
```

### `ListState`

`ListState` tracks which item is selected:

```rust
let mut state = ListState::default();
state.select(Some(0));          // Select first item
state.select(None);             // No selection
let sel = state.selected();      // Option<usize>
```

> **In our app:** Each list panel has its own `ListState`:
> ```rust
> pub repos_state: ratatui::widgets::ListState,
> pub branches_state: ratatui::widgets::ListState,
> pub workflows_state: ratatui::widgets::ListState,
> ```

### Building Lists from Data

```rust
let repo_items: Vec<ListItem> = state.ui.filtered_repo_indices
    .iter()
    .filter_map(|&i| state.data.repos.get(i))
    .map(|r| ListItem::new(r.name.clone()))
    .collect();
```

> This pattern creates `ListItem`s from filtered indices — only showing repos 
> that match the current search.

### Handling Long List Items

If list items can be wider than the panel, slice them using a horizontal offset:

```rust
let visible = slice_with_offset(&r.name, state.ui.repos_hscroll as usize, visible_width as usize);
ListItem::new(visible)
```

> **In our app:** The repos panel supports horizontal scrolling for long names.

## 4. Styling

### Colors

```rust
Color::Red
Color::Green
Color::Blue
Color::Yellow
Color::LightCyan
Color::LightMagenta
Color::Gray
Color::DarkGray
Color::White
Color::Rgb(255, 128, 0)   // Custom RGB
```

### Modifiers

```rust
Modifier::BOLD
Modifier::ITALIC
Modifier::UNDERLINED
Modifier::DIM
```

### Combining Styles

```rust
let style = Style::default()
    .fg(Color::Blue)              // Foreground color
    .bg(Color::Black)             // Background color
    .add_modifier(Modifier::BOLD); // Bold text
```

### Conditional Styling

```rust
let highlight = if matches!(state.ui.focus, Focus::Repo) {
    Style::default().add_modifier(Modifier::BOLD).fg(Color::Blue)
} else {
    Style::default().fg(Color::Gray)
};

let border = if matches!(state.ui.focus, Focus::Repo) {
    Style::default().fg(Color::Blue)
} else {
    Style::default().fg(Color::Gray)
};
```

> **In our app:** Every panel changes color based on focus — blue when active, 
> gray when inactive. This gives the user clear visual feedback.

## 5. Dynamic Titles

Titles can include runtime information:

```rust
let title = if state.ui.search_active && matches!(state.ui.focus, Focus::Repo) {
    format!("Repos /{}█", state.ui.search_query)  // Show search cursor
} else if state.ui.filtered_repo_indices.len() < state.data.repos.len() {
    format!("Repos [{}/{}]", filtered_count, total_count)  // Show filter status
} else {
    "Repos".to_string()  // Normal title
};
```

## 6. The `Clear` Widget

Used for popups — clears an area before drawing over it:

```rust
use ratatui::widgets::Clear;

frame.render_widget(Clear, popup_area);  // Clear the background
frame.render_widget(popup_block, popup_area);  // Draw the popup
```

## 7. Text Alignment

```rust
let paragraph = Paragraph::new("Centered Title")
    .alignment(Alignment::Center);

// Alignment::Left (default)
// Alignment::Center
// Alignment::Right
```

> **In our app:** The title bar uses `Alignment::Center`:
> ```rust
> let title = Paragraph::new("Lazy-Dispatchrr")
>     .style(Color::LightRed)
>     .alignment(Alignment::Center);
> ```

## 8. Building a Complete Panel

Here's a full panel with all the pieces:

```rust
fn render_repos_panel(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let items: Vec<ListItem> = state.ui.filtered_repo_indices
        .iter()
        .filter_map(|&i| state.data.repos.get(i))
        .map(|r| ListItem::new(r.name.clone()))
        .collect();

    let is_focused = matches!(state.ui.focus, Focus::Repo);
    
    let highlight = if is_focused {
        Style::default().add_modifier(Modifier::BOLD).fg(Color::Blue)
    } else {
        Style::default().fg(Color::Gray)
    };
    
    let border = if is_focused {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(Color::Gray)
    };

    let list = List::new(items)
        .block(Block::default()
            .title("Repos")
            .borders(Borders::ALL)
            .border_style(border))
        .highlight_symbol(">> ")
        .highlight_style(highlight);

    frame.render_stateful_widget(list, area, &mut state.ui.repos_state);
}
```

---

## Exercises

1. **Paragraph widget:** Render a `Paragraph` with a title block, borders, and colored text.
   Try different `Wrap` settings.

2. **List widget:** Create a `List` of 5 items with a `ListState`. Render it and verify 
   the first item is highlighted.

3. **Styled output:** Create a multi-line `Paragraph` where each line has a different color.
   Use `Span::styled()` and `Line::from()`.

4. **Conditional styling:** Render two panels side by side. Add a "focus" variable and make 
   the focused panel blue and the unfocused panel gray.

5. **Build all three panels:** Create the repos, branches, and workflows list panels 
   with proper styling. Use dummy data for now.

6. **Connect to the app:** Read `src/ui.rs` and identify:
   - Every `frame.render_widget()` call and what it draws
   - Every `frame.render_stateful_widget()` call and its state
   - How conditional styling works for focused vs unfocused panels

---

## Key Takeaways

- `Block` provides borders and titles — wraps other widgets
- `Paragraph` for text, `List` for selectable items
- `ListState` tracks which item is selected
- `Span` + `Line` for fine-grained text styling
- `Style::default().fg(Color).add_modifier(Modifier)` for styling
- `render_widget` for static widgets, `render_stateful_widget` for interactive ones
- `Clear` to erase an area before drawing a popup

---

**Next:** [Module 14 — Event Loop & Keyboard Input →](./14-event-loop-input.md)
