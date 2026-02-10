# Module 15 — Application State Management

## Learning Objectives

- Design a centralized state struct
- Separate data, UI state, and services
- Implement complex initialization
- Manage state transitions and data loading

---

## 1. The Central State Pattern

Instead of passing many variables around, group everything into one struct:

```rust
#[derive(Debug, Default)]
pub struct AppState {
    pub config: Config,         // Persisted configuration
    pub data: AppData,          // Runtime application data
    pub ui: UiState,            // Visual/interaction state
    pub github: GitHubService,  // External service
}
```

### Why One Struct?

- **Simple function signatures:** `fn handle(state: &mut AppState)` instead of 10 parameters
- **Easy to pass between modules:** event handler and renderer both take `&mut AppState`
- **Clear ownership:** the state struct owns everything

### The Three Sub-Structs

```rust
#[derive(Debug, Default)]
pub struct AppData {
    // What the app knows about the world
    pub repos: Vec<Repo>,
    pub branches: Vec<String>,
    pub workflows: Vec<Workflow>,
    pub inputs: Vec<String>,
    pub input_fields: Vec<InputField>,
    pub replays_list: Vec<ReplayConfig>,
}

#[derive(Debug, Default)]
pub struct UiState {
    // How the app looks right now
    pub repos_state: ListState,
    pub branches_state: ListState,
    pub workflows_state: ListState,
    pub focus: Focus,
    pub output: Option<String>,
    pub repos_hscroll: u16,
    pub output_scroll: u16,
    pub show_add_repo_popup: bool,
    pub search_active: bool,
    pub search_query: String,
    // ... etc
}
```

## 2. Complex Initialization

`AppState::new()` does more than just `Default::default()`:

```rust
impl AppState {
    pub fn new() -> Self {
        // 1. Load config from disk
        let config = load_config();
        
        // 2. Convert config repos to domain repos
        let repos: Vec<Repo> = config.repos.iter()
            .map(|rc| Repo { name: rc.name.clone(), branches: vec![], workflows: vec![] })
            .collect();

        // 3. Initialize list states with first item selected
        let mut repos_state = ListState::default();
        repos_state.select(Some(0));

        // 4. Create filtered index maps
        let filtered_repo_indices: Vec<usize> = (0..repos.len()).collect();

        // 5. Build welcome message
        let has_repos = !repos.is_empty();
        let output = Some(if has_repos {
            "Ready! Select a repo and press Enter.".to_string()
        } else {
            "Welcome! Press 'a' to add a repo.".to_string()
        });

        Self {
            config,
            github: GitHubService::new(),
            data: AppData { repos, /* ... */ },
            ui: UiState { repos_state, output, /* ... */ },
        }
    }
}
```

> **Design principle:** Initialize everything to a valid, consistent state.
> If there are repos, select the first one. If there aren't, show a welcome message.

## 3. State Transitions

Each user action is a method on `AppState` that transforms state:

### Loading Data

```rust
pub fn load_branches(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Get the selected repo
    let idx = self.selected_repo_real_index().ok_or("No repo selected.")?;
    let repo_name = self.data.repos[idx].name.clone();
    
    // 2. Show loading message
    self.ui.output = Some(format!("Fetching branches for '{}'...", repo_name));
    
    // 3. Fetch data from service
    let (branches, workflows) = self.github.fetch_repo_details(owner, name)?;
    
    // 4. Update data
    self.data.branches = branches;
    self.data.workflows = /* convert to Workflow structs */;
    
    // 5. Update UI state
    self.ui.branches_state.select(if self.data.branches.is_empty() { None } else { Some(0) });
    self.ui.filtered_branch_indices = (0..self.data.branches.len()).collect();
    
    // 6. Show success message
    self.ui.output = Some(format!("Loaded {} branches", self.data.branches.len()));
    
    Ok(())
}
```

### Pattern: Every Action Method

1. Validate preconditions (selected items exist)
2. Update output with "loading..." message
3. Call service layer
4. Update data
5. Update UI state (selections, filters)
6. Update output with result

## 4. Filtered Index Pattern

Instead of filtering the data itself, maintain a list of indices:

```rust
// Full data
self.data.repos = vec![repo_a, repo_b, repo_c, repo_d];

// Filtered indices — points into self.data.repos
self.ui.filtered_repo_indices = vec![0, 2, 3];  // Only showing a, c, d

// Get the real index for the selected filtered item
pub fn selected_repo_real_index(&self) -> Option<usize> {
    let sel = self.ui.repos_state.selected()?;           // Index in filtered list
    self.ui.filtered_repo_indices.get(sel).copied()       // Map to real index
}
```

This approach:
- Doesn't modify the original data
- Makes resetting search instant (just regenerate indices)
- Allows the UI list to show fewer items while data stays complete

## 5. State Consistency Rules

Keep these invariants:

1. **Selected index is always valid** — when data changes, reset selection
2. **Filtered indices match data** — when data loads, regenerate filters
3. **Search resets on data load** — new data makes old search irrelevant
4. **Output reflects current state** — every action updates the output panel

```rust
// After loading new branches:
self.ui.branches_state.select(
    if self.data.branches.is_empty() { None } else { Some(0) }
);
self.ui.filtered_branch_indices = (0..self.data.branches.len()).collect();
self.ui.search_active = false;
self.ui.search_query.clear();
```

## 6. Saving State Back to Config

When the user adds a repo or saves a replay, persist to disk:

```rust
fn save_repos_to_config(&self) -> Result<(), Box<dyn std::error::Error>> {
    // Load existing config to preserve replays we didn't touch
    let mut existing = load_config();
    
    let mut repo_configs: Vec<RepoConfig> = Vec::new();
    for repo in &self.data.repos {
        // Find existing replays for this repo
        let replays = existing.repos.iter()
            .find(|rc| rc.name == repo.name)
            .map(|rc| rc.replays.clone())
            .unwrap_or_default();
        
        repo_configs.push(RepoConfig {
            name: repo.name.clone(),
            replays,
        });
    }
    
    existing.repos = repo_configs;
    save_config(&existing)?;
    Ok(())
}
```

> **Key insight:** Re-load config before saving to preserve data (like replays) 
> that other code paths may have modified.

---

## Exercises

1. **Design your AppState:** Create `AppState`, `AppData`, and `UiState` structs.
   Implement `AppState::new()` that loads config and initializes all state.

2. **Implement `load_branches`:** Write the full method that:
   - Gets the selected repo
   - Calls `GitHubService::fetch_repo_details`
   - Updates branches and workflows in data
   - Resets UI selections and filters

3. **Filtered index pattern:** Implement `selected_repo_real_index()` and test it 
   with a filtered list. Verify it maps correctly.

4. **State consistency:** Write a `reset_search()` method that clears the search 
   and regenerates all filter indices.

5. **Connect to the app:** Read `AppState::new()` in `src/app.rs` and list every 
   piece of state that's initialized. For each, explain why it can't just be `Default`.

---

## Key Takeaways

- One `AppState` struct holds everything — simple to pass around
- Separate data, UI state, and services into sub-structs
- Complex initialization in `new()` sets up a valid, consistent state
- Every action method: validate → load → update data → update UI → show result
- Filtered indices avoid modifying source data — great for search/filter UX
- Always maintain state consistency when data changes

---

**Next:** [Module 16 — Focus System & Navigation →](./16-focus-and-navigation.md)
