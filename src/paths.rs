use std::env::var;
use std::path::PathBuf;

use crate::constants::*;
use crate::settings::Settings;

#[derive(Clone, Debug)]
pub struct GenericPaths {
    /// .config or %LOCALAPPDATA%
    pub config: PathBuf,
    /// .local/share or %LOCALAPPDATA%
    pub local: PathBuf,
    /// .cache or %TEMP%
    pub cache: PathBuf,
    /// .local/state or %LOCALAPPDATA%
    pub state: PathBuf,
}

impl Default for GenericPaths {
    fn default() -> Self {
        GenericPaths {
            config: PathBuf::new(),
            local: PathBuf::new(),
            cache: PathBuf::new(),
            state: PathBuf::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct EnvVariables {
    pub home: String,
    pub xdg_data_home: String,
    pub xdg_cache_home: String,
    pub xdg_state_home: String,
    pub app_data_local: String,
    pub xdg_config_home: String,
}

impl Default for EnvVariables {
    fn default() -> Self {
        EnvVariables {
            home: if cfg!(windows) { String::from(USERPROFILE) } else { String::from(HOME) },
            xdg_data_home: String::from(XDG_DATA_HOME),
            xdg_cache_home: String::from(XDG_CACHE_HOME),
            xdg_state_home: String::from(XDG_STATE_HOME),
            app_data_local: String::from(APP_DATA_LOCAL),
            xdg_config_home: String::from(XDG_CONFIG_HOME),
        }
    }
}

pub(crate) fn get_base_paths(settings: &mut Settings) -> GenericPaths {
    create_paths(settings)
}

// --| Nvim Paths are the default locations for Neovim ----
pub(crate) fn get_nvim_paths(settings: &mut Settings) -> GenericPaths {
    let base_paths = &settings.base_paths.clone();
    let mut config = base_paths.config.clone();
    let mut local = base_paths.local.clone();
    let mut cache = base_paths.cache.clone();
    let mut state = base_paths.state.clone();

    let data_dir = if cfg!(target_os = "windows") { NVIM_DATA } else { NVIM };

    config.push(NVIM);
    local.push(data_dir);
    cache.push(NVIM);
    state.push(NVIM);

    GenericPaths { config, local, cache, state }
}

// --| Ncm Paths are the modified destination paths -------
pub(crate) fn get_ncm_paths(settings: &mut Settings) -> GenericPaths {
    let base_paths = &settings.base_paths.clone();
    let mut config = base_paths.config.clone();
    let mut local = base_paths.local.clone();
    let mut cache = base_paths.cache.clone();
    let mut state = base_paths.state.clone();

    let data_dir = if cfg!(target_os = "windows") && !settings.xdg_data_is_set { NCM_DATA_WIN } else { NCM_DATA };

    config.push(NCM_DATA);
    local.push(data_dir);
    cache.push(NCM_DATA);
    state.push(NCM_DATA);

    GenericPaths { config, local, cache, state }
}

fn create_paths(settings: &mut Settings) -> GenericPaths {
    let config: PathBuf;
    let local: PathBuf;
    let cache: PathBuf;
    let state: PathBuf;

    if cfg!(target_os = "windows")
    {
        let win_path = PathBuf::from(var(&settings.env_vars.app_data_local).unwrap_or_else(|_| {
            var(&settings.env_vars.home).unwrap() + APP_DATA_LOCAL_PATH
        }));

        match var(&settings.env_vars.xdg_config_home) {
            Ok(value) => {
                config = PathBuf::from(value);
                settings.xdg_config_is_set = true;
                debug!("Windows: using XDG_CONFIG_HOME as base path: {:?}", config);
            }
            Err(_) => {
                config = win_path.clone();
                debug!("Windows: using APPDATA_LOCAL as base path: {:?}", config);
            }
        }

        match var(&settings.env_vars.xdg_data_home) {
            Ok(value) => {
                local = PathBuf::from(value);
                settings.xdg_data_is_set = true;
                debug!("Windows: using XDG_DATA_HOME as base path: {:?}", local);
            }
            Err(_) => {
                local = win_path.clone();
                debug!("Windows: using APPDATA_LOCAL as base path: {:?}", local);
            }
        }

        cache = PathBuf::from(var(&settings.env_vars.xdg_cache_home).unwrap_or_else(|_| {
            win_path.to_str().unwrap().to_string()
        }));

        state = PathBuf::from(var(&settings.env_vars.xdg_state_home).unwrap_or_else(|_| {
            win_path.to_str().unwrap().to_string()
        }));
    } else {
        config = PathBuf::from(var(&settings.env_vars.xdg_config_home).unwrap_or_else(|_| {
            var(&settings.env_vars.home).unwrap() + XDG_CONFIG_HOME_PATH
        }));

        local = PathBuf::from(var(&settings.env_vars.xdg_data_home).unwrap_or_else(|_| {
            var(&settings.env_vars.home).unwrap() + XDG_DATA_HOME_PATH
        }));

        cache = PathBuf::from(var(&settings.env_vars.xdg_cache_home).unwrap_or_else(|_| {
            var(&settings.env_vars.home).unwrap() + XDG_CACHE_HOME_PATH
        }));

        state = PathBuf::from(var(&settings.env_vars.xdg_state_home).unwrap_or_else(|_| {
            var(&settings.env_vars.home).unwrap() + XDG_STATE_HOME_PATH
        }));
    }

    GenericPaths { config, local, cache, state }
}
