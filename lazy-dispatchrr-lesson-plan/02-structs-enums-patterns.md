# Module 02 — Structs, Enums & Pattern Matching

## Learning Objectives

- Define and use structs to group related data
- Define enums for variants and state
- Use pattern matching with `match`, `if let`, and `while let`
- Understand `Option<T>` and `Result<T, E>`

---

## 1. Structs

Structs group related fields together, like classes without methods (those come via `impl`).

### Named-Field Structs

```rust
struct Repo {
    name: String,
    branches: Vec<String>,
    workflows: Vec<String>,
}

fn main() {
    let repo = Repo {
        name: String::from("owner/my-app"),
        branches: vec![String::from("main"), String::from("dev")],
        workflows: vec![String::from("deploy.yml")],
    };
    println!("Repo: {}", repo.name);
}
```

> **In our app:** `domain.rs` defines `Repo`, `Workflow`, and `InputField` —
> the core data types. `app.rs` defines `AppData`, `UiState`, and `AppState`.

### Struct Update Syntax

```rust
let repo2 = Repo {
    name: String::from("owner/other-app"),
    ..repo  // Take remaining fields from `repo`
};
```

### Tuple Structs and Unit Structs

```rust
struct Color(u8, u8, u8);              // Tuple struct
struct GitHubService;                   // Unit struct (no fields)
```

> **In our app:** `GitHubService` is a unit struct — it has no data, just methods.
> This is a common pattern for service/handler types.

## 2. Implementations (`impl`)

Methods and associated functions live in `impl` blocks:

```rust
struct Repo {
    name: String,
    branches: Vec<String>,
}

impl Repo {
    // Associated function (like a static method) — no `self`
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            branches: vec![],
        }
    }
    
    // Method — takes `&self` (immutable borrow)
    fn branch_count(&self) -> usize {
        self.branches.len()
    }
    
    // Method — takes `&mut self` (mutable borrow)
    fn add_branch(&mut self, branch: String) {
        self.branches.push(branch);
    }
}

fn main() {
    let mut repo = Repo::new("owner/app");
    repo.add_branch(String::from("main"));
    println!("{} has {} branches", repo.name, repo.branch_count());
}
```

> **In our app:** `AppState` has an extensive `impl` block with methods like
> `new()`, `add_repo()`, `load_branches()`, `run_workflow()`, etc.

## 3. Enums

Enums define a type that can be one of several variants:

```rust
enum Focus {
    Repo,
    Branches,
    Workflows,
    Inputs,
    Output,
}
```

### Enums with Data

```rust
enum Command {
    Quit,
    Echo(String),
    Move { x: i32, y: i32 },
    Color(u8, u8, u8),
}
```

> **In our app:** `Focus` is an enum that tracks which panel is active.
> `DispatchOutputColor` is an enum for styling dispatch output lines.

## 4. Pattern Matching with `match`

`match` is Rust's powerful switch/case — it must be **exhaustive** (cover all variants):

```rust
enum Focus {
    Repo,
    Branches,
    Workflows,
    Inputs,
    Output,
}

fn describe_focus(focus: &Focus) -> &str {
    match focus {
        Focus::Repo => "Repository list",
        Focus::Branches => "Branch list",
        Focus::Workflows => "Workflow list",
        Focus::Inputs => "Input fields",
        Focus::Output => "Output panel",
    }
}
```

### Match with Destructuring

```rust
enum AppEvent {
    KeyPress(char),
    Resize(u16, u16),
    Quit,
}

fn handle_event(event: AppEvent) {
    match event {
        AppEvent::KeyPress(c) => println!("Key: {}", c),
        AppEvent::Resize(w, h) => println!("Resize to {}x{}", w, h),
        AppEvent::Quit => println!("Quitting..."),
    }
}
```

### Match Guards and Wildcards

```rust
fn classify_key(code: char) {
    match code {
        'j' | 'k' => println!("Navigation"),
        'q' => println!("Quit"),
        'a'..='z' => println!("Lowercase letter"),
        _ => println!("Something else"),  // Wildcard: catches everything
    }
}
```

> **In our app:** `event.rs` uses massive `match` blocks on `KeyCode` variants
> to handle every possible keyboard input — this is the heart of the event system.

## 5. `Option<T>` — Nullable Values

Rust has no `null`. Instead, use `Option<T>`:

```rust
enum Option<T> {    // This is in the standard library
    Some(T),
    None,
}
```

Usage:

