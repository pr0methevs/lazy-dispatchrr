use std::path::PathBuf;

/// Serializable config format for ~/.config/dispatchrr/config.yml
#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub repos: Vec<RepoConfig>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RepoConfig {
    pub name: String, // "owner/repo"
    #[serde(default)]
    pub replays: Vec<ReplayConfig>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ReplayConfig {
    pub workflow: String,          // workflow filename e.g. "deploy.yml"
    pub description: String,       // auto-generated from inputs e.g. "env=prod, version=1.0"
    pub inputs: Vec<ReplayInput>,  // saved input key=value pairs
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ReplayInput {
    pub name: String,
    pub value: String,
}

fn config_path() -> PathBuf {
    let base = if cfg!(windows) {
        // %LOCALAPPDATA% on Windows
        std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")))
    } else {
        // ~/.config on macOS and Linux (respects XDG_CONFIG_HOME if set)
        std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("~"))
                    .join(".config")
            })
    };
    base.join("dispatchrr").join("config.yml")
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
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let yaml = serde_yaml::to_string(config)?;
    std::fs::write(&path, yaml)?;
    Ok(())
}
