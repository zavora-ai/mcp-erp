//! Zoho Books/Inventory REST backend.
use crate::types::*;
use anyhow::Result;
use reqwest::Client;

const BASE: &str = "https://www.zohoapis.com/books/v3";

#[derive(Clone)]
pub struct ZohoBackend {
    http: Client,
    token: String,
    org_id: String,
}

impl ZohoBackend {
    pub fn new(token: String, org_id: String) -> Self {
        Self { http: Client::new(), token, org_id }
    }

    async fn get(&self, path: &str) -> Result<serde_json::Value> {
        Ok(self.http.get(format!("{BASE}/{path}?organization_id={}", self.org_id))
            .header("Authorization", format!("Zoho-oauthtoken {}", self.token))
            .send().await?.error_for_status()?.json().await?)
    }

    async fn post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        Ok(self.http.post(format!("{BASE}/{path}?organization_id={}", self.org_id))
            .header("Authorization", format!("Zoho-oauthtoken {}", self.token))
            .json(body).send().await?.error_for_status()?.json().await?)
    }

    async fn put(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        Ok(self.http.put(format!("{BASE}/{path}?organization_id={}", self.org_id))
            .header("Authorization", format!("Zoho-oauthtoken {}", self.token))
            .json(body).send().await?.error_for_status()?.json().await?)
    }

    async fn post_action(&self, path: &str) -> Result<serde_json::Value> {
        Ok(self.http.post(format!("{BASE}/{path}?organization_id={}", self.org_id))
            .header("Authorization", format!("Zoho-oauthtoken {}", self.token))
            .send().await?.error_for_status()?.json().await?)
    }
}

#[async_trait::async_trait]
impl ErpBackend for ZohoBackend {
    fn name(&self) -> &str { "zoho" }

    async fn list_customers(&self, limit: u32) -> Result<Vec<Customer>> {
        let resp = self.get(&format!("contacts?contact_type=customer&per_page={limit}")).await?;
        Ok(resp["contacts"].as_array().map(|a| a.iter().map(|c| Customer {
            id: c["contact_id"].as_str().unwrap_or("").into(),
            name: c["contact_name"].as_str().unwrap_or("").into(),
            email: c["email"].as_str().map(Into::into),
            phone: c["phone"].as_str().map(Into::into),
            currency: c["currency_code"].as_str().map(Into::into),
            balance: c["outstanding_receivable_amount"].as_f64(),
            backend: "zoho".into(),
        }).collect()).unwrap_or_default())
    }

    async fn get_customer(&self, id: &str) -> Result<Customer> {
        let resp = self.get(&format!("contacts/{id}")).await?;
        let c = &resp["contact"];
        Ok(Customer {
            id: c["contact_id"].as_str().unwrap_or("").into(),
            name: c["contact_name"].as_str().unwrap_or("").into(),
            email: c["email"].as_str().map(Into::into),
            phone: c["phone"].as_str().map(Into::into),
            currency: c["currency_code"].as_str().map(Into::into),
            balance: c["outstanding_receivable_amount"].as_f64(),
            backend: "zoho".into(),
        })
    }

    async fn create_customer(&self, name: &str, email: Option<&str>, phone: Option<&str>) -> Result<Customer> {
        let mut body = serde_json::json!({"contact_name": name, "contact_type": "customer"});
        if let Some(e) = email { body["email"] = e.into(); }
        if let Some(p) = phone { body["phone"] = p.into(); }
        let resp = self.post("contacts", &body).await?;
        let c = &resp["contact"];
        Ok(Customer { id: c["contact_id"].as_str().unwrap_or("").into(), name: name.into(), email: email.map(Into::into), phone: phone.map(Into::into), currency: None, balance: None, backend: "zoho".into() })
    }

