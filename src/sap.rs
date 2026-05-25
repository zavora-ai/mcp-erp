//! SAP S/4HANA OData backend.
use crate::types::*;
use anyhow::Result;
use reqwest::Client;

#[derive(Clone)]
pub struct SapBackend {
    http: Client,
    base_url: String,
    token: String,
}

impl SapBackend {
    pub fn new(base_url: String, token: String) -> Self {
        let base_url = base_url.trim_end_matches('/').to_string();
        Self { http: Client::new(), base_url, token }
    }

    async fn get(&self, path: &str) -> Result<serde_json::Value> {
        Ok(self.http.get(format!("{}/{path}", self.base_url))
            .bearer_auth(&self.token)
            .header("Accept", "application/json")
            .send().await?.error_for_status()?.json().await?)
    }

    async fn post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        Ok(self.http.post(format!("{}/{path}", self.base_url))
            .bearer_auth(&self.token)
            .header("Accept", "application/json")
            .json(body).send().await?.error_for_status()?.json().await?)
    }

    async fn patch(&self, path: &str, body: &serde_json::Value) -> Result<()> {
        self.http.patch(format!("{}/{path}", self.base_url))
            .bearer_auth(&self.token)
            .header("Accept", "application/json")
            .json(body).send().await?.error_for_status()?;
        Ok(())
    }
}

fn sap_str(v: &serde_json::Value, k: &str) -> Option<String> { v[k].as_str().filter(|s| !s.is_empty()).map(Into::into) }

#[async_trait::async_trait]
impl ErpBackend for SapBackend {
    fn name(&self) -> &str { "sap" }

    async fn list_customers(&self, limit: u32) -> Result<Vec<Customer>> {
        let resp = self.get(&format!("sap/opu/odata/sap/API_BUSINESS_PARTNER/A_BusinessPartner?$filter=BusinessPartnerCategory eq '1'&$top={limit}&$format=json")).await?;
        Ok(resp["d"]["results"].as_array().map(|a| a.iter().map(|c| Customer { id: c["BusinessPartner"].as_str().unwrap_or("").into(), name: c["BusinessPartnerFullName"].as_str().unwrap_or("").into(), email: None, phone: None, currency: None, balance: None, backend: "sap".into() }).collect()).unwrap_or_default())
    }

    async fn get_customer(&self, id: &str) -> Result<Customer> {
        let c = self.get(&format!("sap/opu/odata/sap/API_BUSINESS_PARTNER/A_BusinessPartner('{id}')?$format=json")).await?;
        let d = &c["d"];
        Ok(Customer { id: d["BusinessPartner"].as_str().unwrap_or("").into(), name: d["BusinessPartnerFullName"].as_str().unwrap_or("").into(), email: None, phone: None, currency: None, balance: None, backend: "sap".into() })
    }

    async fn create_customer(&self, name: &str, _email: Option<&str>, _phone: Option<&str>) -> Result<Customer> {
        let body = serde_json::json!({"BusinessPartnerCategory": "1", "BusinessPartnerFullName": name});
        let resp = self.post("sap/opu/odata/sap/API_BUSINESS_PARTNER/A_BusinessPartner", &body).await?;
        Ok(Customer { id: resp["d"]["BusinessPartner"].as_str().unwrap_or("").into(), name: name.into(), email: None, phone: None, currency: None, balance: None, backend: "sap".into() })
    }

    async fn update_customer(&self, id: &str, name: Option<&str>, _email: Option<&str>, _phone: Option<&str>) -> Result<Customer> {
        let mut body = serde_json::json!({});
        if let Some(n) = name { body["BusinessPartnerFullName"] = n.into(); }
        self.patch(&format!("sap/opu/odata/sap/API_BUSINESS_PARTNER/A_BusinessPartner('{id}')"), &body).await?;
        self.get_customer(id).await
    }

