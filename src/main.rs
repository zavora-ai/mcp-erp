//! mcp-erp — Enterprise ERP MCP Server
mod types;
mod server;

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
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    // Validate manifest
    let manifest = adk_mcp_sdk::ServerManifest::from_file(std::path::Path::new("mcp-server.toml"))?;
    let errors = manifest.validate();
    if !errors.is_empty() {
        for e in &errors { tracing::error!("manifest: {e}"); }
        anyhow::bail!("invalid mcp-server.toml ({} error(s))", errors.len());
    }

    // Backend selection — first configured wins
    let backend: Arc<dyn types::ErpBackend> = init_backend().await?;

    tracing::info!("{} v{} starting on stdio (backend: {})", manifest.display_name, manifest.version, backend.name());
    let server = ErpServer { backend };
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

async fn init_backend() -> anyhow::Result<Arc<dyn types::ErpBackend>> {
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

    anyhow::bail!("No ERP backend configured. Set env vars for one of: ZOHO_TOKEN+ZOHO_ORG_ID, ODOO_URL+ODOO_DB+ODOO_USER+ODOO_PASSWORD, BC_TENANT_ID+BC_ENVIRONMENT+BC_COMPANY_ID+BC_TOKEN, NETSUITE_ACCOUNT_ID+..., SAP_BASE_URL+SAP_TOKEN")
}
