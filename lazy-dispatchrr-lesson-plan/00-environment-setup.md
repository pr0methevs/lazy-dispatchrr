# Module 00 — Environment Setup

## Learning Objectives

- Install Rust and Cargo
- Understand the Rust toolchain
- Create and run your first Rust project
- Configure your editor for Rust development

---

## 1. Install Rust

Rust is installed via **rustup**, the official toolchain manager.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After installation, verify:

```bash
rustc --version    # Rust compiler
cargo --version    # Package manager & build tool
rustup --version   # Toolchain manager
```

> **Windows users:** Download from [rustup.rs](https://rustup.rs/) and follow the installer.

## 2. Understanding the Toolchain

| Tool | Purpose |
|------|---------|
| `rustc` | The Rust compiler — you rarely call it directly |
| `cargo` | Build system + package manager (this is your main tool) |
| `rustup` | Manages Rust versions and components |
| `rust-analyzer` | Language server for editor integration |
| `clippy` | Linter for idiomatic Rust |
| `rustfmt` | Auto-formatter |

Install useful components:

```bash
rustup component add clippy rustfmt
```

## 3. Editor Setup

### VS Code (Recommended)

Install these extensions:
- **rust-analyzer** — intelligent code completion, inline errors, go-to-definition
- **Even Better TOML** — syntax highlighting for `Cargo.toml`
- **Error Lens** — inline error display (optional but helpful)

### Settings to add (`.vscode/settings.json`):

```json
{
  "rust-analyzer.check.command": "clippy",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

## 4. Create Your First Project

```bash
cargo new hello-rust
cd hello-rust
```

This creates:

```
hello-rust/
├── Cargo.toml      # Project manifest (name, version, dependencies)
├── src/
│   └── main.rs     # Entry point
```

### Look at `Cargo.toml`

```toml
[package]
name = "hello-rust"
version = "0.1.0"
edition = "2021"

[dependencies]
```

> **Note:** Our target app uses `edition = "2024"`. Editions are Rust's way of
> introducing breaking changes without breaking old code. We'll use whatever
> `cargo new` gives you — the differences are minor for learning.

### Look at `src/main.rs`

```rust
fn main() {
    println!("Hello, world!");
}
```

### Build and run:

```bash
cargo run          # Compiles and runs
cargo build        # Compiles only
cargo check        # Type-checks without generating binary (fastest)
```

## 5. Cargo Commands You'll Use Daily

| Command | Purpose |
|---------|---------|
| `cargo new <name>` | Create a new project |
| `cargo run` | Build and run |
| `cargo build` | Build only |
| `cargo build --release` | Optimized build |
| `cargo check` | Fast type-check (no binary) |
| `cargo clippy` | Run the linter |
| `cargo fmt` | Format your code |
| `cargo add <crate>` | Add a dependency |
| `cargo doc --open` | Generate and open documentation |

## 6. Install GitHub CLI

Our project interacts with GitHub. Install the `gh` CLI:

```bash
# macOS
brew install gh

# Linux (Debian/Ubuntu)
sudo apt install gh

# Windows
winget install GitHub.cli
```

Then authenticate:

```bash
gh auth login
```

Verify it works:

```bash
gh auth status
gh repo list --limit 3
```

---

## Exercises

1. **Create a new project** called `tui-dispatcher` — this will be your capstone project
2. **Run `cargo run`** and verify you see "Hello, world!"
3. **Modify `main.rs`** to print your name and run it again
4. **Run `cargo clippy`** and `cargo fmt` — get used to running these
5. **Verify `gh` works** — run `gh repo list` and confirm you see your repos
6. **Look at the real project's `Cargo.toml`** — compare it to yours. Note the `edition`, `dependencies`, and metadata fields

---

## Key Takeaways

- `cargo` is your primary tool — you almost never call `rustc` directly
- `cargo check` is your fastest feedback loop while learning
- The `Cargo.toml` file is like `package.json` (Node) or `pyproject.toml` (Python)
- Dependencies are called "crates" in Rust — they come from [crates.io](https://crates.io)

---

**Next:** [Module 01 — Rust Fundamentals: Ownership & Types →](./01-rust-fundamentals.md)