    async fn list_vendors(&self, limit: u32) -> Result<Vec<Vendor>> {
        let resp = self.get(&format!("sap/opu/odata/sap/API_BUSINESS_PARTNER/A_BusinessPartner?$filter=BusinessPartnerCategory eq '2'&$top={limit}&$format=json")).await?;
        Ok(resp["d"]["results"].as_array().map(|a| a.iter().map(|v| Vendor { id: v["BusinessPartner"].as_str().unwrap_or("").into(), name: v["BusinessPartnerFullName"].as_str().unwrap_or("").into(), email: None, phone: None, currency: None, balance: None, backend: "sap".into() }).collect()).unwrap_or_default())
    }

    async fn get_vendor(&self, id: &str) -> Result<Vendor> {
        let v = self.get(&format!("sap/opu/odata/sap/API_BUSINESS_PARTNER/A_BusinessPartner('{id}')?$format=json")).await?;
        let d = &v["d"];
        Ok(Vendor { id: d["BusinessPartner"].as_str().unwrap_or("").into(), name: d["BusinessPartnerFullName"].as_str().unwrap_or("").into(), email: None, phone: None, currency: None, balance: None, backend: "sap".into() })
    }

    async fn create_vendor(&self, name: &str, _email: Option<&str>, _phone: Option<&str>) -> Result<Vendor> {
        let body = serde_json::json!({"BusinessPartnerCategory": "2", "BusinessPartnerFullName": name});
        let resp = self.post("sap/opu/odata/sap/API_BUSINESS_PARTNER/A_BusinessPartner", &body).await?;
        Ok(Vendor { id: resp["d"]["BusinessPartner"].as_str().unwrap_or("").into(), name: name.into(), email: None, phone: None, currency: None, balance: None, backend: "sap".into() })
    }

    async fn update_vendor(&self, id: &str, name: Option<&str>, _email: Option<&str>, _phone: Option<&str>) -> Result<Vendor> {
        let mut body = serde_json::json!({});
        if let Some(n) = name { body["BusinessPartnerFullName"] = n.into(); }
        self.patch(&format!("sap/opu/odata/sap/API_BUSINESS_PARTNER/A_BusinessPartner('{id}')"), &body).await?;
        self.get_vendor(id).await
    }

    async fn list_products(&self, limit: u32) -> Result<Vec<Product>> {
        let resp = self.get(&format!("sap/opu/odata/sap/API_PRODUCT_SRV/A_Product?$top={limit}&$format=json")).await?;
        Ok(resp["d"]["results"].as_array().map(|a| a.iter().map(|p| Product { id: p["Product"].as_str().unwrap_or("").into(), name: p["ProductDescription"].as_str().unwrap_or("").into(), sku: sap_str(p, "Product"), unit_price: None, currency: None, stock_on_hand: None, backend: "sap".into() }).collect()).unwrap_or_default())
    }

    async fn get_product(&self, id: &str) -> Result<Product> {
        let p = self.get(&format!("sap/opu/odata/sap/API_PRODUCT_SRV/A_Product('{id}')?$format=json")).await?;
        let d = &p["d"];
        Ok(Product { id: d["Product"].as_str().unwrap_or("").into(), name: d["ProductDescription"].as_str().unwrap_or("").into(), sku: sap_str(d, "Product"), unit_price: None, currency: None, stock_on_hand: None, backend: "sap".into() })
    }

    async fn create_product(&self, name: &str, sku: Option<&str>, _price: Option<f64>) -> Result<Product> {
        let product_id = sku.unwrap_or(name);
        let body = serde_json::json!({"Product": product_id, "ProductDescription": name, "ProductType": "FERT"});
        self.post("sap/opu/odata/sap/API_PRODUCT_SRV/A_Product", &body).await?;
        Ok(Product { id: product_id.into(), name: name.into(), sku: Some(product_id.into()), unit_price: None, currency: None, stock_on_hand: None, backend: "sap".into() })
    }

    async fn update_product(&self, id: &str, name: Option<&str>, _sku: Option<&str>, _price: Option<f64>) -> Result<Product> {
        let mut body = serde_json::json!({});
        if let Some(n) = name { body["ProductDescription"] = n.into(); }
        self.patch(&format!("sap/opu/odata/sap/API_PRODUCT_SRV/A_Product('{id}')"), &body).await?;
        self.get_product(id).await
    }

