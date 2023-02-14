mod args;
mod backup;
mod configs;
mod constants;
mod logger;
mod settings;

use configs::{BackupInfo, ConfigData};
use constants::*;

use crate::args::{Commands, NvCfgArgs};
use crate::settings::Settings;

use ansi_term::Colour::RGB;
use anyhow::{anyhow, Result};
use clap::Parser;
use fs_extra::dir::move_dir;
use inquire::ui::RenderConfig;
use inquire::{Confirm, Text};
use log::{error, info};
use std::env::var;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[macro_use]
extern crate prettytable;

use prettytable::{color, format::Alignment, Attr, Cell, Row, Table};

#[macro_use]
extern crate lazy_static;

use std::sync::{RwLock, RwLockWriteGuard};

// --| Global Settings --------------------------
lazy_static! {
    pub static ref SETTINGS: RwLock<settings::Settings> = RwLock::new(settings::get_settings(XDG_CONFIG_HOME, HOME));
}

fn main() -> Result<()> {
    logger::init().expect("Could not initialize logger");

    let settings = &mut SETTINGS.write().unwrap();

    let setup_complete = settings.config.getbool(NCM, SETUP_COMPLETE).unwrap().expect("Could not read setup_pending from settings.ini");

    // --| Check if setup is needed -------------
    if check_setup(settings, setup_complete).is_err() {
        info!("Please run 'ncm setup' to configure NCM, or follow the manual setup instructions at https://github.com/instance-id/ncm-rs");
        return Ok(());
    }

    let config_json = &settings.configs_path.to_str().unwrap().to_string();

    // --| Parse Arguments ----------------------
    let args = NvCfgArgs::parse();

    match &args.command {
        // --| Set-Default Command --------------
        Commands::SetDefault { name } => configs::set_default(config_json, name)?,

        // --| Add Command ----------------------
        Commands::Add { name, path, description } => {
            if add_config(config_json, name, path, description).is_ok() {
                info!("Adding new config: {name:?} {path:?} {description:?}");
            } else {
                error!("Error adding new config: {name:?} {path:?} {description:?}");
            }
        }

        // --| Remove Command -------------------
        Commands::Remove { name } => {
            configs::remove_config(config_json, &name.clone().unwrap())?;
        }

        // --| List Command ---------------------
        Commands::List => {
            list_configs(config_json);
        }

        // --| Load Command ---------------------
        Commands::Load { name } => load_config(&mut settings.nvim_path, config_json, name),
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

// --| Check Setup ------------------------------
fn check_setup(settings: &mut RwLockWriteGuard<'_, Settings>, setup_complete: bool) -> Result<()> {
    let mut nvim_symlinked: bool = false;

    if settings.nvim_path.is_symlink() {
        nvim_symlinked = true;
    }

    if !nvim_symlinked && !setup_complete {
        info!("New setup detected, creating default config");

        // --| Backup original and move to new location
        match backup_original(&settings.ncm_path, &settings.nvim_path) {
            Ok(backup_info) => {
                let mut nvim_tmp = PathBuf::new();
                let name = backup_info.name.as_str();
                let description: Option<String> = Some("Main Config".to_owned());
                let mut nvim_path_buf = PathBuf::from(settings.nvim_path.to_str().unwrap());

                nvim_tmp.push(backup_info.path.as_str());
                nvim_tmp.push("nvim");

                if add_config(settings.configs_path.to_str().unwrap(), name, &nvim_tmp, &description).is_ok() {
                    load_config(&mut nvim_path_buf, settings.configs_path.to_str().unwrap(), &Some(name.to_string()));
                    info!("Added new config: {name:?} {nvim_tmp:?} {description:?}");

                    settings.config.set(NCM, SETUP_COMPLETE, Some("true".to_string())).expect("Error setting setup_complete to true");
                    settings.write_settings().expect("Error writing settings");

                    let setup_complete = RGB(146, 181, 95).paint("Setup Complete!");
                    println!("");
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

fn backup_original(ncm_path: &Path, nvim_path: &Path) -> Result<BackupInfo> {
    let result = Confirm::new("Create Configuration Directory?")
        .with_default(true)
        .with_help_message("This will create a compressed backup of your original config (always best to have a backup), and relocate it to a new directory.")
        .prompt()
        .unwrap();

    if !result {
        return Err(anyhow!("Please backup your original config manually. Instructions can be found at https://github.com/instance-id/ncm-rs"));
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
        placeholder: Some("Press enter to use default (main)"),
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

        println!(" ");
        let creating_backup_path = RGB(146, 181, 95).paint("Creating backup at: ");
        info!("{} {}\n", creating_backup_path, backup_path.to_str().unwrap());

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
                let backup_success = RGB(146, 181, 95).paint("Backup created successfully");

                println!("");
                info!("{}\n", backup_success);
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
