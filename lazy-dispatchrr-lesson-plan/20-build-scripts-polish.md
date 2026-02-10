# Module 20 — Build Scripts & Polish

## Learning Objectives

- Understand and write `build.rs` build scripts
- Use conditional compilation for platform-specific code
- Add proper Cargo.toml metadata
- Polish the application for release

---

## 1. Build Scripts (`build.rs`)

A `build.rs` file at the project root runs **before** your code compiles. It's used for:

- Generating code
- Embedding resources (icons, version info)
- Running native build tools
- Setting environment variables

```rust
// build.rs
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // Your build logic here
}
```

> **In our app:** `build.rs` embeds Windows resource metadata (product name, version, etc.):

```rust
fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set("ProductName", "Lazy-Dispatchrr");
        res.set("FileDescription", "A TUI app for dispatching GitHub Workflows");
        res.set("LegalCopyright", "Copyright © 2026 Artur Kaminski");
        res.set("CompanyName", "homelab-core");
        res.set("FileVersion", env!("CARGO_PKG_VERSION"));
        res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        res.compile().expect("Failed to compile Windows resources");
    }
}
```

### Key Concepts

- `#[cfg(windows)]` — only runs on Windows (skipped on macOS/Linux)
- `env!("CARGO_PKG_VERSION")` — reads version from `Cargo.toml` at compile time
- Platform-specific build dependencies go in special sections:

```toml
[target.'cfg(windows)'.build-dependencies]
winresource = "0.1"
```

## 2. Conditional Compilation

Rust's `cfg` system lets you compile different code for different platforms:

### `#[cfg()]` Attribute — Compile-Time

```rust
#[cfg(windows)]
fn open_browser(url: &str) {
    std::process::Command::new("cmd").args(["/C", "start", url]).spawn();
}

#[cfg(not(windows))]
fn open_browser(url: &str) {
    std::process::Command::new("open").arg(url).spawn();
}
```

### `cfg!()` Macro — Runtime Check (Compiled-In)

```rust
fn config_path() -> PathBuf {
    if cfg!(windows) {
        // Windows path logic
    } else {
        // Unix path logic
    }
}
```

### Common `cfg` Predicates

| Predicate | Matches |
|-----------|---------|
| `cfg(windows)` | Windows |
| `cfg(unix)` | macOS and Linux |
| `cfg(target_os = "macos")` | macOS only |
| `cfg(target_os = "linux")` | Linux only |
| `cfg(debug_assertions)` | Debug builds |
| `cfg(not(windows))` | Everything except Windows |

## 3. Cargo.toml Metadata

A well-configured `Cargo.toml` communicates what your project is:

```toml
[package]
name = "lazy-dispatchr"
version = "0.1.0"
edition = "2024"
authors = ["Your Name <email@example.com>"]
description = "A TUI app for dispatching GitHub Workflows"
license = "GPL-3.0-only"
repository = "https://github.com/user/repo"
readme = "README.md"
keywords = ["tui", "gha", "github", "workflows"]
categories = ["command-line-utilities"]
```

### Fields Explained

| Field | Purpose |
|-------|---------|
| `name` | Binary/crate name |
| `version` | Semantic versioning |
| `edition` | Rust edition (determines language features) |
| `authors` | Creator(s) |
| `description` | One-line summary (for crates.io) |
| `license` | SPDX license identifier |
| `repository` | Source code URL |
| `keywords` | Up to 5 search keywords |
| `categories` | crates.io categories |

### Dependencies Section

```toml
[dependencies]
base64 = "0.22.1"              # Simple version
serde = { version = "1.0", features = ["derive"] }  # With features
ratatui = "0.30.0"
```

## 4. Release Builds

```bash
# Debug build (fast compile, slow runtime, includes debug symbols)
cargo build

# Release build (slow compile, fast runtime, optimized)
cargo build --release

# The binary is at:
# Debug:   target/debug/lazy-dispatchr
# Release: target/release/lazy-dispatchr
```

### Debug vs Release

| Aspect | Debug | Release |
|--------|-------|---------|
| Compile time | Fast | Slow |
| Runtime speed | Slow | Fast |
| Binary size | Large | Small |
| Debug symbols | Yes | No (by default) |
| Optimizations | None | Full (`opt-level = 3`) |

## 5. Code Quality Tools

### Clippy — The Linter

```bash
cargo clippy
```

Clippy catches common mistakes and suggests idiomatic improvements:
- Unnecessary clones
- Redundant closures
- Better iterator usage
- And hundreds more

### Rustfmt — The Formatter

```bash
cargo fmt
```

Ensures consistent code style across the project.

### Documentation

```bash
cargo doc --open
```

Generate HTML documentation from your doc comments:

```rust
/// Fetch a repo's branches and workflow file names via `gh api graphql`.
///
/// # Errors
/// Returns an error if the `gh` CLI is not authenticated or the repo doesn't exist.
pub fn fetch_repo_details(&self, owner: &str, name: &str) 
    -> Result<(Vec<String>, Vec<String>), Box<dyn std::error::Error>> 
{
    // ...
}
```

## 6. Error Messages and UX Polish

### User-Facing Error Messages

```rust
// Bad: technical error message
return Err(format!("serde_yaml::Error: ...").into());

// Good: user-friendly with context
return Err(format!(
    "Invalid repo format: '{}'. Expected 'owner/name'.",
    repo_name
).into());
```

### Welcome Message

```rust
let output = if has_repos {
    "Ready to dispatch workflows...\n\nSelect a repo and press Enter.\nPress 'a' to add a new repo, '?' for all keybindings."
} else {
    "Welcome to Lazy-Dispatchrr!\n\nPress 'a' to add a repo, '?' for all keybindings."
};
```

### Status Bar

```rust
let help_text = "Tab: focus | j/k: nav | /: search | r: replays | ?: help | q: quit";
```

## 7. README

A good README has:

1. **Title and badges** — project name, language, license
2. **Features** — bullet list of what it does
3. **Prerequisites** — what the user needs installed
4. **Installation** — how to build/install
5. **Usage** — how to run
6. **Keybindings** — reference table
7. **Application flow** — how it works (diagrams help)

> **In our app:** The README includes all of these plus a mermaid flowchart diagram.

---

## Exercises

1. **Build script:** Create a `build.rs` that prints the compile time. Use 
   `println!("cargo:rustc-env=BUILD_TIME={}", chrono::Utc::now())` and read it 
   with `env!("BUILD_TIME")` in your app.

2. **Conditional compilation:** Write a function that opens a URL in the browser, 
   with platform-specific implementations for macOS, Linux, and Windows.

3. **Cargo metadata:** Fill in all the metadata fields in your `Cargo.toml`.
   Run `cargo package --list` to see what would be published.

4. **Code quality:** Run `cargo clippy` on your project. Fix all warnings.
   Run `cargo fmt` to format your code.

5. **Release build:** Build your app in release mode. Compare the binary size 
   with the debug build.

6. **Connect to the app:** Read `build.rs`, `Cargo.toml`, and `README.md` in the 
   actual project. Note:
   - What the build script does on Windows
   - What metadata is in Cargo.toml
   - How the README documents the app

---

## Key Takeaways

- `build.rs` runs before compilation — for code generation and resource embedding
- `#[cfg(platform)]` for compile-time platform selection
- `cfg!()` macro for runtime platform checks
- Full `Cargo.toml` metadata for proper project identification
- `cargo clippy` and `cargo fmt` for code quality
- Release builds for distribution (`cargo build --release`)
- Good UX: friendly errors, welcome messages, help text, README

---

**Next:** [Module 21 — Capstone: Build Your Own →](./21-capstone.md)
