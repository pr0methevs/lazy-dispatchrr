# Module 09 — File I/O & Configuration

## Learning Objectives

- Read and write files with `std::fs`
- Work with paths using `PathBuf` and `Path`
- Handle platform-specific config directories
- Build a complete config loading/saving system

---

## 1. File I/O Basics

### Reading Files

```rust
use std::fs;

// Read entire file to string
let contents = fs::read_to_string("config.yml")?;

// Read to bytes
let bytes = fs::read("binary_file.dat")?;
```

### Writing Files

```rust
// Write a string to a file (creates or overwrites)
fs::write("output.yml", "repos:\n  - name: owner/app\n")?;

// Write bytes
fs::write("data.bin", &[0u8, 1, 2, 3])?;
```

### Creating Directories

```rust
// Create a single directory
fs::create_dir("new_dir")?;

// Create all directories in path (like mkdir -p)
fs::create_dir_all("/home/user/.config/myapp")?;
```

### Checking Existence

```rust
use std::path::Path;

if Path::new("config.yml").exists() {
    let contents = fs::read_to_string("config.yml")?;
}
```

## 2. `PathBuf` and `Path`

`PathBuf` is to `Path` what `String` is to `&str`:

| Type | Ownership | Mutability |
|------|-----------|-----------|
| `PathBuf` | Owned | Mutable |
| `&Path` | Borrowed | Immutable |

```rust
use std::path::PathBuf;

// Building paths
let mut path = PathBuf::from("/home/user");
path.push(".config");
path.push("myapp");
path.push("config.yml");
// /home/user/.config/myapp/config.yml

// Joining (fluent)
let path = PathBuf::from("/home/user")
    .join(".config")
    .join("myapp")
    .join("config.yml");

// Getting parent directory
if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
}
```

## 3. Platform-Specific Config Paths

Different OSes have different conventions for config files:

| OS | Config Location |
|----|----------------|
| macOS | `~/.config/` or `~/Library/Application Support/` |
| Linux | `~/.config/` (XDG_CONFIG_HOME) |
| Windows | `%LOCALAPPDATA%` |

### Using the `dirs` Crate

```bash
cargo add dirs
```

```rust
use std::path::PathBuf;

fn config_dir() -> PathBuf {
    dirs::config_dir()           // ~/.config on macOS/Linux, AppData on Windows
        .unwrap_or_else(|| PathBuf::from("."))
}

fn home_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
}
```

> **In our app:** `config.rs` handles all three platforms:
> ```rust
> fn config_path() -> PathBuf {
>     let base = if cfg!(windows) {
>         std::env::var("LOCALAPPDATA")
>             .map(PathBuf::from)
>             .unwrap_or_else(|_| dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")))
>     } else {
>         std::env::var("XDG_CONFIG_HOME")
>             .map(PathBuf::from)
>             .unwrap_or_else(|_| {
>                 dirs::home_dir()
>                     .unwrap_or_else(|| PathBuf::from("~"))
>                     .join(".config")
>             })
>     };
>     base.join("dispatchrr").join("config.yml")
> }
> ```

## 4. The `cfg!` Macro — Compile-Time Platform Detection

```rust
if cfg!(windows) {
    // Windows-specific path logic
} else if cfg!(target_os = "macos") {
    // macOS-specific logic
} else {
    // Linux and others
}
```

### `#[cfg()]` Attribute — Conditional Compilation

```rust
#[cfg(windows)]
fn open_browser(url: &str) {
    std::process::Command::new("cmd").args(["/C", "start", url]).spawn();
}

#[cfg(target_os = "macos")]
fn open_browser(url: &str) {
    std::process::Command::new("open").arg(url).spawn();
}

#[cfg(target_os = "linux")]
fn open_browser(url: &str) {
    std::process::Command::new("xdg-open").arg(url).spawn();
}
```

## 5. Environment Variables

```rust
// Read an environment variable
let value = std::env::var("XDG_CONFIG_HOME");  // Result<String, VarError>

match std::env::var("XDG_CONFIG_HOME") {
    Ok(path) => println!("Config dir: {}", path),
    Err(_) => println!("Not set, using default"),
}

// Compile-time env vars (from Cargo)
let version = env!("CARGO_PKG_VERSION");   // Panics if not set
let name = env!("CARGO_PKG_NAME");
```

> **In our app:** `build.rs` uses `env!("CARGO_PKG_VERSION")` to embed the 
> version number into Windows resource metadata.

## 6. Complete Config System

Here's the full pattern used in our app:

```rust
use std::path::PathBuf;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub repos: Vec<RepoConfig>,
}

fn config_path() -> PathBuf {
    // Platform-specific base directory
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("~"))
                .join(".config")
        });
    base.join("myapp").join("config.yml")
}

pub fn load_config() -> Config {
    let path = config_path();
    if path.exists() {
        let contents = std::fs::read_to_string(&path).unwrap_or_default();
        serde_yaml::from_str(&contents).unwrap_or_default()
    } else {
        Config::default()
    }
}

pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let path = config_path();
    // Create directory if it doesn't exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let yaml = serde_yaml::to_string(config)?;
    std::fs::write(&path, yaml)?;
    Ok(())
}
```

### Key Design Decisions

1. **`load_config` returns `Config`, never errors** — uses `unwrap_or_default()` for graceful degradation
2. **`save_config` returns `Result`** — saving failures should be reported
3. **`create_dir_all` before writing** — first-time users won't have the directory
4. **Separate read/write functions** — load is called on startup, save after mutations

---

## Exercises

1. **Basic file I/O:** Write a program that creates a YAML config file, writes some data, 
   reads it back, and prints it.

2. **PathBuf building:** Write a `config_path()` function that returns:
   - `~/.config/tui-dispatcher/config.yml` on macOS/Linux
   - `%LOCALAPPDATA%/tui-dispatcher/config.yml` on Windows
   Use `cfg!()` or `#[cfg()]` for platform detection.

3. **Load/save cycle:** Implement `load_config()` and `save_config()` for your project.
   Test by saving a config, loading it, modifying it, and saving again.

4. **Graceful degradation:** Make `load_config()` handle these cases without panicking:
   - Config file doesn't exist → return default
   - Config file has invalid YAML → return default
   - Config file is empty → return default

5. **Connect to the app:** Read `src/config.rs` and trace the config lifecycle:
   - Where is `load_config()` called?
   - Where is `save_config()` called?
   - What happens when a user adds a repo?
   - How are replays preserved when saving?

---

## Key Takeaways

- `std::fs` for all file operations (read, write, create dirs)
- `PathBuf` for building paths — use `.join()` for cross-platform separators
- `dirs` crate for standard directories (`config_dir()`, `home_dir()`)
- `cfg!(windows)` for runtime platform checks, `#[cfg(windows)]` for compile-time
- Config should load gracefully (never crash on missing/bad files)
- Config should save atomically (create dirs, then write)

---

**Next:** [Module 10 — External Processes & CLI Integration →](./10-external-processes.md)
