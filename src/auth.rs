//! Authentication boundary for MCP tool calls.
//!
//! `mcp-erp` is a stdio server, so the transport itself has no user identity.
//! Deployments must therefore choose one of two explicit modes:
//!
//! - `trusted-single-user`: the configured ERP credential belongs to the one
//!   trusted user/process controlling this stdio server.
//! - `delegated`: every tool call must carry an opaque path to a short-lived
//!   bearer file under a locked credential directory. The bearer itself never
//!   appears in the MCP arguments and a missing/invalid credential fails closed.

use anyhow::{Context, Result, anyhow, bail};
use std::path::{Path, PathBuf};

/// Hidden argument injected by a trusted MCP host after model generation.
/// It is removed before typed tool arguments are deserialized.
pub const CREDENTIAL_FILE_ARG: &str = "__credential_file";

tokio::task_local! {
    static CREDENTIAL_FILE: Option<PathBuf>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthMode {
    TrustedSingleUser,
    Delegated,
}

#[derive(Clone, Debug)]
pub struct AuthConfig {
    mode: AuthMode,
    credential_root: Option<PathBuf>,
}

impl AuthConfig {
    pub fn from_env() -> Result<Self> {
        match std::env::var("MCP_ERP_AUTH_MODE").as_deref() {
            Ok("trusted-single-user") => Ok(Self {
                mode: AuthMode::TrustedSingleUser,
                credential_root: None,
            }),
            Ok("delegated") => {
                let configured = std::env::var("MCP_ERP_CREDENTIAL_DIR")
                    .context("MCP_ERP_CREDENTIAL_DIR is required in delegated mode")?;
                Self::delegated(PathBuf::from(configured))
            }
            Ok(other) => bail!(
                "invalid MCP_ERP_AUTH_MODE '{other}'; expected 'trusted-single-user' or 'delegated'"
            ),
            Err(_) => bail!(
                "MCP_ERP_AUTH_MODE must be set explicitly to 'trusted-single-user' or 'delegated'"
            ),
        }
    }

    pub fn delegated(root: PathBuf) -> Result<Self> {
        let root = root
            .canonicalize()
            .with_context(|| format!("credential directory '{}' is unavailable", root.display()))?;
        let metadata = std::fs::metadata(&root)?;
        if !metadata.is_dir() {
            bail!("credential root '{}' is not a directory", root.display());
        }
        validate_private_permissions(&metadata, "credential directory")?;
        Ok(Self {
            mode: AuthMode::Delegated,
            credential_root: Some(root),
        })
    }

    pub fn mode(&self) -> AuthMode {
        self.mode
    }

    pub fn is_delegated(&self) -> bool {
        self.mode == AuthMode::Delegated
    }

    /// Read the caller bearer for the current tool invocation.
    ///
    /// The path must resolve under the configured credential root, must be a
    /// private regular file, and must contain one compact JWT. No service
    /// credential fallback is permitted in delegated mode.
    pub async fn delegated_bearer(&self) -> Result<String> {
        if self.mode != AuthMode::Delegated {
            bail!("delegated bearer requested in trusted-single-user mode");
        }
        let requested = current_credential_file()
            .ok_or_else(|| anyhow!("delegated tool call is missing its caller credential"))?;
        let canonical = tokio::fs::canonicalize(&requested)
            .await
            .context("delegated caller credential is unavailable")?;
        let root = self
            .credential_root
            .as_ref()
            .expect("delegated mode has a root");
        if canonical.parent() != Some(root.as_path()) {
            bail!("delegated caller credential is outside the configured credential directory");
        }

        let file = tokio::fs::File::open(&canonical)
            .await
            .context("delegated caller credential could not be opened")?;
        let metadata = file.metadata().await?;
        if !metadata.is_file() || metadata.len() == 0 || metadata.len() > 16 * 1024 {
            bail!("delegated caller credential is not a valid token file");
        }
        validate_private_permissions(&metadata, "credential file")?;
        validate_same_owner(&metadata, &std::fs::metadata(root)?)?;

        use tokio::io::AsyncReadExt;
        let mut token = String::with_capacity(metadata.len() as usize);
        file.take(16 * 1024).read_to_string(&mut token).await?;
        let token = token.trim();
        if token.split('.').count() != 3 || token.chars().any(char::is_whitespace) {
            bail!("delegated caller credential is not a compact JWT");
        }
        Ok(token.to_owned())
    }
}

#[cfg(unix)]
fn validate_private_permissions(metadata: &std::fs::Metadata, label: &str) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    if metadata.permissions().mode() & 0o077 != 0 {
        bail!("{label} must not be accessible by group or other users");
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_private_permissions(_metadata: &std::fs::Metadata, _label: &str) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn validate_same_owner(file: &std::fs::Metadata, root: &std::fs::Metadata) -> Result<()> {
    use std::os::unix::fs::MetadataExt;
    if file.uid() != root.uid() {
        bail!("credential file owner does not match credential directory owner");
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_same_owner(_file: &std::fs::Metadata, _root: &std::fs::Metadata) -> Result<()> {
    Ok(())
}

pub fn take_credential_file(
    arguments: &mut serde_json::Map<String, serde_json::Value>,
) -> Option<PathBuf> {
    match arguments.remove(CREDENTIAL_FILE_ARG) {
        Some(serde_json::Value::String(path)) if !path.trim().is_empty() => {
            Some(Path::new(&path).to_path_buf())
        }
        _ => None,
    }
}

pub fn current_credential_file() -> Option<PathBuf> {
    CREDENTIAL_FILE.try_with(Clone::clone).ok().flatten()
}

pub async fn scope_credential<F: std::future::Future>(
    credential: Option<PathBuf>,
    future: F,
) -> F::Output {
    CREDENTIAL_FILE.scope(credential, future).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs::OpenOptions;

    #[test]
    fn credential_reference_is_removed_from_arguments() {
        let mut args = json!({"id": "invoice-1", CREDENTIAL_FILE_ARG: "/tmp/caller.jwt"})
            .as_object()
            .unwrap()
            .clone();
        assert_eq!(
            take_credential_file(&mut args),
            Some(PathBuf::from("/tmp/caller.jwt"))
        );
        assert!(!args.contains_key(CREDENTIAL_FILE_ARG));
        assert_eq!(args["id"], "invoice-1");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn delegated_bearer_rejects_files_outside_private_root() {
        use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};

        let base = std::env::temp_dir().join(format!("mcp-erp-auth-{}", std::process::id()));
        let root = base.join("private");
        let outside = base.join("outside.jwt");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::DirBuilder::new()
            .mode(0o700)
            .recursive(true)
            .create(&root)
            .unwrap();
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&outside)
            .unwrap();
        use std::io::Write;
        file.write_all(b"aaa.bbb.ccc").unwrap();

        let config = AuthConfig::delegated(root).unwrap();
        let error = scope_credential(Some(outside), config.delegated_bearer())
            .await
            .unwrap_err();
        assert!(error.to_string().contains("outside"));
        std::fs::remove_dir_all(base).unwrap();
    }
}
