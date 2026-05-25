//! Microsoft Dynamics 365 Business Central OData v4 backend.
use crate::types::*;
use anyhow::Result;
use reqwest::Client;

#[derive(Clone)]
pub struct BusinessCentralBackend {
    http: Client,
    base_url: String,
    token: String,
}

impl BusinessCentralBackend {
    pub fn new(tenant_id: String, environment: String, company_id: String, token: String) -> Self {
        let base_url = format!("https://api.businesscentral.dynamics.com/v2.0/{tenant_id}/{environment}/api/v2.0/companies({company_id})");
        Self { http: Client::new(), base_url, token }
    }

    async fn get(&self, path: &str) -> Result<serde_json::Value> {
        Ok(self.http.get(format!("{}/{path}", self.base_url))
            .bearer_auth(&self.token)
            .send().await?.error_for_status()?.json().await?)
    }

    async fn post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        Ok(self.http.post(format!("{}/{path}", self.base_url))
            .bearer_auth(&self.token)
            .json(body).send().await?.error_for_status()?.json().await?)
    }

    async fn patch(&self, path: &str, body: &serde_json::Value, etag: &str) -> Result<serde_json::Value> {
        Ok(self.http.patch(format!("{}/{path}", self.base_url))
            .bearer_auth(&self.token)
            .header("If-Match", etag)
            .json(body).send().await?.error_for_status()?.json().await?)
    }

    async fn post_action(&self, path: &str) -> Result<()> {
        self.http.post(format!("{}/{path}", self.base_url))
            .bearer_auth(&self.token)
            .send().await?.error_for_status()?;
        Ok(())
    }
}

fn bc_str(v: &serde_json::Value, k: &str) -> Option<String> { v[k].as_str().filter(|s| !s.is_empty()).map(Into::into) }

#[async_trait::async_trait]
impl ErpBackend for BusinessCentralBackend {
    fn name(&self) -> &str { "business_central" }

    async fn list_customers(&self, limit: u32) -> Result<Vec<Customer>> {
        let resp = self.get(&format!("customers?$top={limit}")).await?;
        Ok(resp["value"].as_array().map(|a| a.iter().map(|c| Customer { id: c["id"].as_str().unwrap_or("").into(), name: c["displayName"].as_str().unwrap_or("").into(), email: bc_str(c, "email"), phone: bc_str(c, "phoneNumber"), currency: bc_str(c, "currencyCode"), balance: c["balance"].as_f64(), backend: "business_central".into() }).collect()).unwrap_or_default())
    }

    async fn get_customer(&self, id: &str) -> Result<Customer> {
        let c = self.get(&format!("customers({id})")).await?;
        Ok(Customer { id: c["id"].as_str().unwrap_or("").into(), name: c["displayName"].as_str().unwrap_or("").into(), email: bc_str(&c, "email"), phone: bc_str(&c, "phoneNumber"), currency: bc_str(&c, "currencyCode"), balance: c["balance"].as_f64(), backend: "business_central".into() })
    }

    async fn create_customer(&self, name: &str, email: Option<&str>, phone: Option<&str>) -> Result<Customer> {
        let mut body = serde_json::json!({"displayName": name});
        if let Some(e) = email { body["email"] = e.into(); }
        if let Some(p) = phone { body["phoneNumber"] = p.into(); }
        let c = self.post("customers", &body).await?;
        Ok(Customer { id: c["id"].as_str().unwrap_or("").into(), name: name.into(), email: email.map(Into::into), phone: phone.map(Into::into), currency: None, balance: None, backend: "business_central".into() })
    }

    async fn update_customer(&self, id: &str, name: Option<&str>, email: Option<&str>, phone: Option<&str>) -> Result<Customer> {
        let current = self.get(&format!("customers({id})")).await?;
        let etag = current["@odata.etag"].as_str().unwrap_or("*");
        let mut body = serde_json::json!({});
        if let Some(n) = name { body["displayName"] = n.into(); }
        if let Some(e) = email { body["email"] = e.into(); }
        if let Some(p) = phone { body["phoneNumber"] = p.into(); }
        let c = self.patch(&format!("customers({id})"), &body, etag).await?;
        Ok(Customer { id: id.into(), name: c["displayName"].as_str().unwrap_or("").into(), email: bc_str(&c, "email"), phone: bc_str(&c, "phoneNumber"), currency: None, balance: None, backend: "business_central".into() })
    }

