use anyhow::{Context, Result, bail};
use std::env;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct NordLayerCli {
    bin: String,
}

impl Default for NordLayerCli {
    fn default() -> Self {
        Self {
            bin: env::var("NORDLAYER_BIN").unwrap_or_else(|_| "nordlayer".to_string()),
        }
    }
}

impl NordLayerCli {
    pub fn run(&self, args: &[&str]) -> Result<String> {
        let output = Command::new(&self.bin)
            .args(args)
            .output()
            .with_context(|| format!("failed to start '{}'", self.bin))?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

        if !output.status.success() {
            let message = if stderr.is_empty() {
                format!("command failed: {} {}", self.bin, args.join(" "))
            } else {
                stderr
            };
            bail!(message);
        }

        Ok(stdout)
    }
}

