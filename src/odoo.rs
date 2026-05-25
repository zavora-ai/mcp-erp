//! Odoo JSON-RPC backend.
use crate::types::*;
use anyhow::Result;
use reqwest::Client;

#[derive(Clone)]
pub struct OdooBackend {
    http: Client,
    url: String,
    db: String,
    uid: i64,
    password: String,
}

impl OdooBackend {
    pub async fn connect(url: String, db: String, username: String, password: String) -> Result<Self> {
        let http = Client::new();
        let resp: serde_json::Value = http.post(format!("{url}/jsonrpc"))
            .json(&serde_json::json!({"jsonrpc": "2.0", "method": "call", "id": 1, "params": {"service": "common", "method": "authenticate", "args": [&db, &username, &password, {}]}}))
            .send().await?.error_for_status()?.json().await?;
        let uid = resp["result"].as_i64().ok_or_else(|| anyhow::anyhow!("Odoo auth failed"))?;
        Ok(Self { http, url, db, uid, password })
    }

    async fn call(&self, model: &str, method: &str, args: serde_json::Value, kwargs: serde_json::Value) -> Result<serde_json::Value> {
        let resp: serde_json::Value = self.http.post(format!("{}/jsonrpc", self.url))
            .json(&serde_json::json!({"jsonrpc": "2.0", "method": "call", "id": 1, "params": {"service": "object", "method": "execute_kw", "args": [&self.db, self.uid, &self.password, model, method, args, kwargs]}}))
            .send().await?.error_for_status()?.json().await?;
        if let Some(err) = resp.get("error") {
            anyhow::bail!("Odoo error: {}", err["data"]["message"].as_str().unwrap_or("unknown"));
        }
        Ok(resp["result"].clone())
    }

    async fn search_read(&self, model: &str, domain: &[serde_json::Value], fields: &[&str], limit: u32) -> Result<Vec<serde_json::Value>> {
        let result = self.call(model, "search_read", serde_json::json!([domain]), serde_json::json!({"fields": fields, "limit": limit})).await?;
        Ok(result.as_array().cloned().unwrap_or_default())
    }

    async fn create_record(&self, model: &str, vals: serde_json::Value) -> Result<i64> {
        let result = self.call(model, "create", serde_json::json!([[vals]]), serde_json::json!({})).await?;
        result.as_array().and_then(|a| a.first()).and_then(|v| v.as_i64())
            .or_else(|| result.as_i64())
            .ok_or_else(|| anyhow::anyhow!("create returned no ID"))
    }

    async fn write_record(&self, model: &str, id: i64, vals: serde_json::Value) -> Result<()> {
        self.call(model, "write", serde_json::json!([[id], vals]), serde_json::json!({})).await?;
        Ok(())
    }

    async fn read_one(&self, model: &str, id: i64, fields: &[&str]) -> Result<serde_json::Value> {
        let result = self.call(model, "read", serde_json::json!([[id]]), serde_json::json!({"fields": fields})).await?;
        result.as_array().and_then(|a| a.first()).cloned().ok_or_else(|| anyhow::anyhow!("not found"))
    }
}

fn odoo_id(v: &serde_json::Value) -> String { v["id"].as_i64().map(|i| i.to_string()).unwrap_or_default() }
fn odoo_str(v: &serde_json::Value, k: &str) -> Option<String> { v[k].as_str().filter(|s| !s.is_empty()).map(Into::into) }

#[async_trait::async_trait]
impl ErpBackend for OdooBackend {
    fn name(&self) -> &str { "odoo" }

    async fn list_customers(&self, limit: u32) -> Result<Vec<Customer>> {
        let recs = self.search_read("res.partner", &[serde_json::json!(["customer_rank", ">", 0])], &["id", "name", "email", "phone"], limit).await?;
        Ok(recs.iter().map(|r| Customer { id: odoo_id(r), name: r["name"].as_str().unwrap_or("").into(), email: odoo_str(r, "email"), phone: odoo_str(r, "phone"), currency: None, balance: None, backend: "odoo".into() }).collect())
    }

