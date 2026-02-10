# Module 11 — Domain Modeling

## Learning Objectives

- Design data types that represent your application's domain
- Decide between struct composition and flat structures
- Make types self-documenting with good field names and types
- Understand the relationship between domain types and serialization types

---

## 1. What Is Domain Modeling?

Domain modeling is the process of designing data types that represent the **concepts** 
in your application. Good domain types:

- Make invalid states impossible (or hard to represent)
- Are self-documenting
- Separate "what the app knows" from "how it's displayed" or "how it's stored"

## 2. Our App's Domain

The app deals with these concepts:

```
GitHub Repo → has branches → has workflows → has inputs → can be dispatched
```

Let's model each one:

### Repo

```rust
#[derive(Debug, Default)]
pub struct Repo {
    pub name: String,              // "owner/repo" format
    pub branches: Vec<String>,     // Branch names
    pub workflows: Vec<String>,    // Workflow filenames (e.g., "deploy.yml")
}
```

**Design decisions:**
- `name` stores both owner and repo as one string — matches GitHub's format
- `branches` and `workflows` are `Vec<String>` — simple names are enough
- No IDs needed — we identify repos by name

### Workflow

```rust
#[derive(Debug, Default, Clone)]
pub struct Workflow {
    pub id: String,          // Internal identifier
    pub name: String,        // Filename (e.g., "deploy.yml")
    pub inputs: Vec<String>, // Raw input display strings
}
```

**Design decisions:**
- `id` is generated internally (not from GitHub)
- `name` is the filename — what `gh` needs to dispatch
- `Clone` because workflows might be referenced from multiple places

### InputField

```rust
#[derive(Debug, Clone)]
pub struct InputField {
    pub name: String,           // Input parameter name
    pub description: String,    // Human-readable description
    pub input_type: String,     // "string", "boolean", "choice", "environment"
    pub required: bool,         // Whether the input is required
    pub default_value: String,  // Default from the workflow YAML
    pub options: Vec<String>,   // For choice inputs — the available options
    pub value: String,          // Current user-entered value
}
```

**Design decisions:**
- `input_type` is a `String`, not an enum — keeps parsing simple (YAML types vary)
- `default_value` and `value` are separate — user can reset to default
- `options` is empty for non-choice types
- `Clone` because inputs are copied into replay configs

## 3. Domain Types vs Config Types vs UI Types

Our app has three "layers" of types:

### Domain Types (`domain.rs`)
```
Repo, Workflow, InputField
```
These represent the actual concepts — what the app works with.

### Config Types (`config.rs`)
```
Config, RepoConfig, ReplayConfig, ReplayInput
```
These represent what's persisted to disk. They derive `Serialize`/`Deserialize`.

### UI Types (`app.rs`)
```
Focus, DispatchOutputColor, UiState, AppData
```
These represent the visual state — what's on screen.

### Why Separate Them?

| Layer | Changes When... | Example |
|-------|----------------|---------|
| Domain | The business rules change | Adding a "teams" concept |
| Config | The persistence format changes | Moving from YAML to SQLite |
| UI | The interface changes | Adding a new panel |

```
Config (disk) → Domain (logic) → UI (display)
```

> **In our app:** Config repos are converted to domain repos on startup:
> ```rust
> let repos: Vec<Repo> = config.repos
>     .iter()
>     .map(|rc| Repo {
>         name: rc.name.clone(),
>         branches: vec![],   // Not persisted — fetched fresh each time
>         workflows: vec![],
>     })
>     .collect();
> ```

## 4. The State Struct — Composing Domain + UI + Config

```rust
#[derive(Debug, Default)]
pub struct AppState {
    pub config: Config,         // Persisted configuration
    pub data: AppData,          // Runtime domain data
    pub ui: UiState,            // Visual/interaction state
    pub github: GitHubService,  // Service for API calls
}

#[derive(Debug, Default)]
pub struct AppData {
    pub repos: Vec<Repo>,
    pub branches: Vec<String>,
    pub workflows: Vec<Workflow>,
    pub inputs: Vec<String>,
    pub input_fields: Vec<InputField>,
    pub replays_list: Vec<ReplayConfig>,
}
```

**Design decisions:**
- `AppData` groups all data that changes during the app's lifecycle
- `UiState` groups all visual state (selections, popups, search)
- `config` is separate because it's loaded/saved independently
- `github` is a service — it has methods but no data

## 5. When to Use `String` vs Enum for Types

Our app uses `String` for `input_type`:
```rust
pub input_type: String,  // "string", "boolean", "choice", "environment"
```

An alternative would be an enum:
```rust
pub enum InputType {
    String,
    Boolean,
    Choice,
    Environment,
}
```

**Trade-offs:**

| Approach | Pros | Cons |
|----------|------|------|
| `String` | Easy to parse from YAML, flexible, forward-compatible | No compile-time checks, typos possible |
| `Enum` | Type-safe, exhaustive matching, self-documenting | Must handle unknown types, more code |

> The `gh.rs` docs file in our app actually defines an enum version for reference, 
> but the main app uses strings for simplicity.

## 6. Designing for the Happy Path

Notice how our domain types assume a specific flow:

```
1. User selects a Repo
2. App fetches branches and workflows for that repo
3. User selects a branch and workflow  
4. App fetches inputs for that workflow
5. User fills in inputs
6. App dispatches the workflow
```

The types reflect this:
- `branches` and `workflows` live on `AppData`, not on `Repo` — they represent 
  the *currently loaded* data for the selected repo
- `input_fields` live on `AppData` — they represent the current workflow's inputs
- This keeps the UI simple: there's only one set of branches/workflows/inputs at a time

---

## Exercises

1. **Design your domain types:** Create `src/domain.rs` with `Repo`, `Workflow`, and 
   `InputField`. Think about:
   - What derives each needs
   - Which fields should be `pub`
   - Which types need `Clone`

2. **Create config types:** Create `src/config.rs` types (`Config`, `RepoConfig`, 
   `ReplayConfig`, `ReplayInput`) with `Serialize` and `Deserialize`.

3. **Map between layers:** Write a function that converts `Vec<RepoConfig>` (from config) 
   into `Vec<Repo>` (domain). What information is lost in the conversion?

4. **Design the state struct:** Create `AppData` and `UiState` structs. Think about what 
   belongs in each. Create `AppState` that composes them.

5. **Connect to the app:** Compare `src/domain.rs`, `src/config.rs`, and the structs in 
   `src/app.rs`. Draw a diagram showing how data flows between the three layers.

---

## Key Takeaways

- Domain types model **what the app does**, not how it looks or how data is stored
- Separate domain, config, and UI types — they change for different reasons
- Compose your app state from sub-structs (`data`, `ui`, `config`, `service`)
- Use `String` for flexible parsing, enums for type safety — choose based on context
- Design types around the user's workflow — what data is active at each step

---

**Next:** [Module 12 — Terminal UI: Setup & Layout →](./12-tui-setup-layout.md)