```rust
fn find_repo(repos: &[Repo], name: &str) -> Option<&Repo> {
    repos.iter().find(|r| r.name == name)
}

fn main() {
    let repos = vec![Repo::new("owner/app")];
    
    match find_repo(&repos, "owner/app") {
        Some(repo) => println!("Found: {}", repo.name),
        None => println!("Not found"),
    }
}
```

### Useful `Option` Methods

```rust
let x: Option<i32> = Some(42);

x.unwrap();           // Panics if None — use sparingly!
x.unwrap_or(0);       // Returns 0 if None
x.unwrap_or_default();// Returns type's default if None
x.is_some();          // true
x.is_none();          // false
x.map(|v| v * 2);    // Some(84)

// if let — when you only care about one variant
if let Some(value) = x {
    println!("Got: {}", value);
}
```

> **In our app:** `Option` is everywhere:
> - `output: Option<String>` — the output panel may or may not have text
> - `last_run_id: Option<u64>` — may not have dispatched yet
> - `repos_state.selected()` returns `Option<usize>`

## 6. `Result<T, E>` — Error Handling

```rust
enum Result<T, E> {    // Also in the standard library
    Ok(T),
    Err(E),
}
```

Usage:

```rust
fn parse_number(s: &str) -> Result<i32, String> {
    s.parse::<i32>().map_err(|e| format!("Parse error: {}", e))
}

fn main() {
    match parse_number("42") {
        Ok(n) => println!("Number: {}", n),
        Err(e) => println!("Error: {}", e),
    }
}
```

> **In our app:** Almost every method on `AppState` returns 
> `Result<(), Box<dyn std::error::Error>>` — we'll cover this pattern in Module 03.

## 7. `if let` and `while let`

For when you only care about one pattern:

```rust
// Instead of:
match some_option {
    Some(value) => do_something(value),
    None => {},
}

// Use:
if let Some(value) = some_option {
    do_something(value);
}
```

> **In our app:** `event.rs` uses `if let` for handling specific popup states:
> ```rust
> if let Some(field) = state.data.input_fields.get_mut(state.ui.input_fields_selected) {
>     // work with field
> }
> ```

## 8. The `matches!` Macro

A shorthand for checking if a value matches a pattern:

```rust
let focus = Focus::Repo;

// Instead of:
if let Focus::Repo = focus { /* ... */ }

// Use:
if matches!(focus, Focus::Repo) { /* ... */ }

// With multiple patterns:
if matches!(focus, Focus::Repo | Focus::Branches) { /* ... */ }
```

> **In our app:** `ui.rs` uses `matches!` extensively for conditional styling:
> ```rust
> let highlight = if matches!(state.ui.focus, Focus::Repo) {
>     Style::default().fg(Color::Blue)
> } else {
>     Style::default().fg(Color::Gray)
> };
> ```

---

## Exercises

1. **Define domain types:** Create `Repo`, `Workflow`, and `InputField` structs 
   (look at `src/domain.rs` for reference, but type them yourself):
   - `Repo` should have a `name`, `branches`, and `workflows`
   - `Workflow` should have an `id`, `name`, and `inputs`
   - `InputField` should have `name`, `description`, `input_type`, `required`, 
     `default_value`, `options`, and `value`

2. **Create a `Focus` enum** with variants `Repo`, `Branches`, `Workflows`, `Inputs`, `Output`.
   Write a function that takes a `Focus` and returns the next focus in the cycle (Tab behavior).

3. **Option practice:** Write a function `get_selected_name(repos: &[String], index: Option<usize>) -> Option<&String>`
   that returns the repo name at the given index, or `None` if index is `None` or out of bounds.

4. **Pattern matching drill:** Write a `match` block that handles these `KeyCode`-like scenarios:
   - `'q'` → print "quit"
   - `'j'` or `'k'` → print "navigate"  
   - `'a'` → print "add repo"
   - Any other character → print "unknown"

5. **Connect to the app:** Read through `src/app.rs` and identify all the structs and enums.
   For each one, note:
   - What derive macros it uses (e.g., `Debug`, `Default`)
   - Whether its fields are public (`pub`) or private
   - Which enums are used in `match` blocks in `event.rs`

---

## Key Takeaways

- Structs group data, `impl` blocks add behavior
- Enums represent "one of" — with optional data per variant
- `match` is exhaustive and the primary way to handle enums
- `Option<T>` replaces null — forces you to handle the missing case
- `Result<T, E>` forces you to handle errors — no silent failures
- `matches!()` is a handy shortcut for pattern-checking booleans

---

**Next:** [Module 03 — Error Handling →](./03-error-handling.md)
