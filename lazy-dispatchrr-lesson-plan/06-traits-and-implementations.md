# Module 06 — Traits & Implementations

## Learning Objectives

- Understand traits as Rust's abstraction mechanism
- Use common standard library traits (`Debug`, `Default`, `Clone`, `Display`)
- Derive traits with `#[derive(...)]`
- Write your own trait implementations

---

## 1. What Are Traits?

Traits define shared behavior — similar to interfaces in other languages:

```rust
trait Describable {
    fn describe(&self) -> String;
}

struct Repo {
    name: String,
}

impl Describable for Repo {
    fn describe(&self) -> String {
        format!("Repository: {}", self.name)
    }
}
```

## 2. Derive Macros — Automatic Implementations

Rust can automatically implement many traits using `#[derive]`:

```rust
#[derive(Debug, Default, Clone)]
pub struct Repo {
    pub name: String,
    pub branches: Vec<String>,
    pub workflows: Vec<String>,
}
```

> **In our app:** Every struct uses derive macros. Here's what each does:

### `Debug` — Print with `{:?}`

```rust
#[derive(Debug)]
struct Point { x: i32, y: i32 }

let p = Point { x: 1, y: 2 };
println!("{:?}", p);   // Point { x: 1, y: 2 }
println!("{:#?}", p);  // Pretty-printed
```

> **In our app:** All structs derive `Debug` — essential for troubleshooting.

### `Default` — Create with Default Values

```rust
#[derive(Default)]
struct AppData {
    repos: Vec<Repo>,        // Default: empty vec
    branches: Vec<String>,   // Default: empty vec
}

let data = AppData::default();
// All fields get their type's default value
```

Default values:
| Type | Default |
|------|---------|
| `i32`, `u64`, etc. | `0` |
| `f64` | `0.0` |
| `bool` | `false` |
| `String` | `""` (empty) |
| `Vec<T>` | `[]` (empty) |
| `Option<T>` | `None` |

> **In our app:** `AppState`, `AppData`, `UiState`, `GitHubService` all derive `Default`.

### `Clone` — Deep Copy

```rust
#[derive(Clone)]
struct ReplayConfig {
    workflow: String,
    description: String,
}

let original = ReplayConfig { /* ... */ };
let copy = original.clone();
```

> **In our app:** `ReplayConfig` and `InputField` derive `Clone` because they need
> to be duplicated when saving/loading replays.

### `Copy` — Automatic Bit-wise Copy

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DispatchOutputColor {
    Green,
    Yellow,
    White,
    Blue,
}
```

`Copy` is for small, stack-only types. Enums without data and primitives can be `Copy`.

### `PartialEq` — Equality Comparison

```rust
#[derive(PartialEq)]
enum Focus {
    Repo,
    Branches,
}

let f = Focus::Repo;
if f == Focus::Repo { /* ... */ }
```

## 3. `Default` for Enums

For enums, you must specify which variant is the default:

```rust
#[derive(Debug, Default)]
pub enum Focus {
    #[default]
    Repo,      // This is the default variant
    Branches,
    Workflows,
    Inputs,
    Output,
}
```

> **In our app:** `Focus` defaults to `Repo` — when the app starts, the repo panel 
> is focused.

## 4. Serde Traits (Covered More in Module 08)

```rust
#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub repos: Vec<RepoConfig>,
}
```

`Serialize` and `Deserialize` are traits from the `serde` crate — they enable 
automatic conversion to/from JSON, YAML, TOML, etc.

## 5. Implementing Standard Library Traits Manually

Sometimes you need custom behavior instead of deriving:

### Custom `Display`

```rust
use std::fmt;

struct Repo {
    name: String,
    branch_count: usize,
}

impl fmt::Display for Repo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({} branches)", self.name, self.branch_count)
    }
}

fn main() {
    let repo = Repo { name: "owner/app".to_string(), branch_count: 5 };
    println!("{}", repo);  // "owner/app (5 branches)"
}
```

### Custom `Default`

When derived `Default` isn't enough:

```rust
impl AppState {
    pub fn new() -> Self {
        let config = load_config();
        // ... complex initialization logic ...
        Self {
            config,
            // ... custom defaults with loaded data ...
        }
    }
}
```

> **In our app:** `AppState::new()` does complex initialization (loads config, 
> sets up list states, creates welcome message) instead of using derived `Default`.

## 6. Trait Bounds

When a function needs a type to implement certain traits:

```rust
fn print_debug<T: std::fmt::Debug>(item: &T) {
    println!("{:?}", item);
}

// Or with `where` clause (cleaner for multiple bounds):
fn process<T>(item: &T) 
where
    T: Debug + Clone + Default 
{
    let copy = item.clone();
    println!("{:?}", copy);
}
```

## 7. `dyn Trait` — Trait Objects

When you need runtime polymorphism:

```rust
fn get_error() -> Box<dyn std::error::Error> {
    // Can return any type that implements Error
    "something went wrong".into()
}
```

> **In our app:** `Box<dyn std::error::Error>` is used as the error type everywhere,
> allowing functions to return different concrete error types.

---

## Exercises

1. **Derive practice:** Create a struct with all the derives used in our app:
   ```rust
   #[derive(Debug, Default, Clone)]
   struct MyStruct { /* fields */ }
   ```
   Test each: print with `{:?}`, create with `default()`, call `.clone()`.

2. **Custom Display:** Implement `Display` for your `Repo` struct so that
   `println!("{}", repo)` prints `"owner/name (3 branches, 2 workflows)"`.

3. **Default enum:** Create an enum `Panel` with a `#[default]` variant.
   Verify `Panel::default()` returns the right variant.

4. **Trait bounds:** Write a generic function `print_all<T: Display>(items: &[T])` 
   that prints each item in a slice.

5. **Connect to the app:** List every `#[derive(...)]` used in the project.
   For each derived trait, explain in one sentence why that struct needs it.

---

## Key Takeaways

- Traits define shared behavior — like interfaces
- `#[derive]` auto-implements common traits (`Debug`, `Default`, `Clone`, etc.)
- `Default` gives you zero-value initialization — critical for complex state structs
- `Clone` for explicit deep copies, `Copy` for automatic copies of small types
- `Box<dyn Trait>` enables runtime polymorphism (trait objects)
- Custom `impl` when derived behavior isn't enough

---

**Next:** [Module 07 — Strings & String Manipulation →](./07-strings.md)