    async fn get_customer(&self, id: &str) -> Result<Customer> {
        let r = self.read_one("res.partner", id.parse()?, &["name", "email", "phone"]).await?;
        Ok(Customer { id: id.into(), name: r["name"].as_str().unwrap_or("").into(), email: odoo_str(&r, "email"), phone: odoo_str(&r, "phone"), currency: None, balance: None, backend: "odoo".into() })
    }

    async fn create_customer(&self, name: &str, email: Option<&str>, phone: Option<&str>) -> Result<Customer> {
        let mut vals = serde_json::json!({"name": name, "customer_rank": 1});
        if let Some(e) = email { vals["email"] = e.into(); }
        if let Some(p) = phone { vals["phone"] = p.into(); }
        let id = self.create_record("res.partner", vals).await?;
        Ok(Customer { id: id.to_string(), name: name.into(), email: email.map(Into::into), phone: phone.map(Into::into), currency: None, balance: None, backend: "odoo".into() })
    }

    async fn update_customer(&self, id: &str, name: Option<&str>, email: Option<&str>, phone: Option<&str>) -> Result<Customer> {
        let mut vals = serde_json::json!({});
        if let Some(n) = name { vals["name"] = n.into(); }
        if let Some(e) = email { vals["email"] = e.into(); }
        if let Some(p) = phone { vals["phone"] = p.into(); }
        self.write_record("res.partner", id.parse()?, vals).await?;
        self.get_customer(id).await
    }

    async fn list_vendors(&self, limit: u32) -> Result<Vec<Vendor>> {
        let recs = self.search_read("res.partner", &[serde_json::json!(["supplier_rank", ">", 0])], &["id", "name", "email", "phone"], limit).await?;
        Ok(recs.iter().map(|r| Vendor { id: odoo_id(r), name: r["name"].as_str().unwrap_or("").into(), email: odoo_str(r, "email"), phone: odoo_str(r, "phone"), currency: None, balance: None, backend: "odoo".into() }).collect())
    }

    async fn get_vendor(&self, id: &str) -> Result<Vendor> {
        let r = self.read_one("res.partner", id.parse()?, &["name", "email", "phone"]).await?;
        Ok(Vendor { id: id.into(), name: r["name"].as_str().unwrap_or("").into(), email: odoo_str(&r, "email"), phone: odoo_str(&r, "phone"), currency: None, balance: None, backend: "odoo".into() })
    }

    async fn create_vendor(&self, name: &str, email: Option<&str>, phone: Option<&str>) -> Result<Vendor> {
        let mut vals = serde_json::json!({"name": name, "supplier_rank": 1});
        if let Some(e) = email { vals["email"] = e.into(); }
        if let Some(p) = phone { vals["phone"] = p.into(); }
        let id = self.create_record("res.partner", vals).await?;
        Ok(Vendor { id: id.to_string(), name: name.into(), email: email.map(Into::into), phone: phone.map(Into::into), currency: None, balance: None, backend: "odoo".into() })
    }

    async fn update_vendor(&self, id: &str, name: Option<&str>, email: Option<&str>, phone: Option<&str>) -> Result<Vendor> {
        let mut vals = serde_json::json!({});
        if let Some(n) = name { vals["name"] = n.into(); }
        if let Some(e) = email { vals["email"] = e.into(); }
        if let Some(p) = phone { vals["phone"] = p.into(); }
        self.write_record("res.partner", id.parse()?, vals).await?;
        self.get_vendor(id).await
    }

    async fn list_products(&self, limit: u32) -> Result<Vec<Product>> {
        let recs = self.search_read("product.product", &[], &["id", "name", "default_code", "list_price", "qty_available"], limit).await?;
        Ok(recs.iter().map(|r| Product { id: odoo_id(r), name: r["name"].as_str().unwrap_or("").into(), sku: odoo_str(r, "default_code"), unit_price: r["list_price"].as_f64(), currency: None, stock_on_hand: r["qty_available"].as_f64(), backend: "odoo".into() }).collect())
    }

