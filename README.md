# ERP MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-erp.svg)](https://crates.io/crates/mcp-erp)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)

Unified ERP MCP server with **34 tools** across **5 backends** — SAP S/4HANA, NetSuite, Odoo, Zoho Books, and Microsoft Dynamics 365 Business Central. Lifecycle-based document management with governed write operations.

## Key Principles

- **Unified schema** — agents see consistent types (Customer, Vendor, Product, SalesOrder, Invoice) regardless of backend
- **Lifecycle-based writes** — documents progress through states (draft → pending_approval → approved → released → posted → closed) instead of single-step creates
- **Feature-flagged backends** — compile only what you need
- **Financial governance** — invoice posting and order submission require approval gates
- **No credential exposure** — tokens stay in env vars, never reach LLM context

## Backends

| Backend | Auth | Env Vars |
|---------|------|----------|
| **Zoho Books** | OAuth2 | `ZOHO_TOKEN`, `ZOHO_ORG_ID` |
| **Odoo** | Session (JSON-RPC) | `ODOO_URL`, `ODOO_DB`, `ODOO_USER`, `ODOO_PASSWORD` |
| **Business Central** | Azure AD OAuth2 | `BC_TENANT_ID`, `BC_ENVIRONMENT`, `BC_COMPANY_ID`, `BC_TOKEN` |
| **NetSuite** | OAuth 1.0a | `NETSUITE_ACCOUNT_ID`, `NETSUITE_CONSUMER_KEY`, `NETSUITE_CONSUMER_SECRET`, `NETSUITE_TOKEN_ID`, `NETSUITE_TOKEN_SECRET` |
| **SAP S/4HANA** | OAuth2 / Basic | `SAP_BASE_URL`, `SAP_TOKEN` |

## Tools (34)

| Category | Tools | Risk |
|----------|-------|------|
| Customers | `list_customers`, `get_customer`, `create_customer`, `update_customer` | read / internal_write |
| Vendors | `list_vendors`, `get_vendor`, `create_vendor`, `update_vendor` | read / internal_write |
| Products | `list_products`, `get_product`, `create_product`, `update_product` | read / internal_write |
| Sales Orders | `list_sales_orders`, `get_sales_order`, `create_sales_order_draft`, `submit_sales_order` | read / external_write |
| Purchase Orders | `list_purchase_orders`, `get_purchase_order`, `create_purchase_order_draft`, `submit_purchase_order` | read / external_write |
| Invoices | `list_invoices`, `get_invoice`, `create_invoice_draft`, `submit_invoice`, `post_invoice` | read / financial_action |
| Inventory | `get_stock_levels`, `adjust_stock`, `transfer_stock` | read / internal_write |
| General Ledger | `list_accounts`, `get_journal_entries`, `get_trial_balance` | read_only |
| Governance | `request_erp_approval`, `attach_erp_evidence`, `get_erp_audit_trail` | internal_write / read |

## Document Lifecycle

All transactional documents (orders, invoices) carry a lifecycle state:

```
draft → pending_approval → approved → released → posted → sent → fulfilled → closed
                                                                          ↘ cancelled / voided
```

Write tools create documents in `draft` state. Separate `submit_*` and `post_*` tools advance the lifecycle with appropriate approval gates.

## Installation

```bash
cargo install mcp-erp --features all-backends
```

### Feature flags

```bash
# Default: Zoho + Odoo
cargo install mcp-erp

# All backends
cargo install mcp-erp --features all-backends

# Specific backend
cargo install mcp-erp --no-default-features --features sap
```

## Configuration

### Zoho Books

```bash
export ZOHO_TOKEN="1000.xxxx"
export ZOHO_ORG_ID="12345678"
```

### Odoo

```bash
export ODOO_URL="https://mycompany.odoo.com"
export ODOO_DB="mycompany"
export ODOO_USER="admin"
export ODOO_PASSWORD="secret"
```

### Business Central

```bash
export BC_TENANT_ID="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
export BC_ENVIRONMENT="production"
export BC_COMPANY_ID="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
export BC_TOKEN="eyJ0eXAi..."
```

### NetSuite

```bash
export NETSUITE_ACCOUNT_ID="1234567_SB1"
export NETSUITE_CONSUMER_KEY="xxxx"
export NETSUITE_CONSUMER_SECRET="xxxx"
export NETSUITE_TOKEN_ID="xxxx"
export NETSUITE_TOKEN_SECRET="xxxx"
```

### SAP S/4HANA

```bash
export SAP_BASE_URL="https://myhost.s4hana.cloud.sap"
export SAP_TOKEN="eyJ0eXAi..."
```

## Client Configuration

### Claude Desktop / Kiro / Cursor

```json
{
  "mcpServers": {
    "erp": {
      "command": "mcp-erp",
      "args": [],
      "env": {
        "ZOHO_TOKEN": "1000.xxxx",
        "ZOHO_ORG_ID": "12345678"
      }
    }
  }
}
```

## Usage Examples

```
"List our top 10 customers"
→ list_customers(limit: 10)

"Create a draft sales order for customer C-001 with 50 units of SKU-A at $25 each"
→ create_sales_order_draft(party_id: "C-001", line_items: [...])

"Submit that order for approval"
→ submit_sales_order(id: "SO-12345")

"What's our current stock of product P-100?"
→ get_stock_levels(product_id: "P-100")

"Show me the trial balance as of end of last month"
→ get_trial_balance(as_of: "2026-04-30")
```

## Registry Compliance

This server implements the [ADK MCP SDK](https://crates.io/crates/adk-mcp-sdk) contract:

- **HealthCheck** — verifies backend connectivity on startup
- **mcp-server.toml** — manifest with 34 tools, risk classes, and credential bindings
- **Manifest validation** — startup fails fast on invalid manifest
- **Structured tracing** — `RUST_LOG` env-filter for observability

## License

Apache-2.0

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.
