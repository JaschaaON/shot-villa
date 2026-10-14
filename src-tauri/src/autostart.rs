//! Start bei der Anmeldung: `SMAppService` unter macOS, Registry-Eintrag
//! (über das Autostart-Plugin) unter Windows.

use tauri::AppHandle;

#[cfg(target_os = "macos")]
pub fn set_enabled(_app: &AppHandle, enabled: bool) -> Result<(), String> {
    use objc2_service_management::SMAppService;
    let service = unsafe { SMAppService::mainAppService() };
    let result = if enabled {
        unsafe { service.registerAndReturnError() }
    } else {
        unsafe { service.unregisterAndReturnError() }
    };
    result.map_err(|error| error.localizedDescription().to_string())
}

#[cfg(target_os = "macos")]
pub fn is_enabled(_app: &AppHandle) -> bool {
    use objc2_service_management::{SMAppService, SMAppServiceStatus};
    let status = unsafe { SMAppService::mainAppService().status() };
    status == SMAppServiceStatus::Enabled
}

#[cfg(windows)]
pub fn set_enabled(app: &AppHandle, enabled: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let launcher = app.autolaunch();
    let result = if enabled {
        launcher.enable()
    } else {
        launcher.disable()
    };
    result.map_err(|error| error.to_string())
}

#[cfg(windows)]
pub fn is_enabled(app: &AppHandle) -> bool {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch().is_enabled().unwrap_or(false)
}