    async fn update_customer(&self, id: &str, name: Option<&str>, email: Option<&str>, phone: Option<&str>) -> Result<Customer> {
        let mut body = serde_json::json!({});
        if let Some(n) = name { body["contact_name"] = n.into(); }
        if let Some(e) = email { body["email"] = e.into(); }
        if let Some(p) = phone { body["phone"] = p.into(); }
        let resp = self.put(&format!("contacts/{id}"), &body).await?;
        let c = &resp["contact"];
        Ok(Customer { id: id.into(), name: c["contact_name"].as_str().unwrap_or("").into(), email: c["email"].as_str().map(Into::into), phone: c["phone"].as_str().map(Into::into), currency: None, balance: None, backend: "zoho".into() })
    }

    async fn list_vendors(&self, limit: u32) -> Result<Vec<Vendor>> {
        let resp = self.get(&format!("contacts?contact_type=vendor&per_page={limit}")).await?;
        Ok(resp["contacts"].as_array().map(|a| a.iter().map(|c| Vendor {
            id: c["contact_id"].as_str().unwrap_or("").into(),
            name: c["contact_name"].as_str().unwrap_or("").into(),
            email: c["email"].as_str().map(Into::into),
            phone: c["phone"].as_str().map(Into::into),
            currency: c["currency_code"].as_str().map(Into::into),
            balance: c["outstanding_payable_amount"].as_f64(),
            backend: "zoho".into(),
        }).collect()).unwrap_or_default())
    }

    async fn get_vendor(&self, id: &str) -> Result<Vendor> {
        let resp = self.get(&format!("contacts/{id}")).await?;
        let c = &resp["contact"];
        Ok(Vendor { id: c["contact_id"].as_str().unwrap_or("").into(), name: c["contact_name"].as_str().unwrap_or("").into(), email: c["email"].as_str().map(Into::into), phone: c["phone"].as_str().map(Into::into), currency: c["currency_code"].as_str().map(Into::into), balance: c["outstanding_payable_amount"].as_f64(), backend: "zoho".into() })
    }

    async fn create_vendor(&self, name: &str, email: Option<&str>, phone: Option<&str>) -> Result<Vendor> {
        let mut body = serde_json::json!({"contact_name": name, "contact_type": "vendor"});
        if let Some(e) = email { body["email"] = e.into(); }
        if let Some(p) = phone { body["phone"] = p.into(); }
        let resp = self.post("contacts", &body).await?;
        let c = &resp["contact"];
        Ok(Vendor { id: c["contact_id"].as_str().unwrap_or("").into(), name: name.into(), email: email.map(Into::into), phone: phone.map(Into::into), currency: None, balance: None, backend: "zoho".into() })
    }

    async fn update_vendor(&self, id: &str, name: Option<&str>, email: Option<&str>, phone: Option<&str>) -> Result<Vendor> {
        let mut body = serde_json::json!({});
        if let Some(n) = name { body["contact_name"] = n.into(); }
        if let Some(e) = email { body["email"] = e.into(); }
        if let Some(p) = phone { body["phone"] = p.into(); }
        let resp = self.put(&format!("contacts/{id}"), &body).await?;
        let c = &resp["contact"];
        Ok(Vendor { id: id.into(), name: c["contact_name"].as_str().unwrap_or("").into(), email: c["email"].as_str().map(Into::into), phone: c["phone"].as_str().map(Into::into), currency: None, balance: None, backend: "zoho".into() })
    }

    // Products
    async fn list_products(&self, limit: u32) -> Result<Vec<Product>> {
        let resp = self.get(&format!("items?per_page={limit}")).await?;
        Ok(resp["items"].as_array().map(|a| a.iter().map(|p| Product {
            id: p["item_id"].as_str().unwrap_or("").into(),
            name: p["name"].as_str().unwrap_or("").into(),
            sku: p["sku"].as_str().map(Into::into),
            unit_price: p["rate"].as_f64(),
            currency: None,
            stock_on_hand: p["stock_on_hand"].as_f64(),
            backend: "zoho".into(),
        }).collect()).unwrap_or_default())
    }

    async fn get_product(&self, id: &str) -> Result<Product> {
        let resp = self.get(&format!("items/{id}")).await?;
        let p = &resp["item"];
        Ok(Product { id: p["item_id"].as_str().unwrap_or("").into(), name: p["name"].as_str().unwrap_or("").into(), sku: p["sku"].as_str().map(Into::into), unit_price: p["rate"].as_f64(), currency: None, stock_on_hand: p["stock_on_hand"].as_f64(), backend: "zoho".into() })
    }

