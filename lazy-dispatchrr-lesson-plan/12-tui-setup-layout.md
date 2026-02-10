# Module 12 — Terminal UI: Setup & Layout

## Learning Objectives

- Set up a terminal application with ratatui and crossterm
- Understand raw mode and alternate screen
- Create layouts with `Layout`, `Constraint`, and `Direction`
- Build the main screen structure

---

## 1. What Is a TUI?

A **Terminal User Interface** is a graphical interface rendered entirely in the terminal.
Instead of pixels, you work with a character grid. Libraries like `ratatui` provide widgets 
(lists, tables, paragraphs) and layout systems.

### Our Stack

| Crate | Role |
|-------|------|
| `ratatui` | Widget library, layout engine, rendering |
| `crossterm` | Terminal backend (raw mode, events, cursor control) |

```bash
cargo add ratatui crossterm
```

## 2. Terminal Modes

### Normal Mode
- Input is line-buffered (user types, presses Enter)
- Output goes to the scroll buffer
- Not suitable for TUI apps

### Raw Mode
- Every keypress is immediately available
- No automatic echo, line buffering, or special handling
- Required for TUI apps

### Alternate Screen
- Switches to a separate screen buffer
- When you exit, the original terminal content is restored
- Standard for TUI apps

## 3. Basic Setup

```rust
use color_eyre::eyre::Result;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui;

fn main() -> Result<()> {
    // 1. Install error handler
    color_eyre::install()?;
    
    // 2. Initialize terminal (enters alternate screen + enables raw mode)
    let terminal = ratatui::init();
    enable_raw_mode()?;
    
    // 3. Run the application
    let result = run(terminal);
    
    // 4. Cleanup (ALWAYS runs — even on error)
    disable_raw_mode()?;
    ratatui::restore();
    
    result
}
```

> **In our app:** `main.rs` follows exactly this pattern:
> ```rust
> fn main() -> Result<()> {
>     let mut state = AppState::new();
>     color_eyre::install()?;
>     let terminal = ratatui::init();
>     enable_raw_mode()?;
>     let result = run(terminal, &mut state);
>     disable_raw_mode()?;
>     ratatui::restore();
>     result
> }
> ```
> Note: state is created BEFORE terminal setup — if config loading fails,
> we don't want to be stuck in raw mode.

## 4. The Render Loop

TUI apps use a loop: draw the screen → read input → update state → repeat.

```rust
use ratatui::DefaultTerminal;

pub fn run(mut terminal: DefaultTerminal) -> Result<()> {
    loop {
        // Draw the current frame
        terminal.draw(|frame| {
            // All rendering happens here
            render(frame);
        })?;
        
        // Wait for input (blocking)
        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            if key.code == crossterm::event::KeyCode::Char('q') {
                break;
            }
        }
    }
    Ok(())
}
```

### The `Frame`

The `Frame` is your canvas. It has:
- `frame.area()` — the full terminal `Rect` (x, y, width, height)
- `frame.render_widget(widget, area)` — draw a widget in a rectangle
- `frame.render_stateful_widget(widget, area, state)` — draw a stateful widget

## 5. Layouts

Layouts divide a `Rect` into smaller `Rect`s:

### Vertical Layout

```rust
use ratatui::prelude::*;

let chunks = Layout::vertical([
    Constraint::Length(1),   // Title bar: exactly 1 row
    Constraint::Min(0),      // Main content: fills remaining space
    Constraint::Length(1),   // Status bar: exactly 1 row
])
.split(frame.area());

// chunks[0] = title area
// chunks[1] = main content area
// chunks[2] = status bar area
```

### Horizontal Layout

```rust
let columns = Layout::horizontal([
    Constraint::Percentage(25),   // Left panel: 25% width
    Constraint::Percentage(75),   // Right panel: 75% width
])
.split(main_area);
```

### Constraint Types

| Constraint | Behavior |
|-----------|----------|
| `Length(n)` | Exactly n cells |
| `Min(n)` | At least n cells (fills remaining) |
| `Max(n)` | At most n cells |
| `Percentage(n)` | n% of the parent |
| `Ratio(a, b)` | a/b of the parent |

## 6. Our App's Layout Structure