    async fn list_vendors(&self, limit: u32) -> Result<Vec<Vendor>> {
        let resp = self.get(&format!("vendors?$top={limit}")).await?;
        Ok(resp["value"].as_array().map(|a| a.iter().map(|v| Vendor { id: v["id"].as_str().unwrap_or("").into(), name: v["displayName"].as_str().unwrap_or("").into(), email: bc_str(v, "email"), phone: bc_str(v, "phoneNumber"), currency: bc_str(v, "currencyCode"), balance: v["balance"].as_f64(), backend: "business_central".into() }).collect()).unwrap_or_default())
    }

    async fn get_vendor(&self, id: &str) -> Result<Vendor> {
        let v = self.get(&format!("vendors({id})")).await?;
        Ok(Vendor { id: v["id"].as_str().unwrap_or("").into(), name: v["displayName"].as_str().unwrap_or("").into(), email: bc_str(&v, "email"), phone: bc_str(&v, "phoneNumber"), currency: bc_str(&v, "currencyCode"), balance: v["balance"].as_f64(), backend: "business_central".into() })
    }

    async fn create_vendor(&self, name: &str, email: Option<&str>, phone: Option<&str>) -> Result<Vendor> {
        let mut body = serde_json::json!({"displayName": name});
        if let Some(e) = email { body["email"] = e.into(); }
        if let Some(p) = phone { body["phoneNumber"] = p.into(); }
        let v = self.post("vendors", &body).await?;
        Ok(Vendor { id: v["id"].as_str().unwrap_or("").into(), name: name.into(), email: email.map(Into::into), phone: phone.map(Into::into), currency: None, balance: None, backend: "business_central".into() })
    }

    async fn update_vendor(&self, id: &str, name: Option<&str>, email: Option<&str>, phone: Option<&str>) -> Result<Vendor> {
        let current = self.get(&format!("vendors({id})")).await?;
        let etag = current["@odata.etag"].as_str().unwrap_or("*");
        let mut body = serde_json::json!({});
        if let Some(n) = name { body["displayName"] = n.into(); }
        if let Some(e) = email { body["email"] = e.into(); }
        if let Some(p) = phone { body["phoneNumber"] = p.into(); }
        let v = self.patch(&format!("vendors({id})"), &body, etag).await?;
        Ok(Vendor { id: id.into(), name: v["displayName"].as_str().unwrap_or("").into(), email: bc_str(&v, "email"), phone: bc_str(&v, "phoneNumber"), currency: None, balance: None, backend: "business_central".into() })
    }

    async fn list_products(&self, limit: u32) -> Result<Vec<Product>> {
        let resp = self.get(&format!("items?$top={limit}")).await?;
        Ok(resp["value"].as_array().map(|a| a.iter().map(|p| Product { id: p["id"].as_str().unwrap_or("").into(), name: p["displayName"].as_str().unwrap_or("").into(), sku: bc_str(p, "number"), unit_price: p["unitPrice"].as_f64(), currency: None, stock_on_hand: p["inventory"].as_f64(), backend: "business_central".into() }).collect()).unwrap_or_default())
    }

    async fn get_product(&self, id: &str) -> Result<Product> {
        let p = self.get(&format!("items({id})")).await?;
        Ok(Product { id: p["id"].as_str().unwrap_or("").into(), name: p["displayName"].as_str().unwrap_or("").into(), sku: bc_str(&p, "number"), unit_price: p["unitPrice"].as_f64(), currency: None, stock_on_hand: p["inventory"].as_f64(), backend: "business_central".into() })
    }

    async fn create_product(&self, name: &str, sku: Option<&str>, price: Option<f64>) -> Result<Product> {
        let mut body = serde_json::json!({"displayName": name, "type": "Inventory"});
        if let Some(s) = sku { body["number"] = s.into(); }
        if let Some(p) = price { body["unitPrice"] = p.into(); }
        let p = self.post("items", &body).await?;
        Ok(Product { id: p["id"].as_str().unwrap_or("").into(), name: name.into(), sku: sku.map(Into::into), unit_price: price, currency: None, stock_on_hand: None, backend: "business_central".into() })
    }

