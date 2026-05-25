//! Unified ERP types shared across all backends.
use serde::{Deserialize, Serialize};

/// Document lifecycle state — applies to orders, invoices, and purchase orders.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleState {
    Draft,
    PendingApproval,
    Approved,
    Released,
    Posted,
    Sent,
    Fulfilled,
    PartiallyFulfilled,
    Closed,
    Cancelled,
    Voided,
}

impl std::fmt::Display for LifecycleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::PendingApproval => write!(f, "pending_approval"),
            Self::Approved => write!(f, "approved"),
            Self::Released => write!(f, "released"),
            Self::Posted => write!(f, "posted"),
            Self::Sent => write!(f, "sent"),
            Self::Fulfilled => write!(f, "fulfilled"),
            Self::PartiallyFulfilled => write!(f, "partially_fulfilled"),
            Self::Closed => write!(f, "closed"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Voided => write!(f, "voided"),
        }
    }
}

/// Customer record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub currency: Option<String>,
    pub balance: Option<f64>,
    pub backend: String,
}

/// Vendor/supplier record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vendor {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub currency: Option<String>,
    pub balance: Option<f64>,
    pub backend: String,
}

/// Product/item record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub sku: Option<String>,
    pub unit_price: Option<f64>,
    pub currency: Option<String>,
    pub stock_on_hand: Option<f64>,
    pub backend: String,
}

/// Line item on an order or invoice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItem {
    pub product_id: Option<String>,
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub amount: f64,
}

/// Sales order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesOrder {
    pub id: String,
    pub customer_id: String,
    pub customer_name: Option<String>,
    pub state: LifecycleState,
    pub total: f64,
    pub currency: Option<String>,
    pub line_items: Vec<LineItem>,
    pub created_at: Option<String>,
    pub backend: String,
}

/// Purchase order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrder {
    pub id: String,
    pub vendor_id: String,
    pub vendor_name: Option<String>,
    pub state: LifecycleState,
    pub total: f64,
    pub currency: Option<String>,
    pub line_items: Vec<LineItem>,
    pub created_at: Option<String>,
    pub backend: String,
}

/// Invoice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub customer_id: Option<String>,
    pub customer_name: Option<String>,
    pub state: LifecycleState,
    pub total: f64,
    pub balance_due: Option<f64>,
    pub currency: Option<String>,
    pub line_items: Vec<LineItem>,
    pub due_date: Option<String>,
    pub created_at: Option<String>,
    pub backend: String,
}

/// Stock level for a product at a location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockLevel {
    pub product_id: String,
    pub product_name: Option<String>,
    pub warehouse: Option<String>,
    pub quantity_on_hand: f64,
    pub quantity_available: Option<f64>,
    pub backend: String,
}

/// GL account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub code: Option<String>,
    pub name: String,
    pub account_type: Option<String>,
    pub balance: Option<f64>,
    pub backend: String,
}

/// Journal entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: String,
    pub date: String,
    pub description: Option<String>,
    pub debit_account: Option<String>,
    pub credit_account: Option<String>,
    pub amount: f64,
    pub backend: String,
}

/// Audit trail entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub user: Option<String>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub details: Option<String>,
}

/// Backend trait — each ERP system implements this.
#[async_trait::async_trait]
pub trait ErpBackend: Send + Sync {
    fn name(&self) -> &str;

    // Customers
    async fn list_customers(&self, limit: u32) -> anyhow::Result<Vec<Customer>>;
    async fn get_customer(&self, id: &str) -> anyhow::Result<Customer>;
    async fn create_customer(&self, name: &str, email: Option<&str>, phone: Option<&str>) -> anyhow::Result<Customer>;
    async fn update_customer(&self, id: &str, name: Option<&str>, email: Option<&str>, phone: Option<&str>) -> anyhow::Result<Customer>;

    // Vendors
    async fn list_vendors(&self, limit: u32) -> anyhow::Result<Vec<Vendor>>;
    async fn get_vendor(&self, id: &str) -> anyhow::Result<Vendor>;
    async fn create_vendor(&self, name: &str, email: Option<&str>, phone: Option<&str>) -> anyhow::Result<Vendor>;
    async fn update_vendor(&self, id: &str, name: Option<&str>, email: Option<&str>, phone: Option<&str>) -> anyhow::Result<Vendor>;

    // Products
    async fn list_products(&self, limit: u32) -> anyhow::Result<Vec<Product>>;
    async fn get_product(&self, id: &str) -> anyhow::Result<Product>;
    async fn create_product(&self, name: &str, sku: Option<&str>, price: Option<f64>) -> anyhow::Result<Product>;
    async fn update_product(&self, id: &str, name: Option<&str>, sku: Option<&str>, price: Option<f64>) -> anyhow::Result<Product>;

    // Sales Orders
    async fn list_sales_orders(&self, limit: u32) -> anyhow::Result<Vec<SalesOrder>>;
    async fn get_sales_order(&self, id: &str) -> anyhow::Result<SalesOrder>;
    async fn create_sales_order_draft(&self, customer_id: &str, items: &[LineItemInput]) -> anyhow::Result<SalesOrder>;
    async fn submit_sales_order(&self, id: &str) -> anyhow::Result<SalesOrder>;

    // Purchase Orders
    async fn list_purchase_orders(&self, limit: u32) -> anyhow::Result<Vec<PurchaseOrder>>;
    async fn get_purchase_order(&self, id: &str) -> anyhow::Result<PurchaseOrder>;
    async fn create_purchase_order_draft(&self, vendor_id: &str, items: &[LineItemInput]) -> anyhow::Result<PurchaseOrder>;
    async fn submit_purchase_order(&self, id: &str) -> anyhow::Result<PurchaseOrder>;

    // Invoices
    async fn list_invoices(&self, limit: u32) -> anyhow::Result<Vec<Invoice>>;
    async fn get_invoice(&self, id: &str) -> anyhow::Result<Invoice>;
    async fn create_invoice_draft(&self, customer_id: &str, items: &[LineItemInput]) -> anyhow::Result<Invoice>;
    async fn submit_invoice(&self, id: &str) -> anyhow::Result<Invoice>;
    async fn post_invoice(&self, id: &str) -> anyhow::Result<Invoice>;

    // Inventory
    async fn get_stock_levels(&self, product_id: Option<&str>) -> anyhow::Result<Vec<StockLevel>>;
    async fn adjust_stock(&self, product_id: &str, quantity: f64, reason: &str) -> anyhow::Result<StockLevel>;
    async fn transfer_stock(&self, product_id: &str, from_warehouse: &str, to_warehouse: &str, quantity: f64) -> anyhow::Result<()>;

    // General Ledger
    async fn list_accounts(&self) -> anyhow::Result<Vec<Account>>;
    async fn get_journal_entries(&self, from: &str, to: &str) -> anyhow::Result<Vec<JournalEntry>>;
    async fn get_trial_balance(&self, as_of: &str) -> anyhow::Result<serde_json::Value>;

    // Governance
    async fn get_audit_trail(&self, entity_type: &str, entity_id: &str) -> anyhow::Result<Vec<AuditEntry>>;
}

/// Input for creating line items (used in order/invoice creation).
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct LineItemInput {
    pub product_id: Option<String>,
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
}
