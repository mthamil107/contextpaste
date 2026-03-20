// ContextPaste — Screen Context Reader
//
// Reads text near the cursor to understand what the user is being asked to paste.
// Two strategies:
//   1. OCR: Screenshot a region near the cursor → Windows built-in OCR → extract text
//   2. UIAutomation: Read focused element Name/Value via PowerShell
//   3. Window title fallback
//
// OCR works for terminals where UIAutomation fails.

use crate::prediction::context::get_active_window;
use crate::storage::models::ScreenContext;

/// Read the current screen context including active window info and text near cursor.
#[cfg(target_os = "windows")]
pub fn read_screen_context() -> ScreenContext {
    let window_ctx = get_active_window();

    let mut ctx = ScreenContext {
        app_name: window_ctx.app_name.clone(),
        window_title: window_ctx.window_title.clone(),
        focused_text: None,
        surrounding_text: None,
    };

    // Strategy 1: Read current line (select + copy + read + restore)
    // Most accurate — gets the EXACT prompt text from any app
    match read_current_line() {
        Ok(text) if !text.is_empty() => {
            log::info!("Line read captured: {}", text.chars().take(80).collect::<String>());
            ctx.focused_text = Some(text);
            return ctx;
        }
        Ok(_) => log::debug!("Line read returned empty"),
        Err(e) => log::debug!("Line read failed: {}", e),
    }

    // Strategy 2: Try OCR on screen region near cursor (fallback)
    match read_screen_ocr() {
        Ok(text) if !text.is_empty() => {
            log::info!("OCR captured: {}", text.chars().take(80).collect::<String>());
            ctx.focused_text = Some(text);
            return ctx;
        }
        Ok(_) => log::debug!("OCR returned empty text"),
        Err(e) => log::debug!("OCR failed: {}", e),
    }

    // Strategy 3: Try UIAutomation (works for standard input fields)
    match read_focused_element_text() {
        Ok((focused, _)) => {
            ctx.focused_text = focused;
        }
        Err(e) => {
            log::debug!("UIA context reading failed: {}", e);
        }
    }

    ctx
}

/// Read the current line by selecting it and copying.
/// Simulates: Home → Shift+End → Ctrl+C → read clipboard → restore clipboard.
/// This gets the EXACT text of the current prompt line in any app.
#[cfg(target_os = "windows")]
fn read_current_line() -> Result<String, String> {
    let ps_script = r#"
Add-Type -AssemblyName System.Windows.Forms

# Save current clipboard
$saved = [System.Windows.Forms.Clipboard]::GetText()

# Small delay to ensure we're in the right window
Start-Sleep -Milliseconds 50

# Select current line: Home → Shift+End
[System.Windows.Forms.SendKeys]::SendWait("{HOME}")
Start-Sleep -Milliseconds 30
[System.Windows.Forms.SendKeys]::SendWait("+{END}")
Start-Sleep -Milliseconds 30

# Copy selection
[System.Windows.Forms.SendKeys]::SendWait("^c")
Start-Sleep -Milliseconds 100

# Read what was copied
$line = [System.Windows.Forms.Clipboard]::GetText()

# Restore original clipboard
if ($saved) {
    [System.Windows.Forms.Clipboard]::SetText($saved)
} else {
    [System.Windows.Forms.Clipboard]::Clear()
}

# Deselect: press End
[System.Windows.Forms.SendKeys]::SendWait("{END}")

Write-Output $line
"#;

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-STA", "-Command", ps_script])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|e| format!("Line read failed: {}", e))?;

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(text)
}

/// Use Windows built-in OCR to read text near the cursor position.
/// Captures a screenshot of a 600x100px region around the cursor, runs OCR.
#[cfg(target_os = "windows")]
fn read_screen_ocr() -> Result<String, String> {
    let ps_script = r#"
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms

# Get cursor position
$cursor = [System.Windows.Forms.Cursor]::Position

# Capture region: 600px wide, 100px tall, centered on cursor (biased upward to read prompt above)
$x = [Math]::Max(0, $cursor.X - 300)
$y = [Math]::Max(0, $cursor.Y - 80)
$w = 600
$h = 100

$bmp = New-Object System.Drawing.Bitmap($w, $h)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($x, $y, 0, 0, (New-Object System.Drawing.Size($w, $h)))
$g.Dispose()

# Save to temp file for OCR
$tmp = [System.IO.Path]::GetTempFileName() + ".png"
$bmp.Save($tmp, [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()

# Use Windows.Media.Ocr for text recognition
try {
    Add-Type -AssemblyName System.Runtime.WindowsRuntime
    $null = [Windows.Media.Ocr.OcrEngine, Windows.Foundation, ContentType=WindowsRuntime]
    $null = [Windows.Graphics.Imaging.BitmapDecoder, Windows.Foundation, ContentType=WindowsRuntime]
    $null = [Windows.Storage.StorageFile, Windows.Foundation, ContentType=WindowsRuntime]

    # Helper to await WinRT async operations
    $asTaskGeneric = ([System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object {
        $_.Name -eq 'AsTask' -and
        $_.GetParameters().Count -eq 1 -and
        $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1'
    })[0]

    Function Await($WinRtTask, $ResultType) {
        $asTask = $asTaskGeneric.MakeGenericMethod($ResultType)
        $netTask = $asTask.Invoke($null, @($WinRtTask))
        $netTask.Wait(-1) | Out-Null
        $netTask.Result
    }

    $file = Await ([Windows.Storage.StorageFile]::GetFileFromPathAsync($tmp)) ([Windows.Storage.StorageFile])
    $stream = Await ($file.OpenAsync([Windows.Storage.FileAccessMode]::Read)) ([Windows.Storage.Streams.IRandomAccessStream])
    $decoder = Await ([Windows.Graphics.Imaging.BitmapDecoder]::CreateAsync($stream)) ([Windows.Graphics.Imaging.BitmapDecoder])
    $bitmap = Await ($decoder.GetSoftwareBitmapAsync()) ([Windows.Graphics.Imaging.SoftwareBitmap])

    $engine = [Windows.Media.Ocr.OcrEngine]::TryCreateFromUserProfileLanguages()
    $result = Await ($engine.RecognizeAsync($bitmap)) ([Windows.Media.Ocr.OcrResult])

    Write-Output $result.Text

    $stream.Dispose()
} catch {
    Write-Output ""
} finally {
    Remove-Item $tmp -ErrorAction SilentlyContinue
}
"#;

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", ps_script])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|e| format!("OCR PowerShell failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("OCR error: {}", stderr.chars().take(200).collect::<String>()));
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(text)
}

/// Read focused element text via PowerShell + .NET UIAutomation.
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
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
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

    let focused = value.or(name);
    Ok((focused, None))
}

/// Non-Windows fallback.
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
