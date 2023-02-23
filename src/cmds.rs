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
use crate::paths::get_ncm_paths;

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
pub(crate) fn add_config(name: &str, path: &Path, description: &Option<String>, data_path: Option<String>, cache_path: Option<String>, config_json: &str) {
    if configs::add_config(
        config_json,
        ConfigData {
            name: name.parse().unwrap(),
            path: path.to_str().unwrap().to_string(),
            description: description.clone(),
            data_path,
            cache_path,
        },
    ).is_ok() {
        info!("{}: {name:?} {path:?} {description:?}", INFO_CONFIGS_ADDED);
    } else {
        error!("{}: {name:?} {path:?} {description:?}", ERR_CONFIGS_ADD);
    }
}

// --| Load --------------------------------
// --|--------------------------------------
pub(crate) fn load_config(name: &Option<String>, settings: &mut RwLockWriteGuard<Settings>) {
    let nvim_path = PathBuf::from(settings.nvim_path.to_str().unwrap());
    let nvim_data = settings.base_paths.local.clone().join(NVIM);
    let nvim_cache = settings.base_paths.cache.clone().join(NVIM);
    let name_str = name.to_owned().unwrap();

    let config_json = settings.configs_path.to_str().unwrap();
    let cfg = configs::load_configs(config_json, &name.clone().unwrap()).expect(ERR_CONFIGS_LOAD);
    info!("{}: {:?}", INFO_CONFIGS_LOADING, cfg.name);

    let config_path = PathBuf::from_str(&cfg.path).ok();
    let data_path = PathBuf::from_str(&cfg.data_path.unwrap()).ok();
    let cache_path = PathBuf::from_str(&cfg.cache_path.unwrap()).ok();

    let config_buf = config_path.unwrap();
    let cfg_str = config_buf.to_str().unwrap();

    let mut data_buf = data_path.unwrap();
    data_buf.push(&name_str);
    let data_str = data_buf.to_str().unwrap();

    let mut cache_buf = cache_path.unwrap();
    cache_buf.push(&name_str);
    let cache_str = cache_buf.to_str().unwrap();

    if verify_config_directory(&nvim_path, &config_buf).is_ok() {
        debug!("{}: {:?} - {}: {}", "System Config Path: ", nvim_path, "Config Path: ", cfg_str);
        configs::create_symlink(nvim_path, config_buf).expect(ERR_SYMLINK_CREATE);
    } else {
        error!("{}: {:?} - {}: {}", "System Config Path: ", nvim_path, "Config Path: ", cfg_str);
    }

    if verify_data_directory(&nvim_data, &data_buf, &name_str).is_ok() {
        debug!("{}: {:?} - {}: {}", "System Data Path:   ", nvim_data, "Data Path:   ", data_str);
        configs::create_symlink(nvim_data, data_buf).expect(ERR_SYMLINK_CREATE);
    } else {
        error!("{}: {:?} - {}: {}", "System Data Path:   ", nvim_data, "Data Path:   ", data_str);
    }

    if verify_cache_path(&nvim_cache, &cache_buf, &name_str).is_ok() {
        debug!("{}: {:?} - {}: {}", "System Cache Path:  ", nvim_cache, "Cache Path: ", cache_str);
        configs::create_symlink(nvim_cache, cache_buf).expect(ERR_SYMLINK_CREATE);
    } else {
        error!("{}: {:?} - {}: {}", "System Cache Path:  ", nvim_cache, "Cache Path: ", cache_str);
    }
}

// --| Verify Original Config Directory ---------
fn verify_config_directory(nvim_path: &Path, new_path: &PathBuf) -> Result<()> {
    if !nvim_path.ends_with(NVIM) && !nvim_path.parent().unwrap().ends_with(CONFIG) {
        Err(anyhow!("{}: {:?}", ERR_DIR_CONFIG_VERIFICATION, new_path))?;
    }

    let file_one = new_path.join(INIT_LUA);
    let file_two = new_path.join(INIT_VIM);

    if !file_one.exists() && !file_two.exists() {
        Err(anyhow!("{}: {:?}", ERR_DIR_CONFIG_VERIFICATION, new_path))?;
    }
    Ok(())
}