    async fn list_sales_orders(&self, limit: u32) -> Result<Vec<SalesOrder>> {
        let resp = self.get(&format!("sap/opu/odata/sap/API_SALES_ORDER_SRV/A_SalesOrder?$top={limit}&$format=json")).await?;
        Ok(resp["d"]["results"].as_array().map(|a| a.iter().map(|o| SalesOrder { id: o["SalesOrder"].as_str().unwrap_or("").into(), customer_id: o["SoldToParty"].as_str().unwrap_or("").into(), customer_name: sap_str(o, "SoldToPartyName"), state: sap_order_state(o["OverallSDProcessStatus"].as_str().unwrap_or("")), total: o["TotalNetAmount"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0), currency: sap_str(o, "TransactionCurrency"), line_items: vec![], created_at: sap_str(o, "SalesOrderDate"), backend: "sap".into() }).collect()).unwrap_or_default())
    }

    async fn get_sales_order(&self, id: &str) -> Result<SalesOrder> {
        let o = self.get(&format!("sap/opu/odata/sap/API_SALES_ORDER_SRV/A_SalesOrder('{id}')?$format=json")).await?;
        let d = &o["d"];
        Ok(SalesOrder { id: d["SalesOrder"].as_str().unwrap_or("").into(), customer_id: d["SoldToParty"].as_str().unwrap_or("").into(), customer_name: sap_str(d, "SoldToPartyName"), state: sap_order_state(d["OverallSDProcessStatus"].as_str().unwrap_or("")), total: d["TotalNetAmount"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0), currency: sap_str(d, "TransactionCurrency"), line_items: vec![], created_at: sap_str(d, "SalesOrderDate"), backend: "sap".into() })
    }

    async fn create_sales_order_draft(&self, customer_id: &str, items: &[LineItemInput]) -> Result<SalesOrder> {
        let lines: Vec<_> = items.iter().enumerate().map(|(i, li)| {
            let mut j = serde_json::json!({"SalesOrderItem": format!("{:04}", (i+1)*10), "RequestedQuantity": li.quantity.to_string()});
            if let Some(ref pid) = li.product_id { j["Material"] = pid.clone().into(); }
            j
        }).collect();
        let body = serde_json::json!({"SalesOrderType": "OR", "SoldToParty": customer_id, "to_Item": {"results": lines}});
        let resp = self.post("sap/opu/odata/sap/API_SALES_ORDER_SRV/A_SalesOrder", &body).await?;
        let id = resp["d"]["SalesOrder"].as_str().unwrap_or("");
        self.get_sales_order(id).await
    }

    async fn submit_sales_order(&self, id: &str) -> Result<SalesOrder> { self.get_sales_order(id).await }

    async fn list_purchase_orders(&self, limit: u32) -> Result<Vec<PurchaseOrder>> {
        let resp = self.get(&format!("sap/opu/odata/sap/API_PURCHASEORDER_PROCESS_SRV/A_PurchaseOrder?$top={limit}&$format=json")).await?;
        Ok(resp["d"]["results"].as_array().map(|a| a.iter().map(|o| PurchaseOrder { id: o["PurchaseOrder"].as_str().unwrap_or("").into(), vendor_id: o["Supplier"].as_str().unwrap_or("").into(), vendor_name: sap_str(o, "SupplierName"), state: LifecycleState::Released, total: o["TotalNetAmount"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0), currency: sap_str(o, "DocumentCurrency"), line_items: vec![], created_at: sap_str(o, "PurchaseOrderDate"), backend: "sap".into() }).collect()).unwrap_or_default())
    }