    async fn get_product(&self, id: &str) -> Result<Product> {
        let r = self.read_one("product.product", id.parse()?, &["name", "default_code", "list_price", "qty_available"]).await?;
        Ok(Product { id: id.into(), name: r["name"].as_str().unwrap_or("").into(), sku: odoo_str(&r, "default_code"), unit_price: r["list_price"].as_f64(), currency: None, stock_on_hand: r["qty_available"].as_f64(), backend: "odoo".into() })
    }

    async fn create_product(&self, name: &str, sku: Option<&str>, price: Option<f64>) -> Result<Product> {
        let mut vals = serde_json::json!({"name": name, "type": "product"});
        if let Some(s) = sku { vals["default_code"] = s.into(); }
        if let Some(p) = price { vals["list_price"] = p.into(); }
        let id = self.create_record("product.product", vals).await?;
        Ok(Product { id: id.to_string(), name: name.into(), sku: sku.map(Into::into), unit_price: price, currency: None, stock_on_hand: None, backend: "odoo".into() })
    }

    async fn update_product(&self, id: &str, name: Option<&str>, sku: Option<&str>, price: Option<f64>) -> Result<Product> {
        let mut vals = serde_json::json!({});
        if let Some(n) = name { vals["name"] = n.into(); }
        if let Some(s) = sku { vals["default_code"] = s.into(); }
        if let Some(p) = price { vals["list_price"] = p.into(); }
        self.write_record("product.product", id.parse()?, vals).await?;
        self.get_product(id).await
    }

    // Sales Orders
    async fn list_sales_orders(&self, limit: u32) -> Result<Vec<SalesOrder>> {
        let recs = self.search_read("sale.order", &[], &["id", "name", "partner_id", "state", "amount_total", "date_order"], limit).await?;
        Ok(recs.iter().map(|r| SalesOrder { id: odoo_id(r), customer_id: r["partner_id"].as_array().and_then(|a| a.first()).and_then(|v| v.as_i64()).map(|i| i.to_string()).unwrap_or_default(), customer_name: r["partner_id"].as_array().and_then(|a| a.get(1)).and_then(|v| v.as_str()).map(Into::into), state: odoo_so_state(r["state"].as_str().unwrap_or("")), total: r["amount_total"].as_f64().unwrap_or(0.0), currency: None, line_items: vec![], created_at: r["date_order"].as_str().map(Into::into), backend: "odoo".into() }).collect())
    }

    async fn get_sales_order(&self, id: &str) -> Result<SalesOrder> {
        let r = self.read_one("sale.order", id.parse()?, &["name", "partner_id", "state", "amount_total", "date_order", "order_line"]).await?;
        let line_ids: Vec<i64> = r["order_line"].as_array().map(|a| a.iter().filter_map(|v| v.as_i64()).collect()).unwrap_or_default();
        let items = if !line_ids.is_empty() {
            let lines = self.call("sale.order.line", "read", serde_json::json!([line_ids]), serde_json::json!({"fields": ["product_id", "name", "product_uom_qty", "price_unit", "price_subtotal"]})).await?;
            lines.as_array().map(|a| a.iter().map(|li| LineItem { product_id: li["product_id"].as_array().and_then(|a| a.first()).and_then(|v| v.as_i64()).map(|i| i.to_string()), description: li["name"].as_str().unwrap_or("").into(), quantity: li["product_uom_qty"].as_f64().unwrap_or(0.0), unit_price: li["price_unit"].as_f64().unwrap_or(0.0), amount: li["price_subtotal"].as_f64().unwrap_or(0.0) }).collect()).unwrap_or_default()
        } else { vec![] };
        Ok(SalesOrder { id: id.into(), customer_id: r["partner_id"].as_array().and_then(|a| a.first()).and_then(|v| v.as_i64()).map(|i| i.to_string()).unwrap_or_default(), customer_name: r["partner_id"].as_array().and_then(|a| a.get(1)).and_then(|v| v.as_str()).map(Into::into), state: odoo_so_state(r["state"].as_str().unwrap_or("")), total: r["amount_total"].as_f64().unwrap_or(0.0), currency: None, line_items: items, created_at: r["date_order"].as_str().map(Into::into), backend: "odoo".into() })
    }