    async fn create_product(&self, name: &str, sku: Option<&str>, price: Option<f64>) -> Result<Product> {
        let mut body = serde_json::json!({"name": name});
        if let Some(s) = sku { body["sku"] = s.into(); }
        if let Some(p) = price { body["rate"] = p.into(); }
        let resp = self.post("items", &body).await?;
        let p = &resp["item"];
        Ok(Product { id: p["item_id"].as_str().unwrap_or("").into(), name: name.into(), sku: sku.map(Into::into), unit_price: price, currency: None, stock_on_hand: None, backend: "zoho".into() })
    }

    async fn update_product(&self, id: &str, name: Option<&str>, sku: Option<&str>, price: Option<f64>) -> Result<Product> {
        let mut body = serde_json::json!({});
        if let Some(n) = name { body["name"] = n.into(); }
        if let Some(s) = sku { body["sku"] = s.into(); }
        if let Some(p) = price { body["rate"] = p.into(); }
        let resp = self.put(&format!("items/{id}"), &body).await?;
        let p = &resp["item"];
        Ok(Product { id: id.into(), name: p["name"].as_str().unwrap_or("").into(), sku: p["sku"].as_str().map(Into::into), unit_price: p["rate"].as_f64(), currency: None, stock_on_hand: p["stock_on_hand"].as_f64(), backend: "zoho".into() })
    }

    // Sales Orders
    async fn list_sales_orders(&self, limit: u32) -> Result<Vec<SalesOrder>> {
        let resp = self.get(&format!("salesorders?per_page={limit}")).await?;
        Ok(resp["salesorders"].as_array().map(|a| a.iter().map(|o| SalesOrder {
            id: o["salesorder_id"].as_str().unwrap_or("").into(),
            customer_id: o["customer_id"].as_str().unwrap_or("").into(),
            customer_name: o["customer_name"].as_str().map(Into::into),
            state: zoho_order_state(o["status"].as_str().unwrap_or("")),
            total: o["total"].as_f64().unwrap_or(0.0),
            currency: o["currency_code"].as_str().map(Into::into),
            line_items: vec![],
            created_at: o["date"].as_str().map(Into::into),
            backend: "zoho".into(),
        }).collect()).unwrap_or_default())
    }

    async fn get_sales_order(&self, id: &str) -> Result<SalesOrder> {
        let resp = self.get(&format!("salesorders/{id}")).await?;
        let o = &resp["salesorder"];
        let items = o["line_items"].as_array().map(|a| a.iter().map(|li| LineItem {
            product_id: li["item_id"].as_str().map(Into::into),
            description: li["description"].as_str().unwrap_or("").into(),
            quantity: li["quantity"].as_f64().unwrap_or(0.0),
            unit_price: li["rate"].as_f64().unwrap_or(0.0),
            amount: li["item_total"].as_f64().unwrap_or(0.0),
        }).collect()).unwrap_or_default();
        Ok(SalesOrder { id: o["salesorder_id"].as_str().unwrap_or("").into(), customer_id: o["customer_id"].as_str().unwrap_or("").into(), customer_name: o["customer_name"].as_str().map(Into::into), state: zoho_order_state(o["status"].as_str().unwrap_or("")), total: o["total"].as_f64().unwrap_or(0.0), currency: o["currency_code"].as_str().map(Into::into), line_items: items, created_at: o["date"].as_str().map(Into::into), backend: "zoho".into() })
    }

