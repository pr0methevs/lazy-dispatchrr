# Lazy-Dispatchrr 🚀

A beautiful terminal user interface (TUI) for dispatching GitHub Actions workflows with ease.

![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust)
![License](https://img.shields.io/badge/license-GPLv3-blue)


## Features

- 🎯 **Quick Dispatch** — Select repo → branch → workflow → dispatch in seconds
- 🔍 **Fuzzy Search** — Press `/` to filter repos, branches, or workflows instantly
- 💾 **Replays** — Save workflow input presets and replay them with one keypress
- 📋 **Input Support** — Full support for all GitHub workflow input types (string, number, boolean, choice, environment)
- ✅ **Confirmation Popup** — Preview the exact `gh` command before execution
- 📺 **Live Logs** — Watch workflow run logs directly in the terminal
- 🌐 **Browser Integration** — Open runs in GitHub with a single keypress
- **Repo Name Scrolling** — Scroll horizontally for long repo names
- **Scrollable Output** — Scroll output when logs are long

## Prerequisites

- [GitHub CLI (`gh`)](https://cli.github.com/) installed and authenticated
- Rust 2024 edition (for building from source)

## Installation

```bash
# Clone the repository
git clone https://github.com/pr0methevs/lazy-dispatchr-rs.git
cd lazy-dispatchr-rs

# Build and run
cargo build --release
./target/release/lazy-dispatchr
```

## Usage

```bash
cargo run
# or after building:
./target/release/lazy-dispatchr
```

## Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `h` / `l` / `Left` / `Right` | Scroll repo names (Repos panel) |
| `Enter` | Select / Confirm |
| `Tab` | Cycle focus panels |
| `Shift+Tab` | Cycle focus (reverse) |
| `/` | Fuzzy search in current panel |
| `Esc` | Cancel / Close popup |
| `q` | Quit |
| `?` | Show help |

### Workflow Actions

| Key | Action |
|-----|--------|
| `i` | Open inputs editor (or dispatch if no inputs) |
| `D` | Dispatch workflow (with confirmation) |
| `S` | Save current inputs as a replay |
| `r` | Open replays for selected repo |
| `d` | Delete selected replay |
| `l` | Watch workflow run logs |
| `v` | Open repo/run in browser |
| `a` | Add a new repository |

### In Inputs Editor

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate inputs |
| `Enter` | Edit selected input |
| `Tab` | Cycle choice options forward |
| `Shift+Tab` | Cycle choice options backward |
| `D` | Dispatch workflow |
| `S` | Save as replay |
| `Esc` | Exit editing / Close popup |

### Output Panel

| Key | Action |
|-----|--------|
| `j` / `k` / `Up` / `Down` | Scroll output (when Output panel is focused) |

## Application Flow

### Main Navigation Flow

```mermaid
flowchart TD
    A[Start App] --> B[Repos Panel]
    B -->|Enter| C[Load Branches]
    C --> D[Branches Panel]
    D -->|Enter| E[Load Workflows]
    E --> F[Workflows Panel]
    F -->|Enter| G[Load Inputs]
    G --> H{Has Inputs?}
    H -->|Yes| I[Show Inputs in Output]
    H -->|No| J[Show 'No Inputs' Message]
    I -->|i| K[Inputs Editor Popup]
    J -->|i or Enter| L[Confirmation Popup]
    K -->|D| L
    L -->|y| M[Dispatch Workflow]
    L -->|n| N[Cancel]
    M --> O[Show Success + Log Prompt]
    O -->|l| P[Watch Logs]
    O -->|v| Q[Open in Browser]
```

### Workflow Dispatch Process

```mermaid
sequenceDiagram
    participant User
    participant TUI
    participant GitHub CLI
    participant GitHub API

    User->>TUI: Select Repo
    TUI->>GitHub CLI: gh api (fetch branches/workflows)
    GitHub CLI->>GitHub API: GraphQL Query
    GitHub API-->>GitHub CLI: Branches + Workflow Files
    GitHub CLI-->>TUI: Display Results
    
    User->>TUI: Select Workflow
    TUI->>GitHub CLI: gh api (fetch workflow YAML)
    GitHub CLI->>GitHub API: Get File Contents
    GitHub API-->>GitHub CLI: Workflow YAML
    GitHub CLI-->>TUI: Parse & Display Inputs
    
    User->>TUI: Press 'D' to Dispatch
    TUI->>TUI: Show Confirmation Popup
    User->>TUI: Press 'y' to Confirm
    TUI->>GitHub CLI: gh workflow run ...
    GitHub CLI->>GitHub API: Trigger Workflow
    GitHub API-->>GitHub CLI: Success
    GitHub CLI-->>TUI: Display Success
```

### Replay System Flow

```mermaid
flowchart LR
    subgraph Save Replay
        A[Edit Inputs] -->|S| B[Save to Config]
        B --> C[~/.config/dispatchrr/config.yml]
    end
    
    subgraph Run Replay
        D[Press 'r'] --> E[Load Replays]
        E --> F[Select Replay]
        F -->|Enter| G[Dispatch with Saved Inputs]
    end
    
    C -.-> E
```

### State Management

```mermaid
stateDiagram-v2
    [*] --> ReposFocused
    
    ReposFocused --> BranchesFocused: Enter (load branches)
    BranchesFocused --> WorkflowsFocused: Enter (load workflows)
    WorkflowsFocused --> InputsFocused: Enter (load inputs)
    InputsFocused --> ConfirmPopup: Enter/i/D
    
    ConfirmPopup --> OutputFocused: y (dispatch)
    ConfirmPopup --> InputsFocused: n (cancel)
    
    OutputFocused --> ReposFocused: Tab cycle
    
    state InputsFocused {
        [*] --> Navigating
        Navigating --> Editing: Enter
        Editing --> Navigating: Enter/Esc
    }
```

## Configuration

Configuration is stored at:
- **Linux/macOS**: `~/.config/dispatchrr/config.yml`
- **Windows**: `%LOCALAPPDATA%\dispatchrr\config.yml`

### Example Config

```yaml
repos:
  - name: owner/repo-name
    replays:
      - workflow: deploy.yml
        description: env=production, version=1.0.0
        inputs:
          - name: env
            value: production
          - name: version
            value: 1.0.0
```

## Project Structure

```
src/
├── main.rs        # Entry point
├── app.rs         # Application state & business logic  
├── event.rs       # Keyboard event handling
├── ui.rs          # TUI rendering (ratatui)
├── config.rs      # YAML config persistence
├── domain.rs      # Domain models (Repo, Workflow, InputField)
└── service/
    └── github.rs  # GitHub CLI integration
```

## Tech Stack

- **[Ratatui](https://ratatui.rs/)** — Terminal UI framework
- **[Crossterm](https://github.com/crossterm-rs/crossterm)** — Cross-platform terminal manipulation
- **[GitHub CLI](https://cli.github.com/)** — GitHub API interaction
- **[Serde](https://serde.rs/)** — Serialization/deserialization
- **[Fuzzy Matcher](https://github.com/lotabout/fuzzy-matcher)** — Fuzzy search implementation

## License

GPL-3.0

## Author

Artur Kaminski <artxk92@gmail.com>
