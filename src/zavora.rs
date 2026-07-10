//! Zavora ERA REST backend.
//!
//! Auth is JWT-only with a short (15 min) access TTL, so the backend logs in
//! with a service user's credentials and transparently re-logs-in when the
//! cached token ages out or a request comes back 401. The tenant (entity_id)
//! is taken from the login response and injected where endpoints require it
//! (e.g. `/agent/report`).
use crate::types::*;
use anyhow::{Result, anyhow};
use reqwest::{Client, Method};
use serde_json::{Value, json};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Re-login this long after the last login — comfortably inside the 15-min TTL.
const TOKEN_MAX_AGE: Duration = Duration::from_secs(12 * 60);

struct AuthState {
    token: String,
    entity_id: String,
    issued_at: Instant,
}

pub struct ZavoraBackend {
    http: Client,
    base: String,
    email: String,
    password: String,
    auth: RwLock<Option<AuthState>>,
}

/// Parse Zavora money/number fields, which arrive as JSON strings ("129.04").
fn num(v: &Value) -> Option<f64> {
    v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}

fn s(v: &Value) -> String {
    v.as_str().unwrap_or("").to_string()
}

fn map_doc_state(status: &str) -> LifecycleState {
    match status {
        "draft" => LifecycleState::Draft,
        "pending_approval" => LifecycleState::PendingApproval,
        "approved" => LifecycleState::Approved,
        "sent" => LifecycleState::Sent,
        "partially_paid" => LifecycleState::PartiallyFulfilled,
        "paid" => LifecycleState::Closed,
        "cancelled" => LifecycleState::Cancelled,
        "voided" | "written_off" => LifecycleState::Voided,
        _ => LifecycleState::Posted,
    }
}

impl ZavoraBackend {
    pub fn new(base_url: String, email: String, password: String) -> Self {
        Self {
            http: Client::new(),
            base: format!("{}/api/v1", base_url.trim_end_matches('/')),
            email,
            password,
            auth: RwLock::new(None),
        }
    }