    async fn create_sales_order_draft(&self, customer_id: &str, items: &[LineItemInput]) -> Result<SalesOrder> {
        let line_items: Vec<_> = items.iter().map(|li| {
            let mut j = serde_json::json!({"description": li.description, "quantity": li.quantity, "rate": li.unit_price});
            if let Some(ref pid) = li.product_id { j["item_id"] = pid.clone().into(); }
            j
        }).collect();
        let body = serde_json::json!({"customer_id": customer_id, "line_items": line_items});
        let resp = self.post("salesorders", &body).await?;
        let o = &resp["salesorder"];
        Ok(SalesOrder { id: o["salesorder_id"].as_str().unwrap_or("").into(), customer_id: customer_id.into(), customer_name: None, state: LifecycleState::Draft, total: o["total"].as_f64().unwrap_or(0.0), currency: None, line_items: vec![], created_at: None, backend: "zoho".into() })
    }

    async fn submit_sales_order(&self, id: &str) -> Result<SalesOrder> {
        self.post_action(&format!("salesorders/{id}/status/confirmed")).await?;
        self.get_sales_order(id).await
    }

    // Purchase Orders
    async fn list_purchase_orders(&self, limit: u32) -> Result<Vec<PurchaseOrder>> {
        let resp = self.get(&format!("purchaseorders?per_page={limit}")).await?;
        Ok(resp["purchaseorders"].as_array().map(|a| a.iter().map(|o| PurchaseOrder {
            id: o["purchaseorder_id"].as_str().unwrap_or("").into(),
            vendor_id: o["vendor_id"].as_str().unwrap_or("").into(),
            vendor_name: o["vendor_name"].as_str().map(Into::into),
            state: zoho_order_state(o["status"].as_str().unwrap_or("")),
            total: o["total"].as_f64().unwrap_or(0.0),
            currency: o["currency_code"].as_str().map(Into::into),
            line_items: vec![],
            created_at: o["date"].as_str().map(Into::into),
            backend: "zoho".into(),
        }).collect()).unwrap_or_default())
    }

    async fn get_purchase_order(&self, id: &str) -> Result<PurchaseOrder> {
        let resp = self.get(&format!("purchaseorders/{id}")).await?;
        let o = &resp["purchaseorder"];
        let items = o["line_items"].as_array().map(|a| a.iter().map(|li| LineItem {
            product_id: li["item_id"].as_str().map(Into::into),
            description: li["description"].as_str().unwrap_or("").into(),
            quantity: li["quantity"].as_f64().unwrap_or(0.0),
            unit_price: li["rate"].as_f64().unwrap_or(0.0),
            amount: li["item_total"].as_f64().unwrap_or(0.0),
        }).collect()).unwrap_or_default();
        Ok(PurchaseOrder { id: o["purchaseorder_id"].as_str().unwrap_or("").into(), vendor_id: o["vendor_id"].as_str().unwrap_or("").into(), vendor_name: o["vendor_name"].as_str().map(Into::into), state: zoho_order_state(o["status"].as_str().unwrap_or("")), total: o["total"].as_f64().unwrap_or(0.0), currency: o["currency_code"].as_str().map(Into::into), line_items: items, created_at: o["date"].as_str().map(Into::into), backend: "zoho".into() })
    }

    async fn create_purchase_order_draft(&self, vendor_id: &str, items: &[LineItemInput]) -> Result<PurchaseOrder> {
        let line_items: Vec<_> = items.iter().map(|li| {
            let mut j = serde_json::json!({"description": li.description, "quantity": li.quantity, "rate": li.unit_price});
            if let Some(ref pid) = li.product_id { j["item_id"] = pid.clone().into(); }
            j
        }).collect();
        let body = serde_json::json!({"vendor_id": vendor_id, "line_items": line_items});
        let resp = self.post("purchaseorders", &body).await?;
        let o = &resp["purchaseorder"];
        Ok(PurchaseOrder { id: o["purchaseorder_id"].as_str().unwrap_or("").into(), vendor_id: vendor_id.into(), vendor_name: None, state: LifecycleState::Draft, total: o["total"].as_f64().unwrap_or(0.0), currency: None, line_items: vec![], created_at: None, backend: "zoho".into() })
    }

    async fn submit_purchase_order(&self, id: &str) -> Result<PurchaseOrder> {
        self.post_action(&format!("purchaseorders/{id}/status/open")).await?;
        self.get_purchase_order(id).await
    }

