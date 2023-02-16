use std::env::var;
use std::str::FromStr;
use ansi_term::Color::RGB;
use anyhow::{anyhow, Result};
use log::{debug, error, info};
use inquire::ui::RenderConfig;
use std::path::{Path, PathBuf};
use std::sync::RwLockWriteGuard;
use prettytable::format::Alignment;
use inquire::{Confirm, Select, Text};
use clap::{command, Subcommand, Parser};
use fs_extra::dir::{CopyOptions, move_dir};
use prettytable::{Attr, Cell, color, Row, Table};

use crate::configs;
use crate::constants::*;
use crate::settings::Settings;
use crate::backup::create_backup;
use crate::configs::{BackupInfo, ConfigData, Configs};

#[derive(Parser)]
#[command(name = "ncm")]
#[command(bin_name = "ncm")]
#[command(author, version, about, long_about = None)]
#[clap(about = r#"

Neovim Configuration Manager.

EXAMPLES:
    Add a new configuration directory to the configuration store
    $ ncm add lazyvim /home/username/github/lazyvim/starter

    Load the newly added configuration
    $ ncm load lazyvim"#)]
pub struct NvCfgArgs {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Adds new configuration directory, referenced by name 
    Add { name: String, path: PathBuf, description: Option<String> },

    /// Remove a configuration from the config store
    Remove { name: Option<String> },

    /// Load a configuration by name from the configuration store
    Load { name: Option<String> },

    /// List current stored configurations
    List,

    /// Setup NCM for the first time
    Setup,

    /// Backup all, selected, or current configuration
    Backup { name: Option<String> },
}

// --| Add ---------------------------------
// --|--------------------------------------
pub(crate) fn add_config(name: &str, path: &Path, description: &Option<String>, config_json: &str) {
    if configs::add_config(
        config_json,
        ConfigData {
            name: name.parse().unwrap(),
            path: path.to_str().unwrap().to_string(),
            description: description.clone(),
        },
    ).is_ok() {
        info!("Added new config: {name:?} {path:?} {description:?}");
    } else {
        error!("Error adding new config: {name:?} {path:?} {description:?}");
    }
}

// --| Load --------------------------------
// --|--------------------------------------
pub(crate) fn load_config(name: &Option<String>, nvim_path: &mut PathBuf, config_json: &str) {
    let cfg = configs::load_configs(config_json, &name.clone().unwrap()).expect("Error loading config");
    info!("Loading: {:?}", cfg.name);

    let config_path = PathBuf::from_str(&cfg.path).ok();

    configs::create_symlink(nvim_path, config_path).expect("Error creating symlink");
}


