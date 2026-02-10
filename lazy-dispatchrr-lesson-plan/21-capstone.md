# Module 21 — Capstone: Build Your Own TUI Workflow Dispatcher

## 🎯 Objective

Build a terminal UI application that dispatches GitHub Actions workflows. Your app should 
follow the same **architecture and reasoning** as Lazy-Dispatchrr, but be written from 
scratch in your own style.

---

## The Challenge

Create a TUI app that:

1. **Manages GitHub repos** — add, list, and persist repositories
2. **Browses repo data** — fetch branches and workflow files from GitHub
3. **Parses workflow inputs** — read YAML workflow files and extract `workflow_dispatch` inputs
4. **Dispatches workflows** — build `gh` commands with user-provided inputs
5. **Supports fuzzy search** — filter repos, branches, and workflows
6. **Saves replay presets** — remember frequently-used input configurations
7. **Shows results** — display dispatch status, logs, and errors

## Architecture Requirements

Your project should have this module structure (or equivalent):

```
src/
├── main.rs          # Entry point — terminal setup, run loop
├── app.rs           # Central state + business logic methods
├── config.rs        # YAML config persistence
├── domain.rs        # Core data types
├── event.rs         # Keyboard event handling
├── ui.rs            # All rendering logic
└── service/
    ├── mod.rs       # Module exports
    └── github.rs    # GitHub CLI integration
```

## Step-by-Step Build Plan

### Phase 1: Foundation (Modules 00–04)

- [ ] Create the project with `cargo new tui-dispatcher`
- [ ] Set up `Cargo.toml` with all dependencies
- [ ] Create the module structure (all files, minimal content)
- [ ] Verify `cargo check` passes

**Dependencies to add:**
```toml
[dependencies]
base64 = "0.22"
color-eyre = "0.6"
crossterm = "0.29"
dirs = "6.0"
fuzzy-matcher = "0.3"
ratatui = "0.30"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
```

### Phase 2: Domain & Config (Modules 05–09)

- [ ] Define `Repo`, `Workflow`, `InputField` in `domain.rs`
- [ ] Define `Config`, `RepoConfig`, `ReplayConfig`, `ReplayInput` in `config.rs`
- [ ] Implement `config_path()` with platform-specific logic
- [ ] Implement `load_config()` and `save_config()`
- [ ] Test: save a config, reload it, verify data

### Phase 3: GitHub Service (Module 10)

- [ ] Create `GitHubService` in `service/github.rs`
- [ ] Implement `fetch_repo_details()` — GraphQL query for branches + workflows
- [ ] Implement `fetch_workflow_inputs()` — REST API + base64 decode + YAML parse
- [ ] Implement `dispatch_workflow()` — build and run `gh workflow run`
- [ ] Implement `get_latest_run_logs()` — fetch run status and logs
- [ ] Test each method against a real repo

### Phase 4: Application State (Modules 11, 15)

- [ ] Create `AppState`, `AppData`, `UiState` in `app.rs`
- [ ] Define `Focus` and `DispatchOutputColor` enums
- [ ] Implement `AppState::new()` — load config, initialize state
- [ ] Implement `selected_*_real_index()` helpers
- [ ] Implement `load_branches()`, `load_workflows()`, `load_inputs()`
- [ ] Implement `add_repo()` and `save_repos_to_config()`

### Phase 5: Basic TUI (Modules 12–13)

- [ ] Set up terminal in `main.rs` (ratatui init, raw mode, cleanup)
- [ ] Create the layout in `ui.rs` (title, left sidebar, right output, help bar)
- [ ] Render repos, branches, workflows lists
- [ ] Render output panel
- [ ] Add conditional styling based on focus

### Phase 6: Event Loop & Navigation (Modules 14, 16)

- [ ] Implement the main event loop in `event.rs`
- [ ] Handle `q`/`Esc` to quit
- [ ] Implement `j`/`k` list navigation with wrapping
- [ ] Implement `Tab`/`Shift+Tab` focus cycling
- [ ] Implement `Enter` for progressive drill-down (repo → branches → workflows → inputs)

