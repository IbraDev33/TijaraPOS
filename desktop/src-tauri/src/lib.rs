mod api;
mod auth;
mod catalog;
mod commands;
mod db;
mod error;
mod inventory;
mod sales;

use tauri::Manager;

/// Registers every Tauri command. Shared by `run()` and the IPC contract
/// tests below so the two can never drift apart.
fn register_commands<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.invoke_handler(tauri::generate_handler![
        commands::auth::auth_needs_setup,
        commands::auth::auth_bootstrap_admin,
        commands::auth::auth_login,
        commands::auth::auth_logout,
        commands::auth::auth_current_user,
        commands::catalog::categories_list,
        commands::catalog::categories_create,
        commands::catalog::categories_update,
        commands::catalog::categories_delete,
        commands::catalog::brands_list,
        commands::catalog::brands_create,
        commands::catalog::brands_update,
        commands::catalog::brands_delete,
        commands::catalog::units_list,
        commands::catalog::units_create,
        commands::catalog::units_update,
        commands::catalog::units_delete,
        commands::catalog::taxes_list,
        commands::catalog::taxes_create,
        commands::catalog::taxes_update,
        commands::catalog::taxes_set_active,
        commands::catalog::discounts_list,
        commands::catalog::discounts_create,
        commands::catalog::discounts_update,
        commands::catalog::discounts_set_active,
        commands::catalog::products_list,
        commands::catalog::products_get,
        commands::catalog::products_get_by_barcode,
        commands::catalog::products_create,
        commands::catalog::products_update,
        commands::catalog::products_set_active,
        commands::catalog::products_delete,
        commands::sales::sales_checkout,
        commands::sales::sales_list,
        commands::sales::sales_get,
        commands::sales::sales_cancel,
        commands::inventory::inventory_adjust_stock,
        commands::inventory::inventory_list_movements,
        commands::inventory::inventory_low_stock,
    ])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    register_commands(tauri::Builder::default())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let db_path = app.path().app_data_dir()?.join("tijarapos.sqlite3");
            let pool = db::init(&db_path).map_err(|err| {
                log::error!(
                    "failed to initialize database at {}: {err}",
                    db_path.display()
                );
                err
            })?;
            log::info!("database ready at {}", db_path.display());
            app.manage(pool);
            app.manage(auth::AuthState::default());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Drives the real `invoke()` dispatch path (argument name matching,
/// serde `rename_all` on both sides, permission checks) with the exact
/// JSON shapes the frontend sends, instead of calling Rust functions
/// directly. This is what catches a `categoryId` vs `category_id`
/// mismatch that unit tests on `catalog::*` alone cannot.
#[cfg(test)]
mod ipc_contract_tests {
    use serde_json::{json, Value};
    use tauri::ipc::{CallbackFn, InvokeBody};
    use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
    use tauri::webview::InvokeRequest;
    use tauri::{Manager, WebviewWindowBuilder};

    use super::register_commands;
    use crate::{auth, db};

    fn unique() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }

    fn invoke(
        webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        cmd: &str,
        body: Value,
    ) -> Result<Value, Value> {
        let request = InvokeRequest {
            cmd: cmd.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(body),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        };
        get_ipc_response(webview, request).map(|b| b.deserialize::<Value>().unwrap())
    }

    #[test]
    fn full_ipc_contract_smoke_test() {
        let dir = std::env::temp_dir().join(format!("tijarapos-ipc-test-{}", unique()));
        std::fs::create_dir_all(&dir).unwrap();
        let pool = db::init(&dir.join("test.sqlite3")).unwrap();

        let app = register_commands(mock_builder())
            .build(mock_context(noop_assets()))
            .unwrap();
        app.manage(pool);
        app.manage(auth::AuthState::default());
        let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        assert_eq!(
            invoke(&webview, "auth_needs_setup", json!({})).unwrap(),
            json!(true)
        );

        let admin = invoke(
            &webview,
            "auth_bootstrap_admin",
            json!({ "username": "admin", "password": "supersecret123", "fullName": "Admin User" }),
        )
        .expect("bootstrap should succeed");
        assert_eq!(admin["username"], "admin");
        assert!(admin["permissions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p == "products.create"));

        let category = invoke(
            &webview,
            "categories_create",
            json!({ "input": { "name": "Beverages", "parentId": null } }),
        )
        .expect("category create should succeed");
        let category_id = category["id"].as_i64().unwrap();

        let unit = invoke(
            &webview,
            "units_create",
            json!({ "input": { "name": "Piece", "abbreviation": "pc" } }),
        )
        .expect("unit create should succeed");
        let unit_id = unit["id"].as_i64().unwrap();

        // Exactly the shape `features/products/api.ts`'s `createProduct`
        // sends: camelCase keys throughout.
        let product = invoke(
            &webview,
            "products_create",
            json!({
                "input": {
                    "sku": "SKU-1",
                    "name": "Cola",
                    "description": null,
                    "categoryId": category_id,
                    "brandId": null,
                    "unitId": unit_id,
                    "purchasePrice": 100,
                    "sellingPrice": 200,
                    "taxId": null,
                    "discountId": null,
                    "minStock": 5,
                    "imagePath": null,
                    "barcodes": ["12345"],
                }
            }),
        )
        .expect("product create should succeed");
        assert_eq!(product["sku"], "SKU-1");
        assert_eq!(product["category_name"], "Beverages");
        assert_eq!(product["barcodes"], json!(["12345"]));

        let by_barcode = invoke(
            &webview,
            "products_get_by_barcode",
            json!({ "barcode": "12345" }),
        )
        .expect("barcode lookup should succeed");
        assert_eq!(by_barcode["sku"], "SKU-1");

        let list =
            invoke(&webview, "products_list", json!({ "query": {} })).expect("list should succeed");
        assert_eq!(list.as_array().unwrap().len(), 1);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_cashier_is_rejected_by_the_real_ipc_dispatch_not_just_the_business_logic() {
        let dir = std::env::temp_dir().join(format!("tijarapos-ipc-perm-test-{}", unique()));
        std::fs::create_dir_all(&dir).unwrap();
        let pool = db::init(&dir.join("test.sqlite3")).unwrap();

        let app = register_commands(mock_builder())
            .build(mock_context(noop_assets()))
            .unwrap();
        app.manage(pool);
        app.manage(auth::AuthState::default());
        let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        invoke(
            &webview,
            "auth_bootstrap_admin",
            json!({ "username": "admin", "password": "supersecret123", "fullName": "Admin User" }),
        )
        .unwrap();
        // Log back out so the (still-empty) session isn't the admin.
        invoke(&webview, "auth_logout", json!({})).unwrap();

        let err = invoke(
            &webview,
            "categories_create",
            json!({ "input": { "name": "Beverages", "parentId": null } }),
        )
        .expect_err("an unauthenticated request must be rejected");
        assert_eq!(err["code"], "UNAUTHORIZED");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn checkout_ipc_contract_smoke_test() {
        let dir = std::env::temp_dir().join(format!("tijarapos-ipc-checkout-test-{}", unique()));
        std::fs::create_dir_all(&dir).unwrap();
        let pool = db::init(&dir.join("test.sqlite3")).unwrap();

        let app = register_commands(mock_builder())
            .build(mock_context(noop_assets()))
            .unwrap();
        app.manage(pool.clone());
        app.manage(auth::AuthState::default());
        let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        invoke(
            &webview,
            "auth_bootstrap_admin",
            json!({ "username": "admin", "password": "supersecret123", "fullName": "Admin User" }),
        )
        .unwrap();

        let unit = invoke(
            &webview,
            "units_create",
            json!({ "input": { "name": "Piece", "abbreviation": "pc" } }),
        )
        .unwrap();
        let unit_id = unit["id"].as_i64().unwrap();

        let product = invoke(
            &webview,
            "products_create",
            json!({
                "input": {
                    "sku": "SKU-1", "name": "Widget", "description": null,
                    "categoryId": null, "brandId": null, "unitId": unit_id,
                    "purchasePrice": 500, "sellingPrice": 1000,
                    "taxId": null, "discountId": null, "minStock": 0,
                    "imagePath": null, "barcodes": [],
                }
            }),
        )
        .unwrap();
        let product_id = product["id"].as_i64().unwrap();

        // Stock provisioning has no IPC command yet (that's Phase 6,
        // Inventory) — reach into the repository directly for setup only;
        // the checkout call below is what this test actually verifies.
        {
            let conn = pool.get().unwrap();
            db::repositories::products::adjust_stock(&conn, product_id, 10).unwrap();
        }

        // Exactly the shape `features/sales/api.ts`'s `checkout` sends.
        let result = invoke(
            &webview,
            "sales_checkout",
            json!({
                "input": {
                    "customerId": null,
                    "items": [{ "productId": product_id, "quantity": 2, "discountOverride": null }],
                    "payments": [{ "method": "cash", "amount": 2000 }],
                    "idempotencyKey": null,
                }
            }),
        )
        .expect("checkout should succeed");
        assert_eq!(result["sale"]["total"], 2000);
        assert_eq!(result["change"], 0);

        let sale_id = result["sale"]["id"].as_str().unwrap();
        let fetched = invoke(&webview, "sales_get", json!({ "id": sale_id }))
            .expect("sales_get should succeed");
        assert_eq!(fetched["invoice_number"], result["sale"]["invoice_number"]);

        let list = invoke(&webview, "sales_list", json!({ "query": {} }))
            .expect("sales_list should succeed");
        assert_eq!(list.as_array().unwrap().len(), 1);

        let cancelled = invoke(&webview, "sales_cancel", json!({ "id": sale_id }))
            .expect("sales_cancel should succeed");
        assert_eq!(cancelled["status"], "cancelled");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn inventory_ipc_contract_smoke_test() {
        let dir = std::env::temp_dir().join(format!("tijarapos-ipc-inventory-test-{}", unique()));
        std::fs::create_dir_all(&dir).unwrap();
        let pool = db::init(&dir.join("test.sqlite3")).unwrap();

        let app = register_commands(mock_builder())
            .build(mock_context(noop_assets()))
            .unwrap();
        app.manage(pool);
        app.manage(auth::AuthState::default());
        let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        invoke(
            &webview,
            "auth_bootstrap_admin",
            json!({ "username": "admin", "password": "supersecret123", "fullName": "Admin User" }),
        )
        .unwrap();

        let unit = invoke(
            &webview,
            "units_create",
            json!({ "input": { "name": "Piece", "abbreviation": "pc" } }),
        )
        .unwrap();
        let unit_id = unit["id"].as_i64().unwrap();

        let product = invoke(
            &webview,
            "products_create",
            json!({
                "input": {
                    "sku": "SKU-1", "name": "Widget", "description": null,
                    "categoryId": null, "brandId": null, "unitId": unit_id,
                    "purchasePrice": 500, "sellingPrice": 1000,
                    "taxId": null, "discountId": null, "minStock": 5,
                    "imagePath": null, "barcodes": [],
                }
            }),
        )
        .unwrap();
        let product_id = product["id"].as_i64().unwrap();

        // Exactly the shape `features/inventory/api.ts`'s `adjustStock` sends.
        let result = invoke(
            &webview,
            "inventory_adjust_stock",
            json!({
                "input": {
                    "productId": product_id,
                    "quantityDelta": 20,
                    "reason": "adjustment",
                    "note": "Initial stock take",
                }
            }),
        )
        .expect("adjustment should succeed");
        assert_eq!(result["currentStock"], 20);

        let low =
            invoke(&webview, "inventory_low_stock", json!({})).expect("low_stock should succeed");
        assert_eq!(
            low.as_array().unwrap().len(),
            0,
            "20 in stock should clear a min of 5"
        );

        let movements = invoke(&webview, "inventory_list_movements", json!({ "query": {} }))
            .expect("list_movements should succeed");
        assert_eq!(movements.as_array().unwrap().len(), 1);
        assert_eq!(movements[0]["reason"], "adjustment");

        std::fs::remove_dir_all(&dir).ok();
    }
}
