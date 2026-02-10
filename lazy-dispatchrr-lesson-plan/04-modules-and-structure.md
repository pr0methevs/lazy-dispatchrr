# Module 04 — Modules & Project Structure

## Learning Objectives

- Understand Rust's module system (`mod`, `pub`, `use`)
- Organize code across multiple files
- Create nested module hierarchies with `mod.rs`
- Know the difference between `pub`, `pub(crate)`, and private

---

## 1. Why Modules?

As projects grow, you need to organize code into logical units. Rust's module system:
- Controls visibility (what's public vs private)
- Creates namespaces (avoids name collisions)
- Separates concerns (UI, business logic, services)

## 2. Declaring Modules

### Inline Modules

```rust
mod math {
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }
    
    fn secret() -> i32 {  // Private — only visible inside `math`
        42
    }
}

fn main() {
    println!("{}", math::add(3, 4));
    // math::secret();  // ERROR: private function
}
```

### File-Based Modules

When you write `mod foo;` in a file, Rust looks for:
1. `foo.rs` (same directory)
2. `foo/mod.rs` (subdirectory)

> **In our app:** `main.rs` declares all top-level modules:
> ```rust
> mod app;
> mod event;
> mod ui;
> pub mod config;
> pub mod domain;
> pub mod service;
> ```
> 
> Rust then loads `app.rs`, `event.rs`, `ui.rs`, `config.rs`, `domain.rs`, 
> and `service/mod.rs`.

## 3. Our App's Module Structure

```
src/
├── main.rs           # Declares: mod app, event, ui, config, domain, service
├── app.rs            # Business logic and state
├── config.rs         # Config loading/saving
├── domain.rs         # Core data types
├── event.rs          # Event loop and keyboard handling
├── ui.rs             # All rendering
└── service/
    ├── mod.rs        # Declares: pub mod github; pub use github::GitHubService;
    └── github.rs     # GitHub API integration
```

### The `service/mod.rs` Pattern

When you have a directory module, `mod.rs` is the entry point:

```rust
// src/service/mod.rs
pub mod github;                    // Declares github.rs as a sub-module
pub use github::GitHubService;     // Re-exports for convenience
```

This means code elsewhere can do:
```rust
use crate::service::GitHubService;    // Instead of crate::service::github::GitHubService
```

## 4. Visibility Rules

| Keyword | Visibility |
|---------|-----------|
| (none) | Private — only visible in the current module |
| `pub` | Public — visible to everyone |
| `pub(crate)` | Visible within the crate, but not to external users |
| `pub(super)` | Visible to the parent module |

### Struct Field Visibility

Struct fields have their own visibility:

```rust
pub struct AppState {
    pub config: Config,          // Public field
    pub data: AppData,           // Public field
    pub ui: UiState,             // Public field
    pub github: GitHubService,   // Public field
}
```

> **In our app:** All struct fields in `app.rs` are `pub` because they're accessed 
> from `event.rs` (event handling) and `ui.rs` (rendering). In a library, you'd be 
> more careful about encapsulation.

## 5. The `use` Statement

`use` brings items into scope:

```rust
// Full path
crate::config::load_config();

// With `use`
use crate::config::load_config;
load_config();

// Multiple items
use crate::config::{load_config, save_config, Config, RepoConfig};

// Wildcard (use sparingly)
use crate::config::*;
```

### `crate`, `self`, and `super`

```rust
use crate::domain::Repo;        // From the crate root
use self::helper::do_thing;      // From current module
use super::Config;               // From parent module
```

> **In our app:** Each file starts with `use` statements pulling in what it needs:
> ```rust
> // app.rs
> use crate::config::{load_config, save_config, Config, ReplayConfig, ReplayInput, RepoConfig};
> use crate::domain::{InputField, Repo, Workflow};
> use crate::service::github::GitHubService;
> ```

## 6. Building the Structure from Scratch

Let's build the module structure step by step for your project:

### Step 1: Start with `main.rs`

```rust
// src/main.rs
mod domain;    // Load domain.rs
mod config;    // Load config.rs
mod service;   // Load service/mod.rs
mod app;       // Load app.rs
mod ui;        // Load ui.rs
mod event;     // Load event.rs

fn main() {
    println!("Modules loaded!");
}
```

### Step 2: Create each file with minimal content

```rust
// src/domain.rs
pub struct Repo {
    pub name: String,
}
```

```rust
// src/config.rs
pub struct Config {
    pub repos: Vec<String>,
}
```

```rust
// src/service/mod.rs
pub mod github;
```

```rust
// src/service/github.rs
pub struct GitHubService;
```

```rust
// src/app.rs
use crate::config::Config;
use crate::service::github::GitHubService;

pub struct AppState {
    pub config: Config,
    pub github: GitHubService,
}
```

```rust
// src/ui.rs
use crate::app::AppState;

pub fn render(state: &AppState) {
    println!("Rendering...");
}
```

```rust
// src/event.rs
use crate::app::AppState;

pub fn run(state: &mut AppState) {
    println!("Event loop...");
}
```

### Step 3: Wire it up in `main.rs`

```rust
mod domain;
mod config;
mod service;
mod app;
mod ui;
mod event;

use app::AppState;
use event::run;

fn main() {
    let mut state = AppState { /* ... */ };
    run(&mut state);
}
```

## 7. Module Ordering and Circular Dependencies

Modules are declared in order, but the **compiler sees the whole crate at once**.
You can have `app.rs` use types from `domain.rs` and `event.rs` use types from `app.rs`
without worrying about order.

However, **circular dependencies between crates** are not allowed. Within a single 
crate (our app), any module can use any other module.

---

## Exercises

1. **Create the skeleton:** In your `tui-dispatcher` project, create all the files 
   listed above with minimal content. Run `cargo check` to verify everything compiles.

2. **Add `pub use` re-exports:** In `service/mod.rs`, add a `pub use github::GitHubService;` 
   line. Then use `crate::service::GitHubService` from `app.rs` instead of 
   `crate::service::github::GitHubService`.

3. **Visibility exercise:** Make a struct with one `pub` field and one private field.
   Try to access the private field from another module. Observe the error.

4. **Import styles:** Practice different `use` styles:
   - `use crate::domain::Repo;` (single item)
   - `use crate::config::{Config, load_config};` (multiple items)
   - `use crate::app::AppState;` (from a sibling module)

5. **Connect to the app:** Open `src/main.rs` and trace every `mod` declaration.
   For each one, find the corresponding file. Draw a dependency diagram showing 
   which modules use which other modules.

---

## Key Takeaways

- `mod foo;` loads `foo.rs` or `foo/mod.rs`
- `pub` makes items visible outside their module
- `use` brings items into scope to avoid long paths
- `pub use` re-exports items (convenient APIs)
- All modules in a crate can reference each other — no circular dependency issues within a crate
- Group related types into their own modules: domain types, config, services, UI

---

**Next:** [Module 05 — Collections, Iterators & Closures →](./05-collections-iterators-closures.md)
