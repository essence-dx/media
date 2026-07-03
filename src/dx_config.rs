//! DX Config - Inline TOML-based configuration for the media tool.
//!
//! Loads `dx-config.toml` from the current directory or parent directories.

use std::collections::HashMap;
use serde::Deserialize;
use std::io::Write;
use std::path::PathBuf;

/// Media-specific DX configuration loaded from `dx-config.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct MediaDxConfig {
    /// Root workspace directory (e.g., `G:\Dx`).
    #[serde(default = "default_workspace_root")]
    pub workspace_root: PathBuf,

    /// Directory for source-record files relative to `workspace_root`.
    #[serde(default = "default_sr_dir")]
    pub sr_dir: PathBuf,

    /// Directory for provenance receipts relative to `workspace_root`.
    #[serde(default = "default_receipts_dir")]
    pub receipts_dir: PathBuf,
}

fn default_workspace_root() -> PathBuf {
    PathBuf::from("G:\\Dx")
}

fn default_sr_dir() -> PathBuf {
    PathBuf::from("sr")
}

fn default_receipts_dir() -> PathBuf {
    PathBuf::from("receipts")
}

impl MediaDxConfig {
    /// Load config from `dx-config.toml`, walking up from `start` to find it.
    /// Returns a fully resolved config with all paths absolute.
    pub fn load(start: Option<PathBuf>) -> Self {
        let start = start.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        let found = find_config_file(&start);
        let (mut config, config_dir) = match found {
            Some(path) => {
                let content = std::fs::read_to_string(&path).unwrap_or_default();
                let config = toml::from_str::<MediaDxConfig>(&content).unwrap_or_default();
                let dir = path.parent().map(|p| p.to_path_buf());
                (config, dir)
            }
            None => (MediaDxConfig::default(), None),
        };

        // Resolve workspace_root relative to config file location
        if let Some(config_dir) = config_dir {
            if config.workspace_root.is_relative() {
                config.workspace_root = config_dir.join(&config.workspace_root);
            }
        }

        config
    }

    /// Full path to sr_dir.
    pub fn sr_dir_abs(&self) -> PathBuf {
        self.workspace_root.join(&self.sr_dir)
    }

    /// Full path to receipts_dir.
    pub fn receipts_dir_abs(&self) -> PathBuf {
        self.workspace_root.join(&self.receipts_dir)
    }

    /// Full path to a .sr file.
    pub fn sr_path(&self, name: &str) -> PathBuf {
        self.sr_dir_abs().join(format!("{}.sr", name))
    }

    pub fn write_sr(&self, name: &str, entries: &[(&str, &str)]) -> std::io::Result<()> {
        let path = self.sr_path(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut buf: Vec<u8> = Vec::new();
        for (key, value) in entries {
            write!(buf, "{key}=")?;
            Self::write_llm_value(&mut buf, value)?;
            buf.push(b'\n');
        }
        let tmp = path.with_extension("sr.tmp");
        std::fs::write(&tmp, &buf)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    pub fn read_status(&self, name: &str) -> Option<HashMap<String, String>> {
        let sr_path = self.sr_path(name);
        let (doc, _from_machine) = serializer::try_read_machine_or_sr(&sr_path)?;
        let mut map = HashMap::new();
        for (key, value) in &doc.context {
            map.insert(key.clone(), value.to_string());
        }
        Some(map)
    }

    /// Global cache directory for source-record files.
    pub fn global_sr_dir(&self) -> PathBuf {
        dirs::cache_dir()
            .map(|b| b.join("dx").join("media"))
            .unwrap_or_else(|| PathBuf::from("~/.cache/dx/media"))
    }

    pub fn machine_path(&self, name: &str) -> PathBuf {
        self.sr_dir_abs().join(format!("{}.machine", name))
    }

    pub fn write_global_sr(&self, name: &str, entries: &[(&str, &str)]) -> std::io::Result<()> {
        let path = self.global_sr_dir().join(format!("{}.sr", name));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut buf: Vec<u8> = Vec::new();
        for (key, value) in entries {
            write!(buf, "{key}=")?;
            Self::write_llm_value(&mut buf, value)?;
            buf.push(b'\n');
        }
        let tmp = path.with_extension("sr.tmp");
        std::fs::write(&tmp, &buf)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    fn write_llm_value(buf: &mut Vec<u8>, value: &str) -> std::io::Result<()> {
        if value.is_empty() {
            buf.extend_from_slice(b"\"\"");
            return Ok(());
        }
        let needs_quoting = value.contains(|c: char| {
            c.is_ascii_whitespace() || c == '"' || c == '[' || c == ']' || c == '=' || c == '#'
        });
        if needs_quoting {
            buf.push(b'"');
            for c in value.chars() {
                if c == '"' || c == '\\' { buf.push(b'\\'); }
                let mut tmp = [0u8; 4];
                buf.extend_from_slice(c.encode_utf8(&mut tmp).as_bytes());
            }
            buf.push(b'"');
        } else {
            buf.extend_from_slice(value.as_bytes());
        }
        Ok(())
    }
}

impl Default for MediaDxConfig {
    fn default() -> Self {
        Self {
            workspace_root: default_workspace_root(),
            sr_dir: default_sr_dir(),
            receipts_dir: default_receipts_dir(),
        }
    }
}

/// Walk up from `dir` looking for `dx-config.toml`.
fn find_config_file(dir: &std::path::Path) -> Option<PathBuf> {
    let candidate = dir.join("dx-config.toml");
    if candidate.is_file() {
        return Some(candidate);
    }
    if let Some(parent) = dir.parent() {
        return find_config_file(parent);
    }
    None
}
