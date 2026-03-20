// ContextPaste — Screen Region Capture
//
// Captures a screen region as a PNG file using PowerShell's System.Drawing.
// Returns the path to the temporary PNG file.

/// Capture a screen region and save to a temp PNG file.
/// Returns the file path of the saved PNG.
#[cfg(target_os = "windows")]
pub fn capture_region_to_file(x: i32, y: i32, width: i32, height: i32) -> Result<String, String> {
    let ps_script = format!(
        r#"
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms

$w = {width}; $h = {height}
$bmp = New-Object System.Drawing.Bitmap($w, $h)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen({x}, {y}, 0, 0, (New-Object System.Drawing.Size($w, $h)))
$g.Dispose()

$tmp = [System.IO.Path]::GetTempFileName() + ".png"
$bmp.Save($tmp, [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()

Write-Output $tmp
"#,
        x = x,
        y = y,
        width = width,
        height = height
    );

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_script])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|e| format!("Screen capture failed: {}", e))?;

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Screen capture returned empty path. stderr: {}", stderr));
    }

    Ok(path)
}

/// Capture the full screen as a base64-encoded PNG.
/// Used by the region selector to show the screen as background (since WebView2 doesn't support true transparency).
#[cfg(target_os = "windows")]
pub fn capture_fullscreen_base64() -> Result<String, String> {
    let ps_script = r#"
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms

$bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$bmp = New-Object System.Drawing.Bitmap($bounds.Width, $bounds.Height)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
$g.Dispose()

$ms = New-Object System.IO.MemoryStream
$bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Jpeg)
$bmp.Dispose()
$bytes = $ms.ToArray()
$ms.Dispose()

Write-Output ([Convert]::ToBase64String($bytes))
"#;

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", ps_script])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|e| format!("Fullscreen capture failed: {}", e))?;

    let b64 = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if b64.is_empty() {
        return Err("Fullscreen capture returned empty".to_string());
    }

    Ok(b64)
}

#[cfg(not(target_os = "windows"))]
pub fn capture_fullscreen_base64() -> Result<String, String> {
    Err("Screen capture not supported on this platform".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn capture_region_to_file(_x: i32, _y: i32, _w: i32, _h: i32) -> Result<String, String> {
    Err("Screen capture not supported on this platform".to_string())
}
