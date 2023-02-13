mod args;
mod backup;
mod configs;
mod logger;

use config::{Config, File};
use configs::{BackupInfo, ConfigData};
use std::collections::HashMap;

use ansi_term::Colour::RGB;
use anyhow::{anyhow, Result};
use clap::Parser;
use fs_extra::dir::move_dir;
use inquire::ui::RenderConfig;
use inquire::{Confirm, Text};
use log::{error, info};
use std::env::var;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::args::{Commands, NvCfgArgs};

#[macro_use]
extern crate prettytable;

use prettytable::{color, format::Alignment, Attr, Cell, Row, Table};

const NCMDIR: &str = "ncm-rs";
const SETTINGS: &str = "settings.toml";
const CONFIGS: &str = "configs.json";

fn main() -> Result<()> {
    logger::init().expect("Could not initialize logger");

    let mut ncm_path = PathBuf::new();
    let mut nvim_path = PathBuf::new();

    if let Ok(cfg) = var("XDG_CONFIG_HOME") {
        ncm_path.push(&cfg);
        ncm_path.push(NCMDIR);

        nvim_path.push(&cfg);
        nvim_path.push("nvim");
    } else {
        ncm_path.push(var("HOME").unwrap());
        ncm_path.push(".config");
        ncm_path.push(NCMDIR);

        nvim_path.push(var("HOME").unwrap());
        nvim_path.push(".config");
        nvim_path.push("nvim");
    }

    let settings_path = ncm_path.join(SETTINGS);
    let configs_json_path = ncm_path.join(CONFIGS);
    let mut setup_pending = false;

    if settings_path.exists() {
        let config_data = Config::builder().add_source(File::with_name(settings_path.to_str().unwrap())).build().unwrap();
        let settings = config_data.try_deserialize::<HashMap<String, String>>().unwrap();

        if settings.get("setup_complete").unwrap() == "false" {
            setup_pending = true;
        }
    }

    // --| Check if setup is needed -----------------------
    if check_setup(&mut nvim_path, &ncm_path, setup_pending).is_err() {
        info!("Please run 'ncm setup' to configure NCM, or follow the manual setup instructions at https://github.com/instance-id/ncm-rs");
        return Ok(());
    }

    let config_json = configs_json_path.to_str().unwrap();

    // --| Parse Arguments --------------------------------
    let args = NvCfgArgs::parse();

    match &args.command {
        // --| Set-Default Command ------------------------
        Commands::SetDefault { name } => configs::set_default(config_json, name)?,

        // --| Add Command --------------------------------
        Commands::Add { name, path, description } => {
            if add_config(config_json, name, path, description).is_ok() {
                info!("Adding new config: {name:?} {path:?} {description:?}");
            } else {
                error!("Error adding new config: {name:?} {path:?} {description:?}");
            }
        }

        // --| Remove Command -----------------------------
        Commands::Remove { name } => {
            configs::remove_config(config_json, &name.clone().unwrap())?;
        }

        // --| List Command -------------------------------
        Commands::List => {
            list_configs(config_json);
        }

        // --| Load Command -------------------------------
        Commands::Load { name } => load_config(&mut nvim_path, config_json, name),
        Commands::Backup { name } => {
            let cfg = configs::load_configs(config_json, &name.clone().unwrap())?;
            info!("Loading: {:?}", cfg.name);
        }
        Commands::Setup => {
            info!("Setup");
        }
    }

    Ok(())
}

// --| Check Setup -------------------------------
fn check_setup(nvim_path: &mut Path, ncm_path: &Path, setup_pending: bool) -> Result<()> {
    let mut nvim_symlinked: bool = false;
    let settings_path = ncm_path.join(SETTINGS);
    let configs_json_path = ncm_path.join(CONFIGS);

    if nvim_path.is_symlink() {
        nvim_symlinked = true;
    }

    if !nvim_symlinked && !settings_path.exists() && !configs_json_path.exists() || setup_pending {
        info!("New setup detected, creating default config");

        // --| Create necessary directories -----
        check_directories(&settings_path, &configs_json_path).expect("Error creating directories");

        // --| Backup original and move to new location
        match backup_original(ncm_path, nvim_path) {
            Ok(backup_info) => {
                info!("Backup created: {:?}", backup_info);

                let mut nvim_tmp = PathBuf::new();
                let name = backup_info.name.as_str();
                let description: Option<String> = None;
                let mut nvim_path_buf = PathBuf::from(nvim_path.to_str().unwrap());

                nvim_tmp.push(backup_info.path.as_str());
                nvim_tmp.push("nvim");

                if add_config(ncm_path.join(CONFIGS).to_str().unwrap(), name, &nvim_tmp, &description).is_ok() {
                    info!("Adding new config: {name:?} {nvim_tmp:?} {description:?}");
                    load_config(&mut nvim_path_buf, ncm_path.join(CONFIGS).to_str().unwrap(), &Some(name.to_string()));

                    let mut file = std::fs::File::create(settings_path)?;
                    file.write_all(b"setup_complete = true")?;
                } else {
                    error!("Error adding new config: {} {name:?} {nvim_tmp:?} {description:?}", ncm_path.join(CONFIGS).to_str().unwrap());
                }
            }
            Err(e) => {
                error!("Error creating backup: {:?}", e);
            }
        }
    } else {
        check_directories(&settings_path, &configs_json_path).expect("Error creating directories");
    }

    Ok(())
}

