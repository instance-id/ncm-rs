#![allow(unused_assignments)]

use anyhow::anyhow;
use std::path::PathBuf;
use serde_json::Result;
use serde::{de::Error, Deserialize, Serialize};
use crate::constants::*;

// Configuration Data Container
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Configs {
    #[serde(rename = "default")]
    pub configs_default: String,
    pub configs: Vec<ConfigData>,
}

// Configuration Data Structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigData {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub data_path: Option<String>,
    pub cache_path: Option<String>,
}

// Backup Data Structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupInfo {
    pub name: String,
    pub path: String,
}

// Create BackupInfo new function
impl BackupInfo {
    pub fn new() -> BackupInfo {
        BackupInfo {
            name: String::new(),
            path: String::new(),
        }
    }
}

// --| Load Configs -----------------------------
// Load a configuration file by name. If name is not specified then the default configuration is used.
pub(crate) fn load_configs(config_path: &str, config_name: &str) -> Result<ConfigData> {
    let config_file = std::fs::read_to_string(config_path).expect("Failed to read file");

    let mut target_config: String = String::new();
    let mut configs: Configs = serde_json::from_str(&config_file)?;
    let config_data: Vec<ConfigData> = configs.configs.clone();
    let default_config = configs.configs_default.clone();

    if config_name.is_empty() {
        target_config = default_config;
    } else {
        target_config = config_name.to_string()
    }

    let config_result = find_config(config_data, &target_config);
    match config_result {
        Ok(config) => {
            configs.configs_default = config.name.to_string();
            write_default(&configs, config_path)?;
            Ok(config)
        }
        Err(_e) => Err(serde_json::Error::custom(format!("{} {}", ERR_CONFIGS_NAME, &target_config))),
    }
}

// --| Find Config ------------------------------
// Using config_path as the json file location, and ConfigData struct as input, write the data to the json file.
pub(crate) fn add_config(config_path: &str, config_data: ConfigData) -> Result<()> {
    let config_file = std::fs::read_to_string(config_path).expect(ERR_READ_FILE);
    let name = &config_data.name.clone();

    // --| Create .local data path for config ---
    let data_path = &config_data.data_path.clone();
    if data_path.is_some() {
        let mut path = PathBuf::from(data_path.as_ref().unwrap());
        path.push(name);

        if !path.exists() {
            info!("{}: {}", INFO_DIR_DATA, path.to_str().unwrap());
            std::fs::create_dir_all(&path).expect(ERR_DIR_UCREATE);
        }
    } else { error!("{}", ERR_DIR_DATA); }

    // --| Create .cache data path for config ---
    if !cfg!(target_os = "windows") {
        let cache_path = &config_data.cache_path;
        if cache_path.is_some() {
            let mut path = PathBuf::from(cache_path.as_ref().unwrap());
            path.push(name);

            if !path.exists() {
                info!("{}: {}", INFO_DIR_CACHE, path.to_str().unwrap());
                std::fs::create_dir_all(&path).expect(ERR_DIR_UCREATE);
            }
        } else { error!("{}", ERR_DIR_CACHE); }
    }

    let mut configs: Configs = serde_json::from_str(&config_file)?;
    configs.configs.push(config_data);

    let config_json = serde_json::to_string(&configs)?;

    write_config_to_disk(config_path, config_json)
}

// --| Set Default ------------------------------
// Using config_path as the json file location and return all current configs
pub(crate) fn list_configs(config_path: &str) -> anyhow::Result<Configs> {
    let config_file = std::fs::read_to_string(config_path).expect(ERR_READ_FILE);

    let configs = serde_json::from_str::<Configs>(&config_file)?;
    Ok(configs)
}

// --| Remove Config ----------------------------
// Using config_path as the json file location and config_name as the
// name of the config to remove, remove the config from the json file
pub(crate) fn remove_config(name: &Option<String>, config_path: &str) -> Result<()> {
    let config_file = std::fs::read_to_string(config_path).expect(ERR_READ_FILE);
    let config_name = name.as_ref().unwrap().to_string();

    let mut configs: Configs = serde_json::from_str(&config_file)?;
    configs.configs.retain(|x| x.name != config_name);

    let config_json = serde_json::to_string(&configs)?;

    write_config_to_disk(config_path, config_json)
}

pub(crate) fn create_symlink(nvim_path: PathBuf, new_config: PathBuf) -> anyhow::Result<()> {
    let new_config = new_config.canonicalize()?;

    if !new_config.exists() {
        return Err(anyhow!(format!("{}: {}", ERR_CONFIGS_PATH, new_config.to_str().unwrap())));
    }

    match (&nvim_path.exists(), &nvim_path.is_dir(), true) {
        (true, true, true) => std::fs::remove_dir_all(&nvim_path)?,
        (true, false, true) => std::fs::remove_file(&nvim_path)?,
        (true, _, false) => return Err(anyhow!(ERR_CONFIGS_WRITE)),
        _ => {}
    };

    #[cfg(target_os = "linux")]
    std::os::unix::fs::symlink(new_config, nvim_path)?;

    #[cfg(target_os = "windows")]
    std::os::windows::fs::symlink_dir(new_config, nvim_path)?;

    #[cfg(target_os = "macos")]
    std::os::macos::fs::symlink(new_config, nvim_path)?;


    Ok(())
}

