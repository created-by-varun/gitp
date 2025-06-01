use anyhow::Result;
use clap::Parser;
use colored::Colorize;

mod cli;
mod commands;
mod config;
mod git;
mod utils;

use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up colored output based on environment
    colored::control::set_override(cli.color);

    match run(cli) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::New {
            name,
            user_name,
            user_email,
            signing_key,
            ssh_key_path,
            gpg_key_id,
        } => {
            commands::new::execute(
                name,
                user_name,
                user_email,
                signing_key,
                ssh_key_path,
                gpg_key_id,
            )?;
        }
        Commands::List { verbose } => {
            commands::list::execute(verbose)?;
        }
        Commands::Use {
            name,
            local,
            global,
        } => {
            commands::use_profile::execute(name, local, global)?;
        }
        Commands::Current => {
            commands::current::execute()?;
        }
        Commands::Show { name } => {
            commands::show::execute(name)?;
        }
        Commands::Edit {
            name,
            user_name,
            user_email,
            signing_key,
            ssh_key_path,
            gpg_key_id,
        } => {
            commands::edit::execute(
                name,
                user_name,
                user_email,
                signing_key,
                ssh_key_path,
                gpg_key_id,
            )?;
        }
        Commands::Remove { name, force } => {
            commands::remove::execute(name, force)?;
        }
        Commands::Rename { old_name, new_name } => {
            commands::rename::execute(old_name, new_name)?;
        }
        Commands::SshKey { command } => {
            commands::ssh_key::execute(command)?;
        }
    }

    Ok(())
}
