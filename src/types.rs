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

    // ─── Accounting extensions ───────────────────────────────────────────────
    // Full-ledger operations (vendor bills, payments, financial reports, GL
    // posting). Backends that don't expose these return the default error;
    // results are raw backend JSON since shapes vary too much to unify.

    async fn list_bills(&self, _limit: u32, _status: Option<&str>) -> anyhow::Result<serde_json::Value> {
        Err(anyhow::anyhow!("{}: vendor bills not supported by this backend", self.name()))
    }
    async fn get_bill(&self, _id: &str) -> anyhow::Result<serde_json::Value> {
        Err(anyhow::anyhow!("{}: vendor bills not supported by this backend", self.name()))
    }
    async fn create_bill_draft(&self, _input: &CreateBillInput) -> anyhow::Result<serde_json::Value> {
        Err(anyhow::anyhow!("{}: vendor bills not supported by this backend", self.name()))
    }
    async fn post_bill(&self, _id: &str) -> anyhow::Result<serde_json::Value> {
        Err(anyhow::anyhow!("{}: vendor bills not supported by this backend", self.name()))
    }
    async fn list_payments(&self, _limit: u32) -> anyhow::Result<serde_json::Value> {
        Err(anyhow::anyhow!("{}: payments not supported by this backend", self.name()))
    }
    async fn record_payment(&self, _input: &RecordPaymentInput) -> anyhow::Result<serde_json::Value> {
        Err(anyhow::anyhow!("{}: payments not supported by this backend", self.name()))
    }
    async fn run_report(&self, _report_type: &str, _as_at: Option<&str>, _from: Option<&str>, _to: Option<&str>) -> anyhow::Result<serde_json::Value> {
        Err(anyhow::anyhow!("{}: financial reports not supported by this backend", self.name()))
    }
    async fn get_dashboard(&self) -> anyhow::Result<serde_json::Value> {
        Err(anyhow::anyhow!("{}: dashboard not supported by this backend", self.name()))
    }
    async fn list_bank_accounts(&self) -> anyhow::Result<serde_json::Value> {
        Err(anyhow::anyhow!("{}: bank accounts not supported by this backend", self.name()))
    }
    async fn post_journal_entry(&self, _input: &PostJournalInput) -> anyhow::Result<serde_json::Value> {
        Err(anyhow::anyhow!("{}: journal posting not supported by this backend", self.name()))
    }

    // ─── HR & Payroll extensions ─────────────────────────────────────────────
    // Employees, fiscal periods, and the pay-run lifecycle (run → adjust →
    // recompute → approve → post → paid). Raw backend JSON; only Zavora ERA
    // implements these today.

    async fn list_employees(&self, _limit: u32) -> anyhow::Result<serde_json::Value> {
        Err(anyhow::anyhow!("{}: employees not supported by this backend", self.name()))
    }
    async fn list_fiscal_periods(&self) -> anyhow::Result<serde_json::Value> {
        Err(anyhow::anyhow!("{}: fiscal periods not supported by this backend", self.name()))
    }
    async fn list_departments(&self) -> anyhow::Result<serde_json::Value> {
        Err(anyhow::anyhow!("{}: departments not supported by this backend", self.name()))
    }
    async fn list_pay_runs(&self) -> anyhow::Result<serde_json::Value> {
        Err(anyhow::anyhow!("{}: payroll not supported by this backend", self.name()))
    }
    async fn get_pay_run(&self, _id: &str) -> anyhow::Result<serde_json::Value> {
        Err(anyhow::anyhow!("{}: payroll not supported by this backend", self.name()))
    }
    async fn run_payroll(&self, _period_id: &str, _pay_date: &str) -> anyhow::Result<serde_json::Value> {
        Err(anyhow::anyhow!("{}: payroll not supported by this backend", self.name()))
    }
    async fn add_pay_run_input(&self, _run_id: &str, _input: &PayRunInputInput) -> anyhow::Result<serde_json::Value> {
        Err(anyhow::anyhow!("{}: payroll not supported by this backend", self.name()))
    }
    async fn recompute_pay_run(&self, _id: &str) -> anyhow::Result<serde_json::Value> {
        Err(anyhow::anyhow!("{}: payroll not supported by this backend", self.name()))
    }
    async fn approve_pay_run(&self, _id: &str) -> anyhow::Result<serde_json::Value> {
        Err(anyhow::anyhow!("{}: payroll not supported by this backend", self.name()))
    }
    async fn post_pay_run(&self, _id: &str) -> anyhow::Result<serde_json::Value> {
        Err(anyhow::anyhow!("{}: payroll not supported by this backend", self.name()))
    }
    async fn mark_pay_run_paid(&self, _id: &str) -> anyhow::Result<serde_json::Value> {
        Err(anyhow::anyhow!("{}: payroll not supported by this backend", self.name()))
    }

    // ─── Procurement (P2P) extensions ────────────────────────────────────────
    async fn list_requisitions(&self) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: procurement not supported by this backend", self.name())) }
    async fn create_requisition(&self, _body: &serde_json::Value) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: procurement not supported by this backend", self.name())) }
    async fn approve_requisition(&self, _id: &str) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: procurement not supported by this backend", self.name())) }
    async fn convert_requisition(&self, _id: &str, _body: &serde_json::Value) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: procurement not supported by this backend", self.name())) }
    async fn create_direct_po(&self, _body: &serde_json::Value) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: procurement not supported by this backend", self.name())) }
    async fn send_purchase_order(&self, _id: &str, _body: &serde_json::Value) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: procurement not supported by this backend", self.name())) }
    async fn receive_goods(&self, _po_id: &str, _body: &serde_json::Value) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: procurement not supported by this backend", self.name())) }
    async fn three_way_match(&self, _po_id: &str) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: procurement not supported by this backend", self.name())) }
    async fn create_debit_note(&self, _body: &serde_json::Value) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: procurement not supported by this backend", self.name())) }
    async fn list_expense_claims(&self) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: procurement not supported by this backend", self.name())) }
    async fn create_expense_claim(&self, _body: &serde_json::Value) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: procurement not supported by this backend", self.name())) }
    async fn approve_expense_claim(&self, _id: &str) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: procurement not supported by this backend", self.name())) }
    async fn procurement_analytics(&self) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: procurement not supported by this backend", self.name())) }
    async fn budget_control(&self) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: procurement not supported by this backend", self.name())) }
    // KRA eTIMS (Kenya) — Zavora-specific.
    async fn etims_status(&self) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: eTIMS not supported by this backend", self.name())) }
    async fn etims_transmit_invoice(&self, _id: &str) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: eTIMS not supported by this backend", self.name())) }
    // Banking, period-end and statutory workflows — Zavora-specific.
    async fn list_reconciliations(&self) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: bank reconciliation not supported by this backend", self.name())) }
    async fn compute_reconciliation(&self, _body: &serde_json::Value) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: bank reconciliation not supported by this backend", self.name())) }
    async fn complete_reconciliation(&self, _body: &serde_json::Value) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: bank reconciliation not supported by this backend", self.name())) }
    async fn close_period(&self, _id: &str) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: period close not supported by this backend", self.name())) }
    async fn reopen_period(&self, _id: &str) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: period close not supported by this backend", self.name())) }
    async fn list_tax_filings(&self) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: tax filings not supported by this backend", self.name())) }
    async fn file_tax_return(&self, _body: &serde_json::Value) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: tax filings not supported by this backend", self.name())) }
    async fn remit_tax_filing(&self, _id: &str, _body: &serde_json::Value) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: tax filings not supported by this backend", self.name())) }
    async fn cit_estimate(&self, _fiscal_year: Option<i32>, _adjustments: Option<f64>) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: corporation tax not supported by this backend", self.name())) }
    // AR outreach, asset/FX runs, statement import — Zavora-specific.
    async fn send_customer_statement(&self, _id: &str, _body: &serde_json::Value) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: statement sending not supported by this backend", self.name())) }
    async fn list_fixed_assets(&self) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: fixed assets not supported by this backend", self.name())) }
    async fn run_depreciation(&self, _date: Option<&str>) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: fixed assets not supported by this backend", self.name())) }
    async fn run_fx_revaluation(&self, _date: Option<&str>) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: FX revaluation not supported by this backend", self.name())) }
    async fn import_bank_statement(&self, _body: &serde_json::Value) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: statement import not supported by this backend", self.name())) }
    // Budgets — Zavora-specific.
    async fn list_budgets(&self) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: budgets not supported by this backend", self.name())) }
    async fn set_budget(&self, _body: &serde_json::Value) -> anyhow::Result<serde_json::Value> { Err(anyhow::anyhow!("{}: budgets not supported by this backend", self.name())) }
}

