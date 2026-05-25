# API Reference — mcp-erp

## Tool Categories

- [Customers](#customers)
- [Vendors](#vendors)
- [Products](#products)
- [Sales Orders](#sales-orders)
- [Purchase Orders](#purchase-orders)
- [Invoices](#invoices)
- [Inventory](#inventory)
- [General Ledger](#general-ledger)
- [Governance](#governance)

---

## Customers

### `list_customers`

List customers with optional limit.

| Parameter | Type | Required | Default | Description |
|-----------|------|:---:|---------|-------------|
| `limit` | u32 | No | 20 | Max results |

**Returns:** Array of `Customer` objects.

### `get_customer`

| Parameter | Type | Required | Description |
|-----------|------|:---:|-------------|
| `id` | String | Yes | Customer ID |

**Returns:** Single `Customer` object.

### `create_customer`

| Parameter | Type | Required | Description |
|-----------|------|:---:|-------------|
| `name` | String | Yes | Customer name |
| `email` | String | No | Email address |
| `phone` | String | No | Phone number |

**Returns:** Created `Customer` object. **Risk:** internal_write, requires approval.

### `update_customer`

| Parameter | Type | Required | Description |
|-----------|------|:---:|-------------|
| `id` | String | Yes | Customer ID |
| `name` | String | No | New name |
| `email` | String | No | New email |
| `phone` | String | No | New phone |

**Returns:** Updated `Customer` object. **Risk:** internal_write, requires approval.

---

## Vendors

### `list_vendors`

| Parameter | Type | Required | Default |
|-----------|------|:---:|---------|
| `limit` | u32 | No | 20 |

### `get_vendor`

| Parameter | Type | Required |
|-----------|------|:---:|
| `id` | String | Yes |

### `create_vendor`

| Parameter | Type | Required |
|-----------|------|:---:|
| `name` | String | Yes |
| `email` | String | No |
| `phone` | String | No |

**Risk:** internal_write, requires approval.

### `update_vendor`

| Parameter | Type | Required |
|-----------|------|:---:|
| `id` | String | Yes |
| `name` | String | No |
| `email` | String | No |
| `phone` | String | No |

---

## Products

### `list_products`

| Parameter | Type | Required | Default |
|-----------|------|:---:|---------|
| `limit` | u32 | No | 20 |

### `get_product`

| Parameter | Type | Required |
|-----------|------|:---:|
| `id` | String | Yes |

### `create_product`

| Parameter | Type | Required | Description |
|-----------|------|:---:|-------------|
| `name` | String | Yes | Product name |
| `sku` | String | No | SKU / item code |
| `unit_price` | f64 | No | Unit price |

**Risk:** internal_write, requires approval.

### `update_product`

| Parameter | Type | Required |
|-----------|------|:---:|
| `id` | String | Yes |
| `name` | String | No |
| `sku` | String | No |
| `unit_price` | f64 | No |

---

## Sales Orders

### `list_sales_orders`

| Parameter | Type | Required | Default |
|-----------|------|:---:|---------|
| `limit` | u32 | No | 20 |

### `get_sales_order`

| Parameter | Type | Required |
|-----------|------|:---:|
| `id` | String | Yes |

**Returns:** Sales order with `line_items` array and `state` field.

### `create_sales_order_draft`

Creates a sales order in **draft** state.

| Parameter | Type | Required | Description |
|-----------|------|:---:|-------------|
| `party_id` | String | Yes | Customer ID |
| `line_items` | Array | Yes | Line items (see below) |

**LineItem:**
```json
{"product_id": "P-100", "description": "Widget", "quantity": 10, "unit_price": 25.0}
```

**Risk:** internal_write. No approval required (draft only).

### `submit_sales_order`

Advances a draft sales order to **released** or **pending_approval**.

| Parameter | Type | Required |
|-----------|------|:---:|
| `id` | String | Yes |

**Risk:** external_write, requires approval.

---

## Purchase Orders

### `list_purchase_orders`

| Parameter | Type | Required | Default |
|-----------|------|:---:|---------|
| `limit` | u32 | No | 20 |

### `get_purchase_order`

| Parameter | Type | Required |
|-----------|------|:---:|
| `id` | String | Yes |

### `create_purchase_order_draft`

| Parameter | Type | Required | Description |
|-----------|------|:---:|-------------|
| `party_id` | String | Yes | Vendor ID |
| `line_items` | Array | Yes | Line items |

**Risk:** internal_write. No approval required (draft only).

### `submit_purchase_order`

| Parameter | Type | Required |
|-----------|------|:---:|
| `id` | String | Yes |

**Risk:** external_write, requires approval.

---

## Invoices

### `list_invoices`

| Parameter | Type | Required | Default |
|-----------|------|:---:|---------|
| `limit` | u32 | No | 20 |

### `get_invoice`

| Parameter | Type | Required |
|-----------|------|:---:|
| `id` | String | Yes |

**Returns:** Invoice with `state`, `balance_due`, `line_items`, `due_date`.

### `create_invoice_draft`

| Parameter | Type | Required | Description |
|-----------|------|:---:|-------------|
| `customer_id` | String | Yes | Customer ID |
| `line_items` | Array | Yes | Line items |

**Risk:** internal_write. No approval required (draft only).

> **Note:** SAP does not support direct invoice creation — invoices are generated from the sales order billing flow.

### `submit_invoice`

Submit a draft invoice for approval.

| Parameter | Type | Required |
|-----------|------|:---:|
| `id` | String | Yes |

**Risk:** financial_action, requires approval.

### `post_invoice`

Post an approved invoice to the general ledger. **Irreversible in most ERPs.**

| Parameter | Type | Required |
|-----------|------|:---:|
| `id` | String | Yes |

**Risk:** financial_action, requires approval.

---

## Inventory

### `get_stock_levels`

| Parameter | Type | Required | Description |
|-----------|------|:---:|-------------|
| `product_id` | String | No | Filter to one product (omit for all) |

### `adjust_stock`

| Parameter | Type | Required | Description |
|-----------|------|:---:|-------------|
| `product_id` | String | Yes | Product ID |
| `quantity` | f64 | Yes | Positive = increase, negative = decrease |
| `reason` | String | Yes | Reason for adjustment |

**Risk:** internal_write, requires approval.

### `transfer_stock`

| Parameter | Type | Required | Description |
|-----------|------|:---:|-------------|
| `product_id` | String | Yes | Product ID |
| `from_warehouse` | String | Yes | Source warehouse/location ID |
| `to_warehouse` | String | Yes | Destination warehouse/location ID |
| `quantity` | f64 | Yes | Quantity to transfer |

**Risk:** internal_write, requires approval.

> **Note:** Odoo does not support stock transfers via API (requires picking workflow).

---

## General Ledger

All GL tools are **read-only**.

### `list_accounts`

No parameters. Returns the chart of accounts.

### `get_journal_entries`

| Parameter | Type | Required | Description |
|-----------|------|:---:|-------------|
| `from` | String | Yes | Start date (YYYY-MM-DD) |
| `to` | String | Yes | End date (YYYY-MM-DD) |

### `get_trial_balance`

| Parameter | Type | Required | Description |
|-----------|------|:---:|-------------|
| `as_of` | String | Yes | Date (YYYY-MM-DD) |

---

## Governance

### `request_erp_approval`

| Parameter | Type | Required | Description |
|-----------|------|:---:|-------------|
| `entity_type` | String | Yes | e.g. "sales_order", "invoice" |
| `entity_id` | String | Yes | Document ID |
| `note` | String | No | Approval request note |

### `attach_erp_evidence`

| Parameter | Type | Required | Description |
|-----------|------|:---:|-------------|
| `entity_type` | String | Yes | Entity type |
| `entity_id` | String | Yes | Document ID |
| `description` | String | Yes | Evidence description |
| `url` | String | No | URL to supporting document |

### `get_erp_audit_trail`

| Parameter | Type | Required | Description |
|-----------|------|:---:|-------------|
| `entity_type` | String | Yes | Entity type |
| `entity_id` | String | Yes | Document ID |

---

## Shared Types

### LifecycleState

```
draft | pending_approval | approved | released | posted | sent | fulfilled | partially_fulfilled | closed | cancelled | voided
```

### Customer / Vendor

```json
{"id": "C-001", "name": "Acme Corp", "email": "...", "phone": "...", "currency": "USD", "balance": 1500.00, "backend": "zoho"}
```

### Product

```json
{"id": "P-100", "name": "Widget", "sku": "WDG-100", "unit_price": 25.0, "stock_on_hand": 500.0, "backend": "zoho"}
```

### SalesOrder / PurchaseOrder

```json
{"id": "SO-123", "customer_id": "C-001", "state": "draft", "total": 1250.0, "line_items": [...], "backend": "zoho"}
```

### Invoice

```json
{"id": "INV-789", "customer_id": "C-001", "state": "posted", "total": 1250.0, "balance_due": 0.0, "due_date": "2026-06-15", "backend": "zoho"}
```