fn backup_original(ncm_path: &Path, nvim_path: &Path) -> Result<BackupInfo> {
    let result = Confirm::new("Create Configuration Directory?")
        .with_default(false)
        .with_help_message("This will create a compressed backup of your original config (always best to have a backup), and relocate it to a new directory.")
        .prompt()
        .unwrap();

    if !result {
        return Err(anyhow!("Please backup your original config manually. Instructions can be found at https://github.com/instance-id/ncm-rs"));
    }

    let mut default_path = PathBuf::new();
    // let mut nvim_path = PathBuf::new();

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
        placeholder: Some("Path"),
        help_message: Some("This is the path in which your configurations will be stored. If the directory does not exist, it will be created."),
        formatter: Text::DEFAULT_FORMATTER,
        validators: Vec::new(),
        page_size: Text::DEFAULT_PAGE_SIZE,
        autocompleter: None,
        render_config: RenderConfig::default(),
    }
    .prompt();

    let nvim_config_name = Text {
        message: "Please enter a name for your configuration",
        initial_value: None,
        default: Some("main"),
        placeholder: Some("Name"),
        help_message: Some("This will be used to identify your configuration when loading it."),
        formatter: Text::DEFAULT_FORMATTER,
        validators: Vec::new(),
        page_size: Text::DEFAULT_PAGE_SIZE,
        autocompleter: None,
        render_config: RenderConfig::default(),
    }
    .prompt();

    if let Ok(new_config_path) = &nvim_config_path {
        let mut backup_path = PathBuf::new();
        backup_path.push(ncm_path);
        backup_path.push("nvim_config.zip");

        info!("Creating backup at: {:?}", backup_path);

        // --| Perform Backup -------------------
        perform_backup(nvim_path, new_config_path, &mut backup_path)?;

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

    backup_info.name = nvim_config_name.unwrap();
    backup_info.path = nvim_config_path.unwrap();

    Ok(backup_info)
}

// --| Perform Backup -----------------------------
fn perform_backup(nvim_path: &Path, new_config_path: &String, backup_path: &mut Path) -> Result<()> {
    match backup::create_backup(nvim_path, backup_path) {
        Ok(_) => {
            if backup_path.exists() {
                info!("Backup created successfully");
                info!("Moving original config to: {:?}", new_config_path);

                std::fs::create_dir_all(new_config_path)?;
                move_dir(nvim_path, new_config_path, &fs_extra::dir::CopyOptions::new())?;
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

// --| Check Directories -------------------------
fn check_directories(settings_path: &PathBuf, configs_json_path: &PathBuf) -> Result<()> {
    if !settings_path.exists() {
        std::fs::create_dir_all(settings_path.parent().unwrap())?;
        let mut file = std::fs::File::create(settings_path)?;
        file.write_all(b"setup_complete = false")?;
    }

    // --| Create configs json file and parent directories if it doesn't exist
    if !configs_json_path.exists() {
        std::fs::create_dir_all(configs_json_path.parent().unwrap())?;
        let mut file = std::fs::File::create(configs_json_path)?;
        file.write_all(b"{\n    \"configs\": [\n    ],\n    \"default\": \"\"\n}")?;
    }
    Ok(())
}

// --| Load Configuration -----------------------
fn load_config(nvim_path: &mut PathBuf, config_json: &str, name: &Option<String>) {
    let cfg = configs::load_configs(config_json, &name.clone().unwrap()).expect("Error loading config");
    info!("Loading: {:?}", cfg.name);

    let config_path = PathBuf::from_str(&cfg.path).ok();

    configs::create_symlink(nvim_path, config_path).expect("Error creating symlink");
}

// --| List Configurations ----------------------
fn list_configs(config_json: &str) {
    let cfgs = configs::list_configs(config_json).expect("Error listing configs");
    let current_default = format!("Current Default: {}", cfgs.configs_default);

    let current_str = RGB(70, 130, 180).paint("Current Configurations");
    println!("{}", current_str);
    println!(""); // There is probably a better way to do this, but I don't know what it is...

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

// --| Add Configurations -----------------------
fn add_config(config_json: &str, name: &str, path: &Path, description: &Option<String>) -> serde_json::Result<()> {
    configs::add_config(
        config_json,
        ConfigData {
            name: name.parse().unwrap(),
            path: path.to_str().unwrap().to_string(),
            description: description.clone(),
        },
    )
}
