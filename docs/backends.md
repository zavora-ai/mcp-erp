# Backends — mcp-erp

## Overview

mcp-erp supports 5 ERP backends via feature flags. Only one backend is active at runtime (first configured wins). Each backend implements the `ErpBackend` trait, mapping its native API to the unified schema.

## Backend Selection Priority

When multiple backends are configured, the first match wins:

1. Zoho Books (`ZOHO_TOKEN` + `ZOHO_ORG_ID`)
2. Odoo (`ODOO_URL` + `ODOO_DB` + `ODOO_USER` + `ODOO_PASSWORD`)
3. Business Central (`BC_TENANT_ID` + `BC_ENVIRONMENT` + `BC_COMPANY_ID` + `BC_TOKEN`)
4. NetSuite (`NETSUITE_ACCOUNT_ID` + `NETSUITE_CONSUMER_KEY` + ...)
5. SAP S/4HANA (`SAP_BASE_URL` + `SAP_TOKEN`)

---

## Zoho Books

**Feature flag:** `zoho` (default)  
**Protocol:** REST API  
**Auth:** OAuth2 access token  
**Base URL:** `https://www.zohoapis.com/books/v3`

### Environment Variables

| Variable | Required | Description |
|----------|:---:|-------------|
| `ZOHO_TOKEN` | Yes | OAuth2 access token |
| `ZOHO_ORG_ID` | Yes | Zoho organization ID |

### Getting Credentials

