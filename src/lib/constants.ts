// ContextPaste — Application Constants

export const APP_NAME = "ContextPaste";
export const APP_VERSION = "0.1.0";

// Default settings values
export const DEFAULTS = {
  MAX_HISTORY_ITEMS: 5000,
  CREDENTIAL_AUTO_EXPIRE_MINUTES: 30,
  HOTKEY_QUICK_PASTE: "CommandOrControl+Shift+V",
  HOTKEY_HISTORY: "CommandOrControl+Shift+H",
  OVERLAY_MAX_ITEMS: 8,
  OVERLAY_WIDTH: 420,
  OVERLAY_MAX_HEIGHT: 480,
  DEDUP_WINDOW_SECONDS: 2,
  PREVIEW_MAX_LENGTH: 80,
  SEARCH_DEBOUNCE_MS: 150,
} as const;

// Credential masking
export const MASK_CHAR = "•";
export const MASK_VISIBLE_CHARS = 4; // Show first 4 and last 4

// Prediction scoring weights
export const PREDICTION_WEIGHTS = {
  PIN_BOOST: 100,
  CHAIN_BOOST: 50,
  FREQUENCY: 0.4,
  RECENCY: 0.3,
  TYPE_MATCH: 0.2,
  SOURCE_AFFINITY: 0.1,
} as const;
