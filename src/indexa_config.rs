use crate::Opt;

use serde::Deserialize;
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct IndexaConfig {
    pub flags: FlagConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(default)]
pub struct FlagConfig {
    pub threads: usize,
}

impl Default for FlagConfig {
    fn default() -> Self {
        Self {
            threads: (num_cpus::get() - 1).max(1),
        }
    }
}

impl FlagConfig {
    pub fn merge_opt(&mut self, opt: &Opt) {
        if let Some(threads) = opt.threads {
            self.threads = threads.min(num_cpus::get() - 1).max(1);
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(default)]
pub struct DatabaseConfig {
    pub location: Option<PathBuf>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        let location = dirs::data_dir().map(|data_dir| {
            let mut path = data_dir;
            path.push("indexa");
            path.push("database.db");
            path
        });

        Self { location }
    }
}

pub fn read_config<P>(config_path: Option<P>) -> Result<IndexaConfig, String>
where
    P: AsRef<Path>,
{
    const CONFIG_LOCATION_ERROR_MSG: &str = "Could not determine the location of config file. \
    Please provide the location of config file with -C/--config option.";

    let path = if let Some(path) = config_path.as_ref() {
        Cow::Borrowed(path.as_ref())
    } else if cfg!(windows) {
        let config_dir = dirs::config_dir().ok_or(CONFIG_LOCATION_ERROR_MSG)?;
        let mut path = config_dir;
        path.push("indexa");
        path.push("config.toml");
        Cow::Owned(path)
    } else {
        let home_dir = dirs::home_dir().ok_or(CONFIG_LOCATION_ERROR_MSG)?;
        let mut path = home_dir;
        path.push(".config");
        path.push("indexa");
        path.push("config.toml");
        Cow::Owned(path)
    };

    if let Ok(config_string) = std::fs::read_to_string(&path) {
        toml::from_str(&config_string).map_err(|_| {
            "Invalid config file. Please edit the config file and try again.".to_string()
        })
    } else {
        Err("Failed to read config file".to_string())
    }
}