    async fn create_sales_order_draft(&self, customer_id: &str, items: &[LineItemInput]) -> Result<SalesOrder> {
        let lines: Vec<_> = items.iter().map(|li| {
            let mut v = serde_json::json!({"name": li.description, "product_uom_qty": li.quantity, "price_unit": li.unit_price});
            if let Some(ref pid) = li.product_id { v["product_id"] = pid.parse::<i64>().unwrap_or(0).into(); }
            serde_json::json!([0, 0, v])
        }).collect();
        let vals = serde_json::json!({"partner_id": customer_id.parse::<i64>()?, "order_line": lines});
        let id = self.create_record("sale.order", vals).await?;
        self.get_sales_order(&id.to_string()).await
    }

    async fn submit_sales_order(&self, id: &str) -> Result<SalesOrder> {
        self.call("sale.order", "action_confirm", serde_json::json!([[id.parse::<i64>()?]]), serde_json::json!({})).await?;
        self.get_sales_order(id).await
    }

    // Purchase Orders
    async fn list_purchase_orders(&self, limit: u32) -> Result<Vec<PurchaseOrder>> {
        let recs = self.search_read("purchase.order", &[], &["id", "name", "partner_id", "state", "amount_total", "date_order"], limit).await?;
        Ok(recs.iter().map(|r| PurchaseOrder { id: odoo_id(r), vendor_id: r["partner_id"].as_array().and_then(|a| a.first()).and_then(|v| v.as_i64()).map(|i| i.to_string()).unwrap_or_default(), vendor_name: r["partner_id"].as_array().and_then(|a| a.get(1)).and_then(|v| v.as_str()).map(Into::into), state: odoo_po_state(r["state"].as_str().unwrap_or("")), total: r["amount_total"].as_f64().unwrap_or(0.0), currency: None, line_items: vec![], created_at: r["date_order"].as_str().map(Into::into), backend: "odoo".into() }).collect())
    }

    async fn get_purchase_order(&self, id: &str) -> Result<PurchaseOrder> {
        let r = self.read_one("purchase.order", id.parse()?, &["name", "partner_id", "state", "amount_total", "date_order"]).await?;
        Ok(PurchaseOrder { id: id.into(), vendor_id: r["partner_id"].as_array().and_then(|a| a.first()).and_then(|v| v.as_i64()).map(|i| i.to_string()).unwrap_or_default(), vendor_name: r["partner_id"].as_array().and_then(|a| a.get(1)).and_then(|v| v.as_str()).map(Into::into), state: odoo_po_state(r["state"].as_str().unwrap_or("")), total: r["amount_total"].as_f64().unwrap_or(0.0), currency: None, line_items: vec![], created_at: r["date_order"].as_str().map(Into::into), backend: "odoo".into() })
    }

    async fn create_purchase_order_draft(&self, vendor_id: &str, items: &[LineItemInput]) -> Result<PurchaseOrder> {
        let lines: Vec<_> = items.iter().map(|li| {
            let mut v = serde_json::json!({"name": li.description, "product_qty": li.quantity, "price_unit": li.unit_price});
            if let Some(ref pid) = li.product_id { v["product_id"] = pid.parse::<i64>().unwrap_or(0).into(); }
            serde_json::json!([0, 0, v])
        }).collect();
        let vals = serde_json::json!({"partner_id": vendor_id.parse::<i64>()?, "order_line": lines});
        let id = self.create_record("purchase.order", vals).await?;
        self.get_purchase_order(&id.to_string()).await
    }

    async fn submit_purchase_order(&self, id: &str) -> Result<PurchaseOrder> {
        self.call("purchase.order", "button_confirm", serde_json::json!([[id.parse::<i64>()?]]), serde_json::json!({})).await?;
        self.get_purchase_order(id).await
    }

