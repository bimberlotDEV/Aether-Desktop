use serde::Serialize;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager, Runtime,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

pub const SHOW_SHORTCUT_LABEL: &str = "Ctrl+Shift+Space";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeStatus {
    pub tray_available: bool,
    pub shortcut: String,
    pub shortcut_registered: bool,
    pub notifications_available: bool,
    pub updater_configured: bool,
}

pub fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn setup(
    app: &mut App,
    updater_configured: bool,
) -> Result<NativeStatus, Box<dyn std::error::Error>> {
    let open = MenuItem::with_id(app, "open", "Open Aether", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Aether", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;

    TrayIconBuilder::with_id("aether-main")
        .icon(
            app.default_window_icon()
                .ok_or("Default application icon is missing")?
                .clone(),
        )
        .tooltip("Aether")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_main_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space);
    let shortcut_registered = match app.global_shortcut().register(shortcut) {
        Ok(()) => true,
        Err(error) => {
            log::warn!(
                "Global shortcut {} unavailable: {}",
                SHOW_SHORTCUT_LABEL,
                error
            );
            false
        }
    };

    Ok(NativeStatus {
        tray_available: true,
        shortcut: SHOW_SHORTCUT_LABEL.to_string(),
        shortcut_registered,
        notifications_available: true,
        updater_configured,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_serialization_is_frontend_safe() {
        let value = serde_json::to_value(NativeStatus {
            tray_available: true,
            shortcut: SHOW_SHORTCUT_LABEL.to_string(),
            shortcut_registered: false,
            notifications_available: true,
            updater_configured: false,
        })
        .unwrap();
        assert_eq!(value["shortcut"], SHOW_SHORTCUT_LABEL);
        assert_eq!(value["shortcutRegistered"], false);
        assert_eq!(value["updaterConfigured"], false);
    }
}