    async fn get_purchase_order(&self, id: &str) -> Result<PurchaseOrder> {
        let o = self.get(&format!("sap/opu/odata/sap/API_PURCHASEORDER_PROCESS_SRV/A_PurchaseOrder('{id}')?$format=json")).await?;
        let d = &o["d"];
        Ok(PurchaseOrder { id: d["PurchaseOrder"].as_str().unwrap_or("").into(), vendor_id: d["Supplier"].as_str().unwrap_or("").into(), vendor_name: sap_str(d, "SupplierName"), state: LifecycleState::Released, total: d["TotalNetAmount"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0), currency: sap_str(d, "DocumentCurrency"), line_items: vec![], created_at: sap_str(d, "PurchaseOrderDate"), backend: "sap".into() })
    }

    async fn create_purchase_order_draft(&self, vendor_id: &str, items: &[LineItemInput]) -> Result<PurchaseOrder> {
        let lines: Vec<_> = items.iter().enumerate().map(|(i, li)| {
            let mut j = serde_json::json!({"PurchaseOrderItem": format!("{:04}", (i+1)*10), "OrderQuantity": li.quantity.to_string(), "NetPriceAmount": li.unit_price.to_string()});
            if let Some(ref pid) = li.product_id { j["Material"] = pid.clone().into(); }
            j
        }).collect();
        let body = serde_json::json!({"PurchaseOrderType": "NB", "Supplier": vendor_id, "to_PurchaseOrderItem": {"results": lines}});
        let resp = self.post("sap/opu/odata/sap/API_PURCHASEORDER_PROCESS_SRV/A_PurchaseOrder", &body).await?;
        let id = resp["d"]["PurchaseOrder"].as_str().unwrap_or("");
        self.get_purchase_order(id).await
    }

    async fn submit_purchase_order(&self, id: &str) -> Result<PurchaseOrder> { self.get_purchase_order(id).await }

    async fn list_invoices(&self, limit: u32) -> Result<Vec<Invoice>> {
        let resp = self.get(&format!("sap/opu/odata/sap/API_BILLING_DOCUMENT_SRV/A_BillingDocument?$top={limit}&$format=json")).await?;
        Ok(resp["d"]["results"].as_array().map(|a| a.iter().map(|inv| Invoice { id: inv["BillingDocument"].as_str().unwrap_or("").into(), customer_id: sap_str(inv, "SoldToParty"), customer_name: None, state: LifecycleState::Posted, total: inv["TotalNetAmount"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0), balance_due: None, currency: sap_str(inv, "TransactionCurrency"), line_items: vec![], due_date: None, created_at: sap_str(inv, "BillingDocumentDate"), backend: "sap".into() }).collect()).unwrap_or_default())
    }

    async fn get_invoice(&self, id: &str) -> Result<Invoice> {
        let inv = self.get(&format!("sap/opu/odata/sap/API_BILLING_DOCUMENT_SRV/A_BillingDocument('{id}')?$format=json")).await?;
        let d = &inv["d"];
        Ok(Invoice { id: d["BillingDocument"].as_str().unwrap_or("").into(), customer_id: sap_str(d, "SoldToParty"), customer_name: None, state: LifecycleState::Posted, total: d["TotalNetAmount"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0), balance_due: None, currency: sap_str(d, "TransactionCurrency"), line_items: vec![], due_date: None, created_at: sap_str(d, "BillingDocumentDate"), backend: "sap".into() })
    }

    async fn create_invoice_draft(&self, customer_id: &str, _items: &[LineItemInput]) -> Result<Invoice> {
        anyhow::bail!("SAP invoices are created via billing document processing from sales orders — use create_sales_order_draft + submit flow for customer {customer_id}")
    }

    async fn submit_invoice(&self, id: &str) -> Result<Invoice> { self.get_invoice(id).await }
    async fn post_invoice(&self, id: &str) -> Result<Invoice> { self.get_invoice(id).await }