    async fn update_product(&self, id: &str, name: Option<&str>, sku: Option<&str>, price: Option<f64>) -> Result<Product> {
        let current = self.get(&format!("items({id})")).await?;
        let etag = current["@odata.etag"].as_str().unwrap_or("*");
        let mut body = serde_json::json!({});
        if let Some(n) = name { body["displayName"] = n.into(); }
        if let Some(s) = sku { body["number"] = s.into(); }
        if let Some(p) = price { body["unitPrice"] = p.into(); }
        let p = self.patch(&format!("items({id})"), &body, etag).await?;
        Ok(Product { id: id.into(), name: p["displayName"].as_str().unwrap_or("").into(), sku: bc_str(&p, "number"), unit_price: p["unitPrice"].as_f64(), currency: None, stock_on_hand: None, backend: "business_central".into() })
    }

    // Sales Orders
    async fn list_sales_orders(&self, limit: u32) -> Result<Vec<SalesOrder>> {
        let resp = self.get(&format!("salesOrders?$top={limit}")).await?;
        Ok(resp["value"].as_array().map(|a| a.iter().map(|o| SalesOrder { id: o["id"].as_str().unwrap_or("").into(), customer_id: o["customerId"].as_str().unwrap_or("").into(), customer_name: bc_str(o, "customerName"), state: bc_order_state(o["status"].as_str().unwrap_or("")), total: o["totalAmountIncludingTax"].as_f64().unwrap_or(0.0), currency: bc_str(o, "currencyCode"), line_items: vec![], created_at: bc_str(o, "orderDate"), backend: "business_central".into() }).collect()).unwrap_or_default())
    }

    async fn get_sales_order(&self, id: &str) -> Result<SalesOrder> {
        let o = self.get(&format!("salesOrders({id})?$expand=salesOrderLines")).await?;
        let items = o["salesOrderLines"].as_array().map(|a| a.iter().map(|li| LineItem { product_id: bc_str(li, "itemId"), description: li["description"].as_str().unwrap_or("").into(), quantity: li["quantity"].as_f64().unwrap_or(0.0), unit_price: li["unitPrice"].as_f64().unwrap_or(0.0), amount: li["amountIncludingTax"].as_f64().unwrap_or(0.0) }).collect()).unwrap_or_default();
        Ok(SalesOrder { id: o["id"].as_str().unwrap_or("").into(), customer_id: o["customerId"].as_str().unwrap_or("").into(), customer_name: bc_str(&o, "customerName"), state: bc_order_state(o["status"].as_str().unwrap_or("")), total: o["totalAmountIncludingTax"].as_f64().unwrap_or(0.0), currency: bc_str(&o, "currencyCode"), line_items: items, created_at: bc_str(&o, "orderDate"), backend: "business_central".into() })
    }

    async fn create_sales_order_draft(&self, customer_id: &str, items: &[LineItemInput]) -> Result<SalesOrder> {
        let body = serde_json::json!({"customerId": customer_id});
        let order = self.post("salesOrders", &body).await?;
        let order_id = order["id"].as_str().unwrap_or("");
        for li in items {
            let mut line = serde_json::json!({"description": li.description, "quantity": li.quantity, "unitPrice": li.unit_price, "lineType": "Item"});
            if let Some(ref pid) = li.product_id { line["itemId"] = pid.clone().into(); }
            self.post(&format!("salesOrders({order_id})/salesOrderLines"), &line).await?;
        }
        self.get_sales_order(order_id).await
    }

    async fn submit_sales_order(&self, id: &str) -> Result<SalesOrder> {
        self.post_action(&format!("salesOrders({id})/Microsoft.NAV.ship")).await?;
        self.get_sales_order(id).await
    }

    // Purchase Orders
    async fn list_purchase_orders(&self, limit: u32) -> Result<Vec<PurchaseOrder>> {
        let resp = self.get(&format!("purchaseOrders?$top={limit}")).await?;
        Ok(resp["value"].as_array().map(|a| a.iter().map(|o| PurchaseOrder { id: o["id"].as_str().unwrap_or("").into(), vendor_id: o["vendorId"].as_str().unwrap_or("").into(), vendor_name: bc_str(o, "vendorName"), state: bc_order_state(o["status"].as_str().unwrap_or("")), total: o["totalAmountIncludingTax"].as_f64().unwrap_or(0.0), currency: bc_str(o, "currencyCode"), line_items: vec![], created_at: bc_str(o, "orderDate"), backend: "business_central".into() }).collect()).unwrap_or_default())
    }

