// ContextPaste — AI-Powered Smart Clipboard Manager
// Entry point for Tauri application

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    contextpaste_lib::run()
}
