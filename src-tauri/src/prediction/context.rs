// ContextPaste — Active Window Context Capture

#[derive(Debug, Clone)]
pub struct WindowContext {
    pub app_name: Option<String>,
    pub window_title: Option<String>,
}

/// Get the currently active window context.
/// Returns None values if detection fails (non-fatal).
pub fn get_active_window() -> WindowContext {
    #[cfg(target_os = "windows")]
    {
        get_active_window_windows()
    }

    #[cfg(not(target_os = "windows"))]
    {
        // On non-Windows platforms, return empty context for now.
        WindowContext {
            app_name: None,
            window_title: None,
        }
    }
}

/// Windows implementation: uses PowerShell to get the foreground window title and process name.
#[cfg(target_os = "windows")]
fn get_active_window_windows() -> WindowContext {
    // PowerShell snippet that gets the foreground window's process name and title.
    // Uses Add-Type to call GetForegroundWindow from user32.dll, then resolves
    // the owning process via GetWindowThreadProcessId.
    let ps_script = r#"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class FgWin {
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint pid);
    [DllImport("user32.dll", CharSet=CharSet.Auto)] public static extern int GetWindowText(IntPtr hWnd, System.Text.StringBuilder text, int count);
}
"@
$hw = [FgWin]::GetForegroundWindow()
$sb = New-Object System.Text.StringBuilder 256
[void][FgWin]::GetWindowText($hw, $sb, 256)
$title = $sb.ToString()
$pid = 0
[void][FgWin]::GetWindowThreadProcessId($hw, [ref]$pid)
$proc = (Get-Process -Id $pid -ErrorAction SilentlyContinue).ProcessName
Write-Output "$proc`n$title"
"#;

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", ps_script])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let lines: Vec<&str> = stdout.trim().splitn(2, '\n').collect();
            let app_name = lines.first().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
            let window_title = lines.get(1).map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
            WindowContext {
                app_name,
                window_title,
            }
        }
        Ok(_) => {
            log::debug!("PowerShell active-window detection returned non-zero exit code");
            WindowContext {
                app_name: None,
                window_title: None,
            }
        }
        Err(e) => {
            log::debug!("Failed to detect active window: {}", e);
            WindowContext {
                app_name: None,
                window_title: None,
            }
        }
    }
}