    async fn get_purchase_order(&self, id: &str) -> Result<PurchaseOrder> {
        let o = self.get(&format!("purchaseOrders({id})")).await?;
        Ok(PurchaseOrder { id: o["id"].as_str().unwrap_or("").into(), vendor_id: o["vendorId"].as_str().unwrap_or("").into(), vendor_name: bc_str(&o, "vendorName"), state: bc_order_state(o["status"].as_str().unwrap_or("")), total: o["totalAmountIncludingTax"].as_f64().unwrap_or(0.0), currency: bc_str(&o, "currencyCode"), line_items: vec![], created_at: bc_str(&o, "orderDate"), backend: "business_central".into() })
    }

    async fn create_purchase_order_draft(&self, vendor_id: &str, items: &[LineItemInput]) -> Result<PurchaseOrder> {
        let body = serde_json::json!({"vendorId": vendor_id});
        let order = self.post("purchaseOrders", &body).await?;
        let order_id = order["id"].as_str().unwrap_or("");
        for li in items {
            let mut line = serde_json::json!({"description": li.description, "quantity": li.quantity, "directUnitCost": li.unit_price, "lineType": "Item"});
            if let Some(ref pid) = li.product_id { line["itemId"] = pid.clone().into(); }
            self.post(&format!("purchaseOrders({order_id})/purchaseOrderLines"), &line).await?;
        }
        self.get_purchase_order(order_id).await
    }

    async fn submit_purchase_order(&self, id: &str) -> Result<PurchaseOrder> {
        self.post_action(&format!("purchaseOrders({id})/Microsoft.NAV.receive")).await?;
        self.get_purchase_order(id).await
    }

    // Invoices
    async fn list_invoices(&self, limit: u32) -> Result<Vec<Invoice>> {
        let resp = self.get(&format!("salesInvoices?$top={limit}")).await?;
        Ok(resp["value"].as_array().map(|a| a.iter().map(|inv| Invoice { id: inv["id"].as_str().unwrap_or("").into(), customer_id: bc_str(inv, "customerId"), customer_name: bc_str(inv, "customerName"), state: bc_inv_state(inv["status"].as_str().unwrap_or("")), total: inv["totalAmountIncludingTax"].as_f64().unwrap_or(0.0), balance_due: inv["remainingAmount"].as_f64(), currency: bc_str(inv, "currencyCode"), line_items: vec![], due_date: bc_str(inv, "dueDate"), created_at: bc_str(inv, "invoiceDate"), backend: "business_central".into() }).collect()).unwrap_or_default())
    }

    async fn get_invoice(&self, id: &str) -> Result<Invoice> {
        let inv = self.get(&format!("salesInvoices({id})")).await?;
        Ok(Invoice { id: inv["id"].as_str().unwrap_or("").into(), customer_id: bc_str(&inv, "customerId"), customer_name: bc_str(&inv, "customerName"), state: bc_inv_state(inv["status"].as_str().unwrap_or("")), total: inv["totalAmountIncludingTax"].as_f64().unwrap_or(0.0), balance_due: inv["remainingAmount"].as_f64(), currency: bc_str(&inv, "currencyCode"), line_items: vec![], due_date: bc_str(&inv, "dueDate"), created_at: bc_str(&inv, "invoiceDate"), backend: "business_central".into() })
    }

    async fn create_invoice_draft(&self, customer_id: &str, items: &[LineItemInput]) -> Result<Invoice> {
        let body = serde_json::json!({"customerId": customer_id});
        let inv = self.post("salesInvoices", &body).await?;
        let inv_id = inv["id"].as_str().unwrap_or("");
        for li in items {
            let mut line = serde_json::json!({"description": li.description, "quantity": li.quantity, "unitPrice": li.unit_price, "lineType": "Item"});
            if let Some(ref pid) = li.product_id { line["itemId"] = pid.clone().into(); }
            self.post(&format!("salesInvoices({inv_id})/salesInvoiceLines"), &line).await?;
        }
        self.get_invoice(inv_id).await
    }