    async fn login(&self) -> Result<(String, String)> {
        let resp = self
            .http
            .post(format!("{}/auth/login", self.base))
            .json(&json!({"email": self.email, "password": self.password}))
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(anyhow!("Zavora login failed ({})", resp.status()));
        }
        let body: Value = resp.json().await?;
        let token = body["access_token"]
            .as_str()
            .ok_or_else(|| anyhow!("Zavora login response missing access_token"))?
            .to_string();
        let entity_id = body["user"]["entity_id"].as_str().unwrap_or("").to_string();
        *self.auth.write().await = Some(AuthState {
            token: token.clone(),
            entity_id: entity_id.clone(),
            issued_at: Instant::now(),
        });
        Ok((token, entity_id))
    }

    async fn credentials(&self) -> Result<(String, String)> {
        if let Some(a) = self.auth.read().await.as_ref() {
            if a.issued_at.elapsed() < TOKEN_MAX_AGE {
                return Ok((a.token.clone(), a.entity_id.clone()));
            }
        }
        self.login().await
    }

    async fn request(&self, method: Method, path: &str, body: Option<&Value>) -> Result<Value> {
        // Per-call user token (injected by Amos as `__user_token`, carried in
        // the server's task-local) makes THIS request act as the human who
        // approved it — so the ERP records the person, not the service account,
        // as the actor. Absent ⇒ the service login. On a 401 (e.g. a user token
        // that expired before the client pushed a refresh) we fall back to the
        // service login once, so the operation still completes.
        let user_token = crate::server::current_user_token();
        let mut token = match &user_token {
            Some(t) => t.clone(),
            None => self.credentials().await?.0,
        };
        for attempt in 0..2 {
            let mut req = self
                .http
                .request(method.clone(), format!("{}/{path}", self.base))
                .header("Authorization", format!("Bearer {token}"));
            if let Some(b) = body {
                req = req.json(b);
            }
            let resp = req.send().await?;
            let status = resp.status();
            if status == reqwest::StatusCode::UNAUTHORIZED && attempt == 0 {
                // Re-authenticate as the service account — this also serves as
                // the fallback when a per-call user token has expired.
                token = self.login().await?.0;
                continue;
            }
            let text = resp.text().await?;
            if !status.is_success() {
                return Err(anyhow!("Zavora API {status} on {path}: {text}"));
            }
            return Ok(serde_json::from_str(&text).unwrap_or(Value::String(text)));
        }
        unreachable!()
    }

    async fn get(&self, path: &str) -> Result<Value> {
        self.request(Method::GET, path, None).await
    }

    async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        self.request(Method::POST, path, Some(body)).await
    }

    async fn put(&self, path: &str, body: &Value) -> Result<Value> {
        self.request(Method::PUT, path, Some(body)).await
    }

    async fn post_action(&self, path: &str) -> Result<Value> {
        self.request(Method::POST, path, Some(&json!({}))).await
    }

    /// List endpoints that return `{data: [...]}` (invoices/bills/payments/journal-entries).
    fn rows(resp: Value) -> Vec<Value> {
        match resp {
            Value::Array(a) => a,
            Value::Object(mut o) => match o.remove("data") {
                Some(Value::Array(a)) => a,
                _ => vec![],
            },
            _ => vec![],
        }
    }

    fn map_customer(c: &Value) -> Customer {
        Customer {
            id: s(&c["id"]),
            name: s(&c["name"]),
            email: c["email"][0]["email"].as_str().map(Into::into),
            phone: c["phone"][0]["number"].as_str().map(Into::into),
            currency: c["currency"].as_str().map(Into::into),
            balance: num(&c["balance"]),
            backend: "zavora".into(),
        }
    }

    fn map_vendor(c: &Value) -> Vendor {
        Vendor {
            id: s(&c["id"]),
            name: s(&c["name"]),
            email: c["email"][0]["email"].as_str().map(Into::into),
            phone: c["phone"][0]["number"].as_str().map(Into::into),
            currency: c["currency"].as_str().map(Into::into),
            balance: num(&c["balance"]),
            backend: "zavora".into(),
        }
    }

    fn map_product(p: &Value) -> Product {
        Product {
            id: s(&p["id"]),
            name: s(&p["name"]),
            sku: None,
            unit_price: num(&p["unit_price"]),
            currency: p["currency"].as_str().map(Into::into),
            stock_on_hand: None,
            backend: "zavora".into(),
        }
    }

    fn map_lines(v: &Value) -> Vec<LineItem> {
        v.as_array()
            .map(|a| {
                a.iter()
                    .map(|l| LineItem {
                        product_id: l["product_id"].as_str().map(Into::into),
                        description: s(&l["description"]),
                        quantity: num(&l["quantity"]).unwrap_or(0.0),
                        unit_price: num(&l["unit_price"]).unwrap_or(0.0),
                        amount: num(&l["line_total"]).or_else(|| num(&l["amount"])).unwrap_or(0.0),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn map_invoice(i: &Value) -> Invoice {
        Invoice {
            id: s(&i["id"]),
            customer_id: i["customer_id"].as_str().map(Into::into),
            customer_name: i["customer_name"].as_str().map(Into::into),
            state: map_doc_state(i["status"].as_str().unwrap_or("")),
            total: num(&i["gross_total"]).unwrap_or(0.0),
            balance_due: num(&i["balance_due"]),
            currency: i["currency"].as_str().map(Into::into),
            line_items: Self::map_lines(&i["lines"]),
            due_date: i["due_date"].as_str().map(Into::into),
            created_at: i["issue_date"].as_str().map(Into::into),
            backend: "zavora".into(),
        }
    }
}

#[async_trait::async_trait]
impl ErpBackend for ZavoraBackend {
    fn name(&self) -> &str {
        "zavora"
    }

    // Customers
    async fn list_customers(&self, limit: u32) -> Result<Vec<Customer>> {
        let resp = self.get("customers").await?;
        Ok(Self::rows(resp).iter().take(limit as usize).map(Self::map_customer).collect())
    }

    async fn get_customer(&self, id: &str) -> Result<Customer> {
        Ok(Self::map_customer(&self.get(&format!("customers/{id}")).await?))
    }

    async fn create_customer(&self, name: &str, email: Option<&str>, phone: Option<&str>) -> Result<Customer> {
        let body = json!({
            "name": name,
            "email": email.map(|e| vec![json!({"email": e, "label": null, "is_primary": true})]).unwrap_or_default(),
            "phone": phone.map(|p| vec![json!({"number": p, "label": null, "is_primary": true, "whatsapp_enabled": false})]).unwrap_or_default(),
        });
        Ok(Self::map_customer(&self.post("customers", &body).await?))
    }

    async fn update_customer(&self, id: &str, name: Option<&str>, email: Option<&str>, phone: Option<&str>) -> Result<Customer> {
        let mut body = json!({});
        if let Some(n) = name { body["name"] = n.into(); }
        if let Some(e) = email { body["email"] = json!([{"email": e, "label": null, "is_primary": true}]); }
        if let Some(p) = phone { body["phone"] = json!([{"number": p, "label": null, "is_primary": true, "whatsapp_enabled": false}]); }
        Ok(Self::map_customer(&self.put(&format!("customers/{id}"), &body).await?))
    }

    // Vendors
    async fn list_vendors(&self, limit: u32) -> Result<Vec<Vendor>> {
        let resp = self.get("vendors").await?;
        Ok(Self::rows(resp).iter().take(limit as usize).map(Self::map_vendor).collect())
    }

    async fn get_vendor(&self, id: &str) -> Result<Vendor> {
        Ok(Self::map_vendor(&self.get(&format!("vendors/{id}")).await?))
    }

    async fn create_vendor(&self, name: &str, email: Option<&str>, phone: Option<&str>) -> Result<Vendor> {
        let body = json!({
            "name": name,
            "email": email.map(|e| vec![json!({"email": e, "label": null, "is_primary": true})]).unwrap_or_default(),
            "phone": phone.map(|p| vec![json!({"number": p, "label": null, "is_primary": true, "whatsapp_enabled": false})]).unwrap_or_default(),
        });
        Ok(Self::map_vendor(&self.post("vendors", &body).await?))
    }

    async fn update_vendor(&self, id: &str, name: Option<&str>, email: Option<&str>, phone: Option<&str>) -> Result<Vendor> {
        let mut body = json!({});
        if let Some(n) = name { body["name"] = n.into(); }
        if let Some(e) = email { body["email"] = json!([{"email": e, "label": null, "is_primary": true}]); }
        if let Some(p) = phone { body["phone"] = json!([{"number": p, "label": null, "is_primary": true, "whatsapp_enabled": false}]); }
        Ok(Self::map_vendor(&self.put(&format!("vendors/{id}"), &body).await?))
    }

    // Products
    async fn list_products(&self, limit: u32) -> Result<Vec<Product>> {
        let resp = self.get("products").await?;
        Ok(Self::rows(resp).iter().take(limit as usize).map(Self::map_product).collect())
    }

    async fn get_product(&self, id: &str) -> Result<Product> {
        Ok(Self::map_product(&self.get(&format!("products/{id}")).await?))
    }

    async fn create_product(&self, name: &str, _sku: Option<&str>, price: Option<f64>) -> Result<Product> {
        let body = json!({"name": name, "product_type": "Service", "unit_price": price});
        Ok(Self::map_product(&self.post("products", &body).await?))
    }

    async fn update_product(&self, id: &str, name: Option<&str>, _sku: Option<&str>, price: Option<f64>) -> Result<Product> {
        let mut body = json!({});
        if let Some(n) = name { body["name"] = n.into(); }
        if let Some(p) = price { body["unit_price"] = p.into(); }
        Ok(Self::map_product(&self.put(&format!("products/{id}"), &body).await?))
    }

    // Sales / purchase orders — Zavora ERA uses estimates + invoices/bills instead.
    async fn list_sales_orders(&self, _limit: u32) -> Result<Vec<SalesOrder>> {
        Err(anyhow!("Zavora ERA has no sales orders — use estimates/invoices"))
    }
    async fn get_sales_order(&self, _id: &str) -> Result<SalesOrder> {
        Err(anyhow!("Zavora ERA has no sales orders — use estimates/invoices"))
    }
    async fn create_sales_order_draft(&self, _c: &str, _i: &[LineItemInput]) -> Result<SalesOrder> {
        Err(anyhow!("Zavora ERA has no sales orders — use create_invoice_draft"))
    }
    async fn submit_sales_order(&self, _id: &str) -> Result<SalesOrder> {
        Err(anyhow!("Zavora ERA has no sales orders"))
    }
    async fn list_purchase_orders(&self, _limit: u32) -> Result<Vec<PurchaseOrder>> {
        Err(anyhow!("Zavora ERA has no purchase orders — use vendor bills (list_bills)"))
    }
    async fn get_purchase_order(&self, _id: &str) -> Result<PurchaseOrder> {
        Err(anyhow!("Zavora ERA has no purchase orders — use vendor bills (get_bill)"))
    }
    async fn create_purchase_order_draft(&self, _v: &str, _i: &[LineItemInput]) -> Result<PurchaseOrder> {
        Err(anyhow!("Zavora ERA has no purchase orders — use create_bill_draft"))
    }
    async fn submit_purchase_order(&self, _id: &str) -> Result<PurchaseOrder> {
        Err(anyhow!("Zavora ERA has no purchase orders"))
    }

    // Invoices (AR)
    async fn list_invoices(&self, limit: u32) -> Result<Vec<Invoice>> {
        let resp = self.get(&format!("invoices?limit={limit}")).await?;
        Ok(Self::rows(resp).iter().map(Self::map_invoice).collect())
    }

    async fn get_invoice(&self, id: &str) -> Result<Invoice> {
        Ok(Self::map_invoice(&self.get(&format!("invoices/{id}")).await?))
    }

    async fn create_invoice_draft(&self, customer_id: &str, items: &[LineItemInput]) -> Result<Invoice> {
        let lines: Vec<Value> = items
            .iter()
            .map(|l| json!({
                "product_id": l.product_id,
                "description": l.description,
                "quantity": l.quantity,
                "unit_price": l.unit_price,
            }))
            .collect();
        let body = json!({"customer_id": customer_id, "lines": lines});
        Ok(Self::map_invoice(&self.post("invoices", &body).await?))
    }

    async fn submit_invoice(&self, _id: &str) -> Result<Invoice> {
        Err(anyhow!("Zavora ERA invoices go draft → posted directly; use post_invoice"))
    }

    async fn post_invoice(&self, id: &str) -> Result<Invoice> {
        Ok(Self::map_invoice(&self.post_action(&format!("invoices/{id}/post")).await?))
    }

    // Inventory
    async fn get_stock_levels(&self, product_id: Option<&str>) -> Result<Vec<StockLevel>> {
        let resp = self.get("inventory").await?;
        Ok(Self::rows(resp)
            .iter()
            .filter(|r| product_id.is_none_or(|p| r["id"].as_str() == Some(p) || r["product_id"].as_str() == Some(p)))
            .map(|r| StockLevel {
                product_id: s(&r["id"]),
                product_name: r["name"].as_str().map(Into::into),
                warehouse: None,
                quantity_on_hand: num(&r["quantity_on_hand"]).unwrap_or(0.0),
                quantity_available: num(&r["quantity_available"]),
                backend: "zavora".into(),
            })
            .collect())
    }

    async fn adjust_stock(&self, product_id: &str, quantity: f64, reason: &str) -> Result<StockLevel> {
        let body = json!({"item_id": product_id, "quantity": quantity, "reason": reason});
        let r = self.post("inventory/adjust", &body).await?;
        Ok(StockLevel {
            product_id: product_id.into(),
            product_name: r["name"].as_str().map(Into::into),
            warehouse: None,
            quantity_on_hand: num(&r["quantity_on_hand"]).unwrap_or(quantity),
            quantity_available: None,
            backend: "zavora".into(),
        })
    }

    async fn transfer_stock(&self, _p: &str, _f: &str, _t: &str, _q: f64) -> Result<()> {
        Err(anyhow!("Zavora ERA does not support warehouse transfers"))
    }

    // General ledger
    async fn list_accounts(&self) -> Result<Vec<Account>> {
        let resp = self.get("accounts").await?;
        Ok(Self::rows(resp)
            .iter()
            .map(|a| Account {
                id: s(&a["id"]),
                code: a["code"].as_str().map(Into::into),
                name: s(&a["name"]),
                account_type: a["account_type"].as_str().map(Into::into),
                balance: None,
                backend: "zavora".into(),
            })
            .collect())
    }

    async fn get_journal_entries(&self, from: &str, to: &str) -> Result<Vec<JournalEntry>> {
        let resp = self.get(&format!("journal-entries?from={from}&to={to}&limit=200")).await?;
        Ok(Self::rows(resp)
            .iter()
            .map(|j| JournalEntry {
                id: s(&j["id"]),
                date: s(&j["date"]),
                description: Some(format!("{} — {}", s(&j["number"]), s(&j["description"]))),
                debit_account: None,
                credit_account: None,
                amount: num(&j["total"]).unwrap_or(0.0),
                backend: "zavora".into(),
            })
            .collect())
    }

    async fn get_trial_balance(&self, as_of: &str) -> Result<Value> {
        self.run_report("TrialBalance", Some(as_of), None, None).await
    }

    // Governance
    async fn get_audit_trail(&self, entity_type: &str, entity_id: &str) -> Result<Vec<AuditEntry>> {
        let resp = self.get(&format!("audit/{entity_type}/{entity_id}")).await?;
        Ok(Self::rows(resp)
            .iter()
            .map(|e| AuditEntry {
                timestamp: e["at"].as_str().or(e["created_at"].as_str()).unwrap_or("").into(),
                user: e["user_id"].as_str().map(Into::into),
                action: s(&e["action"]),
                entity_type: entity_type.into(),
                entity_id: entity_id.into(),
                details: e["details"].as_str().map(Into::into),
            })
            .collect())
    }

    // ─── Accountant extensions (Zavora-native) ──────────────────────────────

    async fn list_bills(&self, limit: u32, status: Option<&str>) -> Result<Value> {
        let mut path = format!("bills?limit={limit}");
        if let Some(st) = status {
            path.push_str(&format!("&status={st}"));
        }
        self.get(&path).await
    }

    async fn get_bill(&self, id: &str) -> Result<Value> {
        self.get(&format!("bills/{id}")).await
    }

    async fn create_bill_draft(&self, input: &CreateBillInput) -> Result<Value> {
        let lines: Vec<Value> = input
            .line_items
            .iter()
            .map(|l| json!({
                "product_id": l.product_id,
                "description": l.description,
                "quantity": l.quantity,
                "unit_price": l.unit_price,
            }))
            .collect();
        let body = json!({
            "vendor_id": input.vendor_id,
            "vendor_invoice_number": input.vendor_invoice_number,
            "issue_date": input.issue_date,
            "due_date": input.due_date,
            "currency": input.currency,
            "fx_rate": input.fx_rate,
            "lines": lines,
            "notes": input.notes,
        });
        self.post("bills", &body).await
    }

    async fn post_bill(&self, id: &str) -> Result<Value> {
        self.post_action(&format!("bills/{id}/post")).await
    }

    async fn list_payments(&self, limit: u32) -> Result<Value> {
        self.get(&format!("payments?limit={limit}")).await
    }

    async fn record_payment(&self, input: &RecordPaymentInput) -> Result<Value> {
        let method = match input.method.as_str() {
            "mpesa" => json!({"Mpesa": {"transaction_id": input.reference.clone().unwrap_or_default(), "phone": ""}}),
            "cash" => json!("Cash"),
            "cheque" => json!({"Cheque": {"number": input.reference.clone().unwrap_or_default()}}),
            _ => json!({"BankTransfer": {"reference": input.reference.clone().unwrap_or_default()}}),
        };
        let applications: Vec<Value> = input
            .applications
            .iter()
            .map(|a| json!({"document_id": a.document_id, "amount": a.amount}))
            .collect();
        let body = json!({
            "payment_type": input.payment_type,
            "party_id": input.party_id,
            "payment_date": input.payment_date,
            "amount": input.amount,
            "currency": input.currency,
            "fx_rate": input.fx_rate,
            "method": method,
            "reference": input.reference,
            "bank_account_id": input.bank_account_id,
            "applications": applications,
            "wht_amount": input.wht_amount,
            "funding_account": input.funding_account,
        });
        self.post("payments", &body).await
    }

    async fn run_report(&self, report_type: &str, as_at: Option<&str>, from: Option<&str>, to: Option<&str>) -> Result<Value> {
        let (_, entity_id) = self.credentials().await?;
        let body = json!({
            "entity_id": entity_id,
            "report_type": report_type,
            "parameters": {
                "as_at": as_at,
                "period_from": from,
                "period_to": to,
            },
        });
        self.post("agent/report", &body).await
    }

    async fn get_dashboard(&self) -> Result<Value> {
        self.get("dashboard").await
    }

    async fn list_bank_accounts(&self) -> Result<Value> {
        self.get("bank-accounts").await
    }

    async fn post_journal_entry(&self, input: &PostJournalInput) -> Result<Value> {
        let lines: Vec<Value> = input
            .lines
            .iter()
            .map(|l| json!({
                "account_code": l.account_code,
                "debit": l.debit,
                "credit": l.credit,
                "currency": l.currency.clone().unwrap_or_else(|| "KES".into()),
                "fx_rate": l.fx_rate,
                "description": l.description,
            }))
            .collect();
        let body = json!({
            "date": input.date,
            "source": "Manual",
            "reference": input.reference,
            "description": input.description,
            "lines": lines,
            "post_immediately": true,
        });
        self.post("journal-entries", &body).await
    }

    // ─── HR & Payroll (Zavora-native) ───────────────────────────────────────

    async fn list_employees(&self, limit: u32) -> Result<Value> {
        self.get(&format!("employees?limit={limit}")).await
    }

    async fn list_fiscal_periods(&self) -> Result<Value> {
        self.get("periods").await
    }

    async fn list_departments(&self) -> Result<Value> {
        self.get("payroll/departments").await
    }

    async fn list_pay_runs(&self) -> Result<Value> {
        self.get("payroll").await
    }

    async fn get_pay_run(&self, id: &str) -> Result<Value> {
        self.get(&format!("payroll/{id}")).await
    }

    async fn run_payroll(&self, period_id: &str, pay_date: &str) -> Result<Value> {
        let body = json!({
            "period_id": period_id,
            "pay_date": pay_date,
            "run_by": {"type": "Agent", "id": "amos"},
        });
        self.post("payroll/run", &body).await
    }

    async fn add_pay_run_input(&self, run_id: &str, input: &PayRunInputInput) -> Result<Value> {
        let body = json!({
            "employee_id": input.employee_id,
            "kind": input.kind,
            "name": input.name,
            "amount": input.amount,
            "taxable": input.taxable,
            "type_code": input.type_code,
        });
        self.post(&format!("payroll/{run_id}/inputs"), &body).await
    }

    async fn recompute_pay_run(&self, id: &str) -> Result<Value> {
        self.post_action(&format!("payroll/{id}/recompute")).await
    }

    async fn approve_pay_run(&self, id: &str) -> Result<Value> {
        self.post_action(&format!("payroll/{id}/approve")).await
    }

    async fn post_pay_run(&self, id: &str) -> Result<Value> {
        self.post_action(&format!("payroll/{id}/post")).await
    }

    async fn mark_pay_run_paid(&self, id: &str) -> Result<Value> {
        self.post_action(&format!("payroll/{id}/paid")).await
    }

    // ─── Procurement (P2P) ───────────────────────────────────────────────────
    async fn list_requisitions(&self) -> Result<Value> { self.get("requisitions").await }
    async fn create_requisition(&self, body: &Value) -> Result<Value> { self.post("requisitions", body).await }
    async fn approve_requisition(&self, id: &str) -> Result<Value> { self.post_action(&format!("requisitions/{id}/approve")).await }
    async fn convert_requisition(&self, id: &str, body: &Value) -> Result<Value> { self.post(&format!("requisitions/{id}/convert"), body).await }
    async fn create_direct_po(&self, body: &Value) -> Result<Value> { self.post("purchase-orders", body).await }
    async fn send_purchase_order(&self, id: &str, body: &Value) -> Result<Value> { self.post(&format!("purchase-orders/{id}/send"), body).await }
    async fn receive_goods(&self, po_id: &str, body: &Value) -> Result<Value> { self.post(&format!("purchase-orders/{po_id}/receipts"), body).await }
    async fn three_way_match(&self, po_id: &str) -> Result<Value> { self.get(&format!("purchase-orders/{po_id}/match")).await }
    async fn create_debit_note(&self, body: &Value) -> Result<Value> { self.post("debit-notes", body).await }
    async fn list_expense_claims(&self) -> Result<Value> { self.get("expense-claims").await }
    async fn create_expense_claim(&self, body: &Value) -> Result<Value> { self.post("expense-claims", body).await }
    async fn approve_expense_claim(&self, id: &str) -> Result<Value> { self.post_action(&format!("expense-claims/{id}/approve")).await }
    async fn procurement_analytics(&self) -> Result<Value> { self.get("procurement/analytics").await }
    async fn budget_control(&self) -> Result<Value> { self.get("procurement/budget-control").await }
    async fn etims_status(&self) -> Result<Value> { self.get("etims/config").await }
    async fn etims_transmit_invoice(&self, id: &str) -> Result<Value> { self.post_action(&format!("etims/invoices/{id}/transmit")).await }
    async fn list_reconciliations(&self) -> Result<Value> { self.get("bank/reconciliations").await }
    async fn compute_reconciliation(&self, body: &Value) -> Result<Value> { self.post("bank/reconciliations/compute", body).await }
    async fn complete_reconciliation(&self, body: &Value) -> Result<Value> { self.post("bank/reconciliations/complete", body).await }
    async fn close_period(&self, id: &str) -> Result<Value> { self.post_action(&format!("periods/{id}/close")).await }
    async fn reopen_period(&self, id: &str) -> Result<Value> { self.post_action(&format!("periods/{id}/reopen")).await }
    async fn list_tax_filings(&self) -> Result<Value> { self.get("tax-filings").await }
    async fn file_tax_return(&self, body: &Value) -> Result<Value> { self.post("tax-filings", body).await }
    async fn remit_tax_filing(&self, id: &str, body: &Value) -> Result<Value> { self.post(&format!("tax-filings/{id}/remit"), body).await }
    async fn cit_estimate(&self, fiscal_year: Option<i32>, adjustments: Option<f64>) -> Result<Value> {
        let mut q = Vec::new();
        if let Some(y) = fiscal_year { q.push(format!("fiscal_year={y}")); }
        if let Some(a) = adjustments { q.push(format!("adjustments={a}")); }
        let qs = if q.is_empty() { String::new() } else { format!("?{}", q.join("&")) };
        self.get(&format!("tax/cit/estimate{qs}")).await
    }
    async fn send_customer_statement(&self, id: &str, body: &Value) -> Result<Value> { self.post(&format!("customers/{id}/send-statement"), body).await }
    async fn list_fixed_assets(&self) -> Result<Value> { self.get("assets").await }
    async fn run_depreciation(&self, date: Option<&str>) -> Result<Value> {
        let qs = date.map(|d| format!("?date={d}")).unwrap_or_default();
        self.post_action(&format!("assets/depreciation/run{qs}")).await
    }
    async fn run_fx_revaluation(&self, date: Option<&str>) -> Result<Value> {
        let qs = date.map(|d| format!("?date={d}")).unwrap_or_default();
        self.post_action(&format!("fx/revaluation{qs}")).await
    }
    async fn import_bank_statement(&self, body: &Value) -> Result<Value> { self.post("bank/import", body).await }
    async fn list_budgets(&self) -> Result<Value> { self.get("budgets").await }
    async fn set_budget(&self, body: &Value) -> Result<Value> { self.put("budgets", body).await }
}
