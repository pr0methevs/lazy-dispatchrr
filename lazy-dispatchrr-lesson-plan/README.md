# Rust Curriculum — Build a TUI GitHub Workflow Dispatcher

> **Goal:** By the end of this curriculum, you will have built a fully functional terminal UI
> application in Rust that dispatches GitHub Actions workflows — modeled after **Lazy-Dispatchrr**.

## Who This Is For

- Developers who want to learn Rust through a real-world project
- You should be comfortable with at least one programming language already
- No prior Rust experience required

## Prerequisites

- A computer with macOS, Linux, or Windows
- [GitHub CLI (`gh`)](https://cli.github.com/) installed and authenticated
- A GitHub account with at least one repository that has workflow files
- A text editor (VS Code recommended, with `rust-analyzer` extension)

## Curriculum Overview

| Module | Title | Key Concepts |
|--------|-------|--------------|
| 00 | [Environment Setup](./00-environment-setup.md) | Installing Rust, Cargo, tooling |
| 01 | [Rust Fundamentals — Ownership & Types](./01-rust-fundamentals.md) | Variables, types, ownership, borrowing, lifetimes |
| 02 | [Structs, Enums & Pattern Matching](./02-structs-enums-patterns.md) | Custom types, `match`, `Option`, `Result` |
| 03 | [Error Handling](./03-error-handling.md) | `Result`, `?` operator, `Box<dyn Error>`, `color-eyre` |
| 04 | [Modules & Project Structure](./04-modules-and-structure.md) | `mod`, `pub`, `use`, multi-file projects, `service/mod.rs` |
| 05 | [Collections, Iterators & Closures](./05-collections-iterators-closures.md) | `Vec`, `HashMap`, `.iter()`, `.map()`, `.filter()`, closures |
| 06 | [Traits & Implementations](./06-traits-and-implementations.md) | `impl`, `trait`, `Default`, `Debug`, `Clone`, derive macros |
| 07 | [Strings & String Manipulation](./07-strings.md) | `String` vs `&str`, formatting, splitting, joining |
| 08 | [Serialization with Serde](./08-serde-serialization.md) | `serde`, `serde_json`, `serde_yaml`, derive macros, custom fields |
| 09 | [File I/O & Configuration](./09-file-io-config.md) | `std::fs`, `PathBuf`, `dirs` crate, platform-specific paths, YAML config |
| 10 | [External Processes & CLI Integration](./10-external-processes.md) | `std::process::Command`, capturing output, `gh` CLI, `base64` decoding |
| 11 | [Domain Modeling](./11-domain-modeling.md) | Designing data types for your application domain |
| 12 | [Terminal UI — Setup & Layout](./12-tui-setup-layout.md) | `ratatui`, `crossterm`, raw mode, `Layout`, `Constraint` |
| 13 | [Terminal UI — Widgets & Rendering](./13-tui-widgets-rendering.md) | `List`, `Paragraph`, `Block`, `Borders`, styling, stateful widgets |
| 14 | [Event Loop & Keyboard Input](./14-event-loop-input.md) | `crossterm::event`, key handling, the render loop |
| 15 | [Application State Management](./15-app-state.md) | Central state struct, separating data/UI/service concerns |
| 16 | [Focus System & Navigation](./16-focus-and-navigation.md) | Focus enum, Tab cycling, list navigation helpers |
| 17 | [Popups & Modal Dialogs](./17-popups-and-modals.md) | Layered rendering, `Clear` widget, popup state machines |
| 18 | [Fuzzy Search](./18-fuzzy-search.md) | `fuzzy-matcher`, filtered indices, live search UX |
| 19 | [Workflow Dispatch & Replays](./19-dispatch-and-replays.md) | Building CLI commands, dispatch confirmation, saving/loading presets |
| 20 | [Build Scripts & Polish](./20-build-scripts-polish.md) | `build.rs`, conditional compilation, `cfg`, README, packaging |
| 21 | [Capstone: Build Your Own](./21-capstone.md) | Bring it all together — recreate the app from scratch |

## How to Use This Curriculum

1. **Work through modules in order** — each builds on the previous
2. **Type every exercise by hand** — don't copy-paste; muscle memory matters
3. **Run `cargo check` often** — let the compiler teach you
4. **Refer to the source code** in `src/` as a reference, but write your own version
5. **Complete the exercises** at the end of each module before moving on

## Project Architecture (What You're Building Toward)

```
src/
├── main.rs          # Entry point — initializes terminal and runs event loop
├── app.rs           # Application state, business logic, data loading
├── config.rs        # YAML config persistence (~/.config/dispatchrr/config.yml)
├── domain.rs        # Core data types (Repo, Workflow, InputField)
├── event.rs         # Keyboard event handling, the main loop
├── ui.rs            # All rendering logic (layouts, widgets, popups)
└── service/
    ├── mod.rs       # Module re-exports
    └── github.rs    # GitHub CLI integration (GraphQL, REST, dispatch)
```

## Estimated Time

- **Modules 00–10:** ~20–30 hours (Rust fundamentals + ecosystem)
- **Modules 11–20:** ~20–30 hours (Building the TUI app)
- **Module 21 (Capstone):** ~10–15 hours (Your own implementation)
- **Total:** ~50–75 hours depending on prior experience
