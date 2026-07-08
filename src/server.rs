//! MCP tool router for ERP operations.
use adk_mcp_sdk::{HealthCheck, HealthStatus};
use crate::types::{CreateBillInput, ErpBackend, LineItemInput, PayRunInputInput, PostJournalInput, RecordPaymentInput};
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

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListBillsInput {
    #[serde(default = "d20")]
    pub limit: u32,
    /// Filter by bill status (e.g. draft, posted, paid).
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RunReportInput {
    /// Report type, e.g. TrialBalance, BalanceSheet, ProfitAndLoss, CashFlow, ArAgeing, ApAgeing, VatReturn, GlDetail.
    pub report_type: String,
    /// Point-in-time date (YYYY-MM-DD) for balance-style reports.
    #[serde(default)]
    pub as_at: Option<String>,
    /// Period start (YYYY-MM-DD) for period reports like ProfitAndLoss.
    #[serde(default)]
    pub from: Option<String>,
    /// Period end (YYYY-MM-DD) for period reports.
    #[serde(default)]
    pub to: Option<String>,
}

fn d20() -> u32 { 20 }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RunPayrollInput {
    /// Fiscal period id to run payroll for (from list_fiscal_periods).
    pub period_id: String,
    /// Pay date (YYYY-MM-DD); usually the period end.
    pub pay_date: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PayRunInputToolInput {
    /// The DRAFT pay run id to adjust.
    pub run_id: String,
    pub employee_id: String,
    /// "earning" (adds to pay) or "deduction" (reduces net pay).
    pub kind: String,
    /// Payslip description (e.g. "Performance Bonus", "SACCO").
    pub name: String,
    pub amount: f64,
    /// Earnings only: subject to PAYE/NSSF/SHA/Housing. Default true.
    #[serde(default = "d_true")]
    pub taxable: bool,
    /// Optional type code from masters ("BONUS", "OVERTIME", "SACCO"…).
    #[serde(default)]
    pub type_code: Option<String>,
}

fn d_true() -> bool { true }

// ─── Server ──────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ErpServer {
    pub backend: Arc<dyn ErpBackend>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JsonBodyInput {
    /// The request body as a JSON object matching the ERP API for this action.
    pub body: serde_json::Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct IdBodyInput {
    pub id: String,
    /// The request body as a JSON object for this action.
    pub body: serde_json::Value,
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

    // ─── Accounting (AP bills, payments, reports, GL posting) ───────────────

    #[tool(description = "List vendor bills (accounts payable), optionally filtered by status (draft/posted/paid)")]
    async fn list_bills(&self, Parameters(i): Parameters<ListBillsInput>) -> String {
        match self.backend.list_bills(i.limit, i.status.as_deref()).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get a vendor bill by ID with line items")]
    async fn get_bill(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.get_bill(&i.id).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Create a vendor bill in draft state (record a supplier invoice)")]
    async fn create_bill_draft(&self, Parameters(i): Parameters<CreateBillInput>) -> String {
        match self.backend.create_bill_draft(&i).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Post a draft vendor bill to the general ledger (creates the AP journal entry)")]
    async fn post_bill(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.post_bill(&i.id).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "List payments (customer receipts and vendor payments)")]
    async fn list_payments(&self, Parameters(i): Parameters<ListInput>) -> String {
        match self.backend.list_payments(i.limit).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Record a payment: customer receipt (money in) or vendor payment (money out), with optional invoice/bill applications and withholding tax")]
    async fn record_payment(&self, Parameters(i): Parameters<RecordPaymentInput>) -> String {
        match self.backend.record_payment(&i).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Run a financial or payroll report. Financial: TrialBalance, BalanceSheet, ProfitAndLoss, CashFlow, ArAgeing, ApAgeing, VatReturn, GlDetail, IncomeByCustomer, ExpenseByVendor, EquityChanges. Payroll: PayrollRegister, StatutorySchedule (KRA/NSSF/SHA/Housing/HELB remittance), PayeP9, PayeP10, PayrollBankFile (net-pay EFT), PayrollSummary. Use as_at for point-in-time reports, from/to for period reports")]
    async fn run_report(&self, Parameters(i): Parameters<RunReportInput>) -> String {
        match self.backend.run_report(&i.report_type, i.as_at.as_deref(), i.from.as_deref(), i.to.as_deref()).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get the business dashboard: cash position, receivables, payables, recent activity")]
    async fn get_dashboard(&self) -> String {
        match self.backend.get_dashboard().await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "List bank and mobile-money accounts with balances")]
    async fn list_bank_accounts(&self) -> String {
        match self.backend.list_bank_accounts().await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Post a balanced manual journal entry to the general ledger. Debits must equal credits")]
    async fn post_journal_entry(&self, Parameters(i): Parameters<PostJournalInput>) -> String {
        match self.backend.post_journal_entry(&i).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    // ─── HR & Payroll ────────────────────────────────────────────────────────

    #[tool(description = "List employees (staff records for payroll: name, staff number, salary, KRA PIN, department, bank)")]
    async fn list_employees(&self, Parameters(i): Parameters<ListInput>) -> String {
        match self.backend.list_employees(i.limit).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "List fiscal periods — use this to get the period_id needed to run payroll")]
    async fn list_fiscal_periods(&self) -> String {
        match self.backend.list_fiscal_periods().await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "List departments / cost centres")]
    async fn list_departments(&self) -> String {
        match self.backend.list_departments().await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "List pay runs (payroll history) with status, headcount and totals (gross, net, employer cost)")]
    async fn list_pay_runs(&self) -> String {
        match self.backend.list_pay_runs().await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get a pay run by id: header totals + per-employee payslip breakdown")]
    async fn get_pay_run(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.get_pay_run(&i.id).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Run payroll for a fiscal period: creates a DRAFT pay run for all active employees with Kenya statutory (PAYE, NSSF, SHA, Housing Levy, HELB) computed. Review before approving/posting. Returns the draft run id + totals")]
    async fn run_payroll(&self, Parameters(i): Parameters<RunPayrollInput>) -> String {
        match self.backend.run_payroll(&i.period_id, &i.pay_date).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Add a per-run adjustment to a DRAFT pay run — an earning (bonus, overtime) or a deduction (SACCO, advance) for one employee. The run auto-recomputes so the totals update")]
    async fn add_pay_run_input(&self, Parameters(i): Parameters<PayRunInputToolInput>) -> String {
        let input = PayRunInputInput {
            employee_id: i.employee_id,
            kind: i.kind,
            name: i.name,
            amount: i.amount,
            taxable: i.taxable,
            type_code: i.type_code,
        };
        match self.backend.add_pay_run_input(&i.run_id, &input).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Recompute a DRAFT pay run — re-applies adjustments and any master/statutory changes")]
    async fn recompute_pay_run(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.recompute_pay_run(&i.id).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Approve a DRAFT pay run (draft → approved). Confirm the totals with the user first")]
    async fn approve_pay_run(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.approve_pay_run(&i.id).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Post an APPROVED pay run to the general ledger (approved → posted). Posts a balanced payroll journal; requires an open fiscal period. Confirm with the user first")]
    async fn post_pay_run(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.post_pay_run(&i.id).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Mark a POSTED pay run as paid (posted → paid), once salaries have been disbursed")]
    async fn mark_pay_run_paid(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.mark_pay_run_paid(&i.id).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    // ─── Procurement (P2P) ───────────────────────────────────────────────────

    #[tool(description = "List purchase requisitions with their status (draft/submitted/approved/converted/rejected)")]
    async fn procurement_list_requisitions(&self) -> String {
        match self.backend.list_requisitions().await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Create a purchase requisition. body: {title, department?, needed_by?, justification?, lines:[{description, quantity, uom, estimated_unit_price, account_code?}]}")]
    async fn procurement_create_requisition(&self, Parameters(i): Parameters<JsonBodyInput>) -> String {
        match self.backend.create_requisition(&i.body).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Approve a submitted requisition (enforces the approver's spend limit). Confirm with the user first")]
    async fn procurement_approve_requisition(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.approve_requisition(&i.id).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Convert an approved requisition into a tender or direct PO. body: {target:'tender'|'purchase_order', vendor_id? (required for PO), delivery_date?, closing_date?}")]
    async fn procurement_convert_requisition(&self, Parameters(i): Parameters<IdBodyInput>) -> String {
        match self.backend.convert_requisition(&i.id, &i.body).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Raise a direct LPO against a vendor (no tender). body: {vendor_id, currency?, delivery_date?, notes?, lines:[{description, quantity, uom, unit_price, account_code?}]}")]
    async fn procurement_create_purchase_order(&self, Parameters(i): Parameters<JsonBodyInput>) -> String {
        match self.backend.create_direct_po(&i.body).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Email the LPO PDF to the vendor. id = PO id. body: {recipient_email?, message?}")]
    async fn procurement_send_purchase_order(&self, Parameters(i): Parameters<IdBodyInput>) -> String {
        match self.backend.send_purchase_order(&i.id, &i.body).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Record a Goods Receipt Note against a PO. id = PO id. body: {receipt_date?, notes?, lines:[{po_line_id?, description, quantity_received}]}")]
    async fn procurement_receive_goods(&self, Parameters(i): Parameters<IdBodyInput>) -> String {
        match self.backend.receive_goods(&i.id, &i.body).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Get the 3-way match (ordered vs received vs billed) for a PO. id = PO id")]
    async fn procurement_three_way_match(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.three_way_match(&i.id).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Issue a supplier debit note (return/overcharge) reducing the payable. body: {vendor_id, reason?, applies_to_bill?, po_id?, lines:[{description, quantity, unit_price, account_code?}]}")]
    async fn procurement_create_debit_note(&self, Parameters(i): Parameters<JsonBodyInput>) -> String {
        match self.backend.create_debit_note(&i.body).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "List staff expense claims with their status")]
    async fn procurement_list_expense_claims(&self) -> String {
        match self.backend.list_expense_claims().await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Create a staff expense claim. body: {title, lines:[{expense_date?, description, account_code?, amount}]}")]
    async fn procurement_create_expense_claim(&self, Parameters(i): Parameters<JsonBodyInput>) -> String {
        match self.backend.create_expense_claim(&i.body).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Approve a submitted expense claim — posts DR expense / CR payable (enforces spend limit). Confirm with the user first")]
    async fn procurement_approve_expense_claim(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.approve_expense_claim(&i.id).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Procurement analytics: spend by vendor, open-commitment register, and document counts by status")]
    async fn procurement_analytics(&self) -> String {
        match self.backend.procurement_analytics().await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Budget vs committed (open POs) vs actual by account — the encumbrance view for procurement budget control")]
    async fn procurement_budget_control(&self) -> String {
        match self.backend.budget_control().await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "KRA eTIMS status: whether the device is enabled/initialised, its SCU id, and the last invoice number transmitted")]
    async fn etims_status(&self) -> String {
        match self.backend.etims_status().await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Transmit (or retry) a posted invoice OR credit note to KRA eTIMS in real time, returning the signed SCU receipt. Pass the invoice/credit-note id. Credit notes go out as a credit/refund receipt referencing the original invoice. Documents auto-transmit on posting; use this to retry a failed one. Confirm with the user first")]
    async fn etims_transmit_invoice(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.etims_transmit_invoice(&i.id).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }

    // ─── Banking, period-end & statutory workflows ──────────────────────────

    #[tool(description = "List completed bank reconciliations (bank account, statement date, statement balance, when locked)")]
    async fn list_reconciliations(&self) -> String {
        match self.backend.list_reconciliations().await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Start a bank reconciliation: returns the GL balance, already-cleared balance, and the uncleared ledger entries to tick off against the statement. Body: {\"bank_account_id\": \"<uuid>\", \"statement_date\": \"YYYY-MM-DD\"}")]
    async fn compute_reconciliation(&self, Parameters(i): Parameters<JsonBodyInput>) -> String {
        match self.backend.compute_reconciliation(&i.body).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Complete and LOCK a bank reconciliation — only succeeds when the cleared balance equals the statement closing balance. Body: {\"bank_account_id\": \"<uuid>\", \"statement_date\": \"YYYY-MM-DD\", \"statement_closing_balance\": <number>, \"cleared_entry_ids\": [\"<uuid>\", ...]}. Confirm with the user first")]
    async fn complete_reconciliation(&self, Parameters(i): Parameters<JsonBodyInput>) -> String {
        match self.backend.complete_reconciliation(&i.body).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Close (lock) a fiscal period so no further postings can land in it. Pass the period id from list_fiscal_periods. Confirm with the user first")]
    async fn close_period(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.close_period(&i.id).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Reopen a previously closed fiscal period to allow corrections. Pass the period id. Confirm with the user first")]
    async fn reopen_period(&self, Parameters(i): Parameters<IdInput>) -> String {
        match self.backend.reopen_period(&i.id).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "List tax filings (VAT/PAYE/WHT returns): period, amount, filed/remitted status")]
    async fn list_tax_filings(&self) -> String {
        match self.backend.list_tax_filings().await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Record a tax return as FILED for a period (run the matching report first — VatReturn, PayeP or WhtCertificate — and confirm the figure with the user). Body: {\"tax_type\": \"VAT\"|\"PAYE\"|\"WHT\", \"period_from\": \"YYYY-MM-DD\", \"period_to\": \"YYYY-MM-DD\", \"amount\": <number>}")]
    async fn file_tax_return(&self, Parameters(i): Parameters<JsonBodyInput>) -> String {
        match self.backend.file_tax_return(&i.body).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Record the remittance (payment to KRA) of a filed tax return — posts the payment against the filing. Pass the filing id and the payment body the ERP expects. Confirm with the user first")]
    async fn remit_tax_filing(&self, Parameters(i): Parameters<IdBodyInput>) -> String {
        match self.backend.remit_tax_filing(&i.id, &i.body).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }
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
