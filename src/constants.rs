// --| Environment Variables ------
pub const HOME: &str = "HOME";
pub const APP_DATA_LOCAL: &str = "LOCALAPPDATA";

// XDG_CONFIG_HOME is not a standard environment variable on Windows
// But for nvim, if it is set, it is used. Otherwise, it defaults to %USERPROFILE%\AppData\Local\nvim 
pub const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";

// --| Settings Keys -------------
pub const NCM: &str = "ncm";
pub const SETUP_COMPLETE: &str = "setup_complete";


// --| Directory and File Names --
pub const NCM_DIR: &str = "ncm-rs";
pub const SETTINGS_FILE: &str = "settings.ini";
pub const CONFIGS_FILE: &str = "configs.json";

// --| Information Messages ------
pub const INFO_SETUP_COMPLETE: &str = "Setup complete!";
pub const INFO_NEW_SETUP: &str = "New setup detected, creating configuration directories and settings files";

// --| Help Messages -------------
pub const HELP_BACKUP_MSG: &str = "This will create a compressed backup of your original config (always best to have a backup), and relocate it to a new directory.";
pub const HELP_CONFIG_PATH: &str = "This is the path in which your configurations will be stored. If the directory does not exist, it will be created.";
pub const HELP_CONFIG_NAME: &str = "This will be used to identify your configuration when loading it.";

// --| Error Messages ------------
pub const ERR_SETTINGS_WRITE: &str = "Error writing settings";
pub const ERR_SETTINGS_READ: &str = "Could not read setup_pending from settings.ini";
pub const ERR_BACKUP_MANUALLY: &str = "Please backup your original config manually. Instructions can be found at https://github.com/instance-id/ncm-rs";
pub const ERR_RUN_SETUP: &str = "Please run 'ncm setup' to configure NCM, or follow the manual setup instructions at https://github.com/instance-id/ncm-rs";
pub const ERR_CONFIGS_READ: &str = "Could not read configurations from configs.json";
pub const ERR_CONFIGS_PARSE: &str = "Could not parse configurations from configs.json";
pub const ERR_BACKUP_PATH: &str = "Could not set backup path"; 