    // Invoices
    async fn list_invoices(&self, limit: u32) -> Result<Vec<Invoice>> {
        let resp = self.get(&format!("invoices?per_page={limit}")).await?;
        Ok(resp["invoices"].as_array().map(|a| a.iter().map(|inv| Invoice {
            id: inv["invoice_id"].as_str().unwrap_or("").into(),
            customer_id: inv["customer_id"].as_str().map(Into::into),
            customer_name: inv["customer_name"].as_str().map(Into::into),
            state: zoho_invoice_state(inv["status"].as_str().unwrap_or("")),
            total: inv["total"].as_f64().unwrap_or(0.0),
            balance_due: inv["balance"].as_f64(),
            currency: inv["currency_code"].as_str().map(Into::into),
            line_items: vec![],
            due_date: inv["due_date"].as_str().map(Into::into),
            created_at: inv["date"].as_str().map(Into::into),
            backend: "zoho".into(),
        }).collect()).unwrap_or_default())
    }

    async fn get_invoice(&self, id: &str) -> Result<Invoice> {
        let resp = self.get(&format!("invoices/{id}")).await?;
        let inv = &resp["invoice"];
        let items = inv["line_items"].as_array().map(|a| a.iter().map(|li| LineItem {
            product_id: li["item_id"].as_str().map(Into::into),
            description: li["description"].as_str().unwrap_or("").into(),
            quantity: li["quantity"].as_f64().unwrap_or(0.0),
            unit_price: li["rate"].as_f64().unwrap_or(0.0),
            amount: li["item_total"].as_f64().unwrap_or(0.0),
        }).collect()).unwrap_or_default();
        Ok(Invoice { id: inv["invoice_id"].as_str().unwrap_or("").into(), customer_id: inv["customer_id"].as_str().map(Into::into), customer_name: inv["customer_name"].as_str().map(Into::into), state: zoho_invoice_state(inv["status"].as_str().unwrap_or("")), total: inv["total"].as_f64().unwrap_or(0.0), balance_due: inv["balance"].as_f64(), currency: inv["currency_code"].as_str().map(Into::into), line_items: items, due_date: inv["due_date"].as_str().map(Into::into), created_at: inv["date"].as_str().map(Into::into), backend: "zoho".into() })
    }

    async fn create_invoice_draft(&self, customer_id: &str, items: &[LineItemInput]) -> Result<Invoice> {
        let line_items: Vec<_> = items.iter().map(|li| {
            let mut j = serde_json::json!({"description": li.description, "quantity": li.quantity, "rate": li.unit_price});
            if let Some(ref pid) = li.product_id { j["item_id"] = pid.clone().into(); }
            j
        }).collect();
        let body = serde_json::json!({"customer_id": customer_id, "line_items": line_items, "is_draft": true});
        let resp = self.post("invoices", &body).await?;
        let inv = &resp["invoice"];
        Ok(Invoice { id: inv["invoice_id"].as_str().unwrap_or("").into(), customer_id: Some(customer_id.into()), customer_name: None, state: LifecycleState::Draft, total: inv["total"].as_f64().unwrap_or(0.0), balance_due: None, currency: None, line_items: vec![], due_date: None, created_at: None, backend: "zoho".into() })
    }

    async fn submit_invoice(&self, id: &str) -> Result<Invoice> {
        self.post_action(&format!("invoices/{id}/status/sent")).await?;
        self.get_invoice(id).await
    }

    async fn post_invoice(&self, id: &str) -> Result<Invoice> {
        // Zoho doesn't have a separate "post" — sending marks it as active
        self.submit_invoice(id).await
    }

