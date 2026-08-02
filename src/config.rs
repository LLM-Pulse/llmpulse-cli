use std::{
    env, fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::{CliError, Result};

pub const DEFAULT_BASE_URL: &str = "https://api.llmpulse.ai/api/v1";

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_project_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_profile: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Overrides {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub project_id: Option<u64>,
    pub verbose: bool,
}

#[derive(Clone, Debug)]
pub struct ResolvedConfig {
    pub api_key: String,
    pub base_url: String,
    pub project_id: Option<u64>,
    pub verbose: bool,
}

fn config_dir() -> Result<PathBuf> {
    if let Some(path) = env::var_os("LLMPULSE_CONFIG_DIR") {
        return Ok(PathBuf::from(path));
    }
    let home = dirs::home_dir().ok_or_else(|| {
        CliError::Config("Could not find the current user's home directory".into())
    })?;
    Ok(home.join(".llmpulse"))
}

fn config_file() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.json"))
}

fn profiles_dir() -> Result<PathBuf> {
    Ok(config_dir()?.join("profiles"))
}

fn profile_path(name: &str) -> Result<PathBuf> {
    validate_profile_name(name)?;
    Ok(profiles_dir()?.join(format!("{name}.json")))
}

fn validate_profile_name(name: &str) -> Result<()> {
    let valid = !name.is_empty()
        && name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'));
    if valid {
        Ok(())
    } else {
        Err(CliError::Usage(
            "Profile names may contain letters, numbers, hyphens, and underscores".into(),
        ))
    }
}

fn read_file(path: &Path) -> Result<Config> {
    if !path.exists() {
        return Ok(Config::default());
    }
    let contents = fs::read_to_string(path)?;
    serde_json::from_str(&contents)
        .map_err(|error| CliError::Config(format!("Could not parse {}: {error}", path.display())))
}

fn ensure_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    set_directory_permissions(path)?;
    Ok(())
}

fn write_file(path: &Path, config: &Config) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }
    let contents = serde_json::to_string_pretty(config)? + "\n";
    fs::write(path, contents)?;
    set_file_permissions(path)?;
    Ok(())
}

#[cfg(unix)]
fn set_file_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_file_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn set_directory_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_directory_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

pub fn read_raw_config() -> Result<Config> {
    read_file(&config_file()?)
}

pub fn read_config() -> Result<Config> {
    let mut base = read_raw_config()?;
    let Some(profile_name) = base.active_profile.clone() else {
        return Ok(base);
    };
    let profile = read_file(&profile_path(&profile_name)?)?;
    if profile.api_key.is_some() {
        base.api_key = profile.api_key;
    }
    if profile.base_url.is_some() {
        base.base_url = profile.base_url;
    }
    if profile.default_project_id.is_some() {
        base.default_project_id = profile.default_project_id;
    }
    Ok(base)
}

pub fn write_config(config: &Config) -> Result<()> {
    write_file(&config_file()?, config)
}

pub fn set_value(key: &str, value: &str) -> Result<()> {
    if !matches!(key, "api_key" | "base_url" | "default_project_id") {
        return Err(CliError::Usage(format!(
            "Unknown config key '{key}'. Use api_key, base_url, or default_project_id"
        )));
    }

    let mut base = read_raw_config()?;
    if let Some(profile_name) = base.active_profile.clone() {
        let path = profile_path(&profile_name)?;
        let mut profile = read_file(&path)?;
        assign_value(&mut profile, key, value)?;
        return write_file(&path, &profile);
    }

    assign_value(&mut base, key, value)?;
    write_config(&base)
}

fn assign_value(config: &mut Config, key: &str, value: &str) -> Result<()> {
    match key {
        "api_key" => config.api_key = Some(value.to_owned()),
        "base_url" => config.base_url = Some(validate_base_url(value)?),
        "default_project_id" => {
            config.default_project_id = Some(value.parse().map_err(|_| {
                CliError::Usage("default_project_id must be a positive integer".into())
            })?)
        }
        _ => unreachable!(),
    }
    Ok(())
}