/// Input for creating a vendor bill draft.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateBillInput {
    pub vendor_id: String,
    /// The supplier's own invoice number (the legal document reference).
    #[serde(default)]
    pub vendor_invoice_number: Option<String>,
    /// ISO date (YYYY-MM-DD); defaults to today.
    #[serde(default)]
    pub issue_date: Option<String>,
    #[serde(default)]
    pub due_date: Option<String>,
    /// ISO 4217 code; defaults to the vendor's currency.
    #[serde(default)]
    pub currency: Option<String>,
    /// Spot rate to functional currency for FCY bills.
    #[serde(default)]
    pub fx_rate: Option<f64>,
    pub line_items: Vec<LineItemInput>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// One document a payment settles.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PaymentApplicationInput {
    /// Invoice or bill id.
    pub document_id: String,
    /// Amount applied, in the payment currency.
    pub amount: f64,
}

/// Input for recording a payment (customer receipt or vendor payment).
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RecordPaymentInput {
    /// "customer_payment" (money in) or "vendor_payment" (money out).
    pub payment_type: String,
    /// Customer or vendor id.
    pub party_id: String,
    /// ISO date (YYYY-MM-DD); defaults to today.
    #[serde(default)]
    pub payment_date: Option<String>,
    pub amount: f64,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub fx_rate: Option<f64>,
    /// "bank_transfer" | "mpesa" | "cash" | "cheque".
    pub method: String,
    #[serde(default)]
    pub reference: Option<String>,
    #[serde(default)]
    pub bank_account_id: Option<String>,
    #[serde(default)]
    pub applications: Vec<PaymentApplicationInput>,
    /// Withholding tax deducted at source, in functional currency (KES).
    #[serde(default)]
    pub wht_amount: Option<f64>,
    /// GL account funding a non-cash payment (e.g. a director's loan account).
    #[serde(default)]
    pub funding_account: Option<String>,
}

