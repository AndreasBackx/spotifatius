use std::{collections::HashMap, env, fs::File, io::Read, path::PathBuf};

use anyhow::{Context, Error, Result};
use serde::Deserialize;
use tracing::{debug, warn};

pub const DEFAULT_CONFIG_FOLDER: &str = "~/.config/spotifatius";
pub const DEFAULT_CONFIG_PATH: &str = "~/.config/spotifatius/config.toml";

#[derive(Clone, Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default = "default_polybar_config")]
    pub polybar: PolybarConfig,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct PolybarConfig {
    #[serde(default)]
    pub colors: HashMap<String, String>,
}

fn default_polybar_config() -> PolybarConfig {
    PolybarConfig {
        colors: HashMap::new(),
    }
}

pub fn resolve_home_path(path: PathBuf) -> Result<PathBuf> {
    if path.starts_with("~/") {
        let home_path = env::var_os("HOME")
            .context("could not find the $HOME env var to use for the default config path")?;
        let relative_path = path
            .to_str()
            .with_context(|| {
                format!("invalid path specified: {}", path.display())
            })?
            .to_string()
            .split_off(2);
        Ok(PathBuf::from(home_path).join(relative_path))
    } else {
        Ok(path)
    }
}

pub fn get_config(config_path: PathBuf) -> Result<Config> {
    let path = resolve_home_path(config_path)?;
    debug!("Using config: {}", path.display());

    let config_file = File::open(path.clone())
        .with_context(|| format!("could not open the file {}", path.display()));

    match config_file {
        Ok(mut config) => {
            let mut config_content = String::new();
            config.read_to_string(&mut config_content)?;
            toml::from_str::<Config>(config_content.as_str())
                .map_err(|error| -> Error { error.into() })
                .with_context(|| format!("could not parse {}", path.display()))
        }
        Err(err) => {
            warn!("{err}");
            Ok(Config::default())
        }
    }
}
