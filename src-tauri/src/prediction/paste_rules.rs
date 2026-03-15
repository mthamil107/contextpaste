// ContextPaste — Paste Rules Engine
//
// Evaluates user-defined paste rules against the current screen context.
// Rules match on app name, window title, and focused element text using regex patterns.

use regex::Regex;

use crate::storage::database::DbPool;
use crate::storage::models::{ClipItem, PasteRule, ScreenContext};
use crate::storage::queries;

/// Evaluate all enabled paste rules against the current screen context.
/// Returns the first matching rule and its resolved clipboard item, if any.
pub fn evaluate_rules(
    db: &DbPool,
    screen: &ScreenContext,
) -> Result<Option<(PasteRule, ClipItem)>, String> {
    let rules = queries::get_enabled_paste_rules(db)?;

    for rule in rules {
        if matches_rule(&rule, screen) {
            if let Some(item) = resolve_rule_action(db, &rule)? {
                queries::increment_rule_trigger(db, &rule.id)?;
                return Ok(Some((rule, item)));
            }
        }
    }
    Ok(None)
}

/// Check whether all non-null conditions in a rule match the screen context (AND logic).
fn matches_rule(rule: &PasteRule, screen: &ScreenContext) -> bool {
    if let Some(ref pattern) = rule.app_pattern {
        match Regex::new(pattern) {
            Ok(re) => {
                if let Some(ref app) = screen.app_name {
                    if !re.is_match(app) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            Err(e) => {
                log::debug!("Invalid app_pattern regex '{}': {}", pattern, e);
                return false;
            }
        }
    }

    if let Some(ref pattern) = rule.window_title_pattern {
        match Regex::new(pattern) {
            Ok(re) => {
                if let Some(ref title) = screen.window_title {
                    if !re.is_match(title) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            Err(e) => {
                log::debug!("Invalid window_title_pattern regex '{}': {}", pattern, e);
                return false;
            }
        }
    }

    if let Some(ref pattern) = rule.context_pattern {
        match Regex::new(pattern) {
            Ok(re) => {
                let context_text = screen
                    .focused_text
                    .as_deref()
                    .or(screen.surrounding_text.as_deref())
                    .unwrap_or("");
                if !re.is_match(context_text) {
                    return false;
                }
            }
            Err(e) => {
                log::debug!("Invalid context_pattern regex '{}': {}", pattern, e);
                return false;
            }
        }
    }

    true
}

/// Resolve a rule's action to a concrete clipboard item.
fn resolve_rule_action(db: &DbPool, rule: &PasteRule) -> Result<Option<ClipItem>, String> {
    match rule.action_type.as_str() {
        "paste_recent_type" => queries::get_most_recent_by_type(db, &rule.action_value),
        "paste_item" => {
            let item = queries::get_item(db, &rule.action_value)?;
            Ok(Some(item))
        }
        other => {
            log::debug!("Unknown rule action_type: {}", other);
            Ok(None)
        }
    }
}
