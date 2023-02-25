use std::io::Write;
use std::path::{PathBuf};
use configparser::ini::Ini;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

use crate::constants::*;
use crate::paths::*;

#[derive(Debug, Clone)]
pub struct Settings {
    pub settings: Ini,
    pub dot_path: PathBuf,
    pub nvim_path: PathBuf,
    pub data_path: PathBuf,
    pub cache_path: PathBuf,
    pub ncm_cfg_path: PathBuf,
    pub configs_path: PathBuf,
    pub settings_path: PathBuf,
    pub env_vars: EnvVariables,
    /// Custom ncm paths in which to store moved nvim files
    pub ncm_paths: GenericPaths,
    pub base_paths: GenericPaths,
    /// Original Nvim paths
    pub nvim_paths: GenericPaths,
    pub settings_map: HashMap<String, HashMap<String, Option<String>>>,
    pub xdg_data_is_set: bool,
    pub xdg_config_is_set: bool,
}

impl Settings {
    pub fn new(env_vars: &EnvVariables) -> Settings {
        Settings {
            settings: Ini::new(),
            dot_path: PathBuf::new(),
            nvim_path: PathBuf::new(),
            data_path: PathBuf::new(),
            cache_path: PathBuf::new(),
            ncm_cfg_path: PathBuf::new(),
            configs_path: PathBuf::new(),
            settings_map: HashMap::new(),
            settings_path: PathBuf::new(),
            ncm_paths: GenericPaths::default(),
            base_paths: GenericPaths::default(),
            nvim_paths: GenericPaths::default(),
            env_vars: env_vars.clone(),
            xdg_data_is_set: false,
            xdg_config_is_set: false,
        }
    }

    pub fn get_paths(&mut self) -> &mut Settings {
        self.base_paths = get_base_paths(self);
        self.nvim_paths = get_nvim_paths(self);
        self.ncm_paths = get_ncm_paths(self);

        self.dot_path = self.base_paths.config.to_owned();
        self.ncm_cfg_path = self.base_paths.config.join(NCM_DIR);
        self.nvim_path = self.nvim_paths.config.to_owned();
        self.data_path = self.nvim_paths.local.to_owned();
        self.cache_path = self.nvim_paths.cache.to_owned();

        self.settings_path.push(&self.ncm_cfg_path);
        self.settings_path.push(SETTINGS_FILE);

        self.configs_path.push(&self.ncm_cfg_path);
        self.configs_path.push(CONFIGS_FILE);

        self
    }

    pub fn check_directories(&mut self) -> Result<()> {
        if !self.ncm_cfg_path.exists() { std::fs::create_dir_all(&self.ncm_cfg_path)?; }
        if !self.data_path.exists() { std::fs::create_dir_all(&self.data_path)?; }
        if !self.cache_path.exists() { std::fs::create_dir_all(&self.cache_path)?; }
        if !self.ncm_paths.local.exists() { std::fs::create_dir_all(&self.ncm_paths.local)?; }

        if !self.settings_path.exists() {
            std::fs::create_dir_all(self.settings_path.parent().unwrap())?;
            self.settings.read(String::from(
                "[ncm]
                    setup_complete = false
            backup_path=none")).expect(ERR_SETTINGS_UREAD);
            self.settings.write(&self.settings_path).expect(ERR_SETTINGS_UWRITE);
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
            Err(anyhow!(ERR_SETTINGS_UWRITE))
        } else { Ok(()) }
    }
}

impl Default for Settings {
    fn default() -> Self { Settings::new(&EnvVariables::default()) }
}

// --| Create and load Settings -----------------
// pub fn get_settings(config_home: &str, home: &str) -> Settings {
pub fn get_settings(env_vars: &EnvVariables) -> Settings {
    let mut settings = Settings::new(env_vars);
    let mut settings = settings.get_paths();
    
    settings.check_directories().expect(ERR_DIR_UCREATE);
    settings.settings.load(&settings.settings_path).unwrap();

    let mut config = Ini::new();
    let map = config.load(&settings.settings_path).unwrap();
    settings.settings_map = map;
    
    debug!("Settings: {:?}", settings);
    settings.to_owned()
}

// --| Tests ------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::env::{set_var, var};
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
        let env_vars = EnvVariables {
            home: tmp_home.to_string(),
            app_data_local: tmp_home.to_string(),
            xdg_data_home: tmp_config_home.to_string(),
            xdg_cache_home: tmp_config_home.to_string(),
            xdg_state_home: tmp_config_home.to_string(),
            xdg_config_home: tmp_config_home.to_string(),
        };

        let settings = get_settings(&env_vars);

        info!("settings: {:?}", settings);

        if var(tmp_config_home).is_ok() {
            assert_eq!(var(tmp_config_home).unwrap(), dot_path.to_str().unwrap());
        }

        assert_eq!(var(tmp_home).unwrap(), home_path.to_str().unwrap());
        assert_eq!(home_path.exists(), true, "home_path: {:?}", home_path);

        assert_eq!(settings.ncm_cfg_path, dot_path.join(NCM_DIR));
        assert_eq!(settings.nvim_path, dot_path.join(NVIM));

        assert_eq!(settings.settings_path, dot_path.join(NCM_DIR).join(SETTINGS_FILE));
        assert_eq!(settings.configs_path, dot_path.join(NCM_DIR).join(CONFIGS_FILE));
        assert_eq!(settings.settings.getbool(NCM, SETUP_COMPLETE).unwrap().unwrap(), false);

        std::fs::remove_dir_all(dir).unwrap();
    }
}