    async fn submit_invoice(&self, id: &str) -> Result<Invoice> {
        self.post_action(&format!("salesInvoices({id})/Microsoft.NAV.post")).await?;
        self.get_invoice(id).await
    }

    async fn post_invoice(&self, id: &str) -> Result<Invoice> {
        self.submit_invoice(id).await
    }

    // Inventory
    async fn get_stock_levels(&self, product_id: Option<&str>) -> Result<Vec<StockLevel>> {
        let path = match product_id {
            Some(id) => format!("items({id})"),
            None => "items?$top=50&$select=id,displayName,inventory".into(),
        };
        let resp = self.get(&path).await?;
        if let Some(arr) = resp["value"].as_array() {
            Ok(arr.iter().map(|p| StockLevel { product_id: p["id"].as_str().unwrap_or("").into(), product_name: bc_str(p, "displayName"), warehouse: None, quantity_on_hand: p["inventory"].as_f64().unwrap_or(0.0), quantity_available: None, backend: "business_central".into() }).collect())
        } else {
            Ok(vec![StockLevel { product_id: resp["id"].as_str().unwrap_or("").into(), product_name: bc_str(&resp, "displayName"), warehouse: None, quantity_on_hand: resp["inventory"].as_f64().unwrap_or(0.0), quantity_available: None, backend: "business_central".into() }])
        }
    }

    async fn adjust_stock(&self, product_id: &str, quantity: f64, reason: &str) -> Result<StockLevel> {
        let body = serde_json::json!({"itemId": product_id, "adjustedQuantity": quantity, "reason": reason});
        self.post("itemAdjustments", &body).await?;
        let levels = self.get_stock_levels(Some(product_id)).await?;
        levels.into_iter().next().ok_or_else(|| anyhow::anyhow!("product not found"))
    }

    async fn transfer_stock(&self, product_id: &str, from: &str, to: &str, qty: f64) -> Result<()> {
        let body = serde_json::json!({"itemId": product_id, "fromLocationCode": from, "toLocationCode": to, "quantity": qty});
        self.post("transferOrders", &body).await?;
        Ok(())
    }

    // General Ledger
    async fn list_accounts(&self) -> Result<Vec<Account>> {
        let resp = self.get("accounts?$top=200").await?;
        Ok(resp["value"].as_array().map(|a| a.iter().map(|acc| Account { id: acc["id"].as_str().unwrap_or("").into(), code: bc_str(acc, "number"), name: acc["displayName"].as_str().unwrap_or("").into(), account_type: bc_str(acc, "category"), balance: acc["balance"].as_f64(), backend: "business_central".into() }).collect()).unwrap_or_default())
    }

    async fn get_journal_entries(&self, from: &str, to: &str) -> Result<Vec<JournalEntry>> {
        let resp = self.get(&format!("generalLedgerEntries?$filter=postingDate ge {from} and postingDate le {to}&$top=100")).await?;
        Ok(resp["value"].as_array().map(|a| a.iter().map(|j| JournalEntry { id: j["id"].as_str().unwrap_or("").into(), date: j["postingDate"].as_str().unwrap_or("").into(), description: bc_str(j, "description"), debit_account: bc_str(j, "accountNumber"), credit_account: None, amount: j["amount"].as_f64().unwrap_or(0.0), backend: "business_central".into() }).collect()).unwrap_or_default())
    }

    async fn get_trial_balance(&self, _as_of: &str) -> Result<serde_json::Value> {
        self.get("trialBalance").await
    }

    async fn get_audit_trail(&self, _entity_type: &str, _entity_id: &str) -> Result<Vec<AuditEntry>> {
        Ok(vec![]) // BC doesn't expose audit via standard API
    }
}

fn bc_order_state(s: &str) -> LifecycleState {
    match s { "Draft" => LifecycleState::Draft, "Open" => LifecycleState::Released, "Released" => LifecycleState::Released, "Pending Approval" => LifecycleState::PendingApproval, _ => LifecycleState::Draft }
}

fn bc_inv_state(s: &str) -> LifecycleState {
    match s { "Draft" => LifecycleState::Draft, "Open" => LifecycleState::Posted, "Paid" => LifecycleState::Closed, "Canceled" | "Corrective" => LifecycleState::Voided, _ => LifecycleState::Draft }
}
