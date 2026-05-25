# ERP MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-erp.svg)](https://crates.io/crates/mcp-erp)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)
[![Registry Ready](https://img.shields.io/badge/ADK_Registry-Ready-green.svg)](https://www.zavora.ai)

The most complete multi-backend ERP MCP server. **34 tools** across **5 backends** — SAP S/4HANA, NetSuite, Odoo, Zoho Books, and Microsoft Dynamics 365 Business Central. Lifecycle-based document management with governed write operations. Single Rust binary with feature-flagged backends and enterprise governance.

## Architecture

<p align="center">
  <img src="https://raw.githubusercontent.com/zavora-ai/mcp-erp/main/docs/assets/architecture.svg" alt="MCP ERP Architecture" width="800"/>
</p>

## Key Principles

- **Unified schema** — agents see consistent types (Customer, Vendor, Product, SalesOrder, Invoice) regardless of backend
- **Lifecycle-based writes** — documents progress through states (draft → pending_approval → approved → released → posted → closed) instead of single-step creates
- **Feature-flagged backends** — compile only what you need (`--features zoho,odoo,sap`)
- **Financial governance** — invoice posting and order submission require approval gates
- **No credential exposure** — tokens stay in env vars, never reach LLM context
- **Single binary** — no Node.js, no Python, no runtime dependencies

## Comparison with Other ERP Integrations

| Feature | Generic REST | Zapier/Make | **mcp-erp** |
|---------|:---:|:---:|:---:|
| Multi-backend (5 ERPs) | ❌ | Partial | ✅ |
| Unified schema | ❌ | ❌ | ✅ |
| Lifecycle state tracking | ❌ | ❌ | ✅ |
| Draft → Submit → Post flow | ❌ | ❌ | ✅ |
| Risk classification per tool | ❌ | ❌ | ✅ |
| Approval gates | ❌ | ❌ | ✅ |
| Audit trail | ❌ | Partial | ✅ |
| Single binary | ❌ | ❌ | ✅ |
| Registry governance | ❌ | ❌ | ✅ |
| Agent-native (MCP) | ❌ | ❌ | ✅ |

## Tools (34)

### Customers (4)

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `list_customers` | List customers with optional filters | Read-only |
| `get_customer` | Get a customer by ID | Read-only |
| `create_customer` | Create a new customer record | Internal write |
| `update_customer` | Update an existing customer record | Internal write |

### Vendors (4)

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `list_vendors` | List vendors/suppliers with optional filters | Read-only |
| `get_vendor` | Get a vendor by ID | Read-only |
| `create_vendor` | Create a new vendor/supplier record | Internal write |
| `update_vendor` | Update an existing vendor record | Internal write |

### Products (4)

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `list_products` | List products/items with optional filters | Read-only |
| `get_product` | Get a product by ID | Read-only |
| `create_product` | Create a new product/item record | Internal write |
| `update_product` | Update an existing product record | Internal write |

### Sales Orders (4)

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `list_sales_orders` | List sales orders with optional filters | Read-only |
| `get_sales_order` | Get a sales order by ID with line items | Read-only |
| `create_sales_order_draft` | Create a sales order in draft state | Internal write |
| `submit_sales_order` | Submit a draft sales order for approval/release | External write |

### Purchase Orders (4)

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `list_purchase_orders` | List purchase orders with optional filters | Read-only |
| `get_purchase_order` | Get a purchase order by ID with line items | Read-only |
| `create_purchase_order_draft` | Create a purchase order in draft state | Internal write |
| `submit_purchase_order` | Submit a draft purchase order for approval/release | External write |

### Invoices (5)

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `list_invoices` | List invoices with optional filters | Read-only |
| `get_invoice` | Get an invoice by ID with line items and payment status | Read-only |
| `create_invoice_draft` | Create an invoice in draft state | Internal write |
| `submit_invoice` | Submit a draft invoice for approval | Financial action |
| `post_invoice` | Post an approved invoice to the ledger | Financial action |

### Inventory (3)

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `get_stock_levels` | Get current stock levels for products | Read-only |
| `adjust_stock` | Adjust stock quantity (increase/decrease) | Internal write |
| `transfer_stock` | Transfer stock between warehouses/locations | Internal write |

### General Ledger (3) — Read-only

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `list_accounts` | List chart of accounts | Read-only |
| `get_journal_entries` | Get journal entries for a date range | Read-only |
| `get_trial_balance` | Get trial balance for a period | Read-only |

### Governance (3)

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `request_erp_approval` | Request approval for a pending ERP document | Internal write |
| `attach_erp_evidence` | Attach supporting evidence/documents to a record | Internal write |
| `get_erp_audit_trail` | Get the audit trail/history for a document | Read-only |

## Document Lifecycle

All transactional documents (orders, invoices) carry a lifecycle state:

```
draft → pending_approval → approved → released → posted → sent → fulfilled → closed
                                                                          ↘ cancelled / voided
```

- **`create_*_draft`** tools create documents in `draft` state (low risk, no side effects)
- **`submit_*`** tools advance to `pending_approval` or `released` (requires approval)
- **`post_invoice`** advances to `posted` (financial action, irreversible in most ERPs)

## Backends

| Backend | Protocol | Auth | Default Feature |
|---------|----------|------|:---:|
| **Zoho Books** | REST | OAuth2 | ✅ |
| **Odoo** | JSON-RPC | Session | ✅ |
| **Business Central** | OData v4 | Azure AD OAuth2 | ❌ |
| **NetSuite** | REST | OAuth 1.0a | ❌ |
| **SAP S/4HANA** | OData | OAuth2 / Basic | ❌ |

### Backend Capabilities

| Capability | Zoho | Odoo | BC | NetSuite | SAP |
|-----------|:---:|:---:|:---:|:---:|:---:|
| Customers/Vendors | ✅ | ✅ | ✅ | ✅ | ✅ |
| Products | ✅ | ✅ | ✅ | ✅ | ✅ |
| Sales Orders | ✅ | ✅ | ✅ | ✅ | ✅ |
| Purchase Orders | ✅ | ✅ | ✅ | ✅ | ✅ |
| Invoices (draft/post) | ✅ | ✅ | ✅ | ✅ | Via SO flow |
| Stock Levels | ✅ | ✅ | ✅ | ✅ | ✅ |
| Stock Adjustments | ✅ | ✅ | ✅ | ✅ | ✅ |
| Stock Transfers | ✅ | ❌¹ | ✅ | ✅ | ✅ |
| Chart of Accounts | ✅ | ✅ | ✅ | ✅ | ✅ |
| Journal Entries | ✅ | ✅ | ✅ | ✅ | ✅ |
| Audit Trail | ❌ | ✅² | ❌ | ❌ | ❌ |

¹ Odoo stock transfers require the picking workflow — use the Odoo UI  
² Odoo audit trail uses the `mail.message` model

## Installation

```bash
cargo install mcp-erp --features all-backends
```

Or build from source:

```bash
git clone https://github.com/zavora-ai/mcp-erp
cd mcp-erp
cargo build --release --features all-backends
```

### Feature flags

```bash
# Default: Zoho + Odoo (lightest)
cargo install mcp-erp

# All backends
cargo install mcp-erp --features all-backends

# Specific backends
cargo install mcp-erp --no-default-features --features sap
cargo install mcp-erp --no-default-features --features "zoho,business-central"
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

### Microsoft Dynamics 365 Business Central

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

### Claude Desktop

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

### Kiro

Add to `.kiro/settings/mcp.json`:

```json
{
  "mcpServers": {
    "erp": {
      "command": "mcp-erp",
      "args": [],
      "env": {
        "ODOO_URL": "https://mycompany.odoo.com",
        "ODOO_DB": "mycompany",
        "ODOO_USER": "admin",
        "ODOO_PASSWORD": "secret"
      }
    }
  }
}
```

### Cursor

Add to `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "erp": {
      "command": "mcp-erp",
      "args": [],
      "env": {
        "SAP_BASE_URL": "https://myhost.s4hana.cloud.sap",
        "SAP_TOKEN": "eyJ0eXAi..."
      }
    }
  }
}
```

### Windsurf

Add to `~/.codeium/windsurf/mcp_config.json`:

```json
{
  "mcpServers": {
    "erp": {
      "command": "mcp-erp",
      "args": [],
      "env": {
        "BC_TENANT_ID": "xxx",
        "BC_ENVIRONMENT": "production",
        "BC_COMPANY_ID": "xxx",
        "BC_TOKEN": "eyJ..."
      }
    }
  }
}
```

## Usage Examples

### List and inspect customers
```
"List our top 10 customers"
→ list_customers(limit: 10)

