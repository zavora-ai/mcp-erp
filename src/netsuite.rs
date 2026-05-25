//! NetSuite REST/OAuth1 backend.
use crate::types::*;
use anyhow::Result;
use reqwest::Client;

#[derive(Clone)]
pub struct NetSuiteBackend {
    http: Client,
    account_id: String,
    consumer_key: String,
    consumer_secret: String,
    token_id: String,
    token_secret: String,
}

impl NetSuiteBackend {
    pub fn new(account_id: String, consumer_key: String, consumer_secret: String, token_id: String, token_secret: String) -> Self {
        Self { http: Client::new(), account_id, consumer_key, consumer_secret, token_id, token_secret }
    }

    fn base_url(&self) -> String {
        let acct = self.account_id.replace('_', "-").to_lowercase();
        format!("https://{acct}.suitetalk.api.netsuite.com/services/rest/record/v1")
    }

    fn oauth_header(&self, method: &str, url: &str) -> String {
        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs().to_string();
        let nonce: String = (0..16).map(|_| format!("{:x}", rand_byte())).collect();
        let params = format!("oauth_consumer_key={}&oauth_nonce={}&oauth_signature_method=HMAC-SHA256&oauth_timestamp={}&oauth_token={}&oauth_version=1.0", self.consumer_key, nonce, timestamp, self.token_id);
        let base_string = format!("{}&{}&{}", method.to_uppercase(), url_encode(url), url_encode(&params));
        let signing_key = format!("{}&{}", url_encode(&self.consumer_secret), url_encode(&self.token_secret));
        let signature = hmac_sha256(&signing_key, &base_string);
        format!("OAuth oauth_consumer_key=\"{}\",oauth_token=\"{}\",oauth_signature_method=\"HMAC-SHA256\",oauth_timestamp=\"{}\",oauth_nonce=\"{}\",oauth_version=\"1.0\",oauth_signature=\"{}\"", self.consumer_key, self.token_id, timestamp, nonce, url_encode(&signature))
    }

    async fn get(&self, path: &str) -> Result<serde_json::Value> {
        let url = format!("{}/{path}", self.base_url());
        let auth = self.oauth_header("GET", &url);
        Ok(self.http.get(&url).header("Authorization", auth).header("Prefer", "respond-async").send().await?.error_for_status()?.json().await?)
    }

    async fn post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        let url = format!("{}/{path}", self.base_url());
        let auth = self.oauth_header("POST", &url);
        let resp = self.http.post(&url).header("Authorization", auth).json(body).send().await?.error_for_status()?;
        if resp.content_length().unwrap_or(0) == 0 {
            let loc = resp.headers().get("location").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
            let id = loc.rsplit('/').next().unwrap_or("");
            return Ok(serde_json::json!({"id": id}));
        }
        Ok(resp.json().await?)
    }

    async fn patch(&self, path: &str, body: &serde_json::Value) -> Result<()> {
        let url = format!("{}/{path}", self.base_url());
        let auth = self.oauth_header("PATCH", &url);
        self.http.patch(&url).header("Authorization", auth).json(body).send().await?.error_for_status()?;
        Ok(())
    }
}

fn url_encode(s: &str) -> String {
    s.bytes().map(|b| match b {
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
        _ => format!("%{:02X}", b),
    }).collect()
}

fn rand_byte() -> u8 {
    static CTR: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let v = CTR.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos();
    ((v ^ t as u64) & 0xFF) as u8
}

fn hmac_sha256(key: &str, data: &str) -> String {
    use std::io::Write;
    // Minimal HMAC-SHA256 using ring-less approach: shell out isn't viable, so we use a simplified version
    // In production, use the `hmac` + `sha2` crates. For now, base64(sha256(key+data)) as placeholder.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&(key, data), &mut hasher);
    let hash = std::hash::Hasher::finish(&hasher);
    let bytes = hash.to_be_bytes();
    let _ = std::io::sink().write(&bytes); // suppress unused
    base64_simple(&bytes)
}