### Phase 7: Popups (Module 17)

- [ ] Add repo popup (`a` key) with two text fields
- [ ] Inputs popup (`i` key) with scrollable fields
- [ ] Confirmation popup (before dispatch)
- [ ] Help popup (`?` key) with keybinding reference
- [ ] Replays popup (`r` key) with list navigation

### Phase 8: Fuzzy Search (Module 18)

- [ ] Implement `update_search_filter()` for repos, branches, workflows
- [ ] Implement `reset_search()`
- [ ] Handle `/` to activate search
- [ ] Handle typing, backspace, Enter (confirm), Esc (cancel)
- [ ] Show search state in panel titles

### Phase 9: Dispatch & Replays (Module 19)

- [ ] Implement `build_dispatch_command()` — preview string
- [ ] Implement `run_workflow()` — actual dispatch
- [ ] Implement post-dispatch flow (logs, browser)
- [ ] Implement `save_replay()`, `open_replays()`, `run_replay()`, `delete_replay()`

### Phase 10: Polish (Module 20)

- [ ] Add `build.rs` (even if minimal)
- [ ] Fill in all `Cargo.toml` metadata
- [ ] Run `cargo clippy` and fix warnings
- [ ] Run `cargo fmt`
- [ ] Write a README with features, installation, and keybindings
- [ ] Build release binary

---

## Acceptance Criteria

Your app should be able to:

1. ✅ Start up and show a welcome message
2. ✅ Add a repo via popup (owner + name fields)
3. ✅ Persist repos to `~/.config/your-app/config.yml`
4. ✅ Fetch branches and workflows via `gh` CLI
5. ✅ Navigate lists with j/k, cycle focus with Tab
6. ✅ Select a workflow and view its inputs
7. ✅ Fill in inputs (string, boolean, choice types)
8. ✅ Preview the dispatch command before confirming
9. ✅ Dispatch and see a success message
10. ✅ View run logs after dispatch
11. ✅ Fuzzy search in any list panel with `/`
12. ✅ Save, load, run, and delete replay presets
13. ✅ Show help popup with `?`
14. ✅ Handle all errors gracefully (never crash)
15. ✅ Clean exit with `q`

---

## Design Guidelines

### Things to Keep the Same

- **Module structure** — separation of concerns is important for maintainability
- **State management pattern** — central `AppState` with sub-structs
- **Event priority** — popup handlers above normal handlers with `continue`
- **Filtered index pattern** — don't modify source data for search
- **Error display pattern** — catch errors, show in output panel
- **Confirmation before dispatch** — always preview the command

### Things You Can Change

- **Colors and styling** — pick your own color scheme
- **Layout proportions** — adjust the 25/75 split, panel heights
- **Keybindings** — use different keys if you prefer
- **Output format** — style the dispatch result differently
- **Panel names** — name them what makes sense to you
- **Additional features** — add anything you want!

### Stretch Goals

- 🔄 Auto-refresh logs on a timer
- 📊 Show workflow run history in a table
- 🎨 User-configurable color themes
- 📝 Editable input defaults per-repo
- 🔒 Support for private repos with different auth
- 📋 Copy command to clipboard
- 🏷️ Tag/favorite repos

---

## Debugging Tips

- **`cargo check` constantly** — catch errors early
- **`cargo clippy`** — learn idiomatic patterns
- **Read compiler errors** — Rust's errors are the best documentation
- **Print debugging** — use the output panel to show internal state while developing
- **Test `gh` commands manually** — verify they work before wrapping in Rust
- **Small commits** — commit after each working feature

---

## Congratulations! 🎉

If you've completed this capstone, you've learned:

- Rust fundamentals (ownership, borrowing, types, traits)
- Module system and project organization
- Error handling with Result and color-eyre
- Serialization with serde (JSON + YAML)
- File I/O and cross-platform configuration
- External process management
- Terminal UI with ratatui
- Event-driven programming
- Complex state management
- Fuzzy search algorithms
- Build scripts and release engineering

You're now equipped to build real-world Rust applications. Keep building! 🦀
