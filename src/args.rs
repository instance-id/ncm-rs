use std::path::PathBuf;
use clap::{arg, command, value_parser, Subcommand, ArgAction, Parser};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[clap(about = r#"Neovim Configuration Manager.

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
    /// Set default configuration in which to load if not specified
    SetDefault { name: String }, 
    /// List current stored configurations
    List,
}
