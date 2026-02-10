# Module 10 — External Processes & CLI Integration

## Learning Objectives

- Spawn external processes with `std::process::Command`
- Capture stdout, stderr, and exit status
- Integrate with the GitHub CLI (`gh`)
- Decode base64 content from API responses

---

## 1. `std::process::Command`

Rust can run external programs and capture their output:

```rust
use std::process::Command;

let output = Command::new("echo")
    .arg("Hello, World!")
    .output()?;

println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
println!("success: {}", output.status.success());
```

### Building Complex Commands

```rust
let output = Command::new("gh")
    .args([
        "api", "graphql",
        "-f", &format!("query={}", graphql_query),
        "-F", &format!("owner={}", owner),
        "-F", &format!("name={}", name),
    ])
    .output()?;
```

> **In our app:** `github.rs` builds `gh` commands with complex argument lists
> for GraphQL queries, REST API calls, and workflow dispatches.

## 2. Handling Command Output

### Check for Success

```rust
let output = Command::new("gh")
    .args(["auth", "status"])
    .output()?;

if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(format!("Command failed: {}", stderr.trim()).into());
}
```

### Parse JSON Output

```rust
let output = Command::new("gh")
    .args(["run", "list", "--json", "databaseId,status", "--limit", "1"])
    .output()?;

if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(format!("Failed: {}", stderr.trim()).into());
}

let runs: serde_json::Value = serde_json::from_slice(&output.stdout)?;
let run_id = runs[0]["databaseId"].as_u64().ok_or("No run found")?;
```

> **In our app:** This exact pattern is used in `get_latest_run_logs()`.

## 3. The `gh` CLI — GitHub from the Terminal

Our app wraps `gh` commands. Here are the key ones:

### GraphQL Query — Fetch Repo Details

```rust
let query = "query($owner: String!, $name: String!) {
    repository(owner: $owner, name: $name) {
        refs(refPrefix: \"refs/heads/\", first: 100) {
            nodes { name }
        }
        object(expression: \"HEAD:.github/workflows/\") {
            ... on Tree {
                entries { name }
            }
        }
    }
}";

let output = Command::new("gh")
    .args([
        "api", "graphql",
        "-f", &format!("query={}", query),
        "-F", &format!("owner={}", owner),
        "-F", &format!("name={}", name),
    ])
    .output()?;
```

> This single query fetches both branches AND workflow filenames in one API call.

### REST API — Fetch File Content

```rust
let api_path = format!("repos/{}/contents/.github/workflows/{}", repo_name, filename);
let output = Command::new("gh")
    .args(["api", &api_path, "--jq", ".content"])
    .output()?;
```

### Workflow Dispatch

```rust
let mut args = vec![
    "workflow".to_string(),
    "run".to_string(),
    workflow_filename.to_string(),
    "--repo".to_string(),
    repo_name.to_string(),
    "--ref".to_string(),
    branch.to_string(),
];

for field in inputs {
    if !field.value.is_empty() {
        args.push("-f".to_string());
        args.push(format!("{}={}", field.name, field.value));
    }
}

let output = Command::new("gh").args(&args).output()?;
```

### Run Logs

```rust
let output = Command::new("gh")
    .args(["run", "view", &run_id.to_string(), "--repo", repo_name, "--log"])
    .output()?;
```

## 4. Base64 Decoding

GitHub's API returns file contents as base64-encoded strings. We need to decode them:

```bash
cargo add base64
```

```rust
use base64::Engine;

// GitHub returns base64 with newlines
let raw_b64 = String::from_utf8_lossy(&output.stdout)
    .replace('\n', "")
    .replace('\r', "");

let decoded_bytes = base64::engine::general_purpose::STANDARD
    .decode(&raw_b64)
    .map_err(|e| format!("Base64 decode error: {}", e))?;

let content = String::from_utf8_lossy(&decoded_bytes);
```