```
┌──────────────────────────────────────────────────────────┐
│                    Title Bar (1 row)                      │
├──────────────┬───────────────────────────────────────────┤
│  Left 25%    │          Right 75%                         │
│ ┌──────────┐ │   ┌────────────────────────────────────┐  │
│ │  Repos   │ │   │                                    │  │
│ │  (33%)   │ │   │          Output Panel               │  │
│ ├──────────┤ │   │                                    │  │
│ │ Branches │ │   │                                    │  │
│ │  (33%)   │ │   │                                    │  │
│ ├──────────┤ │   │                                    │  │
│ │Workflows │ │   │                                    │  │
│ │  (34%)   │ │   │                                    │  │
│ └──────────┘ │   └────────────────────────────────────┘  │
├──────────────┴───────────────────────────────────────────┤
│                   Help Bar (1 row)                        │
└──────────────────────────────────────────────────────────┘
```

In code:

```rust
pub fn render(frame: &mut Frame, state: &mut AppState) {
    // Top-level: title, main, bottom
    let main_layout = Layout::vertical([
        Constraint::Length(1),  // Title
        Constraint::Min(0),     // Main
        Constraint::Length(1),  // Help bar
    ])
    .split(frame.area());

    // Main: left 25%, right 75%
    let areas = Layout::horizontal([
        Constraint::Percentage(25),
        Constraint::Percentage(75),
    ])
    .split(main_layout[1]);

    // Left: three stacked panels
    let left_columns = Layout::vertical([
        Constraint::Percentage(33),
        Constraint::Percentage(33),
        Constraint::Percentage(34),
    ])
    .split(areas[0]);

    // Now render widgets into each area...
    // left_columns[0] → Repos
    // left_columns[1] → Branches
    // left_columns[2] → Workflows
    // areas[1]        → Output
}
```

## 7. Rect — The Building Block

Every area is a `Rect`:

```rust
pub struct Rect {
    pub x: u16,       // Left edge
    pub y: u16,       // Top edge
    pub width: u16,   // Width
    pub height: u16,  // Height
}
```

You rarely create these manually — `Layout::split()` does it for you.

## 8. Putting It Together — Minimal TUI App

```rust
mod ui;
mod event;

use color_eyre::eyre::Result;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    enable_raw_mode()?;
    
    let result = event::run(terminal);
    
    disable_raw_mode()?;
    ratatui::restore();
    result
}
```

```rust
// event.rs
use color_eyre::eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::DefaultTerminal;

pub fn run(mut terminal: DefaultTerminal) -> Result<()> {
    loop {
        terminal.draw(|frame| crate::ui::render(frame))?;
        
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                    break;
                }
            }
        }
    }
    Ok(())
}
```

```rust
// ui.rs
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame) {
    let layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(frame.area());

    let title = Paragraph::new("My TUI App")
        .alignment(Alignment::Center);
    frame.render_widget(title, layout[0]);

    let content = Paragraph::new("Press 'q' to quit")
        .block(Block::default().title("Main").borders(Borders::ALL));
    frame.render_widget(content, layout[1]);

    let help = Paragraph::new("q: quit");
    frame.render_widget(help, layout[2]);
}
```

---

## Exercises

1. **Minimal TUI:** Create a program that enters raw mode, draws "Hello TUI!" centered on 
   screen, and quits on 'q'. Follow the setup pattern from section 3.

2. **Layout practice:** Create a 3-row layout (title, content, footer) with a 2-column 
   split in the content area. Put a `Paragraph` in each section.

3. **Match our layout:** Recreate the app's layout structure — title bar, left sidebar 
   with 3 panels, right output area, and bottom help bar.

4. **Dynamic sizing:** Experiment with `Constraint::Min(0)` vs `Constraint::Percentage(50)`.
   Resize your terminal and observe how the layout changes.

5. **Connect to the app:** Read the first 20 lines of `src/ui.rs` — the `render()` function's 
   layout code. Map each `Layout::split()` call to a region in the UI.

---

## Key Takeaways

- `ratatui::init()` + `enable_raw_mode()` sets up the terminal
- Always cleanup with `disable_raw_mode()` + `ratatui::restore()`
- The render loop: draw → read input → update state → repeat
- Layouts split rectangles: `Layout::vertical()` / `Layout::horizontal()`
- `Constraint::Length(n)` for fixed sizes, `Constraint::Min(0)` for flexible fill
- Our app: title(1) + main(flex) + help(1), main = left(25%) + right(75%)

---

**Next:** [Module 13 — Terminal UI: Widgets & Rendering →](./13-tui-widgets-rendering.md)
