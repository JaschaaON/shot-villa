mod actions;
mod autostart;
mod capture;
mod clipboard;
mod commands;
mod display;
mod files;
mod hotkeys;
mod imaging;
mod menu;
mod notify;
mod ocr;
mod output;
mod permission;
mod settings;
mod shots;
mod tray;
mod windows;

use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{App, AppHandle, Manager, RunEvent, Window, WindowEvent};

/// Dateien, die macOS schon vor dem Ende von `setup` zum Öffnen übergibt.
#[derive(Default)]
struct PendingFiles(Mutex<Vec<PathBuf>>);

pub fn run() {
    let builder = tauri::Builder::default();
    // Das App-Menü gibt es nur auf macOS; unter Windows hinge es an jedem Fenster.
    #[cfg(target_os = "macos")]
    let builder = builder.menu(menu::build);
    #[cfg(windows)]
    let builder = builder
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_notification::init());

    let app = builder
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .manage(shots::Shots::default())
        .manage(actions::Session::default())
        .manage(PendingFiles::default())
        .on_menu_event(|app, event| menu::handle(app, &event))
        .setup(setup)
        .on_window_event(on_window_event)
        .invoke_handler(tauri::generate_handler![
            commands::current_shot,
            commands::shot_info,
            commands::shot_png,
            commands::run_action,
            commands::get_settings,
            commands::set_settings,
            commands::suspend_hotkeys,
            commands::resume_hotkeys,
            commands::choose_save_folder,
            commands::overlay_info,
            commands::overlay_ready,
            commands::overlay_finish,
            commands::export_image,
            commands::export_shot,
            commands::recognize_text,
            commands::pin_image,
            commands::pin_shot,
            commands::edit_current,
            commands::open_images,
            commands::close_window,
            commands::copy_text,
            commands::screen_access,
            commands::open_screen_access_settings,
            commands::restart_app,
        ])
        .build(tauri::generate_context!());

    match app {
        Ok(app) => app.run(on_run_event),
        Err(error) => log::error!("App konnte nicht starten: {error}"),
    }
}

/// Ohne Fenster weiterlaufen: Die App lebt in der Menüleiste bzw. im Infobereich.
#[cfg(windows)]
fn on_run_event(_app: &AppHandle, event: RunEvent) {
    if let RunEvent::ExitRequested {
        api, code: None, ..
    } = event
    {
        api.prevent_exit();
    }
}

/// Ohne Fenster weiterlaufen; Bilder per „Öffnen mit“ oder aufs Dock-Icon öffnen.
#[cfg(target_os = "macos")]
fn on_run_event(app: &AppHandle, event: RunEvent) {
    match event {
        RunEvent::ExitRequested {
            api, code: None, ..
        } => api.prevent_exit(),
        RunEvent::Opened { urls } => {
            if let Some(path) = urls.iter().find_map(|url| url.to_file_path().ok()) {
                // Beim Start per „Öffnen mit“ kommt die Datei vor `setup`.
                if app.try_state::<settings::Store>().is_some() {
                    open_file(app, &path);
                } else if let Ok(mut pending) = app.state::<PendingFiles>().0.lock() {
                    pending.push(path);
                }
            }
        }
        _ => {}
    }
}

/// Lädt die Einstellungen, legt Menüleisten-Symbol und Kürzel an und begrüßt
/// beim ersten Start.
fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    let path = app.path().app_config_dir()?.join("settings.json");
    let store = settings::Store::load(path);
    let mut settings = store.get();
    let hotkeys = settings.hotkeys.clone();
    let first_start = !settings.welcome_shown;
    if first_start {
        settings.welcome_shown = true;
        if let Err(error) = store.set(settings) {
            log::warn!("Einstellungen nicht gespeichert: {error}");
        }
    }
    app.manage(store);
    notify::init();
    if first_start {
        windows::show_welcome(app.handle())?;
    }
    tray::create(app.handle(), &hotkeys)?;
    for problem in hotkeys::apply(app.handle(), &hotkeys) {
        log::warn!("Kürzel nicht verfügbar: {problem}");
    }
    // Unter Windows kommt eine per „Öffnen mit“ gewählte Datei als Argument.
    #[cfg(windows)]
    if let Some(path) = std::env::args_os().nth(1).map(PathBuf::from)
        && path.is_file()
    {
        open_file(app.handle(), &path);
    }
    let pending = app
        .state::<PendingFiles>()
        .0
        .lock()
        .map(|mut pending| std::mem::take(&mut *pending))
        .unwrap_or_default();
    for path in pending {
        open_file(app.handle(), &path);
    }
    Ok(())
}

fn open_file(app: &AppHandle, path: &std::path::Path) {
    if let Err(error) = actions::open_file(app, path) {
        log::error!("{error}");
    }
}

/// Editor und Einstellungen nur verstecken; beim Schließen aufräumen.
fn on_window_event(window: &Window, event: &WindowEvent) {
    match event {
        WindowEvent::CloseRequested { api, .. } if windows::is_persistent(window.label()) => {
            api.prevent_close();
            if let Err(error) = windows::hide(window.app_handle(), window) {
                log::error!("{error}");
            }
        }
        WindowEvent::Destroyed => {
            if let Err(error) = windows::update_dock(window.app_handle(), Some(window.label())) {
                log::warn!("{error}");
            }
            if let Some(id) = window
                .label()
                .strip_prefix(windows::PIN_PREFIX)
                .and_then(|id| id.parse().ok())
            {
                window.state::<shots::Shots>().remove(id);
            }
        }
        _ => {}
    }
}