// --| List --------------------------------
// --|--------------------------------------
pub(crate) fn list_configs(config_json: &str) {
    let cfgs = configs::list_configs(config_json).expect("Error listing configs");
    let current_default = format!("Current Default: {}", cfgs.configs_default);

    let current_str = RGB(70, 130, 180).paint("Current Configurations");
    println!("{}", current_str);
    println!(" "); // There is probably a better way to do this, but I don't know what it is...

    let name_str = RGB(70, 130, 180).paint("Name");
    let path_str = RGB(70, 130, 180).paint("Path");
    let desc_str = RGB(70, 130, 180).paint("Description");

    let mut table = Table::new();
    table.set_titles(row![b->name_str, b->path_str, b->desc_str]);

    table.set_format(*prettytable::format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    for cfg in cfgs.configs {
        table.add_row(row![cfg.name, cfg.path, cfg.description.unwrap_or("".to_string())]);
    }

    table.add_row(Row::new(vec![Cell::new_align(&current_default, Alignment::LEFT).with_style(Attr::Bold).with_style(Attr::ForegroundColor(color::GREEN)).with_hspan(3)]));

    table.printstd();
}

// --| Setup -------------------------------
// --|--------------------------------------
pub(crate) fn check_setup(settings: &mut RwLockWriteGuard<'_, Settings>, setup_complete: bool) -> Result<()> {
    let mut nvim_symlinked: bool = false;

    if settings.nvim_path.is_symlink() {
        nvim_symlinked = true;
    }

    if !nvim_symlinked && !setup_complete {
        info!("{}", INFO_NEW_SETUP);

        // --| Backup original and move to new location
        match backup_original(settings) {
            Ok(backup_info) => {
                let mut nvim_tmp = PathBuf::new();
                let name = backup_info.name.as_str();
                let description: Option<String> = Some("Main Config".to_owned());
                let mut nvim_path_buf = PathBuf::from(settings.nvim_path.to_str().unwrap());

                nvim_tmp.push(backup_info.path.as_str());
                nvim_tmp.push("nvim");

                if configs::add_config(
                    settings.configs_path.to_str().unwrap(),
                    ConfigData {
                        name: name.parse()?,
                        path: nvim_tmp.to_str().unwrap().to_string(),
                        description: description.clone(),
                    },
                ).is_ok() {
                    load_config(&Some(name.to_string()), &mut nvim_path_buf, settings.configs_path.to_str().unwrap());
                    info!("Added new config: {name:?} {nvim_tmp:?}");

                    settings.settings
                        .set(NCM, SETUP_COMPLETE, Some("true".to_string()))
                        .expect(ERR_SETTINGS_WRITE);

                    settings.write_settings().expect(ERR_SETTINGS_WRITE);

                    let setup_complete = RGB(146, 181, 95).paint(INFO_SETUP_COMPLETE);
                    println!(" ");
                    info!("{}\n", setup_complete);
                } else {
                    error!("Error adding new config: {} {name:?} {nvim_tmp:?} {description:?}", settings.configs_path.to_str().unwrap());
                }
            }
            Err(e) => {
                error!("Error creating backup: {:?}", e);
            }
        }
    } else {
        settings.check_directories().expect("Error creating directories");
    }

    Ok(())
}

// --| Backup ------------------------------
// --|--------------------------------------
pub(crate) fn initiate_backup(name: &Option<String>, settings: &mut RwLockWriteGuard<Settings>) {
    let config_path = PathBuf::from_str(settings.configs_path.to_str().unwrap()).ok().unwrap();
    let config_file = std::fs::read_to_string(config_path).expect(ERR_CONFIGS_READ);
    let configs: Configs = serde_json::from_str(&config_file).expect(ERR_CONFIGS_PARSE);

    let config_name: String;
    

    if let Some(n) = name {
        config_name = n.to_string();
    } else {
        let mut options = Vec::new();
        
        for cfg in &configs.configs {
            options.push(cfg.name.to_string());
        }

        options.push("all".to_string());
        config_name = Select::new("Backup which configuration?", options).prompt().unwrap();
    }
    
    if config_name == "all" {
        for cfg in &configs.configs {
            backup_selected(&settings, &configs, &cfg.name);
        }
    } else {
        backup_selected(&settings, &configs, &config_name);
    }
}

fn backup_selected(settings: &&mut RwLockWriteGuard<Settings>, configs: &Configs, config_name: &String) {
    let mut backup_info = BackupInfo::new();
    for cfg in &configs.configs {
        if cfg.name == config_name.as_str() {
            backup_info.name = cfg.name.to_string();
            backup_info.path = cfg.path.to_string();
        }
    }

    let mut backup_path = PathBuf::new();
    let mut backup_source = PathBuf::new();

    backup_path.push(&settings.ncm_path);
    backup_path.push("backups");

    if !backup_path.exists() {
        std::fs::create_dir_all(&backup_path).expect(ERR_BACKUP_PATH);
    }

    backup_path.push(format!("{}.zip", &config_name));

    let backup_str = backup_path.to_str().unwrap();
    debug!("Backup Path: {}", backup_str);

    let creating_backup_path = RGB(146, 181, 95).paint("Creating backup at: ");
    info!("{} {}", creating_backup_path, backup_path.to_str().unwrap());

    backup_source.push(backup_info.path.as_str());

    // --| Perform Backup -------------------
    match create_backup(backup_source.as_path(), backup_path.as_path()) {
        Ok(_) => {
            if backup_path.exists() {
                let backup_success = RGB(146, 181, 95).paint("Backup created successfully");
                info!("{}", backup_success);
            } else {
                error!("Error creating backup");
            }
        }
        Err(e) => {
            error!("Error creating backup: {:?}", e);
        }
    }
}


fn backup_original(settings: &mut RwLockWriteGuard<Settings>) -> Result<BackupInfo> {
    let result = Confirm::new("Create Configuration Directory?")
        .with_default(true)
        .with_help_message(HELP_BACKUP_MSG)
        .prompt()
        .unwrap();

    if !result {
        return Err(anyhow!(ERR_BACKUP_MANUALLY));
    }

    let mut default_path = PathBuf::new();

    if let Ok(cfg) = var("XDG_CONFIG_HOME") {
        default_path.push(&cfg);
        default_path.push("nvim_configs");
    } else {
        default_path.push(var("HOME").unwrap());
        default_path.push(".config");
        default_path.push("nvim_configs");
    }

    let mut backup_info = BackupInfo {
        name: "nvim_configs".to_string(),
        path: default_path.to_str().unwrap().to_string(),
    };

    let nvim_config_path = Text {
        message: "Please enter a path in which to store your configurations",
        initial_value: None,
        default: Some(backup_info.path.as_str()),
        placeholder: Some("Press enter to use default (~/.config/nvim_configs)"),
        help_message: Some(HELP_CONFIG_PATH),
        formatter: Text::DEFAULT_FORMATTER,
        validators: Vec::new(),
        page_size: Text::DEFAULT_PAGE_SIZE,
        autocompleter: None,
        render_config: RenderConfig::default(),
    }.prompt();

    let nvim_config_name = Text {
        message: "Please enter a name for your configuration",
        initial_value: None,
        default: Some("main"),
        placeholder: Some("Press enter to use default (main)"),
        help_message: Some(HELP_CONFIG_NAME),
        formatter: Text::DEFAULT_FORMATTER,
        validators: Vec::new(),
        page_size: Text::DEFAULT_PAGE_SIZE,
        autocompleter: None,
        render_config: RenderConfig::default(),
    }.prompt().unwrap();

    if let Ok(new_config_path) = &nvim_config_path {
        let mut backup_path = PathBuf::new();
        let mut backup_file = PathBuf::new();

        backup_path.push(&settings.ncm_path);
        backup_path.push("backups");

        if !backup_path.exists() {
            std::fs::create_dir_all(&backup_path).expect(ERR_BACKUP_PATH);
        }

        let backup_str = backup_path.to_str().unwrap();
        debug!("Backup Path: {}", backup_str);

        settings.settings
            .setstr(NCM, "backup_path", Option::from(backup_str))
            .expect(ERR_BACKUP_PATH);

        backup_file.push(backup_path.to_str().unwrap());
        backup_file.push(format!("{}.zip", &nvim_config_name));
        
        let creating_backup_path = RGB(146, 181, 95).paint("Creating backup at: ");
        info!("{} {}\n", creating_backup_path, backup_file.to_str().unwrap());

        // --| Perform Backup -------------------
        perform_backup(&settings.nvim_path, new_config_path, &backup_file)?;

        let mut init_lua = PathBuf::new();
        init_lua.push(new_config_path);
        init_lua.push("nvim");
        init_lua.push("init.lua");

        let mut init_vim = PathBuf::new();
        init_vim.push(new_config_path);
        init_vim.push("nvim");
        init_vim.push("init.vim");

        if !init_lua.exists() && !init_vim.exists() {
            return Err(anyhow!("No init.lua or init.vim found in new config path"));
        }
    }

    backup_info.name = nvim_config_name;
    backup_info.path = nvim_config_path.unwrap();
    settings.settings.write(&settings.settings_path).expect(ERR_SETTINGS_WRITE);

    Ok(backup_info)
}

// --| Perform Backup -----------------------------
pub(crate) fn perform_backup(nvim_path: &Path, new_config_path: &String, backup_path: &Path) -> Result<()> {
    match create_backup(nvim_path, backup_path) {
        Ok(_) => {
            if backup_path.exists() {
                let backup_success = RGB(146, 181, 95).paint("Backup created successfully");

                println!(" ");
                info!("{}\n", backup_success);
                info!("Moving original config to: {:?}", new_config_path);

                std::fs::create_dir_all(new_config_path)?;
      
                move_dir(nvim_path, new_config_path, &CopyOptions::new().copy_inside(true))?;
            } else {
                error!("Error creating backup");
                return Err(anyhow!("Error creating backup"));
            }
        }
        Err(e) => {
            error!("Error creating backup: {:?}", e);
            return Err(anyhow!("Error creating backup: {:?}", e));
        }
    }

    Ok(())
}
