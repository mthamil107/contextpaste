// ContextPaste — Screen Context Reader
//
// Reads text from the focused UI element to provide context for auto-paste.
// Uses PowerShell + .NET UIAutomation on Windows for reliable cross-app support.

use crate::prediction::context::get_active_window;
use crate::storage::models::ScreenContext;

/// Read the current screen context including active window info and focused element text.
#[cfg(target_os = "windows")]
pub fn read_screen_context() -> ScreenContext {
    let window_ctx = get_active_window();

    let mut ctx = ScreenContext {
        app_name: window_ctx.app_name.clone(),
        window_title: window_ctx.window_title.clone(),
        focused_text: None,
        surrounding_text: None,
    };

    match read_focused_element_text() {
        Ok((focused, surrounding)) => {
            ctx.focused_text = focused;
            ctx.surrounding_text = surrounding;
        }
        Err(e) => {
            log::debug!("UIA context reading failed: {}", e);
        }
    }

    ctx
}

/// Read focused element text via PowerShell + .NET UIAutomation.
/// Returns (element_name_or_value, None) — surrounding text is not reliably
/// available through the PowerShell approach.
#[cfg(target_os = "windows")]
fn read_focused_element_text() -> Result<(Option<String>, Option<String>), String> {
    let ps_script = r#"
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$element = [System.Windows.Automation.AutomationElement]::FocusedElement
$name = $element.Current.Name
$val = ''
try {
    $vp = $element.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
    $val = $vp.Current.Value
} catch {}
Write-Output "$name`n$val"
"#;

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", ps_script])
        .output()
        .map_err(|e| format!("PowerShell execution failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("PowerShell UIA failed: {}", stderr.chars().take(200).collect::<String>()));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = text.trim().lines().collect();

    let name = lines
        .first()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let value = lines
        .get(1)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    // Use value (actual input content) as focused_text if available, else fall back to name
    let focused = value.or(name);
    Ok((focused, None))
}

/// Non-Windows fallback: returns basic window context without focused element text.
#[cfg(not(target_os = "windows"))]
pub fn read_screen_context() -> ScreenContext {
    let window_ctx = get_active_window();
    ScreenContext {
        app_name: window_ctx.app_name.clone(),
        window_title: window_ctx.window_title.clone(),
        focused_text: None,
        surrounding_text: None,
    }
}
