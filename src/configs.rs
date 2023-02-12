use serde::{de::Error, Deserialize, Serialize};
use serde_json::Result;

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
}

// --| Load Configs -----------------------------
// Load a configuration file by name. If name is not specified then the default configuration is used.
pub(crate) fn load_configs(config_path: &str, config_name: &str) -> Result<ConfigData> {
    let config_file = std::fs::read_to_string(config_path).expect("Failed to read file");

    let target_config: String;
    let mut configs: Configs = serde_json::from_str(&config_file)?;
    let config_data: Vec<ConfigData> = configs.configs.clone();
    let default_config = configs.configs_default.clone();

    if config_name == "" {
        target_config = default_config;
    } else {
        target_config = config_name.to_string();
    }

    let config_result = find_config(config_data, &target_config);
    match config_result {
        Ok(config) => {
            configs.configs_default = config.name.to_string();
            write_default(&configs, config_path)?;
            Ok(config)
        }
        Err(_e) => return Err(serde_json::Error::custom(format!("No configuration found with name {target_config}"))),
    }
}

// --| Find Config ------------------------------
// Using config_path as the json file location, and ConfigData struct as input, write the data to the json file.
pub(crate) fn add_config(config_path: &str, config_data: ConfigData) -> anyhow::Result<()> {
    let config_file = std::fs::read_to_string(config_path).expect("Failed to read file");

    let mut configs: Configs = serde_json::from_str(&config_file)?;
    configs.configs.push(config_data);

    let config_json = serde_json::to_string(&configs)?;

    if std::fs::write(config_path, config_json).is_ok() {
        Ok(())
    } else {
        Err(serde_json::Error::custom("Failed to write configuration to disk").into())
    }
}

// --| Set Default ------------------------------
// Using config_path as the json file location and return all current configs
pub(crate) fn list_configs(config_path: &str) -> anyhow::Result<Configs> {
    let config_file = std::fs::read_to_string(config_path).expect("Failed to read file");

    let configs = serde_json::from_str::<Configs>(&config_file)?;
    Ok(configs)
}

// --| Remove Config ----------------------------
// Using config_path as the json file location and config_name as the
// name of the config to remove, remove the config from the json file
pub(crate) fn remove_config(config_path: &str, config_name: &str) -> anyhow::Result<()> {
    let config_file = std::fs::read_to_string(config_path).expect("Failed to read file");

    let mut configs: Configs = serde_json::from_str(&config_file)?;
    configs.configs.retain(|x| x.name != config_name);

    let config_json = serde_json::to_string(&configs)?;

    if std::fs::write(config_path, config_json).is_ok() {
        Ok(())
    } else {
        Err(serde_json::Error::custom("Failed to write configuration to disk").into())
    }
}

// --| Set Default ------------------------------
// Using config_path as the json file location and config_name as the name
// of the config to set as default, set the config as default in the json file
pub(crate) fn set_default(config_path: &str, config_name: &str) -> anyhow::Result<()> {
    let config_file = std::fs::read_to_string(config_path).expect("Failed to read file");

    let mut configs: Configs = serde_json::from_str(&config_file)?;

    if configs.configs.iter().find(|x| x.name == config_name).is_none() {
        return Err(serde_json::Error::custom("No config found").into());
    }

    configs.configs_default = config_name.to_string();
    let config_json = serde_json::to_string(&configs)?;

    if std::fs::write(config_path, config_json).is_ok() {
        Ok(())
    } else {
        Err(serde_json::Error::custom("Failed to write configuration to disk").into())
    }
}

// --| Helper Functions -------------------------
// Check if a configuration exists by name
fn find_config(configs: Vec<ConfigData>, config_name: &str) -> Result<ConfigData> {
    return if let Some(config) = configs.into_iter().find(|x| x.name == config_name) {
        Ok(config)
    } else {
        Err(serde_json::Error::custom(format!("No configuration found with name {config_name}")))
    };
}

// Write currently loaded configuration as the default configuration
fn write_default(configs: &Configs, config_path: &str) -> Result<()> {
    let config_json = serde_json::to_string(&configs)?;

    if std::fs::write(config_path, config_json).is_ok() {
        return Ok(());
    } else {
        Err(serde_json::Error::custom("Failed to write configuration to disk").into())
    }
}

