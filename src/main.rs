//! DX Media CLI - Universal Digital Asset Acquisition
//!
//! Usage:
//!   dx search "sunset mountains" --type image
//!   dx download openverse:abc123
//!   dx providers --available

use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    // Load dx-config.toml and create required directories
    let dx_config = dx_media::dx_config::MediaDxConfig::load(None);
    let _ = std::fs::create_dir_all(&dx_config.sr_dir_abs());
    let _ = std::fs::create_dir_all(&dx_config.receipts_dir_abs());

    match dx_media::cli::run().await {
        Ok(()) => {
            let _ = dx_config.write_sr("media", &[("tool", "media"), ("action", "run"), ("status", "ok")]);
            let _ = dx_config.write_global_sr("media", &[("tool", "media"), ("action", "run"), ("status", "ok")]);
            if let Some(status) = dx_config.read_status("media") {
                eprintln!("[media] sr cache verified: {} entries", status.len());
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");

            // Print chain of errors
            let mut source = std::error::Error::source(&e);
            while let Some(cause) = source {
                eprintln!("  Caused by: {cause}");
                source = std::error::Error::source(cause);
            }

            ExitCode::FAILURE
        }
    }
}
