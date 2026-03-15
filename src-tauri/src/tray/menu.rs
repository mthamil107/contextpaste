// ContextPaste — System Tray Menu

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Emitter, Manager,
};

pub fn setup_tray(app: &App) -> Result<(), String> {
    let show_item =
        MenuItem::with_id(app, "quick_paste", "Quick Paste (Ctrl+Shift+V)", true, None::<&str>)
            .map_err(|e| format!("Failed to create menu item: {}", e))?;

    let history_item =
        MenuItem::with_id(app, "history", "History (Ctrl+Shift+H)", true, None::<&str>)
            .map_err(|e| format!("Failed to create menu item: {}", e))?;

    let separator1 =
        MenuItem::with_id(app, "sep1", "───────────", false, None::<&str>)
            .map_err(|e| format!("Failed to create separator: {}", e))?;

    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)
        .map_err(|e| format!("Failed to create menu item: {}", e))?;

    let separator2 =
        MenuItem::with_id(app, "sep2", "───────────", false, None::<&str>)
            .map_err(|e| format!("Failed to create separator: {}", e))?;

    let quit_item = MenuItem::with_id(app, "quit", "Quit ContextPaste", true, None::<&str>)
        .map_err(|e| format!("Failed to create menu item: {}", e))?;

    let menu = Menu::with_items(
        app,
        &[
            &show_item,
            &history_item,
            &separator1,
            &settings_item,
            &separator2,
            &quit_item,
        ],
    )
    .map_err(|e| format!("Failed to create tray menu: {}", e))?;

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("ContextPaste — AI Clipboard Manager")
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            // Left-click on tray icon shows the window
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quick_paste" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "history" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = app.emit("nav:history", ());
                }
            }
            "settings" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = app.emit("nav:settings", ());
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)
        .map_err(|e| format!("Failed to build tray icon: {}", e))?;

    log::info!("System tray initialized");
    Ok(())
}
