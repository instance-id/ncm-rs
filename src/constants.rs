#![allow(unused_assignments)]

// --| Environment Variables ------
pub const HOME: &str = "HOME";
pub const USERPROFILE: &str = "USERPROFILE";
pub const APP_DATA_LOCAL: &str = "LOCALAPPDATA";

// XDG_* are not a standard environment variables on Windows
// But for nvim, if it is set, it is used. Otherwise, it defaults to %USERPROFILE%\AppData\Local\nvim 
pub const XDG_DATA_HOME: &str = "XDG_DATA_HOME";
pub const XDG_CACHE_HOME: &str = "XDG_CACHE_HOME";
pub const XDG_STATE_HOME: &str = "XDG_STATE_HOME";
pub const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";

// --| Directory Paths -----------
// --| Linux ---------------------
pub const XDG_DATA_HOME_PATH: &str = "/.local/share";
pub const XDG_CACHE_HOME_PATH: &str = "/.cache";
pub const XDG_STATE_HOME_PATH: &str = "/.local/state";
pub const XDG_CONFIG_HOME_PATH: &str = "/.config";

// --| Windows -------------------
pub const APP_DATA_LOCAL_PATH: &str = "\\AppData\\Local";

// --| Settings Keys -------------
pub const NCM: &str = "ncm";
pub const SETUP_COMPLETE: &str = "setup_complete";

// --| Directory and File Names --
pub const DATA: &str = ".local";
pub const SHARE: &str = "share";
pub const CACHE: &str = ".cache";
pub const STATE: &str = ".state";
pub const CONFIG: &str = ".config";
pub const WIN_DATA: &str = "Local";

pub const ZIP: &str = "zip";
pub const MAIN: &str = "main";
pub const NVIM: &str = "nvim";
pub const NCM_DIR: &str = "ncm-rs";
pub const INIT_LUA: &str = "init.lua";
pub const INIT_VIM: &str = "init.vim";
pub const NVIM_DATA: &str = "nvim-data";

pub const BACKUPS: &str = "backups";
pub const BACKUP_PATH: &str = "backup_path";
pub const CONFIGS_FILE: &str = "configs.json";
pub const NCM_DATA: &str = "nvim-ncm";
pub const SETTINGS_FILE: &str = "settings.ini";


// --| CLI Commands --------------
pub const CLI_SPACER: &str = " ";
pub const CLI_CURRENT_CONFIGS: &str = "Current Configurations";
pub const CLI_TABLE_NAME: &str = "Name";
pub const CLI_TABLE_PATH: &str = "Path";
pub const CLI_TABLE_DESC: &str = "Description";

// --| Default Values ------------
pub const DEFAULT_CONFIG_DESC: &str = "Main Config";
pub const DEFAULT_CURRENT: &str = "Current default";

// --| Information Messages ------
pub const INFO_BACKUP_PATH: &str = "Backup path";
pub const INFO_BACKUP_PATH_AT: &str = "Creating backup at: ";
pub const INFO_BACKUP_COMPLETE: &str = "Backup created successfully";
pub const INFO_BACKUP_SELECT: &str = "Backup which configuration?";

pub const INFO_CONFIGS_ADDED: &str = "Added new config";
pub const INFO_CONFIGS_LOADING: &str = "Loading";
pub const INFO_CONFIG_PATH: &str = "Please enter a path in which to store your configurations";
pub const INFO_CONFIG_NAME: &str = "Please enter a name for your configuration";
pub const INFO_CONFIG_PATH_PLACEHOLDER: &str = "Press enter to use default (~/.config/nvim-ncm)";
pub const INFO_CONFIG_NAME_PLACEHOLDER: &str = "Press enter to use default";

pub const INFO_DIR_CACHE: &str = "Creating cache directory: ";
pub const INFO_DIR_DATA: &str = "Creating data directory: ";

pub const INFO_NEW_SETUP: &str = "New setup detected, creating configuration directories and settings files";
pub const INFO_MOVING_ORIGINAL: &str = "Moving original config to";
pub const INFO_SELECT_ALL: &str = "all";
pub const INFO_SETUP_COMPLETE: &str = "Setup complete!";

pub const DEBUG_CONFIG_VALIDATION_SUCCESS: &str = "init.lua or init.vim was found in the directory";

// --| Help Messages -------------
pub const HELP_BACKUP_MSG: &str = "This will create a compressed backup of your original config (always best to have a backup), and relocate it to a new directory.";

pub const HELP_CONFIG_PATH: &str = "This is the path in which your configurations will be stored. If the directory does not exist, it will be created.";
pub const HELP_CONFIG_NAME: &str = "This will be used to identify your configuration when loading it.";

// --| Error Messages ------------
pub const ERR_BACKUP_CREATE: &str = "Error creating backup";
pub const ERR_BACKUP_PATH: &str = "Could not set backup path";
pub const ERR_BACKUP_MANUALLY: &str = "Please backup your original config manually. Instructions can be found at https://github.com/instance-id/ncm-rs";

pub const ERR_CONFIGS_ADD: &str = "Error adding new config";
pub const ERR_CONFIGS_LIST: &str = "Error listing configs";
pub const ERR_CONFIGS_LOAD: &str = "Error loading configs";
pub const ERR_CONFIGS_NAME: &str = "No configuration found with name";
pub const ERR_CONFIGS_PATH: &str = "Configuration path not found";
pub const ERR_CONFIGS_PARSE: &str = "Could not parse configurations from configs.json";
pub const ERR_CONFIGS_READ: &str = "Could not read configurations from configs.json";
pub const ERR_CONFIGS_WRITE: &str = "Failed to write configuration to disk";

pub const ERR_DIR_UCREATE: &str = "Unable to create directories";
pub const ERR_DIR_DATA: &str = "Could not create data directory";
pub const ERR_DIR_CACHE: &str = "Could not create cache directory";
pub const ERR_DIR_CONFIG_VERIFICATION: &str = "No init.lua or init.vim found in new config path";
pub const ERR_DIR_DATA_VERIFICATION: &str = "Data directory path is not correct";
pub const ERR_DIR_CACHE_VERIFICATION: &str = "Cache directory path is not correct";

pub const ERR_READ_FILE: &str = "Failed to read file";
pub const ERR_RUN_SETUP: &str = "Please run 'ncm setup' to configure NCM, or follow the manual setup instructions at https://github.com/instance-id/ncm-rs";

pub const ERR_SETTINGS_WRITE: &str = "Error writing settings";
pub const ERR_SETTINGS_READ: &str = "Could not read setup_pending from settings.ini";
pub const ERR_SETTINGS_UREAD: &str = "Unable to read settings file";
pub const ERR_SETTINGS_UWRITE: &str = "Unable to write settings file";
pub const ERR_SYMLINK_CREATE: &str = "Error creating symlink";
