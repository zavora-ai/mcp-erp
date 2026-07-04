# Changelog

## [1.1.0] - 2026-07-04

### Added
- **Zavora ERA backend** (`--features zavora`) — JWT email/password auth with
  automatic re-login (15-minute access TTL), tenant scoping from the token,
  and REST mapping for customers, vendors, products, invoices, and the GL
- **Accounting extension tools (10)** on the `ErpBackend` trait as
  default-error methods (existing backends unaffected): `list_bills`,
  `get_bill`, `create_bill_draft`, `post_bill`, `list_payments`,
  `record_payment` (document applications, Kenyan KES-denominated withholding
  tax, non-cash funding accounts), `run_report`, `get_dashboard`,
  `list_bank_accounts`, `post_journal_entry` — 44 tools total

### Fixed
- `mcp-server.toml` lookup now falls back to the crate root relative to the
  executable, so the server can be spawned from any working directory (e.g.
  by an MCP server manager)

## [1.0.0] - 2026-05-25

### Added
- Initial release with 34 tools across 9 categories
- **5 backends:** Zoho Books, Odoo, Business Central, NetSuite, SAP S/4HANA
- **Unified ErpBackend trait** — all backends implement the same interface
- **Lifecycle-based writes** — draft → submit → post flow with approval gates
- **Document lifecycle states:** draft, pending_approval, approved, released, posted, sent, fulfilled, partially_fulfilled, closed, cancelled, voided
- **Feature flags** — compile only the backends you need (default: zoho + odoo)
- **Manifest validation** on startup (adk-mcp-sdk 0.1.3)
- **Health check** verifies backend connectivity
- Customers: list, get, create, update
- Vendors: list, get, create, update
- Products: list, get, create, update
- Sales Orders: list, get, create_draft, submit
- Purchase Orders: list, get, create_draft, submit
- Invoices: list, get, create_draft, submit, post
- Inventory: get_stock_levels, adjust_stock, transfer_stock
- General Ledger: list_accounts, get_journal_entries, get_trial_balance (read-only)
- Governance: request_erp_approval, attach_erp_evidence, get_erp_audit_trail
- Architecture SVG diagram
- Comprehensive README with client configs for Claude, Kiro, Cursor, Windsurf
- API reference documentation
- Backend setup guides
