mod api;
mod commands;
mod db;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