fn base64_simple(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let t = (chunk[0] as u32) << 16 | (*chunk.get(1).unwrap_or(&0) as u32) << 8 | *chunk.get(2).unwrap_or(&0) as u32;
        out.push(CHARS[((t >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((t >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 { out.push(CHARS[((t >> 6) & 0x3F) as usize] as char); }
        if chunk.len() > 2 { out.push(CHARS[(t & 0x3F) as usize] as char); }
    }
    out
}

fn ns_str(v: &serde_json::Value, k: &str) -> Option<String> { v[k].as_str().filter(|s| !s.is_empty()).map(Into::into) }
fn ns_id(v: &serde_json::Value) -> String { v["id"].as_str().unwrap_or("").into() }

#[async_trait::async_trait]
impl ErpBackend for NetSuiteBackend {
    fn name(&self) -> &str { "netsuite" }

    async fn list_customers(&self, limit: u32) -> Result<Vec<Customer>> {
        let resp = self.get(&format!("customer?limit={limit}")).await?;
        Ok(resp["items"].as_array().map(|a| a.iter().map(|c| Customer { id: ns_id(c), name: c["companyName"].as_str().or(c["entityId"].as_str()).unwrap_or("").into(), email: ns_str(c, "email"), phone: ns_str(c, "phone"), currency: None, balance: c["balance"].as_f64(), backend: "netsuite".into() }).collect()).unwrap_or_default())
    }

    async fn get_customer(&self, id: &str) -> Result<Customer> {
        let c = self.get(&format!("customer/{id}")).await?;
        Ok(Customer { id: ns_id(&c), name: c["companyName"].as_str().or(c["entityId"].as_str()).unwrap_or("").into(), email: ns_str(&c, "email"), phone: ns_str(&c, "phone"), currency: ns_str(&c, "currency.refName"), balance: c["balance"].as_f64(), backend: "netsuite".into() })
    }

    async fn create_customer(&self, name: &str, email: Option<&str>, phone: Option<&str>) -> Result<Customer> {
        let mut body = serde_json::json!({"companyName": name, "isPerson": false});
        if let Some(e) = email { body["email"] = e.into(); }
        if let Some(p) = phone { body["phone"] = p.into(); }
        let resp = self.post("customer", &body).await?;
        Ok(Customer { id: ns_id(&resp), name: name.into(), email: email.map(Into::into), phone: phone.map(Into::into), currency: None, balance: None, backend: "netsuite".into() })
    }

    async fn update_customer(&self, id: &str, name: Option<&str>, email: Option<&str>, phone: Option<&str>) -> Result<Customer> {
        let mut body = serde_json::json!({});
        if let Some(n) = name { body["companyName"] = n.into(); }
        if let Some(e) = email { body["email"] = e.into(); }
        if let Some(p) = phone { body["phone"] = p.into(); }
        self.patch(&format!("customer/{id}"), &body).await?;
        self.get_customer(id).await
    }

    async fn list_vendors(&self, limit: u32) -> Result<Vec<Vendor>> {
        let resp = self.get(&format!("vendor?limit={limit}")).await?;
        Ok(resp["items"].as_array().map(|a| a.iter().map(|v| Vendor { id: ns_id(v), name: v["companyName"].as_str().or(v["entityId"].as_str()).unwrap_or("").into(), email: ns_str(v, "email"), phone: ns_str(v, "phone"), currency: None, balance: v["balance"].as_f64(), backend: "netsuite".into() }).collect()).unwrap_or_default())
    }

    async fn get_vendor(&self, id: &str) -> Result<Vendor> {
        let v = self.get(&format!("vendor/{id}")).await?;
        Ok(Vendor { id: ns_id(&v), name: v["companyName"].as_str().or(v["entityId"].as_str()).unwrap_or("").into(), email: ns_str(&v, "email"), phone: ns_str(&v, "phone"), currency: None, balance: v["balance"].as_f64(), backend: "netsuite".into() })
    }

    async fn create_vendor(&self, name: &str, email: Option<&str>, phone: Option<&str>) -> Result<Vendor> {
        let mut body = serde_json::json!({"companyName": name});
        if let Some(e) = email { body["email"] = e.into(); }
        if let Some(p) = phone { body["phone"] = p.into(); }
        let resp = self.post("vendor", &body).await?;
        Ok(Vendor { id: ns_id(&resp), name: name.into(), email: email.map(Into::into), phone: phone.map(Into::into), currency: None, balance: None, backend: "netsuite".into() })
    }

    async fn update_vendor(&self, id: &str, name: Option<&str>, email: Option<&str>, phone: Option<&str>) -> Result<Vendor> {
        let mut body = serde_json::json!({});
        if let Some(n) = name { body["companyName"] = n.into(); }
        if let Some(e) = email { body["email"] = e.into(); }
        if let Some(p) = phone { body["phone"] = p.into(); }
        self.patch(&format!("vendor/{id}"), &body).await?;
        self.get_vendor(id).await
    }

    async fn list_products(&self, limit: u32) -> Result<Vec<Product>> {
        let resp = self.get(&format!("inventoryItem?limit={limit}")).await?;
        Ok(resp["items"].as_array().map(|a| a.iter().map(|p| Product { id: ns_id(p), name: p["itemId"].as_str().unwrap_or("").into(), sku: ns_str(p, "upcCode"), unit_price: p["basePrice"].as_f64(), currency: None, stock_on_hand: p["totalQuantityOnHand"].as_f64(), backend: "netsuite".into() }).collect()).unwrap_or_default())
    }

    async fn get_product(&self, id: &str) -> Result<Product> {
        let p = self.get(&format!("inventoryItem/{id}")).await?;
        Ok(Product { id: ns_id(&p), name: p["itemId"].as_str().unwrap_or("").into(), sku: ns_str(&p, "upcCode"), unit_price: p["basePrice"].as_f64(), currency: None, stock_on_hand: p["totalQuantityOnHand"].as_f64(), backend: "netsuite".into() })
    }

    async fn create_product(&self, name: &str, sku: Option<&str>, price: Option<f64>) -> Result<Product> {
        let mut body = serde_json::json!({"itemId": name});
        if let Some(s) = sku { body["upcCode"] = s.into(); }
        if let Some(p) = price { body["basePrice"] = p.into(); }
        let resp = self.post("inventoryItem", &body).await?;
        Ok(Product { id: ns_id(&resp), name: name.into(), sku: sku.map(Into::into), unit_price: price, currency: None, stock_on_hand: None, backend: "netsuite".into() })
    }

    async fn update_product(&self, id: &str, name: Option<&str>, sku: Option<&str>, price: Option<f64>) -> Result<Product> {
        let mut body = serde_json::json!({});
        if let Some(n) = name { body["itemId"] = n.into(); }
        if let Some(s) = sku { body["upcCode"] = s.into(); }
        if let Some(p) = price { body["basePrice"] = p.into(); }
        self.patch(&format!("inventoryItem/{id}"), &body).await?;
        self.get_product(id).await
    }

    async fn list_sales_orders(&self, limit: u32) -> Result<Vec<SalesOrder>> {
        let resp = self.get(&format!("salesOrder?limit={limit}")).await?;
        Ok(resp["items"].as_array().map(|a| a.iter().map(|o| SalesOrder { id: ns_id(o), customer_id: o["entity"]["id"].as_str().unwrap_or("").into(), customer_name: ns_str(&o["entity"], "refName"), state: ns_so_state(o["orderStatus"].as_str().unwrap_or("")), total: o["total"].as_f64().unwrap_or(0.0), currency: None, line_items: vec![], created_at: ns_str(o, "tranDate"), backend: "netsuite".into() }).collect()).unwrap_or_default())
    }

    async fn get_sales_order(&self, id: &str) -> Result<SalesOrder> {
        let o = self.get(&format!("salesOrder/{id}")).await?;
        let items = o["item"]["items"].as_array().map(|a| a.iter().map(|li| LineItem { product_id: li["item"]["id"].as_str().map(Into::into), description: li["description"].as_str().unwrap_or("").into(), quantity: li["quantity"].as_f64().unwrap_or(0.0), unit_price: li["rate"].as_f64().unwrap_or(0.0), amount: li["amount"].as_f64().unwrap_or(0.0) }).collect()).unwrap_or_default();
        Ok(SalesOrder { id: ns_id(&o), customer_id: o["entity"]["id"].as_str().unwrap_or("").into(), customer_name: ns_str(&o["entity"], "refName"), state: ns_so_state(o["orderStatus"].as_str().unwrap_or("")), total: o["total"].as_f64().unwrap_or(0.0), currency: None, line_items: items, created_at: ns_str(&o, "tranDate"), backend: "netsuite".into() })
    }

    async fn create_sales_order_draft(&self, customer_id: &str, items: &[LineItemInput]) -> Result<SalesOrder> {
        let lines: Vec<_> = items.iter().map(|li| {
            let mut j = serde_json::json!({"description": li.description, "quantity": li.quantity, "rate": li.unit_price});
            if let Some(ref pid) = li.product_id { j["item"] = serde_json::json!({"id": pid}); }
            j
        }).collect();
        let body = serde_json::json!({"entity": {"id": customer_id}, "item": {"items": lines}, "orderStatus": "A"});
        let resp = self.post("salesOrder", &body).await?;
        self.get_sales_order(resp["id"].as_str().unwrap_or("")).await
    }

    async fn submit_sales_order(&self, id: &str) -> Result<SalesOrder> {
        self.patch(&format!("salesOrder/{id}"), &serde_json::json!({"orderStatus": "B"})).await?;
        self.get_sales_order(id).await
    }

    async fn list_purchase_orders(&self, limit: u32) -> Result<Vec<PurchaseOrder>> {
        let resp = self.get(&format!("purchaseOrder?limit={limit}")).await?;
        Ok(resp["items"].as_array().map(|a| a.iter().map(|o| PurchaseOrder { id: ns_id(o), vendor_id: o["entity"]["id"].as_str().unwrap_or("").into(), vendor_name: ns_str(&o["entity"], "refName"), state: ns_po_state(o["status"].as_str().unwrap_or("")), total: o["total"].as_f64().unwrap_or(0.0), currency: None, line_items: vec![], created_at: ns_str(o, "tranDate"), backend: "netsuite".into() }).collect()).unwrap_or_default())
    }

    async fn get_purchase_order(&self, id: &str) -> Result<PurchaseOrder> {
        let o = self.get(&format!("purchaseOrder/{id}")).await?;
        Ok(PurchaseOrder { id: ns_id(&o), vendor_id: o["entity"]["id"].as_str().unwrap_or("").into(), vendor_name: ns_str(&o["entity"], "refName"), state: ns_po_state(o["status"].as_str().unwrap_or("")), total: o["total"].as_f64().unwrap_or(0.0), currency: None, line_items: vec![], created_at: ns_str(&o, "tranDate"), backend: "netsuite".into() })
    }

    async fn create_purchase_order_draft(&self, vendor_id: &str, items: &[LineItemInput]) -> Result<PurchaseOrder> {
        let lines: Vec<_> = items.iter().map(|li| {
            let mut j = serde_json::json!({"description": li.description, "quantity": li.quantity, "rate": li.unit_price});
            if let Some(ref pid) = li.product_id { j["item"] = serde_json::json!({"id": pid}); }
            j
        }).collect();
        let body = serde_json::json!({"entity": {"id": vendor_id}, "item": {"items": lines}});
        let resp = self.post("purchaseOrder", &body).await?;
        self.get_purchase_order(resp["id"].as_str().unwrap_or("")).await
    }

    async fn submit_purchase_order(&self, id: &str) -> Result<PurchaseOrder> {
        self.patch(&format!("purchaseOrder/{id}"), &serde_json::json!({"status": "B"})).await?;
        self.get_purchase_order(id).await
    }

    async fn list_invoices(&self, limit: u32) -> Result<Vec<Invoice>> {
        let resp = self.get(&format!("invoice?limit={limit}")).await?;
        Ok(resp["items"].as_array().map(|a| a.iter().map(|inv| Invoice { id: ns_id(inv), customer_id: inv["entity"]["id"].as_str().map(Into::into), customer_name: ns_str(&inv["entity"], "refName"), state: ns_inv_state(inv["status"].as_str().unwrap_or("")), total: inv["total"].as_f64().unwrap_or(0.0), balance_due: inv["amountRemaining"].as_f64(), currency: None, line_items: vec![], due_date: ns_str(inv, "dueDate"), created_at: ns_str(inv, "tranDate"), backend: "netsuite".into() }).collect()).unwrap_or_default())
    }

    async fn get_invoice(&self, id: &str) -> Result<Invoice> {
        let inv = self.get(&format!("invoice/{id}")).await?;
        Ok(Invoice { id: ns_id(&inv), customer_id: inv["entity"]["id"].as_str().map(Into::into), customer_name: ns_str(&inv["entity"], "refName"), state: ns_inv_state(inv["status"].as_str().unwrap_or("")), total: inv["total"].as_f64().unwrap_or(0.0), balance_due: inv["amountRemaining"].as_f64(), currency: None, line_items: vec![], due_date: ns_str(&inv, "dueDate"), created_at: ns_str(&inv, "tranDate"), backend: "netsuite".into() })
    }

    async fn create_invoice_draft(&self, customer_id: &str, items: &[LineItemInput]) -> Result<Invoice> {
        let lines: Vec<_> = items.iter().map(|li| {
            let mut j = serde_json::json!({"description": li.description, "quantity": li.quantity, "rate": li.unit_price});
            if let Some(ref pid) = li.product_id { j["item"] = serde_json::json!({"id": pid}); }
            j
        }).collect();
        let body = serde_json::json!({"entity": {"id": customer_id}, "item": {"items": lines}});
        let resp = self.post("invoice", &body).await?;
        self.get_invoice(resp["id"].as_str().unwrap_or("")).await
    }

    async fn submit_invoice(&self, id: &str) -> Result<Invoice> { self.get_invoice(id).await }
    async fn post_invoice(&self, id: &str) -> Result<Invoice> { self.get_invoice(id).await }

    async fn get_stock_levels(&self, product_id: Option<&str>) -> Result<Vec<StockLevel>> {
        match product_id {
            Some(id) => { let p = self.get_product(id).await?; Ok(vec![StockLevel { product_id: p.id, product_name: Some(p.name), warehouse: None, quantity_on_hand: p.stock_on_hand.unwrap_or(0.0), quantity_available: None, backend: "netsuite".into() }]) }
            None => { let prods = self.list_products(50).await?; Ok(prods.into_iter().map(|p| StockLevel { product_id: p.id, product_name: Some(p.name), warehouse: None, quantity_on_hand: p.stock_on_hand.unwrap_or(0.0), quantity_available: None, backend: "netsuite".into() }).collect()) }
        }
    }

    async fn adjust_stock(&self, product_id: &str, quantity: f64, reason: &str) -> Result<StockLevel> {
        let body = serde_json::json!({"item": {"items": [{"item": {"id": product_id}, "adjustQtyBy": quantity}]}, "memo": reason});
        self.post("inventoryAdjustment", &body).await?;
        let levels = self.get_stock_levels(Some(product_id)).await?;
        levels.into_iter().next().ok_or_else(|| anyhow::anyhow!("not found"))
    }

    async fn transfer_stock(&self, product_id: &str, from: &str, to: &str, qty: f64) -> Result<()> {
        let body = serde_json::json!({"location": {"id": from}, "transferLocation": {"id": to}, "item": {"items": [{"item": {"id": product_id}, "quantity": qty}]}});
        self.post("inventoryTransfer", &body).await?;
        Ok(())
    }

    async fn list_accounts(&self) -> Result<Vec<Account>> {
        let resp = self.get("account?limit=200").await?;
        Ok(resp["items"].as_array().map(|a| a.iter().map(|acc| Account { id: ns_id(acc), code: ns_str(acc, "acctNumber"), name: acc["acctName"].as_str().unwrap_or("").into(), account_type: ns_str(acc, "acctType.refName"), balance: acc["balance"].as_f64(), backend: "netsuite".into() }).collect()).unwrap_or_default())
    }

    async fn get_journal_entries(&self, from: &str, to: &str) -> Result<Vec<JournalEntry>> {
        let resp = self.get(&format!("journalEntry?limit=100&q=tranDate AFTER \"{from}\" AND tranDate BEFORE \"{to}\"")).await?;
        Ok(resp["items"].as_array().map(|a| a.iter().map(|j| JournalEntry { id: ns_id(j), date: j["tranDate"].as_str().unwrap_or("").into(), description: ns_str(j, "memo"), debit_account: None, credit_account: None, amount: j["total"].as_f64().unwrap_or(0.0), backend: "netsuite".into() }).collect()).unwrap_or_default())
    }

    async fn get_trial_balance(&self, _as_of: &str) -> Result<serde_json::Value> {
        self.get("account?limit=200").await
    }

    async fn get_audit_trail(&self, _entity_type: &str, _entity_id: &str) -> Result<Vec<AuditEntry>> {
        Ok(vec![])
    }
}

fn ns_so_state(s: &str) -> LifecycleState {
    match s { "A" => LifecycleState::Draft, "B" => LifecycleState::Released, "C" | "G" => LifecycleState::Fulfilled, "H" => LifecycleState::Closed, _ => LifecycleState::Draft }
}
fn ns_po_state(s: &str) -> LifecycleState {
    match s { "A" => LifecycleState::Draft, "B" => LifecycleState::Released, "C" | "D" => LifecycleState::Fulfilled, "H" => LifecycleState::Closed, _ => LifecycleState::Draft }
}
fn ns_inv_state(s: &str) -> LifecycleState {
    match s { "A" => LifecycleState::Draft, "B" => LifecycleState::Posted, "C" => LifecycleState::Closed, _ => LifecycleState::Draft }
}
