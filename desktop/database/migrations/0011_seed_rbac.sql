-- Static reference data: the permission catalog and the five roles from
-- the architecture brief, with a role->permission mapping. This is schema
-- reference data, not authentication logic — no user accounts are created
-- here. Phase 3 wires up login, password hashing, and enforcement against
-- this data, and creates the first admin account via a setup flow (never
-- a hardcoded seeded credential).

INSERT INTO permissions (key, description) VALUES
  ('products.view', 'View products'),
  ('products.create', 'Create products'),
  ('products.update', 'Edit products'),
  ('products.delete', 'Delete products'),
  ('sales.view', 'View sales'),
  ('sales.create', 'Create sales'),
  ('sales.cancel', 'Cancel a sale'),
  ('sales.refund', 'Refund a sale'),
  ('stock.view', 'View stock levels and movements'),
  ('stock.adjust', 'Create manual stock adjustments'),
  ('customers.view', 'View customers'),
  ('customers.create', 'Create customers'),
  ('purchases.view', 'View purchases'),
  ('purchases.create', 'Create purchases'),
  ('cash_register.view', 'View cash register sessions'),
  ('cash_register.manage', 'Open/close cash register sessions'),
  ('reports.view', 'View reports'),
  ('devices.view', 'View paired devices'),
  ('devices.manage', 'Rename, disable, or revoke paired devices'),
  ('settings.manage', 'Change business/application settings');

INSERT INTO roles (name, description) VALUES
  ('admin', 'Full access to every feature'),
  ('manager', 'Day-to-day operations and reporting, excluding system settings'),
  ('cashier', 'Point-of-sale checkout and customer lookup'),
  ('mobile_seller', 'Mobile app: browse catalog, create sales, look up customers'),
  ('inventory_user', 'Stock lookup and adjustments');

-- admin: every permission
INSERT INTO role_permissions (role_id, permission_id)
SELECT (SELECT id FROM roles WHERE name = 'admin'), id FROM permissions;

-- manager: everything except devices.manage and settings.manage
INSERT INTO role_permissions (role_id, permission_id)
SELECT (SELECT id FROM roles WHERE name = 'manager'), id
FROM permissions
WHERE key NOT IN ('devices.manage', 'settings.manage');

-- cashier: checkout-focused
INSERT INTO role_permissions (role_id, permission_id)
SELECT (SELECT id FROM roles WHERE name = 'cashier'), id
FROM permissions
WHERE key IN (
  'products.view', 'sales.view', 'sales.create', 'sales.cancel',
  'stock.view', 'customers.view', 'customers.create', 'cash_register.view'
);

-- mobile_seller: same as cashier, from the mobile app
INSERT INTO role_permissions (role_id, permission_id)
SELECT (SELECT id FROM roles WHERE name = 'mobile_seller'), id
FROM permissions
WHERE key IN (
  'products.view', 'sales.view', 'sales.create',
  'stock.view', 'customers.view', 'customers.create'
);

-- inventory_user: stock + read-only catalog/purchasing
INSERT INTO role_permissions (role_id, permission_id)
SELECT (SELECT id FROM roles WHERE name = 'inventory_user'), id
FROM permissions
WHERE key IN (
  'products.view', 'products.update', 'stock.view', 'stock.adjust',
  'purchases.view', 'purchases.create'
);
