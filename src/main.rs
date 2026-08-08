//! mcp-erp — Enterprise ERP MCP Server
mod types;
mod server;
mod auth;

#[cfg(feature = "zavora")]
mod zavora;
#[cfg(feature = "zoho")]
mod zoho;
#[cfg(feature = "odoo")]
mod odoo;
#[cfg(feature = "business-central")]
mod business_central;
#[cfg(feature = "netsuite")]
mod netsuite;
#[cfg(feature = "sap")]
mod sap;

use rmcp::{ServiceExt, transport::stdio};
use server::ErpServer;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        // stdout is the MCP wire; diagnostics must never corrupt JSON-RPC.
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Validate manifest — check the cwd first, then fall back to the crate root
    // relative to the executable (target/{profile}/mcp-erp → ../../mcp-server.toml)
    // so the server can be spawned from any working directory.
    let manifest_path = ["mcp-server.toml"]
        .iter()
        .map(std::path::PathBuf::from)
        .chain(std::env::current_exe().ok().and_then(|exe| {
            exe.parent().and_then(|p| p.parent()).and_then(|p| p.parent()).map(|root| root.join("mcp-server.toml"))
        }))
        .find(|p| p.exists())
        .ok_or_else(|| anyhow::anyhow!("mcp-server.toml not found in cwd or next to the executable"))?;
    let manifest = adk_mcp_sdk::ServerManifest::from_file(&manifest_path)?;
    let errors = manifest.validate();
    if !errors.is_empty() {
        for e in &errors { tracing::error!("manifest: {e}"); }
        anyhow::bail!("invalid mcp-server.toml ({} error(s))", errors.len());
    }

    // Stdio carries no identity of its own. Require an explicit deployment
    // model so a shared agent cannot accidentally run with a service account.
    let auth = auth::AuthConfig::from_env()?;

    // Backend selection — first configured wins
    let backend: Arc<dyn types::ErpBackend> = init_backend(&auth).await?;

    tracing::info!("{} v{} starting on stdio (backend: {})", manifest.display_name, manifest.version, backend.name());
    let server = ErpServer { backend, auth };
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

async fn init_backend(auth: &auth::AuthConfig) -> anyhow::Result<Arc<dyn types::ErpBackend>> {
    // Zavora ERA
    #[cfg(feature = "zavora")]
    if let Ok(url) = std::env::var("ZAVORA_API_URL") {
        tracing::info!("Using Zavora ERA backend at {url}");
        return match auth.mode() {
            auth::AuthMode::Delegated => Ok(Arc::new(zavora::ZavoraBackend::delegated(url, auth.clone()))),
            auth::AuthMode::TrustedSingleUser => {
                let email = std::env::var("ZAVORA_EMAIL")?;
                let pass = std::env::var("ZAVORA_PASSWORD")?;
                Ok(Arc::new(zavora::ZavoraBackend::trusted_single_user(url, email, pass, auth.clone())))
            }
        };
    }

    if auth.is_delegated() {
        anyhow::bail!("delegated authentication currently requires the Zavora backend and ZAVORA_API_URL");
    }

    // Zoho
    #[cfg(feature = "zoho")]
    if let (Ok(token), Ok(org)) = (std::env::var("ZOHO_TOKEN"), std::env::var("ZOHO_ORG_ID")) {
        tracing::info!("Using Zoho Books backend");
        return Ok(Arc::new(zoho::ZohoBackend::new(token, org)));
    }

    // Odoo
    #[cfg(feature = "odoo")]
    if let (Ok(url), Ok(db), Ok(user), Ok(pass)) = (std::env::var("ODOO_URL"), std::env::var("ODOO_DB"), std::env::var("ODOO_USER"), std::env::var("ODOO_PASSWORD")) {
        tracing::info!("Connecting to Odoo at {url}");
        return Ok(Arc::new(odoo::OdooBackend::connect(url, db, user, pass).await?));
    }

    // Business Central
    #[cfg(feature = "business-central")]
    if let (Ok(tenant), Ok(env), Ok(company), Ok(token)) = (std::env::var("BC_TENANT_ID"), std::env::var("BC_ENVIRONMENT"), std::env::var("BC_COMPANY_ID"), std::env::var("BC_TOKEN")) {
        tracing::info!("Using Business Central backend");
        return Ok(Arc::new(business_central::BusinessCentralBackend::new(tenant, env, company, token)));
    }

    // NetSuite
    #[cfg(feature = "netsuite")]
    if let (Ok(acct), Ok(ck), Ok(cs), Ok(ti), Ok(ts)) = (std::env::var("NETSUITE_ACCOUNT_ID"), std::env::var("NETSUITE_CONSUMER_KEY"), std::env::var("NETSUITE_CONSUMER_SECRET"), std::env::var("NETSUITE_TOKEN_ID"), std::env::var("NETSUITE_TOKEN_SECRET")) {
        tracing::info!("Using NetSuite backend");
        return Ok(Arc::new(netsuite::NetSuiteBackend::new(acct, ck, cs, ti, ts)));
    }

    // SAP
    #[cfg(feature = "sap")]
    if let (Ok(url), Ok(token)) = (std::env::var("SAP_BASE_URL"), std::env::var("SAP_TOKEN")) {
        tracing::info!("Using SAP S/4HANA backend");
        return Ok(Arc::new(sap::SapBackend::new(url, token)));
    }

    anyhow::bail!("No ERP backend configured. Set env vars for one of: ZAVORA_API_URL (+ ZAVORA_EMAIL+ZAVORA_PASSWORD in trusted-single-user mode), ZOHO_TOKEN+ZOHO_ORG_ID, ODOO_URL+ODOO_DB+ODOO_USER+ODOO_PASSWORD, BC_TENANT_ID+BC_ENVIRONMENT+BC_COMPANY_ID+BC_TOKEN, NETSUITE_ACCOUNT_ID+..., SAP_BASE_URL+SAP_TOKEN")
}