1. Go to [Zoho API Console](https://api-console.zoho.com/)
2. Create a **Self Client**
3. Generate a token with scopes: `ZohoBooks.fullaccess.all`
4. Note your Organization ID from Zoho Books → Settings → Organization

### API Mapping

| Entity | Zoho Endpoint |
|--------|---------------|
| Customers | `contacts?contact_type=customer` |
| Vendors | `contacts?contact_type=vendor` |
| Products | `items` |
| Sales Orders | `salesorders` |
| Purchase Orders | `purchaseorders` |
| Invoices | `invoices` |
| Stock | `items` (stock_on_hand field) |
| Accounts | `chartofaccounts` |
| Journals | `journals` |

### Lifecycle Mapping

| Zoho Status | Lifecycle State |
|-------------|-----------------|
| draft | Draft |
| open / confirmed | Released |
| closed / fulfilled | Fulfilled |
| sent | Sent |
| paid | Closed |
| void | Voided |
| cancelled | Cancelled |

---

## Odoo

**Feature flag:** `odoo` (default)  
**Protocol:** JSON-RPC  
**Auth:** Session (username/password → uid)  
**Base URL:** `{ODOO_URL}/jsonrpc`

### Environment Variables

| Variable | Required | Description |
|----------|:---:|-------------|
| `ODOO_URL` | Yes | Odoo instance URL (e.g. `https://mycompany.odoo.com`) |
| `ODOO_DB` | Yes | Database name |
| `ODOO_USER` | Yes | Username (email) |
| `ODOO_PASSWORD` | Yes | Password or API key |

### Getting Credentials

1. Use your Odoo login credentials
2. For API keys: Settings → Users → API Keys → Generate
3. Database name is visible in the URL or Settings → Database

### API Mapping

| Entity | Odoo Model |
|--------|------------|
| Customers | `res.partner` (customer_rank > 0) |
| Vendors | `res.partner` (supplier_rank > 0) |
| Products | `product.product` |
| Sales Orders | `sale.order` |
| Purchase Orders | `purchase.order` |
| Invoices | `account.move` (move_type = out_invoice) |
| Stock | `stock.quant` |
| Accounts | `account.account` |
| Journals | `account.move.line` |
| Audit | `mail.message` |

### Limitations

- **Stock transfers** require the picking workflow and cannot be done via JSON-RPC alone
- **Trial balance** returns the chart of accounts (no native report API)

---

## Microsoft Dynamics 365 Business Central

**Feature flag:** `business-central`  
**Protocol:** OData v4  
**Auth:** Azure AD OAuth2 (Bearer token)  
**Base URL:** `https://api.businesscentral.dynamics.com/v2.0/{tenant}/{environment}/api/v2.0/companies({company})`

### Environment Variables

| Variable | Required | Description |
|----------|:---:|-------------|
| `BC_TENANT_ID` | Yes | Azure AD tenant ID |
| `BC_ENVIRONMENT` | Yes | Environment name (e.g. `production`, `sandbox`) |
| `BC_COMPANY_ID` | Yes | Company GUID |
| `BC_TOKEN` | Yes | Azure AD OAuth2 access token |

### Getting Credentials

1. Register an app in [Azure Portal](https://portal.azure.com/#blade/Microsoft_AAD_RegisteredApps)
2. Add API permission: `Dynamics 365 Business Central → Financials.ReadWrite.All`
3. Generate a client secret
4. Use OAuth2 client credentials flow to obtain a token
5. Find your Company ID: `GET /api/v2.0/companies`

### API Mapping

| Entity | BC Endpoint |
|--------|-------------|
| Customers | `customers` |
| Vendors | `vendors` |
| Products | `items` |
| Sales Orders | `salesOrders` (+ `salesOrderLines`) |
| Purchase Orders | `purchaseOrders` |
| Invoices | `salesInvoices` (+ `salesInvoiceLines`) |
| Stock | `items` (inventory field) |
| Accounts | `accounts` |
| Journals | `generalLedgerEntries` |

### Notes

- Uses **ETag-based optimistic concurrency** for PATCH operations
- Order lines are created separately after the header
- `Microsoft.NAV.ship` / `Microsoft.NAV.post` actions for lifecycle transitions

---

## NetSuite

**Feature flag:** `netsuite`  
**Protocol:** SuiteTalk REST API  
**Auth:** OAuth 1.0a (Token-Based Authentication)  
**Base URL:** `https://{account}.suitetalk.api.netsuite.com/services/rest/record/v1`

### Environment Variables

| Variable | Required | Description |
|----------|:---:|-------------|
| `NETSUITE_ACCOUNT_ID` | Yes | Account ID (e.g. `1234567_SB1`) |
| `NETSUITE_CONSUMER_KEY` | Yes | OAuth consumer key |
| `NETSUITE_CONSUMER_SECRET` | Yes | OAuth consumer secret |
| `NETSUITE_TOKEN_ID` | Yes | Token ID |
| `NETSUITE_TOKEN_SECRET` | Yes | Token secret |

### Getting Credentials

1. Setup → Integration → Manage Integrations → New
2. Enable **Token-Based Authentication**
3. Create an access token: Setup → Users/Roles → Access Tokens → New
4. Note: Account ID uses underscore format (e.g. `1234567_SB1` for sandbox)

### API Mapping

| Entity | NetSuite Record Type |
|--------|---------------------|
| Customers | `customer` |
| Vendors | `vendor` |
| Products | `inventoryItem` |
| Sales Orders | `salesOrder` |
| Purchase Orders | `purchaseOrder` |
| Invoices | `invoice` |
| Stock | `inventoryItem` (totalQuantityOnHand) |
| Accounts | `account` |
| Journals | `journalEntry` |

### Status Codes

| Code | Sales Order | Purchase Order | Invoice |
|------|-------------|----------------|---------|
| A | Draft / Pending | Draft | Open |
| B | Released | Released | Posted |
| C | Fulfilled | Received | Paid |
| H | Closed | Closed | — |

---

## SAP S/4HANA

**Feature flag:** `sap`  
**Protocol:** OData (v2/v4)  
**Auth:** OAuth2 client credentials or Basic auth  
**Base URL:** `{SAP_BASE_URL}/sap/opu/odata/sap/`

### Environment Variables

| Variable | Required | Description |
|----------|:---:|-------------|
| `SAP_BASE_URL` | Yes | S/4HANA host (e.g. `https://myhost.s4hana.cloud.sap`) |
| `SAP_TOKEN` | Yes | OAuth2 access token or Basic auth header |

### Getting Credentials

1. **Cloud:** SAP BTP → Service Keys → Create for Communication Arrangement
2. **On-premise:** Transaction `SOAUTH2` or Basic auth via ICF service
3. Ensure the following Communication Scenarios are active:
   - `SAP_COM_0008` (Business Partner)
   - `SAP_COM_0109` (Sales Order)
   - `SAP_COM_0053` (Purchase Order)
   - `SAP_COM_0112` (Billing Document)

### API Mapping

| Entity | SAP OData Service |
|--------|-------------------|
| Customers | `API_BUSINESS_PARTNER` (Category 1) |
| Vendors | `API_BUSINESS_PARTNER` (Category 2) |
| Products | `API_PRODUCT_SRV` |
| Sales Orders | `API_SALES_ORDER_SRV` |
| Purchase Orders | `API_PURCHASEORDER_PROCESS_SRV` |
| Invoices | `API_BILLING_DOCUMENT_SRV` |
| Stock | `API_MATERIAL_STOCK_SRV` |
| Stock Adjustments | `API_MATERIAL_DOCUMENT_SRV` |
| Accounts | `API_GLACCOUNTINCHARTOFACCOUNTS_SRV` |
| Journals | `API_JOURNALENTRYITEMBASIC_SRV` |

### Limitations

- **Invoice creation** is not supported directly — SAP creates billing documents from the sales order fulfillment flow
- **Amounts** are returned as strings in OData responses (parsed to f64)
- **Goods movements** use movement types: 501 (receipt), 502 (issue), 311 (transfer)