// --| Helper Functions -------------------------
// --|-------------------------------------------
// Write the configuration to disk
fn write_config_to_disk(config_path: &str, config_json: String) -> Result<(), > {
    if std::fs::write(config_path, config_json).is_ok() {
        Ok(())
    } else {
        Err(serde_json::Error::custom(ERR_CONFIGS_WRITE))
    }
}

// Check if a configuration exists by name
fn find_config(configs: Vec<ConfigData>, config_name: &str) -> Result<ConfigData> {
    if let Some(config) = configs.into_iter().find(|x| x.name == config_name) {
        Ok(config)
    } else {
        Err(serde_json::Error::custom(format!("{} {config_name}", ERR_CONFIGS_NAME)))
    }
}

// Write currently loaded configuration as the default configuration
fn write_default(configs: &Configs, config_path: &str) -> Result<()> {
    let config_json = serde_json::to_string(&configs)?;

    if std::fs::write(config_path, config_json).is_ok() {
        Ok(())
    } else {
        Err(serde_json::Error::custom(ERR_CONFIGS_WRITE))
    }
}

// --| Tests ------------------------------------
// --|-------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::str::FromStr;
    use serde_json::Result;
    use std::path::{Path, PathBuf};
    use pretty_assertions::assert_eq;

    #[cfg(test)]
    #[ctor::ctor]
    fn init() {}

    // --| Load Config --------------------------
    #[test]
    fn load_config_test() {
        let mut tmp_nvim = std::env::temp_dir();
        tmp_nvim.push("ncm_tmp_load");

        let tmp_config_dir = tmp_nvim.join("config");
        let tmp_data_dir = tmp_nvim.join("data");
        let tmp_cache_dir = tmp_nvim.join("cache");
        let tmp_nvim_dir = tmp_nvim.join("nvim");

        std::fs::create_dir_all(tmp_config_dir.clone()).unwrap();
        std::fs::create_dir_all(tmp_data_dir.clone()).unwrap();
        std::fs::create_dir_all(tmp_cache_dir.clone()).unwrap();
        std::fs::create_dir_all(tmp_nvim_dir.clone()).unwrap();

        println!("{:?}", &tmp_config_dir);
        println!("{:?}", &tmp_data_dir);
        println!("{:?}", &tmp_cache_dir);

        let _result = create_test_data(&tmp_config_dir, &tmp_data_dir, &tmp_cache_dir);

        let config_file = &tmp_config_dir.join("configs.json");
        println!("{}", config_file.to_str().unwrap());

        // --| Load Configuration Test ----------
        let config = load_configs(config_file.to_str().unwrap(), "test");

        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.path, tmp_data_dir.join("config_two").to_str().unwrap());

        if !cfg!(target_os = "windows") {
            assert_eq!(config.cache_path.unwrap(), tmp_cache_dir.join("config_two_cache").to_str().unwrap());
        }

        let config_file = std::fs::read_to_string(config_file.to_str().unwrap()).expect("Failed to read file");
        let configs: Configs = serde_json::from_str(&config_file).expect("Failed to parse json");
        assert_eq!(configs.configs_default, "test");

        let config_path = PathBuf::from_str(&config.path).ok();

        // --| Symlink Test ---------------------
        create_symlink(tmp_nvim_dir.clone(), config_path.unwrap()).expect(" Failed to create symlink");

        assert_eq!(&tmp_nvim_dir.read_link().unwrap(), &tmp_data_dir.join("config_two"));
        std::fs::remove_dir_all(tmp_nvim).unwrap();
    }

    // --| Add Config ---------------------------
    #[test]
    fn add_config_test() {
        let mut tmp_nvim = std::env::temp_dir();
        tmp_nvim.push("ncm_tmp_add");

        let tmp_config_dir = tmp_nvim.join("config");
        let tmp_data_dir = tmp_nvim.join("data");
        let tmp_cache_dir = tmp_nvim.join("cache");

        std::fs::create_dir_all(tmp_config_dir.clone()).unwrap();
        std::fs::create_dir_all(tmp_data_dir.clone()).unwrap();
        std::fs::create_dir_all(tmp_cache_dir.clone()).unwrap();

        println!("{:?}", &tmp_config_dir);
        println!("{:?}", &tmp_data_dir);
        println!("{:?}", &tmp_cache_dir);

        let _result = create_test_data(&tmp_config_dir, &tmp_data_dir, &tmp_cache_dir);

        let config_file = &tmp_config_dir.join("configs.json");
        println!("{}", config_file.to_str().unwrap());

        let config = ConfigData {
            name: "test3".to_string(),
            path: tmp_data_dir.join("config_three").to_str().unwrap().to_string(),
            description: Some("test3".to_string()),
            data_path: Some(tmp_data_dir.join("config_three").to_str().unwrap().to_string()),
            cache_path: Some(tmp_data_dir.join("config_three").to_str().unwrap().to_string()),
        };

        // --| Add Configuration Test -----------
        let result = add_config(config_file.to_str().unwrap(), config);

        assert!(result.is_ok());

        let config_file = std::fs::read_to_string(config_file.to_str().unwrap()).expect("Failed to read file");
        let configs: Configs = serde_json::from_str(&config_file).expect("Failed to parse json");

        assert_eq!(configs.configs.len(), 3);
        assert_eq!(configs.configs[2].name, "test3");
        assert_eq!(configs.configs[2].path, tmp_data_dir.join("config_three").to_str().unwrap());

        std::fs::remove_dir_all(tmp_nvim).unwrap();
    }

    // --| Remove Config ------------------------
    #[test]
    fn remove_config_test() {
        let mut tmp_nvim = std::env::temp_dir();
        tmp_nvim.push("ncm_tmp_remove");

        let tmp_config_dir = tmp_nvim.join("config");
        let tmp_data_dir = tmp_nvim.join("data");
        let tmp_cache_dir = tmp_nvim.join("cache");

        std::fs::create_dir_all(tmp_config_dir.clone()).unwrap();
        std::fs::create_dir_all(tmp_data_dir.clone()).unwrap();
        std::fs::create_dir_all(tmp_cache_dir.clone()).unwrap();

        println!("{:?}", &tmp_config_dir);
        println!("{:?}", &tmp_data_dir);
        println!("{:?}", &tmp_cache_dir);

        let _result = create_test_data(&tmp_config_dir, &tmp_data_dir, &tmp_cache_dir);

        let config_file = &tmp_config_dir.join("configs.json");
        println!("{}", config_file.to_str().unwrap());

        let result = remove_config(&Some("default".to_string()), config_file.to_str().unwrap());

        assert!(result.is_ok());

        let config_file = std::fs::read_to_string(config_file.to_str().unwrap()).expect("Failed to read file");
        let configs: Configs = serde_json::from_str(&config_file).expect("Failed to parse json");

        assert_eq!(configs.configs.len(), 1);
        assert_eq!(configs.configs[0].name, "test");
        assert_eq!(configs.configs[0].path, tmp_data_dir.join("config_two").to_str().unwrap());

        std::fs::remove_dir_all(tmp_nvim).unwrap();
    }

    // --| Create Test Data ---------------------
    fn create_test_data(config_path: &Path, data_dir: &Path, cache_dir: &Path) -> Result<Configs> {
        let file_path = config_path.join("configs.json");

        let mut configs = Configs { configs: Vec::new(), configs_default: String::new() };
        configs.configs_default = "default".to_string();

        let path_one = data_dir.join("config_one");
        std::fs::create_dir_all(&path_one).unwrap();

        let path_one_data = data_dir.join("config_one_data");
        std::fs::create_dir_all(&path_one_data).unwrap();

        let path_one_cache = cache_dir.join("config_one_cache");
        std::fs::create_dir_all(&path_one_cache).unwrap();

        let path_two = data_dir.join("config_two");
        std::fs::create_dir_all(&path_two).unwrap();

        let path_two_data = data_dir.join("config_two_data");
        std::fs::create_dir_all(&path_two_data).unwrap();

        let path_two_cache = cache_dir.join("config_two_cache");
        std::fs::create_dir_all(&path_two_cache).unwrap();

        let file_one = path_one.join("init.lua");
        let file_two = path_two.join("init.lua");

        let mut file = File::create(file_one).unwrap();
        file.write_all(b"vim.g.loaded_netrw = 1").unwrap();

        let mut file = File::create(file_two).unwrap();
        file.write_all(b"vim.g.loaded_netrw = 1").unwrap();

        configs.configs.push(ConfigData {
            name: "default".to_string(),
            path: path_one.to_str().unwrap().to_string(),
            description: Some("Default configuration".to_string()),
            data_path: Some(path_one_data.to_str().unwrap().to_string()),
            cache_path: Some(path_one_cache.to_str().unwrap().to_string()),
        });
        configs.configs.push(ConfigData {
            name: "test".to_string(),
            path: path_two.to_str().unwrap().to_string(),
            description: Some("Test configuration".to_string()),
            data_path: Some(path_two_data.to_str().unwrap().to_string()),
            cache_path: Some(path_two_cache.to_str().unwrap().to_string()),
        });

        let config_json = serde_json::to_string(&configs)?;
        std::fs::write(file_path, config_json).expect("Failed to write configuration to disk");

        Ok(configs)
    }
}
