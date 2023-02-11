use serde::{de::Error, Deserialize, Serialize};
use serde_json::{Result};


#[derive(Debug, Serialize, Deserialize)]
pub struct Configs {
    #[serde(rename = "default")]
    pub configs_default: String,
    pub configs: Vec<ConfigData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigData {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
}

// Load a configuration file by name. If name is not specified then the default configuration is used.
pub(crate) fn load_configs(config_path: &str, config_name: &str) -> Result<ConfigData> {
    let config_file = std::fs::read_to_string(config_path).expect("Failed to read file");

    let target_config: String;
    let configs: Configs = serde_json::from_str(&config_file)?;
    let config_data: Vec<ConfigData> = configs.configs;
    let default_config = configs.configs_default.clone();

    if config_name == "" {
        target_config = default_config;
    } else {
        target_config = config_name.to_string();
    }

    return if let Some(config) = config_data.into_iter().find(|x| x.name == target_config) { 
        Ok(config)
    } 
    else { 
        Err(serde_json::Error::custom(format!("No configuration found with name {target_config}")))
    };
}

// Using config_path as the json file location, and ConfigData struct as input, write the data to the json file.
pub(crate) fn add_config(config_path: &str, config_data: ConfigData) ->  anyhow::Result<()>{
    // Load file from path 'config_path'
    let config_file = std::fs::read_to_string(config_path).expect("Failed to read file");

    // Parse the string of data into serde_json::Value.
    let mut configs: Configs = serde_json::from_str(&config_file)?;

    // Add new config to the vector
    configs.configs.push(config_data);

    // Write the vector to the json file
    let config_json = serde_json::to_string(&configs)?;

    if std::fs::write(config_path, config_json).is_ok() {
        Ok(())
    } else {
        Err(serde_json::Error::custom("Failed to write configuration to disk").into())
    }
}

// Using config_path as the json file location and return all current configs
pub(crate) fn list_configs(config_path: &str) -> anyhow::Result<Configs> {
    let config_file = std::fs::read_to_string(config_path).expect("Failed to read file");

    let configs = serde_json::from_str::<Configs>(&config_file)?;
    Ok(configs)
}

// Using config_path as the json file location and config_name as the name of the config to remove, remove the config from the json file
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

// Using config_path as the json file location and config_name as the name of the config to set as default, set the config as default in the json file
pub(crate) fn set_default(config_path: &str, config_name: &str) -> anyhow::Result<()> {
    let config_file = std::fs::read_to_string(config_path).expect("Failed to read file");

    let mut configs: Configs = serde_json::from_str(&config_file)?;

    // Check if name exists in the configs
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
