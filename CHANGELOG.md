# Changelog

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