    // Invoices
    async fn list_invoices(&self, limit: u32) -> Result<Vec<Invoice>> {
        let recs = self.search_read("account.move", &[serde_json::json!(["move_type", "=", "out_invoice"])], &["id", "name", "partner_id", "state", "amount_total", "amount_residual", "invoice_date", "invoice_date_due"], limit).await?;
        Ok(recs.iter().map(|r| Invoice { id: odoo_id(r), customer_id: r["partner_id"].as_array().and_then(|a| a.first()).and_then(|v| v.as_i64()).map(|i| i.to_string()), customer_name: r["partner_id"].as_array().and_then(|a| a.get(1)).and_then(|v| v.as_str()).map(Into::into), state: odoo_inv_state(r["state"].as_str().unwrap_or("")), total: r["amount_total"].as_f64().unwrap_or(0.0), balance_due: r["amount_residual"].as_f64(), currency: None, line_items: vec![], due_date: r["invoice_date_due"].as_str().map(Into::into), created_at: r["invoice_date"].as_str().map(Into::into), backend: "odoo".into() }).collect())
    }

    async fn get_invoice(&self, id: &str) -> Result<Invoice> {
        let r = self.read_one("account.move", id.parse()?, &["name", "partner_id", "state", "amount_total", "amount_residual", "invoice_date", "invoice_date_due"]).await?;
        Ok(Invoice { id: id.into(), customer_id: r["partner_id"].as_array().and_then(|a| a.first()).and_then(|v| v.as_i64()).map(|i| i.to_string()), customer_name: r["partner_id"].as_array().and_then(|a| a.get(1)).and_then(|v| v.as_str()).map(Into::into), state: odoo_inv_state(r["state"].as_str().unwrap_or("")), total: r["amount_total"].as_f64().unwrap_or(0.0), balance_due: r["amount_residual"].as_f64(), currency: None, line_items: vec![], due_date: r["invoice_date_due"].as_str().map(Into::into), created_at: r["invoice_date"].as_str().map(Into::into), backend: "odoo".into() })
    }

    async fn create_invoice_draft(&self, customer_id: &str, items: &[LineItemInput]) -> Result<Invoice> {
        let lines: Vec<_> = items.iter().map(|li| {
            let mut v = serde_json::json!({"name": li.description, "quantity": li.quantity, "price_unit": li.unit_price});
            if let Some(ref pid) = li.product_id { v["product_id"] = pid.parse::<i64>().unwrap_or(0).into(); }
            serde_json::json!([0, 0, v])
        }).collect();
        let vals = serde_json::json!({"move_type": "out_invoice", "partner_id": customer_id.parse::<i64>()?, "invoice_line_ids": lines});
        let id = self.create_record("account.move", vals).await?;
        self.get_invoice(&id.to_string()).await
    }

    async fn submit_invoice(&self, id: &str) -> Result<Invoice> {
        self.call("account.move", "action_post", serde_json::json!([[id.parse::<i64>()?]]), serde_json::json!({})).await?;
        self.get_invoice(id).await
    }

    async fn post_invoice(&self, id: &str) -> Result<Invoice> {
        self.submit_invoice(id).await
    }

    // Inventory
    async fn get_stock_levels(&self, product_id: Option<&str>) -> Result<Vec<StockLevel>> {
        let domain = match product_id {
            Some(id) => vec![serde_json::json!(["product_id", "=", id.parse::<i64>()?])],
            None => vec![],
        };
        let recs = self.search_read("stock.quant", &domain, &["product_id", "location_id", "quantity", "available_quantity"], 50).await?;
        Ok(recs.iter().map(|r| StockLevel { product_id: r["product_id"].as_array().and_then(|a| a.first()).and_then(|v| v.as_i64()).map(|i| i.to_string()).unwrap_or_default(), product_name: r["product_id"].as_array().and_then(|a| a.get(1)).and_then(|v| v.as_str()).map(Into::into), warehouse: r["location_id"].as_array().and_then(|a| a.get(1)).and_then(|v| v.as_str()).map(Into::into), quantity_on_hand: r["quantity"].as_f64().unwrap_or(0.0), quantity_available: r["available_quantity"].as_f64(), backend: "odoo".into() }).collect())
    }