// --| Tests ------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::str::FromStr;
    use std::path::PathBuf;
    use serde_json::Result;
    use pretty_assertions::assert_eq;

    #[cfg(test)]
    #[ctor::ctor]
    fn init() {}

    // --| Load Config --------------------------
    #[test]
    fn load_config_test() {
        let tmp_nvim = PathBuf::from_str("/tmp/ncm_tmp_load").expect("Failed to create temp dir");

        let tmp_config_dir = PathBuf::from(tmp_nvim.join("config"));
        let tmp_data_dir = PathBuf::from(tmp_nvim.join("data"));
        let tmp_nvim_dir = PathBuf::from(tmp_nvim.join("nvim"));

        std::fs::create_dir_all(tmp_config_dir.clone()).unwrap();
        std::fs::create_dir_all(tmp_data_dir.clone()).unwrap();
        std::fs::create_dir_all(tmp_nvim_dir.clone()).unwrap();

        print!("{:?}\n", &tmp_config_dir);
        print!("{:?}\n", &tmp_data_dir);

        let _result = create_test_data(&tmp_config_dir, &tmp_data_dir);

        let config_file = &tmp_config_dir.join("configs.json");
        println!("{:?}", config_file);

        // --| Load Configuration Test ----------
        let config = load_configs(config_file.to_str().unwrap(), "test");

        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.path, tmp_data_dir.join("config_two").to_str().unwrap());

        let config_file = std::fs::read_to_string(config_file.to_str().unwrap()).expect("Failed to read file");
        let configs: Configs = serde_json::from_str(&config_file).expect("Failed to parse json");
        assert_eq!(configs.configs_default, "test");

        let config_path = PathBuf::from_str(&config.path).ok();

        // --| Symlink Test ---------------------
        if let Some(new_config) = config_path {
            let new_config = new_config.canonicalize().expect("Failed to canonicalize path");

            if !new_config.exists() {
                return Err(anyhow::anyhow!("Config file does not exist")).expect("Failed to remove file");
            }

            match (tmp_nvim_dir.exists(), tmp_nvim_dir.is_dir(), true) {
                (true, true, true) => std::fs::remove_dir_all(&tmp_nvim_dir).expect("Failed to remove directory"),
                (true, false, true) => std::fs::remove_file(&tmp_nvim_dir).expect("Failed to remove file"),
                (true, _, false) => return Err(anyhow::anyhow!("Config file does not exist")).expect("Failed to remove file"),
                _ => {}
            };

            std::os::unix::fs::symlink(new_config, &tmp_nvim_dir).expect("Failed to create symlink");
        }

        assert_eq!(&tmp_nvim_dir.read_link().unwrap(), &tmp_data_dir.join("config_two"));
        std::fs::remove_dir_all(tmp_nvim).unwrap();
    }

    // --| Add Config ---------------------------
    #[test]
    fn add_config_test() {
        let tmp_nvim = PathBuf::from_str("/tmp/ncm_tmp_add").expect("Failed to create temp dir");
        let tmp_config_dir = PathBuf::from(tmp_nvim.join("config"));
        let tmp_data_dir = PathBuf::from(tmp_nvim.join("data"));

        std::fs::create_dir_all(tmp_config_dir.clone()).unwrap();
        std::fs::create_dir_all(tmp_data_dir.clone()).unwrap();

        print!("{:?}\n", &tmp_config_dir);
        print!("{:?}\n", &tmp_data_dir);

        let _result = create_test_data(&tmp_config_dir, &tmp_data_dir);

        let config_file = &tmp_config_dir.join("configs.json");
        println!("{:?}", config_file);

        let config = ConfigData {
            name: "test3".to_string(),
            path: tmp_data_dir.join("config_three").to_str().unwrap().to_string(),
            description: Some("test3".to_string()),
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
        let tmp_nvim = PathBuf::from_str("/tmp/ncm_tmp_remove").expect("Failed to create temp dir");
        let tmp_config_dir = PathBuf::from(tmp_nvim.join("config"));
        let tmp_data_dir = PathBuf::from(tmp_nvim.join("data"));

        std::fs::create_dir_all(tmp_config_dir.clone()).unwrap();
        std::fs::create_dir_all(tmp_data_dir.clone()).unwrap();

        print!("{:?}\n", &tmp_config_dir);
        print!("{:?}\n", &tmp_data_dir);

        let _result = create_test_data(&tmp_config_dir, &tmp_data_dir);

        let config_file = &tmp_config_dir.join("configs.json");
        println!("{:?}", config_file);

        let result = remove_config(config_file.to_str().unwrap(), "default");

        assert!(result.is_ok());

        let config_file = std::fs::read_to_string(config_file.to_str().unwrap()).expect("Failed to read file");
        let configs: Configs = serde_json::from_str(&config_file).expect("Failed to parse json");

        assert_eq!(configs.configs.len(), 1);
        assert_eq!(configs.configs[0].name, "test");
        assert_eq!(configs.configs[0].path, tmp_data_dir.join("config_two").to_str().unwrap());

        std::fs::remove_dir_all(tmp_nvim).unwrap();
    }

    // --| Create Test Data ---------------------
    fn create_test_data(config_path: &PathBuf, data_dir: &PathBuf) -> Result<Configs> {
        let file_path = config_path.join("configs.json");

        let mut configs = Configs { configs: Vec::new(), configs_default: String::new() };
        configs.configs_default = "default".to_string();

        let path_one = data_dir.join("config_one");
        std::fs::create_dir_all(&path_one).unwrap();

        let path_two = data_dir.join("config_two");
        std::fs::create_dir_all(&path_two).unwrap();

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
        });
        configs.configs.push(ConfigData {
            name: "test".to_string(),
            path: path_two.to_str().unwrap().to_string(),
            description: Some("Test configuration".to_string()),
        });

        let config_json = serde_json::to_string(&configs)?;
        std::fs::write(file_path, config_json).expect("Failed to write configuration to disk");

        Ok(configs)
    }
}
