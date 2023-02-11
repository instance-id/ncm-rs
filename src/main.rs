mod args;
mod configs;
mod logger;

use configs::ConfigData;

use clap::Parser;
use log::{debug, error, info};
use std::env::var;
use std::path::PathBuf;
use std::str::FromStr;
use ansi_term::Colour::RGB;

use crate::args::{Commands, NvCfgArgs};

#[macro_use]
extern crate prettytable;
use prettytable::{
    color,
    format::Alignment,
    Attr, Cell, Row, Table,
};

fn main() -> anyhow::Result<()> {
    // Initialize logger
    logger::init().expect("Could not initialize logger");

    let args = NvCfgArgs::parse();

    let mut nvim_path = PathBuf::new();
    let mut path = PathBuf::new();
    let settings_file = "settings.toml";
    let configs_json = "configs.json";

    if let Ok(cfg) = var("XDG_CONFIG_HOME") {
        path.push(&cfg);
        path.push("nvcfg");

        nvim_path.push(&cfg);
        nvim_path.push("nvim");
    } else {
        path.push(var("HOME").unwrap());
        path.push(".config");
        path.push("nvcfg");

        nvim_path.push(var("HOME").unwrap());
        nvim_path.push(".config");
        nvim_path.push("nvim");
    }

    let settings_path = path.join(settings_file);
    let configs_json_path = path.join(configs_json);

    if !settings_path.exists() {
        std::fs::create_dir_all(settings_path.parent().unwrap())?;
        std::fs::File::create(&settings_path)?;
    }

    // --| Create configs json file and parent directories if it doesn't exist
    if !configs_json_path.exists() {
        std::fs::create_dir_all(configs_json_path.parent().unwrap())?;
        std::fs::File::create(&configs_json_path)?;
    }

    let config_json = configs_json_path.to_str().unwrap();
    match &args.command {
        Commands::SetDefault { name } => configs::set_default(config_json, name)?,
        Commands::Add { name, path, description } => {
            info!("Adding new config: {name:?} {path:?} {description:?}");

            configs::add_config(
                config_json,
                ConfigData {
                    name: name.clone(),
                    path: path.to_str().unwrap().to_string(),
                    description: description.clone(),
                },
            )?;
        }

        Commands::Remove { name } => {
            configs::remove_config(config_json, &name.clone().unwrap())?;
        }
        Commands::List => {
            let cfgs = configs::list_configs(config_json)?;
            let current_default = format!("Current Default: {}", cfgs.configs_default);

            let current_str = RGB(70, 130, 180).paint("Current Configurations");
            println!("{}",current_str);
            println!(""); // There is probably a better way to do this, but I don't know what it is...

            let name_str = RGB(70, 130, 180).paint("Name");
            let path_str = RGB(70, 130, 180).paint("Path");
            let desc_str = RGB(70, 130, 180).paint("Decription");

            let mut table = Table::new();
            table.set_titles(row![b->name_str, b->path_str, b->desc_str]);

            table.set_format(*prettytable::format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
            for cfg in cfgs.configs {
                table.add_row(row![cfg.name, cfg.path, cfg.description.unwrap_or("".to_string())]);
            }

            table.add_row(Row::new(vec![Cell::new_align(&current_default, Alignment::LEFT)
                                   .with_style(Attr::Bold)
                                   .with_style(Attr::ForegroundColor(color::GREEN))
                                   .with_hspan(3)]));
            table.printstd();
        }
        Commands::Load { name } => {
            let cfg = configs::load_configs(config_json, &name.clone().unwrap())?;
            info!("Loading: {:?}", cfg.name);

            let config_path = PathBuf::from_str(&cfg.path).ok();

            if let Some(new_config) = config_path {
                let new_config = new_config.canonicalize()?;

                if !new_config.exists() {
                    return Err(anyhow::anyhow!("Config file does not exist"));
                }

                match (nvim_path.exists(), nvim_path.is_dir(), true) {
                    (true, true, true) => std::fs::remove_dir_all(&nvim_path)?,
                    (true, false, true) => std::fs::remove_file(&nvim_path)?,
                    (true, _, false) => return Err(anyhow::anyhow!("Cannot overwrite file with directory")),
                    _ => {}
                };

                std::os::unix::fs::symlink(new_config, nvim_path)?;
            } else {
                return Err(anyhow::anyhow!("Config file does not exist"));
            }
        }
    }

    Ok(())
}