/// One GL line of a manual journal entry.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JournalLineInput {
    pub account_code: String,
    #[serde(default)]
    pub debit: Option<f64>,
    #[serde(default)]
    pub credit: Option<f64>,
    /// ISO 4217 code; defaults to the functional currency.
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub fx_rate: Option<f64>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Input for posting a balanced manual journal entry.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PostJournalInput {
    /// ISO date (YYYY-MM-DD).
    pub date: String,
    pub reference: String,
    pub description: String,
    pub lines: Vec<JournalLineInput>,
}

/// Input for creating line items (used in order/invoice creation).
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct LineItemInput {
    pub product_id: Option<String>,
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
}

/// A per-run payroll adjustment (bonus/overtime earning, or a voluntary/loan deduction).
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PayRunInputInput {
    pub employee_id: String,
    /// "earning" (adds to pay) or "deduction" (reduces net pay).
    pub kind: String,
    /// Description shown on the payslip (e.g. "Performance Bonus", "SACCO").
    pub name: String,
    pub amount: f64,
    /// Earnings only: whether the amount is subject to PAYE/NSSF/SHA/Housing. Default true.
    #[serde(default = "default_true_pri")]
    pub taxable: bool,
    /// Optional earning/deduction type code from the masters (e.g. "BONUS", "SACCO").
    #[serde(default)]
    pub type_code: Option<String>,
}

fn default_true_pri() -> bool { true }
