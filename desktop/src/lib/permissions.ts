/**
 * The single source of truth for permission key strings on the frontend,
 * mirroring the catalog seeded by
 * `desktop/database/migrations/0011_seed_rbac.sql`. Features reference
 * `PERMISSIONS.ProductsCreate` etc. instead of typing raw strings, so a
 * typo shows up as a TypeScript error instead of a silently-always-false
 * permission check.
 */
export const PERMISSIONS = {
  ProductsView: 'products.view',
  ProductsCreate: 'products.create',
  ProductsUpdate: 'products.update',
  ProductsDelete: 'products.delete',
  SalesView: 'sales.view',
  SalesCreate: 'sales.create',
  SalesCancel: 'sales.cancel',
  SalesRefund: 'sales.refund',
  StockView: 'stock.view',
  StockAdjust: 'stock.adjust',
  CustomersView: 'customers.view',
  CustomersCreate: 'customers.create',
  PurchasesView: 'purchases.view',
  PurchasesCreate: 'purchases.create',
  CashRegisterView: 'cash_register.view',
  CashRegisterManage: 'cash_register.manage',
  ReportsView: 'reports.view',
  DevicesView: 'devices.view',
  DevicesManage: 'devices.manage',
  SettingsManage: 'settings.manage',
} as const

export type PermissionKey = (typeof PERMISSIONS)[keyof typeof PERMISSIONS]
