mod cmds;
mod backup;
mod logger;
mod configs;
mod settings;
mod constants;

use constants::*;
use crate::cmds::{Commands, NvCfgArgs};

use anyhow::{Result};
use clap::Parser;

#[macro_use] extern crate log;
extern crate simplelog;

#[macro_use]
extern crate prettytable;

#[macro_use]
extern crate lazy_static;

use std::sync::{RwLock};


// --| Global Settings ---------------------
lazy_static! {
    pub static ref SETTINGS: RwLock<settings::Settings> = RwLock::new(settings::get_settings(XDG_CONFIG_HOME, HOME));
}

fn main() -> Result<()> {
    logger::initialize();

    let settings = &mut SETTINGS.write().unwrap();
    let setup_complete = settings.settings.getbool(NCM, SETUP_COMPLETE)
        .unwrap()
        .expect(ERR_SETTINGS_READ);

    // --| Check if setup is needed --------
    if cmds::check_setup(settings, setup_complete).is_err() {
        info!("{}", ERR_RUN_SETUP);
        return Ok(());
    }

    let config_json = &settings.configs_path.to_str().unwrap().to_string();

    // --| Parse Arguments -----------------
    let args = NvCfgArgs::parse();

    match &args.command {
        // --| Add Command -----------------
        Commands::Add { name, path, description } => {
            cmds::add_config(name, path, description, config_json);
        }

        // --| Remove Command --------------
        Commands::Remove { name } => {
            configs::remove_config(name, config_json)?;
        }

        // --| List Command ----------------
        Commands::List => {
            cmds::list_configs(config_json);
        }

        // --| Load Command ----------------
        Commands::Load { name } => {
            cmds::load_config(name, &mut settings.nvim_path, config_json);
        }

        // --| Backup Command --------------
        Commands::Backup { name } => {
            cmds::initiate_backup(name, settings);
        }

        Commands::Setup => {
            info!("Setup");
        }
    }

    Ok(())
}