"Get details for customer C-001"
→ get_customer(id: "C-001")
```

### Create and submit a sales order
```
"Create a draft sales order for customer C-001 with 50 units of SKU-A at $25 each"
→ create_sales_order_draft(party_id: "C-001", line_items: [{description: "SKU-A", quantity: 50, unit_price: 25.0}])

"Submit that order for approval"
→ submit_sales_order(id: "SO-12345")
```

### Invoice lifecycle
```
"Create a draft invoice for customer C-001"
→ create_invoice_draft(customer_id: "C-001", line_items: [...])

"Submit the invoice for approval"
→ submit_invoice(id: "INV-789")

"Post the approved invoice to the ledger"
→ post_invoice(id: "INV-789")
```

### Inventory management
```
"What's our current stock of product P-100?"
→ get_stock_levels(product_id: "P-100")

"Add 200 units of P-100 — received from supplier"
→ adjust_stock(product_id: "P-100", quantity: 200, reason: "PO receipt")

"Transfer 50 units from warehouse A to warehouse B"
→ transfer_stock(product_id: "P-100", from_warehouse: "WH-A", to_warehouse: "WH-B", quantity: 50)
```

### Financial reporting
```
"Show me the trial balance as of end of last month"
→ get_trial_balance(as_of: "2026-04-30")

"Get journal entries for May 2026"
→ get_journal_entries(from: "2026-05-01", to: "2026-05-31")
```

### Governance
```
"Request approval for sales order SO-12345"
→ request_erp_approval(entity_type: "sales_order", entity_id: "SO-12345")

"Show the audit trail for invoice INV-789"
→ get_erp_audit_trail(entity_type: "invoice", entity_id: "INV-789")
```

## Documentation

| Document | Description |
|----------|-------------|
| [Architecture](docs/assets/architecture.svg) | System diagram |
| [mcp-server.toml](mcp-server.toml) | ADK-Rust Enterprise registry manifest |
| [LICENSE](LICENSE) | Apache-2.0 license |

## Registry Compliance

This server implements the [ADK MCP SDK](https://crates.io/crates/adk-mcp-sdk) contract:

- **HealthCheck** — verifies backend connectivity on startup
- **mcp-server.toml** — manifest with 34 tools, risk classes, and credential bindings
- **Manifest validation** — startup fails fast on invalid manifest (SDK 0.1.3+)
- **Structured tracing** — `RUST_LOG` env-filter for observability

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START -->
| [<img src="https://github.com/jkmaina.png" width="80px;" alt=""/><br /><sub><b>James Karanja Maina</b></sub>](https://github.com/jkmaina) |
|:---:|
<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

Apache-2.0 — see [LICENSE](LICENSE) for details.

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.

Built with ❤️ by [Zavora AI](https://zavora.ai)