// --| Verify Original Data Directory -----------
fn verify_data_directory(nvim_data: &PathBuf, new_path: &PathBuf, name: &str) -> Result<()> {
    if !nvim_data.exists() {
        Err(anyhow!("{}: {:?}", ERR_DIR_DATA_VERIFICATION, nvim_data))?;
    }

    if !nvim_data.ends_with(NVIM) && !nvim_data.parent().unwrap().ends_with(SHARE) {
        Err(anyhow!("{}: {:?}", ERR_DIR_DATA_VERIFICATION, nvim_data))?;
    }

    if !new_path.ends_with(name) && !new_path.parent().unwrap().ends_with(NCM_DATA) {
        Err(anyhow!("{}: {:?}", ERR_DIR_DATA_VERIFICATION, new_path))?;
    }
    Ok(())
}

// --| Verify Original Cache Path ---------------
fn verify_cache_path(nvim_cache: &PathBuf, new_path: &PathBuf, name: &str) -> Result<()> {
    if !nvim_cache.exists() {
        Err(anyhow!("{}: {:?}", ERR_DIR_CACHE_VERIFICATION, nvim_cache))?;
    }

    if !nvim_cache.ends_with(NVIM) && !nvim_cache.parent().unwrap().ends_with(CACHE) {
        Err(anyhow!("{}: {:?}", ERR_DIR_CACHE_VERIFICATION, nvim_cache))?;
    }

    if !new_path.ends_with(name) && !new_path.parent().unwrap().ends_with(NCM_DATA) {
        Err(anyhow!("{}: {:?}", ERR_DIR_DATA_VERIFICATION, new_path))?;
    }
    Ok(())
}

