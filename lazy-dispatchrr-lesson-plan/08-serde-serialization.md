# Module 08 — Serialization with Serde

## Learning Objectives

- Understand serialization/deserialization concepts
- Use `serde` with derive macros for JSON and YAML
- Handle dynamic/untyped data with `serde_json::Value` and `serde_yaml::Value`
- Use serde attributes (`#[serde(default)]`, `#[serde(rename)]`, etc.)

---

## 1. What Is Serde?

**Serde** (Serialize + Deserialize) is Rust's universal serialization framework. It lets you
convert Rust structs to/from formats like JSON, YAML, TOML, and more.

```
Rust Struct  ←→  serde  ←→  JSON / YAML / TOML / ...
```

### Adding Serde to Your Project

```bash
cargo add serde --features derive
cargo add serde_json
cargo add serde_yaml
```

This adds to `Cargo.toml`:
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
```

## 2. Basic Serialization / Deserialization

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    repos: Vec<RepoConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
struct RepoConfig {
    name: String,
}
```

### To JSON

```rust
let config = Config {
    repos: vec![
        RepoConfig { name: "owner/app".to_string() },
    ],
};

// Serialize to JSON string
let json = serde_json::to_string(&config)?;
// {"repos":[{"name":"owner/app"}]}

let pretty_json = serde_json::to_string_pretty(&config)?;

// Deserialize from JSON
let parsed: Config = serde_json::from_str(&json)?;
```

### To YAML

```rust
// Serialize to YAML
let yaml = serde_yaml::to_string(&config)?;
// repos:
// - name: owner/app

// Deserialize from YAML
let parsed: Config = serde_yaml::from_str(&yaml)?;
```

> **In our app:** Config is stored as YAML at `~/.config/dispatchrr/config.yml`:
> ```rust
> pub fn load_config() -> Config {
>     let contents = std::fs::read_to_string(&path).unwrap_or_default();
>     serde_yaml::from_str(&contents).unwrap_or_default()
> }
> 
> pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
>     let yaml = serde_yaml::to_string(config)?;
>     std::fs::write(&path, yaml)?;
>     Ok(())
> }
> ```

## 3. Serde Attributes

### `#[serde(default)]` — Use Default When Missing

```rust
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    #[serde(default)]       // If "repos" key is missing in YAML, use vec![]
    pub repos: Vec<RepoConfig>,
}
```

> **In our app:** `Config` uses `#[serde(default)]` on the `repos` field so that
> an empty config file doesn't cause a parse error.

### `#[serde(rename)]` — Different Field Names

```rust
#[derive(Deserialize)]
struct Input {
    #[serde(rename = "type")]     // YAML key is "type", Rust field is "input_type"
    input_type: Option<InputType>,
}
```

### `#[serde(rename_all)]` — Case Conversion

```rust
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum InputType {
    String,      // matches "string" in YAML
    Boolean,     // matches "boolean"
    Choice,      // matches "choice"
    Environment, // matches "environment"
}
```

## 4. Dynamic JSON with `serde_json::Value`

When you don't know the exact structure upfront, use `Value`:

```rust
let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;

// Access nested fields with indexing
let repo = &json["data"]["repository"];
let branch_name = json["data"]["repository"]["refs"]["nodes"][0]["name"]
    .as_str()
    .unwrap_or("unknown");

// Check if a value is null
if repo.is_null() {
    return Err("Repository not found".into());
}

// Extract arrays
let branches: Vec<String> = repo["refs"]["nodes"]
    .as_array()
    .map(|nodes| {
        nodes.iter()
            .filter_map(|n| n["name"].as_str().map(String::from))
            .collect()
    })
    .unwrap_or_default();
```

> **In our app:** `github.rs` parses GraphQL responses as dynamic `serde_json::Value`
> because the GitHub API returns complex, nested JSON.

### Key `Value` Methods

```rust
let val: serde_json::Value = /* ... */;

val.as_str()     // Option<&str>
val.as_u64()     // Option<u64>
val.as_bool()    // Option<bool>
val.as_array()   // Option<&Vec<Value>>
val.as_object()  // Option<&Map<String, Value>>
val.is_null()    // bool
val[key]         // Index access (returns Null for missing keys)
```

## 5. Dynamic YAML with `serde_yaml::Value`

Similar to JSON, but for YAML:

```rust
let yaml_value: serde_yaml::Value = serde_yaml::from_str(&yaml_str)?;

// Access YAML mappings
if let Some(inputs_map) = yaml_value["on"]["workflow_dispatch"]["inputs"].as_mapping() {
    for (key, val) in inputs_map {
        let name = key.as_str().unwrap_or("unknown");
        let required = val["required"].as_bool().unwrap_or(false);
        let input_type = val["type"].as_str().unwrap_or("string");
        
        // Handle different YAML value types
        let default_value = match &val["default"] {
            serde_yaml::Value::String(s) => s.clone(),
            serde_yaml::Value::Bool(b) => b.to_string(),
            serde_yaml::Value::Number(n) => n.to_string(),
            _ => String::new(),
        };
    }
}
```

> **In our app:** `github.rs` parses workflow YAML files to extract `workflow_dispatch` 
> inputs, handling different types (string, bool, number, choice, environment).

### YAML Sequences

```rust
let options: Vec<String> = val["options"]
    .as_sequence()
    .map(|opts| {
        opts.iter()
            .filter_map(|o| o.as_str().map(String::from))
            .collect()
    })
    .unwrap_or_default();
```

## 6. Full Config Example from Our App

```rust
#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub repos: Vec<RepoConfig>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RepoConfig {
    pub name: String,
    #[serde(default)]
    pub replays: Vec<ReplayConfig>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ReplayConfig {
    pub workflow: String,
    pub description: String,
    pub inputs: Vec<ReplayInput>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ReplayInput {
    pub name: String,
    pub value: String,
}
```

This serializes to YAML like:
```yaml
repos:
  - name: owner/app
    replays:
      - workflow: deploy.yml
        description: "env=prod, version=1.0"
        inputs:
          - name: env
            value: prod
          - name: version
            value: "1.0"
```

---

## Exercises

1. **Basic serde:** Create a `Config` struct with `Serialize` and `Deserialize`.
   Serialize it to YAML, write to a file, read it back, and deserialize it.

2. **Dynamic JSON:** Parse this JSON string and extract the branch names:
   ```json
   {"data":{"repository":{"refs":{"nodes":[{"name":"main"},{"name":"dev"}]}}}}
   ```

3. **YAML value types:** Write a function that takes a `serde_yaml::Value` and extracts 
   a "default" field, handling `String`, `Bool`, and `Number` variants.

4. **Serde attributes:** Create a struct with `#[serde(default)]` on a field. Deserialize 
   YAML that's missing that field and verify it uses the default.

5. **Connect to the app:** Read `src/config.rs` and `src/service/github.rs`. Trace the 
   complete flow of:
   - Config: YAML file → `Config` struct → modify → YAML file
   - GitHub API: JSON response → `serde_json::Value` → extract fields → domain types

---

## Key Takeaways

- `serde` provides `Serialize` and `Deserialize` traits
- `#[derive(Serialize, Deserialize)]` auto-generates the code
- `serde_json` for JSON, `serde_yaml` for YAML
- Use typed structs when the format is known and fixed
- Use `Value` (dynamic) when the structure varies or is complex
- `#[serde(default)]` prevents parse errors for missing fields
- `as_str()`, `as_bool()`, `as_array()` etc. for safe access to dynamic values

---

**Next:** [Module 09 — File I/O & Configuration →](./09-file-io-config.md)
