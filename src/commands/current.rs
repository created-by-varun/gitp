use anyhow::Result;
use colored::Colorize;

use crate::git::{get_git_config, GitConfigScope};

fn print_config_value(label: &str, local_val: Option<String>, global_val: Option<String>) {
    match (local_val, global_val) {
        (Some(l), _) => println!("  {}: {} {}", label.dimmed(), l.green(), "(local)".cyan()),
        (None, Some(g)) => println!("  {}: {} {}", label.dimmed(), g.green(), "(global)".blue()),
        (None, None) => println!("  {}: {}", label.dimmed(), "Not set".yellow()),
    }
}

pub fn execute() -> Result<()> {
    println!("{}", "Current Git Configuration:".bold().underline());

    let user_name_local = get_git_config("user.name", GitConfigScope::Local)?;
    let user_name_global = get_git_config("user.name", GitConfigScope::Global)?;
    print_config_value("User Name", user_name_local, user_name_global);

    let user_email_local = get_git_config("user.email", GitConfigScope::Local)?;
    let user_email_global = get_git_config("user.email", GitConfigScope::Global)?;
    print_config_value("User Email", user_email_local, user_email_global);

    let signing_key_local = get_git_config("user.signingkey", GitConfigScope::Local)?;
    let signing_key_global = get_git_config("user.signingkey", GitConfigScope::Global)?;
    print_config_value("Signing Key", signing_key_local, signing_key_global);

    println!(
        "\n{}",
        "Note: Values are read directly from Git. Local settings override global settings."
            .dimmed()
    );

    Ok(())
}
