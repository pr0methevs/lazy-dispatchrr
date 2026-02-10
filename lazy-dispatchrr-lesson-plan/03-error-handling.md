# Module 03 — Error Handling

## Learning Objectives

- Understand Rust's error handling philosophy (no exceptions)
- Use the `?` operator for ergonomic error propagation
- Work with `Box<dyn Error>` for flexible error types
- Set up `color-eyre` for beautiful error reports

---

## 1. Rust's Error Philosophy

Rust has **no exceptions**. Errors are values:

- **Recoverable errors:** `Result<T, E>` — the function might fail, caller decides what to do
- **Unrecoverable errors:** `panic!()` — the program crashes (bugs, impossible states)

```rust
// This panics — use for programming errors, not expected failures
panic!("Something went terribly wrong");

// This returns an error — use for expected failures
fn read_file(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}
```

## 2. The `?` Operator

The `?` operator is shorthand for "if this is an error, return it; otherwise, unwrap the Ok":

```rust
// Without ?
fn read_config() -> Result<String, std::io::Error> {
    let contents = match std::fs::read_to_string("config.yml") {
        Ok(c) => c,
        Err(e) => return Err(e),
    };
    Ok(contents)
}

// With ? — much cleaner
fn read_config() -> Result<String, std::io::Error> {
    let contents = std::fs::read_to_string("config.yml")?;
    Ok(contents)
}
```

You can chain `?`:

```rust
fn load_and_parse() -> Result<Config, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string("config.yml")?;   // May fail
    let config: Config = serde_yaml::from_str(&contents)?;     // May fail
    Ok(config)
}
```

## 3. `Box<dyn Error>` — The Universal Error Type

Different operations return different error types (`io::Error`, `serde_yaml::Error`, etc.).
`Box<dyn Error>` accepts any error type:

```rust
use std::error::Error;

fn do_stuff() -> Result<(), Box<dyn Error>> {
    let file = std::fs::read_to_string("data.json")?;   // io::Error
    let data: serde_json::Value = serde_json::from_str(&file)?;  // serde_json::Error
    println!("{}", data);
    Ok(())
}
```

> **In our app:** Almost every method uses this pattern:
> ```rust
> pub fn add_repo(&mut self, owner: &str, name: &str) -> Result<(), Box<dyn std::error::Error>> {
>     let (branches, workflows) = self.github.fetch_repo_details(owner, name)?;
>     // ...
>     Ok(())
> }
> ```

## 4. Creating Errors from Strings

You can convert strings into `Box<dyn Error>` using `.into()`:

```rust
fn validate_repo_name(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = name.splitn(2, '/').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid repo format: '{}'. Expected 'owner/name'.", name).into());
    }
    Ok(())
}
```

> **In our app:** This exact pattern is used in `load_branches()`:
> ```rust
> return Err(format!("Invalid repo format: '{}'. Expected 'owner/name'.", repo_name).into());
> ```

## 5. `unwrap()` and `expect()` — Use Sparingly

```rust
let value = some_result.unwrap();        // Panics with generic message if Err
let value = some_result.expect("msg");   // Panics with your message if Err
```

**When to use them:**
- In tests
- When you've already checked the error can't happen
- During prototyping (replace with proper handling later)

**When NOT to use them:**
- In library code
- In production application code
- When the error is expected (network, file I/O, user input)

> **In our app:** `load_config()` uses `unwrap_or_default()` — if parsing fails,
> just use an empty config. This is intentional graceful degradation.

## 6. `Option` to `Result` Conversion

```rust
let selected = state.repos_state.selected()  // Returns Option<usize>
    .ok_or("No repo selected.")?;            // Converts None → Err
```

> **In our app:** This pattern appears in nearly every action method:
> ```rust
> let selected_repo_idx = self.selected_repo_real_index()
>     .ok_or("No repo selected.")?;
> ```

## 7. `color-eyre` — Beautiful Error Reports

`color-eyre` wraps errors with colored, contextual output including backtraces.

### Setup

Add to `Cargo.toml`:
```toml
[dependencies]
color-eyre = "0.6.5"
```

### Initialize in `main()`:

```rust
use color_eyre::eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;  // Install the error handler
    
    // Now any error returned from main() gets a beautiful report
    let config = load_config()?;
    run_app(config)?;
    
    Ok(())
}
```

> **In our app:** `main.rs` installs `color-eyre` early and uses its `Result` type:
> ```rust
> use color_eyre::eyre::Result;
> 
> fn main() -> Result<()> {
>     color_eyre::install()?;
>     // ...
>     result
> }
> ```

### The `eyre::Result` type

`color_eyre::eyre::Result<T>` is `Result<T, eyre::Report>` — a rich error type that:
- Captures backtraces
- Supports error chaining
- Pretty-prints in the terminal with colors

## 8. Error Handling Patterns Summary

| Pattern | When to Use |
|---------|------------|
| `?` | Propagate errors up to the caller |
| `Box<dyn Error>` | Functions that can fail multiple ways |
| `.ok_or("msg")?` | Convert `Option` → `Result` |
| `.map_err(|e| ...)?` | Transform error type/message |
| `unwrap_or_default()` | Graceful fallback for non-critical operations |
| `if let Err(e) = ...` | Handle error locally without propagating |
| `color_eyre::Result` | App-level error handling with nice output |

> **In our app's event loop**, errors are caught and displayed to the user:
> ```rust
> if let Err(e) = state.add_repo(&owner, &name) {
>     state.ui.output = Some(format!("Error adding repo: {}", e));
>     state.ui.output_is_error = true;
> }
> ```
>
> This is the "catch and display" pattern — the app doesn't crash, it shows the 
> error in the output panel.

---

## Exercises

1. **`?` operator drill:** Write a function `read_and_count(path: &str) -> Result<usize, Box<dyn Error>>`
   that reads a file and returns the number of lines. Use `?` for both file reading and any errors.

2. **Option → Result:** Write a function that takes a `Vec<String>` and an index, and returns
   `Result<&String, String>`. Use `.get()` (returns `Option`) and `.ok_or()`.

3. **Error display pattern:** Write a function that can fail, then call it in `main()` using:
   ```rust
   if let Err(e) = my_function() {
       println!("Error: {}", e);
   }
   ```

4. **Set up color-eyre:** Add `color-eyre` to your `tui-dispatcher` project, install it in `main()`,
   and make `main()` return `Result<()>`.

5. **Connect to the app:** Read `src/event.rs` and find every `if let Err(e) = ...` block.
   Note how the app handles errors — it never crashes, it always shows the error to the user.

---

## Key Takeaways

- Rust uses `Result<T, E>` instead of exceptions — errors are explicit
- The `?` operator makes error propagation concise and readable
- `Box<dyn Error>` is the go-to for functions with multiple error types
- `color-eyre` gives you beautiful error reports with backtraces
- In TUI apps, catch errors and display them — don't crash the UI

---

**Next:** [Module 04 — Modules & Project Structure →](./04-modules-and-structure.md)