    async fn get_stock_levels(&self, product_id: Option<&str>) -> Result<Vec<StockLevel>> {
        let path = match product_id {
            Some(id) => format!("sap/opu/odata/sap/API_MATERIAL_STOCK_SRV/A_MatlStkInAcctMod?$filter=Material eq '{id}'&$format=json"),
            None => "sap/opu/odata/sap/API_MATERIAL_STOCK_SRV/A_MatlStkInAcctMod?$top=50&$format=json".into(),
        };
        let resp = self.get(&path).await?;
        Ok(resp["d"]["results"].as_array().map(|a| a.iter().map(|s| StockLevel { product_id: s["Material"].as_str().unwrap_or("").into(), product_name: None, warehouse: sap_str(s, "Plant"), quantity_on_hand: s["MatlWrhsStkQtyInMatlBaseUnit"].as_str().and_then(|v| v.parse().ok()).unwrap_or(0.0), quantity_available: None, backend: "sap".into() }).collect()).unwrap_or_default())
    }

    async fn adjust_stock(&self, product_id: &str, quantity: f64, reason: &str) -> Result<StockLevel> {
        let body = serde_json::json!({"Material": product_id, "Quantity": quantity.to_string(), "GoodsMovementType": if quantity > 0.0 { "501" } else { "502" }, "DocumentHeaderText": reason});
        self.post("sap/opu/odata/sap/API_MATERIAL_DOCUMENT_SRV/A_MaterialDocumentHeader", &body).await?;
        let levels = self.get_stock_levels(Some(product_id)).await?;
        levels.into_iter().next().ok_or_else(|| anyhow::anyhow!("not found"))
    }

    async fn transfer_stock(&self, product_id: &str, from: &str, to: &str, qty: f64) -> Result<()> {
        let body = serde_json::json!({"Material": product_id, "Plant": from, "StorageLocation": to, "Quantity": qty.to_string(), "GoodsMovementType": "311"});
        self.post("sap/opu/odata/sap/API_MATERIAL_DOCUMENT_SRV/A_MaterialDocumentHeader", &body).await?;
        Ok(())
    }

    async fn list_accounts(&self) -> Result<Vec<Account>> {
        let resp = self.get("sap/opu/odata/sap/API_GLACCOUNTINCHARTOFACCOUNTS_SRV/A_GLAccountInChartOfAccounts?$top=200&$format=json").await?;
        Ok(resp["d"]["results"].as_array().map(|a| a.iter().map(|acc| Account { id: acc["GLAccount"].as_str().unwrap_or("").into(), code: sap_str(acc, "GLAccount"), name: acc["GLAccountLongName"].as_str().unwrap_or("").into(), account_type: sap_str(acc, "GLAccountType"), balance: None, backend: "sap".into() }).collect()).unwrap_or_default())
    }

    async fn get_journal_entries(&self, from: &str, to: &str) -> Result<Vec<JournalEntry>> {
        let resp = self.get(&format!("sap/opu/odata/sap/API_JOURNALENTRYITEMBASIC_SRV/A_JournalEntryItemBasic?$filter=PostingDate ge datetime'{from}T00:00:00' and PostingDate le datetime'{to}T23:59:59'&$top=100&$format=json")).await?;
        Ok(resp["d"]["results"].as_array().map(|a| a.iter().map(|j| JournalEntry { id: j["AccountingDocument"].as_str().unwrap_or("").into(), date: j["PostingDate"].as_str().unwrap_or("").into(), description: sap_str(j, "DocumentItemText"), debit_account: sap_str(j, "GLAccount"), credit_account: None, amount: j["AmountInCompanyCodeCurrency"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0), backend: "sap".into() }).collect()).unwrap_or_default())
    }

    async fn get_trial_balance(&self, _as_of: &str) -> Result<serde_json::Value> {
        self.get("sap/opu/odata/sap/API_GLACCOUNTINCHARTOFACCOUNTS_SRV/A_GLAccountInChartOfAccounts?$top=200&$format=json").await
    }

    async fn get_audit_trail(&self, _entity_type: &str, _entity_id: &str) -> Result<Vec<AuditEntry>> {
        Ok(vec![]) // SAP change documents require separate API
    }
}

fn sap_order_state(status: &str) -> LifecycleState {
    match status { "A" | "" => LifecycleState::Draft, "B" => LifecycleState::PartiallyFulfilled, "C" => LifecycleState::Fulfilled, _ => LifecycleState::Released }
}
