use anyhow::Result;
use std::fs::File;
use std::io::{self, Write};

pub fn generate_completions(shell: &str, out_path: Option<&str>) -> Result<()> {
    use clap::CommandFactory;
    use clap_complete::{generate, shells::{Bash, Zsh}};

    // Build the clap Command from the Args derive
    let mut cmd = crate::cli::Args::command();

    let mut writer: Box<dyn Write> = if let Some(p) = out_path {
        Box::new(File::create(p)?)
    } else {
        Box::new(io::stdout())
    };

    match shell.to_lowercase().as_str() {
        "bash" => {
            generate(Bash, &mut cmd, "check_vpn", &mut writer);
        }
        "zsh" => {
            generate(Zsh, &mut cmd, "check_vpn", &mut writer);
        }
        other => anyhow::bail!("unsupported shell: {} (supported: bash, zsh)", other),
    }
    Ok(())
}