> **In our app:** This decodes workflow YAML files fetched from the GitHub API,
> which are then parsed with `serde_yaml` to extract input definitions.

## 5. Opening a Browser

```rust
pub fn open_in_browser(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    Command::new("open").arg(url).spawn()?;

    #[cfg(target_os = "linux")]
    Command::new("xdg-open").arg(url).spawn()?;

    #[cfg(target_os = "windows")]
    Command::new("cmd").args(["/C", "start", url]).spawn()?;

    Ok(())
}
```

> **In our app:** `open_repo_in_browser()` and `open_run_in_browser()` use 
> `Command::new("open")` on macOS.

## 6. `.output()` vs `.spawn()`

| Method | Behavior | Use Case |
|--------|----------|----------|
| `.output()` | Runs to completion, captures stdout/stderr | API calls, CLI commands |
| `.spawn()` | Starts process, returns immediately | Opening browser, background tasks |
| `.status()` | Runs to completion, returns exit code only | Simple success/failure checks |

```rust
// .output() — wait and capture everything
let output = Command::new("gh").args(["..."])
    .output()?;

// .spawn() — fire and forget
Command::new("open").arg("https://github.com")
    .spawn()?;
```

## 7. Building the GitHub Service

Putting it all together in a service struct:

```rust
use crate::domain::InputField;

#[derive(Debug, Default)]
pub struct GitHubService;

impl GitHubService {
    pub fn new() -> Self {
        Self
    }

    pub fn fetch_repo_details(&self, owner: &str, name: &str) 
        -> Result<(Vec<String>, Vec<String>), Box<dyn std::error::Error>> 
    {
        // 1. Build GraphQL query
        // 2. Run `gh api graphql` command
        // 3. Check for errors
        // 4. Parse JSON response
        // 5. Extract branches and workflows
        // 6. Return (branches, workflows)
        todo!()
    }

    pub fn fetch_workflow_inputs(&self, repo: &str, workflow: &str)
        -> Result<(Vec<String>, Vec<InputField>), Box<dyn std::error::Error>>
    {
        // 1. Fetch workflow file via REST API
        // 2. Decode base64 content
        // 3. Parse YAML
        // 4. Extract workflow_dispatch inputs
        // 5. Build InputField structs
        todo!()
    }

    pub fn dispatch_workflow(&self, repo: &str, branch: &str, workflow: &str, inputs: &[InputField])
        -> Result<(Vec<String>, String), Box<dyn std::error::Error>>
    {
        // 1. Build args list
        // 2. Run `gh workflow run` command
        // 3. Check for errors
        // 4. Return (args, preview_string)
        todo!()
    }
}
```

---

## Exercises

1. **Basic command:** Run `gh auth status` from Rust and print whether the user is authenticated.

2. **Capture JSON:** Run `gh repo list --json name --limit 3` and parse the output to extract 
   repo names into a `Vec<String>`.

3. **GraphQL query:** Use the GraphQL query from the app to fetch branches for one of your repos.
   Extract the branch names from the JSON response.

4. **Base64 decode:** Fetch a workflow file's base64 content via `gh api` and decode it.
   Print the decoded YAML.

5. **Build the service:** Create a `GitHubService` struct with `fetch_repo_details()`.
   Start with just fetching branches via GraphQL. Test it against one of your own repos.

6. **Connect to the app:** Read `src/service/github.rs` end-to-end. For each method, 
   trace:
   - What `gh` command is built
   - How the output is parsed
   - What's returned to the caller

---

## Key Takeaways

- `Command::new("program").args([...]).output()?` — run and capture
- Always check `output.status.success()` before parsing stdout
- `String::from_utf8_lossy()` for converting process output to strings
- `serde_json::from_slice()` to parse JSON directly from bytes
- `base64::decode()` for GitHub's encoded file contents
- Use `.spawn()` for fire-and-forget (browser opening), `.output()` for everything else

---

**Next:** [Module 11 — Domain Modeling →](./11-domain-modeling.md)
