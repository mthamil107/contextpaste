// ContextPaste — Screenshot + OCR Commands
//
// Tauri IPC commands for screen region capture with OCR.

use tauri::State;

use crate::prediction::engine;
use crate::screenshot::{capture, ocr};
use crate::storage::database::DbPool;
use crate::storage::models::RankedItem;

/// Capture a screen region, run OCR, and return the extracted text
/// along with context-ranked clipboard predictions.
#[tauri::command]
pub fn capture_and_ocr_region(
    db: State<'_, DbPool>,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<(String, Vec<RankedItem>), String> {
    let text = ocr::capture_and_ocr(x, y, width, height)?;
    log::info!(
        "Region OCR text: {}",
        text.chars().take(80).collect::<String>()
    );

    if text.is_empty() {
        return Ok(("".to_string(), Vec::new()));
    }

    let predictions = engine::get_context_predictions(&db, &text, 8)?;

    Ok((text, predictions))
}

/// Capture the full screen as base64 JPEG.
/// Called before showing the region selector overlay so it can use the screenshot as background.
#[tauri::command]
pub fn capture_fullscreen() -> Result<String, String> {
    capture::capture_fullscreen_base64()
}