    // Inventory
    async fn get_stock_levels(&self, product_id: Option<&str>) -> Result<Vec<StockLevel>> {
        let path = match product_id {
            Some(id) => format!("items/{id}"),
            None => "items?per_page=50".into(),
        };
        let resp = self.get(&path).await?;
        if let Some(item) = resp.get("item") {
            Ok(vec![StockLevel { product_id: item["item_id"].as_str().unwrap_or("").into(), product_name: item["name"].as_str().map(Into::into), warehouse: None, quantity_on_hand: item["stock_on_hand"].as_f64().unwrap_or(0.0), quantity_available: item["available_stock"].as_f64(), backend: "zoho".into() }])
        } else {
            Ok(resp["items"].as_array().map(|a| a.iter().map(|p| StockLevel { product_id: p["item_id"].as_str().unwrap_or("").into(), product_name: p["name"].as_str().map(Into::into), warehouse: None, quantity_on_hand: p["stock_on_hand"].as_f64().unwrap_or(0.0), quantity_available: p["available_stock"].as_f64(), backend: "zoho".into() }).collect()).unwrap_or_default())
        }
    }

    async fn adjust_stock(&self, product_id: &str, quantity: f64, reason: &str) -> Result<StockLevel> {
        let body = serde_json::json!({"date": chrono::Utc::now().format("%Y-%m-%d").to_string(), "reason": reason, "line_items": [{"item_id": product_id, "quantity_adjusted": quantity}]});
        self.post("inventoryadjustments", &body).await?;
        let levels = self.get_stock_levels(Some(product_id)).await?;
        levels.into_iter().next().ok_or_else(|| anyhow::anyhow!("product not found"))
    }

    async fn transfer_stock(&self, product_id: &str, from_warehouse: &str, to_warehouse: &str, quantity: f64) -> Result<()> {
        let body = serde_json::json!({"date": chrono::Utc::now().format("%Y-%m-%d").to_string(), "from_warehouse_id": from_warehouse, "to_warehouse_id": to_warehouse, "line_items": [{"item_id": product_id, "quantity_transfer": quantity}]});
        self.post("transferorders", &body).await?;
        Ok(())
    }

    // General Ledger
    async fn list_accounts(&self) -> Result<Vec<Account>> {
        let resp = self.get("chartofaccounts").await?;
        Ok(resp["chartofaccounts"].as_array().map(|a| a.iter().map(|acc| Account {
            id: acc["account_id"].as_str().unwrap_or("").into(),
            code: acc["account_code"].as_str().map(Into::into),
            name: acc["account_name"].as_str().unwrap_or("").into(),
            account_type: acc["account_type"].as_str().map(Into::into),
            balance: None,
            backend: "zoho".into(),
        }).collect()).unwrap_or_default())
    }

    async fn get_journal_entries(&self, from: &str, to: &str) -> Result<Vec<JournalEntry>> {
        let resp = self.get(&format!("journals?date_start={from}&date_end={to}")).await?;
        Ok(resp["journals"].as_array().map(|a| a.iter().map(|j| JournalEntry {
            id: j["journal_id"].as_str().unwrap_or("").into(),
            date: j["journal_date"].as_str().unwrap_or("").into(),
            description: j["notes"].as_str().map(Into::into),
            debit_account: None,
            credit_account: None,
            amount: j["total"].as_f64().unwrap_or(0.0),
            backend: "zoho".into(),
        }).collect()).unwrap_or_default())
    }

    async fn get_trial_balance(&self, as_of: &str) -> Result<serde_json::Value> {
        self.get(&format!("reports/trialbalance?date={as_of}")).await
    }

    // Governance
    async fn get_audit_trail(&self, _entity_type: &str, _entity_id: &str) -> Result<Vec<AuditEntry>> {
        // Zoho Books doesn't expose a generic audit API — return empty
        Ok(vec![])
    }
}

fn zoho_order_state(s: &str) -> LifecycleState {
    match s {
        "draft" => LifecycleState::Draft,
        "open" | "confirmed" => LifecycleState::Released,
        "closed" | "fulfilled" => LifecycleState::Fulfilled,
        "cancelled" | "void" => LifecycleState::Cancelled,
        _ => LifecycleState::Draft,
    }
}

fn zoho_invoice_state(s: &str) -> LifecycleState {
    match s {
        "draft" => LifecycleState::Draft,
        "sent" => LifecycleState::Sent,
        "overdue" | "unpaid" => LifecycleState::Posted,
        "paid" => LifecycleState::Closed,
        "void" => LifecycleState::Voided,
        _ => LifecycleState::Draft,
    }
}