pub fn get_value(key: &str) -> Result<Option<String>> {
    let config = read_config()?;
    let value = match key {
        "api_key" => config.api_key,
        "base_url" => config.base_url,
        "default_project_id" => config.default_project_id.map(|id| id.to_string()),
        "active_profile" => config.active_profile,
        _ => {
            return Err(CliError::Usage(format!(
                "Unknown config key '{key}'. Use api_key, base_url, default_project_id, or active_profile"
            )));
        }
    };
    Ok(value)
}

pub fn set_active_profile(name: &str) -> Result<()> {
    validate_profile_name(name)?;
    let mut config = read_raw_config()?;
    config.active_profile = Some(name.to_owned());
    let path = profile_path(name)?;
    if !path.exists() {
        write_file(&path, &Config::default())?;
    }
    write_config(&config)
}

pub fn list_profiles() -> Result<Vec<String>> {
    let directory = profiles_dir()?;
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut profiles = fs::read_dir(directory)?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            (path.extension()?.to_str()? == "json")
                .then(|| path.file_stem()?.to_str().map(str::to_owned))
                .flatten()
        })
        .collect::<Vec<_>>();
    profiles.sort();
    Ok(profiles)
}

pub fn active_profile() -> Result<Option<String>> {
    Ok(read_raw_config()?.active_profile)
}

pub fn validate_base_url(raw: &str) -> Result<String> {
    let parsed =
        Url::parse(raw).map_err(|_| CliError::Config(format!("Invalid base URL: {raw}")))?;
    let localhost = matches!(parsed.host_str(), Some("localhost" | "127.0.0.1" | "::1"));
    if parsed.scheme() != "https" && !(parsed.scheme() == "http" && localhost) {
        return Err(CliError::Config(format!(
            "Refusing to use insecure base URL '{raw}'. Use https://. HTTP is allowed only for localhost."
        )));
    }
    Ok(raw.trim_end_matches('/').to_owned())
}

pub fn resolve(overrides: Overrides) -> Result<ResolvedConfig> {
    let config = read_config()?;
    let api_key = overrides
        .api_key
        .or_else(|| env::var("LLMPULSE_API_KEY").ok())
        .or(config.api_key)
        .unwrap_or_default();
    let base_url = overrides
        .base_url
        .or_else(|| env::var("LLMPULSE_BASE_URL").ok())
        .or(config.base_url)
        .unwrap_or_else(|| DEFAULT_BASE_URL.to_owned());
    let project_id = overrides
        .project_id
        .or_else(|| {
            env::var("LLMPULSE_PROJECT_ID")
                .ok()
                .and_then(|id| id.parse().ok())
        })
        .or(config.default_project_id);

    Ok(ResolvedConfig {
        api_key,
        base_url: validate_base_url(&base_url)?,
        project_id,
        verbose: overrides.verbose,
    })
}

pub fn mask_api_key(key: &str) -> String {
    if key.len() < 15 {
        return "***".into();
    }
    format!("{}***{}", &key[..12], &key[key.len() - 3..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_secure_and_local_urls() {
        assert!(validate_base_url("https://api.example.com/v1").is_ok());
        assert!(validate_base_url("http://127.0.0.1:3000/api/v1").is_ok());
        assert!(validate_base_url("http://api.example.com/v1").is_err());
        assert!(validate_base_url("not a url").is_err());
    }

    #[test]
    fn masks_api_keys() {
        assert_eq!(mask_api_key("short"), "***");
        assert_eq!(mask_api_key("llmpulse_abcdefghijk"), "llmpulse_abc***ijk");
    }

    #[test]
    fn rejects_unsafe_profile_names() {
        assert!(validate_profile_name("production").is_ok());
        assert!(validate_profile_name("team-1_us").is_ok());
        assert!(validate_profile_name("../secrets").is_err());
    }
}
