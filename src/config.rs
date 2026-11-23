use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};
use std::{fs, io};

use adw::glib;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

impl Theme {
    pub const ALL: [Theme; 3] = [Theme::System, Theme::Light, Theme::Dark];

    pub fn label(self) -> &'static str {
        match self {
            Theme::System => "System",
            Theme::Light => "Light",
            Theme::Dark => "Dark",
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub frosted_glass: bool,
    pub theme: Theme,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frosted_glass: true,
            theme: Theme::default(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = path();
        Self::read(&path).unwrap_or_else(|error| {
            if error.kind() != io::ErrorKind::NotFound {
                eprintln!("ignoring {}: {error}", path.display());
            }
            Self::default()
        })
    }

    pub fn save(&self) {
        let path = path();
        if let Err(error) = self.write(&path) {
            eprintln!("failed to save {}: {error}", path.display());
        }
    }

    fn read(path: &Path) -> io::Result<Self> {
        Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
    }

    fn write(&self, path: &Path) -> io::Result<()> {
        if let Some(dir) = path.parent() {
            fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(dir)?;
        }
        let mut json = serde_json::to_string_pretty(self)?;
        json.push('\n');
        fs::write(path, json)
    }
}

fn path() -> PathBuf {
    glib::user_config_dir()
        .join(env!("CARGO_PKG_NAME"))
        .join("config.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_dir(test: &str) -> PathBuf {
        let name = format!("{}-{}-{test}", env!("CARGO_PKG_NAME"), std::process::id());
        let dir = std::env::temp_dir().join(name);
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn a_saved_config_is_read_back() {
        let dir = scratch_dir("round-trip");
        let path = dir.join("missing/directories/config.json");
        let config = Config {
            frosted_glass: false,
            theme: Theme::Dark,
        };

        config.write(&path).unwrap();
        assert_eq!(Config::read(&path).unwrap(), config);

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_missing_file_is_reported_as_not_found() {
        let path = scratch_dir("missing").join("config.json");
        assert_eq!(
            Config::read(&path).unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
    }

    #[test]
    fn settings_missing_from_the_file_keep_their_defaults() {
        assert_eq!(
            serde_json::from_str::<Config>("{}").unwrap(),
            Config::default()
        );
    }

    #[test]
    fn settings_this_version_does_not_know_are_ignored() {
        let config: Config =
            serde_json::from_str(r#"{"frosted_glass": false, "newer": 1}"#).unwrap();
        assert!(!config.frosted_glass);
    }

    #[test]
    fn the_theme_is_stored_as_a_lowercase_name() {
        let config: Config = serde_json::from_str(r#"{"theme": "light"}"#).unwrap();
        assert_eq!(config.theme, Theme::Light);
    }
}
