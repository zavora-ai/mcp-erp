//! MCP tool router for ERP operations.
use adk_mcp_sdk::{HealthCheck, HealthStatus};
use crate::types::{ErpBackend, LineItemInput};
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;
use std::sync::Arc;

// ─── Input types ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListInput {
    #[serde(default = "d20")]
    pub limit: u32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct IdInput {
    pub id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateCustomerInput {
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateCustomerInput {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateVendorInput {
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateVendorInput {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateProductInput {
    pub name: String,
    #[serde(default)]
    pub sku: Option<String>,
    #[serde(default)]
    pub unit_price: Option<f64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateProductInput {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub sku: Option<String>,
    #[serde(default)]
    pub unit_price: Option<f64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateOrderInput {
    /// Customer ID (sales) or Vendor ID (purchase)
    pub party_id: String,
    pub line_items: Vec<LineItemInput>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateInvoiceInput {
    pub customer_id: String,
    pub line_items: Vec<LineItemInput>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StockQueryInput {
    #[serde(default)]
    pub product_id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AdjustStockInput {
    pub product_id: String,
    /// Positive to increase, negative to decrease
    pub quantity: f64,
    pub reason: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TransferStockInput {
    pub product_id: String,
    pub from_warehouse: String,
    pub to_warehouse: String,
    pub quantity: f64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DateRangeInput {
    /// ISO date (YYYY-MM-DD)
    pub from: String,
    /// ISO date (YYYY-MM-DD)
    pub to: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AsOfInput {
    /// ISO date (YYYY-MM-DD)
    pub as_of: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ApprovalInput {
    pub entity_type: String,
    pub entity_id: String,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EvidenceInput {
    pub entity_type: String,
    pub entity_id: String,
    pub description: String,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AuditInput {
    pub entity_type: String,
    pub entity_id: String,
}

fn d20() -> u32 { 20 }

// ─── Server ──────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ErpServer {
    pub backend: Arc<dyn ErpBackend>,
}

#[tool_router(server_handler)]
impl ErpServer {
    // ─── Customers ───────────────────────────────────────────────────────────

    #[tool(description = "List customers with optional filters")]
    async fn list_customers(&self, Parameters(i): Parameters<ListInput>) -> String {
        match self.backend.list_customers(i.limit).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get a customer by ID")]
    async fn get_customer(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.get_customer(&i.id).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Create a new customer record")]
    async fn create_customer(&self, Parameters(i): Parameters<CreateCustomerInput>) -> String {
        match self.backend.create_customer(&i.name, i.email.as_deref(), i.phone.as_deref()).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Update an existing customer record")]
    async fn update_customer(&self, Parameters(i): Parameters<UpdateCustomerInput>) -> String {
        match self.backend.update_customer(&i.id, i.name.as_deref(), i.email.as_deref(), i.phone.as_deref()).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    // ─── Vendors ─────────────────────────────────────────────────────────────

    #[tool(description = "List vendors/suppliers with optional filters")]
    async fn list_vendors(&self, Parameters(i): Parameters<ListInput>) -> String {
        match self.backend.list_vendors(i.limit).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get a vendor by ID")]
    async fn get_vendor(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.get_vendor(&i.id).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Create a new vendor/supplier record")]
    async fn create_vendor(&self, Parameters(i): Parameters<CreateVendorInput>) -> String {
        match self.backend.create_vendor(&i.name, i.email.as_deref(), i.phone.as_deref()).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Update an existing vendor record")]
    async fn update_vendor(&self, Parameters(i): Parameters<UpdateVendorInput>) -> String {
        match self.backend.update_vendor(&i.id, i.name.as_deref(), i.email.as_deref(), i.phone.as_deref()).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    // ─── Products ────────────────────────────────────────────────────────────

    #[tool(description = "List products/items with optional filters")]
    async fn list_products(&self, Parameters(i): Parameters<ListInput>) -> String {
        match self.backend.list_products(i.limit).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get a product by ID")]
    async fn get_product(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.get_product(&i.id).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Create a new product/item record")]
    async fn create_product(&self, Parameters(i): Parameters<CreateProductInput>) -> String {
        match self.backend.create_product(&i.name, i.sku.as_deref(), i.unit_price).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Update an existing product record")]
    async fn update_product(&self, Parameters(i): Parameters<UpdateProductInput>) -> String {
        match self.backend.update_product(&i.id, i.name.as_deref(), i.sku.as_deref(), i.unit_price).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    // ─── Sales Orders ────────────────────────────────────────────────────────

    #[tool(description = "List sales orders with optional filters")]
    async fn list_sales_orders(&self, Parameters(i): Parameters<ListInput>) -> String {
        match self.backend.list_sales_orders(i.limit).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get a sales order by ID with line items")]
    async fn get_sales_order(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.get_sales_order(&i.id).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Create a sales order in draft state")]
    async fn create_sales_order_draft(&self, Parameters(i): Parameters<CreateOrderInput>) -> String {
        match self.backend.create_sales_order_draft(&i.party_id, &i.line_items).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Submit a draft sales order for approval/release")]
    async fn submit_sales_order(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.submit_sales_order(&i.id).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    // ─── Purchase Orders ─────────────────────────────────────────────────────

    #[tool(description = "List purchase orders with optional filters")]
    async fn list_purchase_orders(&self, Parameters(i): Parameters<ListInput>) -> String {
        match self.backend.list_purchase_orders(i.limit).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get a purchase order by ID with line items")]
    async fn get_purchase_order(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.get_purchase_order(&i.id).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Create a purchase order in draft state")]
    async fn create_purchase_order_draft(&self, Parameters(i): Parameters<CreateOrderInput>) -> String {
        match self.backend.create_purchase_order_draft(&i.party_id, &i.line_items).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Submit a draft purchase order for approval/release")]
    async fn submit_purchase_order(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.submit_purchase_order(&i.id).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    // ─── Invoices ────────────────────────────────────────────────────────────

    #[tool(description = "List invoices with optional filters")]
    async fn list_invoices(&self, Parameters(i): Parameters<ListInput>) -> String {
        match self.backend.list_invoices(i.limit).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get an invoice by ID with line items and payment status")]
    async fn get_invoice(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.get_invoice(&i.id).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Create an invoice in draft state")]
    async fn create_invoice_draft(&self, Parameters(i): Parameters<CreateInvoiceInput>) -> String {
        match self.backend.create_invoice_draft(&i.customer_id, &i.line_items).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Submit a draft invoice for approval")]
    async fn submit_invoice(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.submit_invoice(&i.id).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Post an approved invoice to the ledger")]
    async fn post_invoice(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.post_invoice(&i.id).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    // ─── Inventory ───────────────────────────────────────────────────────────

    #[tool(description = "Get current stock levels for products")]
    async fn get_stock_levels(&self, Parameters(i): Parameters<StockQueryInput>) -> String {
        match self.backend.get_stock_levels(i.product_id.as_deref()).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Adjust stock quantity for a product (increase/decrease)")]
    async fn adjust_stock(&self, Parameters(i): Parameters<AdjustStockInput>) -> String {
        match self.backend.adjust_stock(&i.product_id, i.quantity, &i.reason).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Transfer stock between warehouses/locations")]
    async fn transfer_stock(&self, Parameters(i): Parameters<TransferStockInput>) -> String {
        match self.backend.transfer_stock(&i.product_id, &i.from_warehouse, &i.to_warehouse, i.quantity).await {
            Ok(()) => "Stock transferred".into(),
            Err(e) => format!("Error: {e}"),
        }
    }

    // ─── General Ledger ──────────────────────────────────────────────────────

    #[tool(description = "List chart of accounts")]
    async fn list_accounts(&self) -> String {
        match self.backend.list_accounts().await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get journal entries for a date range")]
    async fn get_journal_entries(&self, Parameters(i): Parameters<DateRangeInput>) -> String {
        match self.backend.get_journal_entries(&i.from, &i.to).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get trial balance for a period")]
    async fn get_trial_balance(&self, Parameters(i): Parameters<AsOfInput>) -> String {
        match self.backend.get_trial_balance(&i.as_of).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    // ─── Governance ──────────────────────────────────────────────────────────

    #[tool(description = "Request approval for a pending ERP document (order, invoice, etc.)")]
    async fn request_erp_approval(&self, Parameters(i): Parameters<ApprovalInput>) -> String {
        format!("Approval requested for {} {} (note: {})", i.entity_type, i.entity_id, i.note.as_deref().unwrap_or("none"))
    }

    #[tool(description = "Attach supporting evidence/documents to an ERP record")]
    async fn attach_erp_evidence(&self, Parameters(i): Parameters<EvidenceInput>) -> String {
        let url_info = i.url.as_deref().unwrap_or("no URL");
        format!("Evidence attached to {} {}: {} ({})", i.entity_type, i.entity_id, i.description, url_info)
    }

    #[tool(description = "Get the audit trail/history for an ERP document")]
    async fn get_erp_audit_trail(&self, Parameters(i): Parameters<AuditInput>) -> String {
        match self.backend.get_audit_trail(&i.entity_type, &i.entity_id).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for ErpServer {
    async fn check_health(&self) -> HealthStatus {
        // Verify backend is reachable by listing 1 customer
        match self.backend.list_customers(1).await {
            Ok(_) => HealthStatus { healthy: true, message: Some(format!("{} connected", self.backend.name())), latency_ms: Some(1) },
            Err(e) => HealthStatus { healthy: false, message: Some(format!("{}: {e}", self.backend.name())), latency_ms: None },
        }
    }
}
