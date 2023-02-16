use std::env::var;
use std::io::Write;
use std::path::{PathBuf};
use configparser::ini::Ini;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

use crate::constants::*;

#[derive(Debug, Clone)]
pub struct Settings {
    pub settings: Ini,
    pub ncm_path: PathBuf,
    pub dot_path: PathBuf,
    pub nvim_path: PathBuf,
    pub configs_path: PathBuf,
    pub settings_path: PathBuf,
    pub settings_map: HashMap<String, HashMap<String, Option<String>>>,
}

impl Settings {
    pub fn new() -> Settings {
        Settings {
            settings: Ini::new(),
            ncm_path: PathBuf::new(),
            dot_path: PathBuf::new(),
            nvim_path: PathBuf::new(),
            settings_map: HashMap::new(),
            configs_path: PathBuf::new(),
            settings_path: PathBuf::new(),
        }
    }

    pub fn check_directories(&mut self) -> Result<()> {
        if !self.ncm_path.exists() {
            std::fs::create_dir_all(&self.ncm_path)?;
        }

        if !self.settings_path.exists() {
            std::fs::create_dir_all(self.settings_path.parent().unwrap())?;
            self.settings.read(String::from(
                "[ncm]
                    setup_complete = false
            backup_path=none")).expect("Unable to read settings file");
            self.settings.write(&self.settings_path).expect("Unable to write settings file");
        }

        if !self.configs_path.exists() {
            std::fs::create_dir_all(self.configs_path.parent().unwrap())?;
            let mut file = std::fs::File::create(&self.configs_path)?;
            file.write_all(b"{\n    \"configs\": [\n    ],\n    \"default\": \"\"\n}")?;
        }

        Ok(())
    }

    pub fn write_settings(&mut self) -> Result<()> {
        if self.settings.write(&self.settings_path).is_err() {
            Err(anyhow!("Unable to write settings file"))
        } else { Ok(()) }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings::new()
    }
}

// --| Create and load Settings -----------------
pub fn get_settings(config_home: &str, home: &str) -> Settings {
    let mut settings = Settings::new();

    // --| Use XDG_CONFIG_HOME is set, else -----
    // --| use $HOME/.config or $LOCALAPPDATA ---
    if let Ok(cfg) = var(config_home) {
        
        settings.dot_path.push(&cfg);
        settings.ncm_path.push(&settings.dot_path);
        settings.ncm_path.push(NCM_DIR);

        settings.nvim_path.push(&settings.dot_path);
        settings.nvim_path.push("nvim");
    } else {
        settings.dot_path.push(var(home).unwrap());

        // --| Windows doesn't use .config
        if !cfg!(windows) { settings.dot_path.push(".config"); }

        settings.ncm_path.push(&settings.dot_path);
        settings.ncm_path.push(NCM_DIR);

        settings.nvim_path.push(&settings.dot_path);
        settings.nvim_path.push("nvim");
    }

    settings.settings_path.push(&settings.ncm_path);
    settings.settings_path.push(SETTINGS_FILE);

    settings.configs_path.push(&settings.ncm_path);
    settings.configs_path.push(CONFIGS_FILE);

    settings.check_directories().expect("Unable to create directories");
    settings.settings.load(&settings.settings_path).unwrap();

    let mut config = Ini::new();
    let map = config.load(&settings.settings_path).unwrap();
    settings.settings_map = map;
    settings
}

// --| Tests ------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::env::set_var;
    use std::path::{Path, PathBuf};
    use pretty_assertions::assert_eq;

    // Test that the settings file is created
    #[test]
    fn test_create_settings() {
        let tmp_home = "TMP_HOME";
        let tmp_config_home = "TMP_CONFIG_HOME";

        // --| First run emulates no XDG_CONFIG_HOME ----
        let (dir, mut dot_path, mut home_path) = create_directories();
        set_var(tmp_home, home_path.clone());
        run_get_config(dir, &mut dot_path, &mut home_path, tmp_config_home, tmp_home);

        // --| Second run emulates XDG_CONFIG_HOME ----
        let (dir, mut dot_path, mut home_path) = create_directories();
        set_var(tmp_config_home, dot_path.clone());
        run_get_config(dir, &mut dot_path, &mut home_path, tmp_config_home, tmp_home);
    }

    // --| Helper functions for multiple passes utilizing -
    // --| different environment variable configurations --
    fn create_directories() -> (PathBuf, PathBuf, PathBuf) {
        let mut dir = std::env::temp_dir();
        dir.push("ncm_tmp_config");

        std::fs::create_dir_all(dir.clone()).unwrap();

        let mut dot_path = PathBuf::new();

        dot_path.push(dir.as_path());
        if !cfg!(windows) { dot_path.push(".config"); }

        let mut home_path = PathBuf::new();
        home_path.push(dir.as_path());
        (dir, dot_path, home_path)
    }

    fn run_get_config(dir: PathBuf, dot_path: &mut Path, home_path: &mut Path, tmp_config_home: &str, tmp_home: &str) {
        let settings = get_settings(tmp_config_home, tmp_home);

        if var(tmp_config_home).is_ok() {
            assert_eq!(var(tmp_config_home).unwrap(), dot_path.to_str().unwrap());
        }

        assert_eq!(var(tmp_home).unwrap(), home_path.to_str().unwrap());

        assert!(dot_path.exists());
        assert!(home_path.exists());

        assert_eq!(settings.dot_path, dot_path.to_path_buf());
        assert_eq!(settings.ncm_path, dot_path.join(NCM_DIR));
        assert_eq!(settings.nvim_path, dot_path.join("nvim"));


        assert_eq!(settings.settings_path, dot_path.join(NCM_DIR).join(SETTINGS_FILE));
        assert_eq!(settings.configs_path, dot_path.join(NCM_DIR).join(CONFIGS_FILE));
        assert_eq!(settings.settings.getbool(NCM, SETUP_COMPLETE).unwrap().unwrap(), false);

        std::fs::remove_dir_all(dir).unwrap();
    }
}
