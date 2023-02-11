use std::path::PathBuf;
use clap::{arg, command, value_parser, Subcommand, ArgAction, Parser};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[clap(about = r#"Neovim Configuration Manager.

EXAMPLES:
    Set default config path /target/path to current directory
    $ ncm -d /target/path

    create a symlink at ./my_path pointing to /other/path
    $ ncm ./my_path /other/path"#)]
pub struct NvCfgArgs {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Adds new config 
    Add { name: String, path: PathBuf, description: Option<String> },
    /// Removes files from myapp
    Remove { name: Option<String> },
    /// Loads files from myapp
    Load { name: Option<String> },
    /// Set default config
    SetDefault { name: String }, 
    /// List current configs
    List,
}