// --| List --------------------------------
// --|--------------------------------------
pub(crate) fn list_configs(config_json: &str) {
    let cfgs = configs::list_configs(config_json).expect(ERR_CONFIGS_LIST);
    let current_default = format!("{}: {}", DEFAULT_CURRENT, cfgs.configs_default);

    let current_str = RGB(70, 130, 180).paint(CLI_CURRENT_CONFIGS);
    println!("{}", current_str);
    println!("{}", CLI_SPACER); // There is probably a better way to do this, but I don't know what it is...

    let name_str = RGB(70, 130, 180).paint(CLI_TABLE_NAME);
    let path_str = RGB(70, 130, 180).paint(CLI_TABLE_PATH);
    let desc_str = RGB(70, 130, 180).paint(CLI_TABLE_DESC);

    let mut table = Table::new();
    table.set_titles(row![b->name_str, b->path_str, b->desc_str]);

    table.set_format(*prettytable::format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    for cfg in cfgs.configs {
        table.add_row(row![cfg.name, cfg.path, cfg.description.unwrap_or("".to_string())]);
    }

    table.add_row(Row::new(vec![Cell::new_align(
        &current_default, Alignment::LEFT)
        .with_style(Attr::Bold)
        .with_style(Attr::ForegroundColor(color::GREEN))
        .with_hspan(3)]));

    table.printstd();
}

fn check_for_nvim(nvim_path: &Path) -> bool {
    if !nvim_path.exists() { return false; }

    let file_one = nvim_path.join(INIT_LUA);
    let file_two = nvim_path.join(INIT_VIM);

    if !file_one.exists() && !file_two.exists() { return false; }
    true
}

// --| Setup -------------------------------
// --|--------------------------------------
pub(crate) fn check_setup(settings: &mut RwLockWriteGuard<'_, Settings>, setup_complete: bool) -> Result<()> {
    let mut nvim_symlinked: bool = false;

    if !check_for_nvim(&settings.nvim_path) {
        if !&settings.xdg_config_is_set {
            if cfg!(windows) {
                warn!("{} {} ", ERR_NVIM_NOT_FOUND_WIN, ERR_NVIM_NOT_FOUND_WIN_NO_XDG);
            } else {
                warn!("{} {} ", ERR_NVIM_NOT_FOUND_LINUX, ERR_NVIM_NOT_FOUND_LINUX_NO_XDG);
            }
        } else {
            if cfg!(windows) {
                warn!("{} {} ", ERR_NVIM_NOT_FOUND, ERR_NVIM_NOT_FOUND_WIN_XDG);
            } else {
                warn!("{}", ERR_NVIM_NOT_FOUND_LINUX);
            }
            return Err(anyhow!("{}: {:?}", ERR_NVIM_NOT_FOUND, settings.nvim_path));
        }
        return Err(anyhow!("{}: {:?}", ERR_NVIM_NOT_FOUND, settings.nvim_path));
    }

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
                let description: Option<String> = Some(DEFAULT_CONFIG_DESC.to_owned());
                let data_path: Option<String> = Some(settings.data_path.to_str().unwrap().to_string());
                let cache_path: Option<String> = Some(settings.cache_path.to_str().unwrap().to_string());

                nvim_tmp.push(backup_info.path.as_str());
                nvim_tmp.push(backup_info.name.as_str());

                if configs::add_config(
                    settings.configs_path.to_str().unwrap(),
                    ConfigData {
                        name: name.parse()?,
                        path: nvim_tmp.to_str().unwrap().to_string(),
                        description: description.clone(),
                        data_path,
                        cache_path,
                    },
                ).is_ok() {
                    load_config(&Some(name.to_string()), settings);
                    info!("{}: {name:?} {nvim_tmp:?}", INFO_CONFIGS_ADDED);

                    settings.settings
                        .set(NCM, SETUP_COMPLETE, Some("true".to_string()))
                        .expect(ERR_SETTINGS_WRITE);

                    settings.write_settings().expect(ERR_SETTINGS_WRITE);

                    let setup_complete = RGB(146, 181, 95).paint(INFO_SETUP_COMPLETE);
                    info!("{}\n", setup_complete);
                } else {
                    error!("{}: {} {name:?} {nvim_tmp:?} {description:?}", ERR_CONFIGS_ADD, settings.configs_path.to_str().unwrap());
                }
            }
            Err(e) => {
                error!("{}: {:?}", ERR_BACKUP_CREATE, e);
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

        options.push(INFO_SELECT_ALL.to_string());
        config_name = Select::new(INFO_BACKUP_SELECT, options).prompt().unwrap();
    }

    if config_name == INFO_SELECT_ALL {
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
    backup_path.push(BACKUPS);

    if !backup_path.exists() {
        std::fs::create_dir_all(&backup_path).expect(ERR_BACKUP_PATH);
    }

    backup_path.push(format!("{}.{}", &config_name, ZIP));

    let backup_str = backup_path.to_str().unwrap();
    debug!("{}: {}", INFO_BACKUP_PATH, backup_str);

    let creating_backup_path = RGB(146, 181, 95).paint(INFO_BACKUP_PATH_AT);
    info!("{} {}", creating_backup_path, backup_path.to_str().unwrap());

    backup_source.push(backup_info.path.as_str());

    // --| Perform Backup -------------------
    match create_backup(backup_source.as_path(), backup_path.as_path()) {
        Ok(_) => {
            if backup_path.exists() {
                let backup_success = RGB(146, 181, 95).paint(INFO_BACKUP_COMPLETE);
                info!("{}", backup_success);
            } else {
                error!("{}", ERR_BACKUP_CREATE);
            }
        }
        Err(e) => {
            error!("{}: {:?}",ERR_BACKUP_CREATE, e);
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

    let default_paths = get_ncm_paths(settings);
    let mut backup_info = BackupInfo {
        name: NCM_DATA.to_string(),
        path: default_paths.config.to_str().unwrap().to_string(),
    };

    let nvim_config_path = Text {
        message: INFO_CONFIG_PATH,
        initial_value: None,
        default: Some(backup_info.path.as_str()),
        placeholder: Some(INFO_CONFIG_PATH_PLACEHOLDER),
        help_message: Some(HELP_CONFIG_PATH),
        formatter: Text::DEFAULT_FORMATTER,
        validators: Vec::new(),
        page_size: Text::DEFAULT_PAGE_SIZE,
        autocompleter: None,
        render_config: RenderConfig::default(),
    }.prompt();

    let nvim_config_name = Text {
        message: INFO_CONFIG_NAME,
        initial_value: None,
        default: Some(MAIN),
        placeholder: Some(INFO_CONFIG_NAME_PLACEHOLDER),
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
        backup_path.push(BACKUPS);

        if !backup_path.exists() {
            std::fs::create_dir_all(&backup_path).expect(ERR_BACKUP_PATH);
        }

        let backup_str = backup_path.to_str().unwrap();
        debug!("{}: {}", INFO_BACKUP_PATH, backup_str);

        settings.settings
            .setstr(NCM, BACKUP_PATH, Option::from(backup_str))
            .expect(ERR_BACKUP_PATH);

        backup_file.push(backup_path.to_str().unwrap());
        backup_file.push(format!("{}.{}", &nvim_config_name, ZIP));

        let creating_backup_path = RGB(146, 181, 95).paint(INFO_BACKUP_PATH_AT);
        info!("{} {}\n", creating_backup_path, backup_file.to_str().unwrap());

        // --| Perform Backup -------------------
        perform_backup(&settings.nvim_path, new_config_path, &backup_file, &nvim_config_name)?;

        let mut init_lua = PathBuf::new();
        init_lua.push(new_config_path);
        init_lua.push(&nvim_config_name);
        init_lua.push(INIT_LUA);

        let mut init_vim = PathBuf::new();
        init_vim.push(new_config_path);
        init_vim.push(&nvim_config_name);
        init_vim.push(INIT_VIM);

        if !init_lua.exists() && !init_vim.exists() {
            return Err(anyhow!(ERR_DIR_CONFIG_VERIFICATION));
        }
    }

    backup_info.name = nvim_config_name;
    backup_info.path = nvim_config_path.unwrap();
    settings.settings.write(&settings.settings_path).expect(ERR_SETTINGS_WRITE);

    Ok(backup_info)
}

// --| Perform Backup -----------------------------
pub(crate) fn perform_backup(nvim_path: &Path, new_config_path: &String, backup_path: &Path, name: &str) -> Result<()> {
    match create_backup(nvim_path, backup_path) {
        Ok(_) => {
            if backup_path.exists() {
                let backup_success = RGB(146, 181, 95).paint(INFO_BACKUP_COMPLETE);

                info!("{}\n", backup_success);
                info!("{}: {:?}", INFO_MOVING_ORIGINAL, new_config_path);

                std::fs::create_dir_all(new_config_path)?;

                let mut rename_original = PathBuf::from(new_config_path);
                rename_original.push(NVIM);

                let mut rename_path = PathBuf::from(new_config_path);
                rename_path.push(name);

                #[cfg(not(target_os = "windows"))]
                {
                    move_dir(nvim_path, new_config_path, &CopyOptions::new().copy_inside(true))?;
                    std::fs::rename(rename_original, rename_path)?;
                }

                #[cfg(target_os = "windows")]
                {
                    fs_extra::copy_items(&[nvim_path], new_config_path, &CopyOptions::new().copy_inside(true))?;
                    std::fs::rename(rename_original, rename_path)?;
                }
            } else {
                error!("{}", ERR_BACKUP_CREATE);
                return Err(anyhow!(ERR_BACKUP_CREATE));
            }
        }
        Err(e) => {
            error!("{}: {:?}",ERR_BACKUP_CREATE, e);
            return Err(anyhow!("{}: {:?}", ERR_BACKUP_CREATE, e));
        }
    }

    Ok(())
}