    async fn adjust_stock(&self, product_id: &str, quantity: f64, _reason: &str) -> Result<StockLevel> {
        let vals = serde_json::json!({"product_id": product_id.parse::<i64>()?, "new_quantity": quantity});
        self.call("stock.quant", "action_apply_inventory", serde_json::json!([[vals]]), serde_json::json!({})).await?;
        let levels = self.get_stock_levels(Some(product_id)).await?;
        levels.into_iter().next().ok_or_else(|| anyhow::anyhow!("product not found"))
    }

    async fn transfer_stock(&self, _product_id: &str, _from: &str, _to: &str, _qty: f64) -> Result<()> {
        anyhow::bail!("Odoo stock transfers require picking workflow — use the Odoo UI")
    }

    // General Ledger
    async fn list_accounts(&self) -> Result<Vec<Account>> {
        let recs = self.search_read("account.account", &[], &["id", "code", "name", "account_type"], 200).await?;
        Ok(recs.iter().map(|r| Account { id: odoo_id(r), code: odoo_str(r, "code"), name: r["name"].as_str().unwrap_or("").into(), account_type: odoo_str(r, "account_type"), balance: None, backend: "odoo".into() }).collect())
    }

    async fn get_journal_entries(&self, from: &str, to: &str) -> Result<Vec<JournalEntry>> {
        let recs = self.search_read("account.move.line", &[serde_json::json!(["date", ">=", from]), serde_json::json!(["date", "<=", to])], &["id", "date", "name", "debit", "credit", "account_id"], 100).await?;
        Ok(recs.iter().map(|r| JournalEntry { id: odoo_id(r), date: r["date"].as_str().unwrap_or("").into(), description: odoo_str(r, "name"), debit_account: r["account_id"].as_array().and_then(|a| a.get(1)).and_then(|v| v.as_str()).map(Into::into), credit_account: None, amount: r["debit"].as_f64().unwrap_or(0.0).max(r["credit"].as_f64().unwrap_or(0.0)), backend: "odoo".into() }).collect())
    }

    async fn get_trial_balance(&self, _as_of: &str) -> Result<serde_json::Value> {
        let accounts = self.list_accounts().await?;
        Ok(serde_json::to_value(accounts)?)
    }

    // Governance
    async fn get_audit_trail(&self, model: &str, id: &str) -> Result<Vec<AuditEntry>> {
        let recs = self.search_read("mail.message", &[serde_json::json!(["res_id", "=", id.parse::<i64>()?]), serde_json::json!(["model", "=", model])], &["date", "author_id", "body", "subtype_id"], 50).await?;
        Ok(recs.iter().map(|r| AuditEntry { timestamp: r["date"].as_str().unwrap_or("").into(), user: r["author_id"].as_array().and_then(|a| a.get(1)).and_then(|v| v.as_str()).map(Into::into), action: "message".into(), entity_type: model.into(), entity_id: id.into(), details: odoo_str(r, "body") }).collect())
    }
}

fn odoo_so_state(s: &str) -> LifecycleState {
    match s { "draft" | "sent" => LifecycleState::Draft, "sale" => LifecycleState::Released, "done" => LifecycleState::Fulfilled, "cancel" => LifecycleState::Cancelled, _ => LifecycleState::Draft }
}

fn odoo_po_state(s: &str) -> LifecycleState {
    match s { "draft" | "sent" => LifecycleState::Draft, "purchase" => LifecycleState::Released, "done" => LifecycleState::Fulfilled, "cancel" => LifecycleState::Cancelled, _ => LifecycleState::Draft }
}

fn odoo_inv_state(s: &str) -> LifecycleState {
    match s { "draft" => LifecycleState::Draft, "posted" => LifecycleState::Posted, "cancel" => LifecycleState::Cancelled, _ => LifecycleState::Draft }
}
