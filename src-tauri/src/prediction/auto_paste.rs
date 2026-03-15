// ContextPaste — Auto-Paste Engine
//
// Orchestrates context-aware auto-paste by combining:
// 1. User-defined paste rules (highest priority)
// 2. Keyword-based content type inference from screen context
// 3. Multi-signal scoring (type match, keyword overlap, recency, affinity, pin/star)
//
// Credentials are NEVER auto-pasted.

use crate::storage::database::DbPool;
use crate::storage::models::{AutoPasteResult, ClipItem};
use crate::prediction::{context_reader, paste_rules};

/// Keyword groups mapped to content types for inference from screen context.
const KEYWORD_MAP: &[(&[&str], &str)] = &[
    (&["token", "api key", "api_key", "apikey", "secret", "password", "passwd", "credentials", "auth"], "Credential"),
    (&["url", "link", "website", "http", "endpoint", "webhook"], "Url"),
    (&["email", "e-mail", "mail"], "Email"),
    (&["ip", "ip address", "host", "server address"], "IpAddress"),
    (&["json", "payload", "body", "request body"], "Json"),
    (&["sql", "query", "select", "database query"], "Sql"),
    (&["command", "run", "execute", "terminal", "shell"], "ShellCommand"),
    (&["path", "file", "directory", "folder", "filename"], "FilePath"),
    (&["arn", "aws", "resource name"], "AwsArn"),
    (&["connection string", "dsn", "jdbc", "connect", "database url"], "ConnectionString"),
];

/// Attempt context-aware auto-paste.
///
/// Reads the current screen context, evaluates paste rules, and if no rule matches,
/// scores recent clipboard items against the context. Returns an `AutoPasteResult`
/// indicating whether to auto-paste or show the overlay.
///
/// `threshold` is the minimum confidence score (0.0-1.0) required for auto-paste.
pub fn try_auto_paste(
    db: &DbPool,
    threshold: f64,
) -> Result<AutoPasteResult, String> {
    // Step 1: Read screen context
    let screen = context_reader::read_screen_context();
    log::debug!(
        "Screen context: app={:?}, title={:?}, focused={:?}",
        screen.app_name,
        screen.window_title,
        screen.focused_text
    );

    // Step 2: Check user-defined paste rules first (highest priority)
    if let Some((rule, item)) = paste_rules::evaluate_rules(db, &screen)? {
        // Never auto-paste credentials
        if item.is_credential {
            log::debug!("Rule matched credential item, falling back to overlay");
            return Ok(AutoPasteResult {
                action: "ShowOverlay".to_string(),
                item: None,
                confidence: 0.0,
                matched_rule: Some(rule.name),
                reason: "Matched rule but item is a credential — cannot auto-paste".to_string(),
            });
        }
        let reason = format!("Matched rule: {}", rule.name);
        return Ok(AutoPasteResult {
            action: "AutoPaste".to_string(),
            item: Some(item),
            confidence: 1.0,
            matched_rule: Some(rule.name),
            reason,
        });
    }

    // Step 2.5: Check learned patterns (from previous manual pastes)
    let inferred_from_context = infer_content_types(&build_context_string(&screen));
    for inferred_type in &inferred_from_context {
        if let Ok(Some(pattern)) = crate::storage::queries::find_matching_pattern(
            db,
            inferred_type,
            screen.app_name.as_deref(),
        ) {
            // Found a learned pattern — get the most recent item of that type
            if let Ok(Some(item)) = crate::storage::queries::get_most_recent_by_type(db, &pattern.content_type) {
                if !item.is_credential {
                    let confidence = 0.5 + (pattern.frequency as f64 * 0.05).min(0.4); // 0.55-0.90 based on frequency
                    if confidence >= threshold {
                        return Ok(AutoPasteResult {
                            action: "AutoPaste".to_string(),
                            item: Some(item),
                            confidence,
                            matched_rule: Some(format!("Learned: {} in {}", pattern.content_type, pattern.target_app.as_deref().unwrap_or("?"))),
                            reason: format!("Learned pattern (used {} times)", pattern.frequency),
                        });
                    }
                }
            }
        }
    }

    // Step 3: Build combined context string for heuristic matching
    let context_text = build_context_string(&screen);
    if context_text.is_empty() {
        return Ok(AutoPasteResult {
            action: "ShowOverlay".to_string(),
            item: None,
            confidence: 0.0,
            matched_rule: None,
            reason: "No screen context available".to_string(),
        });
    }

    // Step 4: Infer expected content types from context keywords
    let inferred_types = infer_content_types(&context_text);

    // Step 5: Get candidate items from DB
    let candidates = crate::storage::queries::get_recent_items(db, 50, 0)?;

    if candidates.is_empty() {
        return Ok(AutoPasteResult {
            action: "ShowOverlay".to_string(),
            item: None,
            confidence: 0.0,
            matched_rule: None,
            reason: "No clipboard items available".to_string(),
        });
    }

    // Step 6: Score each candidate (skip credentials)
    let mut best_item: Option<&ClipItem> = None;
    let mut best_score: f64 = 0.0;
    let mut best_reason = String::new();

    for item in &candidates {
        // Never auto-paste credentials
        if item.is_credential {
            continue;
        }

        let score = compute_context_score(item, &screen, &inferred_types, &context_text);
        if score > best_score {
            best_score = score;
            best_item = Some(item);
            best_reason = format!(
                "type={}, score={:.2}, context={}",
                item.content_type.as_str(),
                score,
                context_text.chars().take(50).collect::<String>()
            );
        }
    }

    // Step 7: Decision gate — only auto-paste if confidence exceeds threshold
    if best_score >= threshold {
        if let Some(item) = best_item {
            return Ok(AutoPasteResult {
                action: "AutoPaste".to_string(),
                item: Some(item.clone()),
                confidence: best_score,
                matched_rule: None,
                reason: best_reason,
            });
        }
    }

    Ok(AutoPasteResult {
        action: "ShowOverlay".to_string(),
        item: best_item.cloned(),
        confidence: best_score,
        matched_rule: None,
        reason: format!(
            "Confidence {:.2} below threshold {:.2}",
            best_score, threshold
        ),
    })
}

/// Build a lowercase combined string from all available screen context.
fn build_context_string(
    screen: &crate::storage::models::ScreenContext,
) -> String {
    let mut parts = Vec::new();
    if let Some(ref t) = screen.window_title {
        parts.push(t.as_str());
    }
    if let Some(ref t) = screen.focused_text {
        parts.push(t.as_str());
    }
    if let Some(ref t) = screen.surrounding_text {
        parts.push(t.as_str());
    }
    parts.join(" ").to_lowercase()
}

/// Infer which content types are likely expected based on keywords in context text.
fn infer_content_types(context: &str) -> Vec<String> {
    let lower = context.to_lowercase();
    let mut types = Vec::new();

    for (keywords, content_type) in KEYWORD_MAP {
        for kw in *keywords {
            if lower.contains(kw) {
                types.push(content_type.to_string());
                break;
            }
        }
    }
    types
}

/// Compute a multi-signal score for how well a clip item matches the current context.
///
/// Signals:
/// 1. Content type match (0.0-0.4) — does the item's type match what we inferred?
/// 2. Keyword overlap (0.0-0.2) — do context words appear in the item content?
/// 3. Recency (0.0-0.2) — exponential decay with 1-hour half-life
/// 4. Cross-app affinity (0.0-0.05) — cross-app pastes are more intentional
/// 5. Pin/star boost (0.0-0.15)
fn compute_context_score(
    item: &ClipItem,
    screen: &crate::storage::models::ScreenContext,
    inferred_types: &[String],
    context_text: &str,
) -> f64 {
    let mut score = 0.0;

    // Signal 1: Content type match (0.0-0.4)
    let type_str = item.content_type.as_str().to_string();
    if inferred_types.contains(&type_str) {
        score += 0.4;
    }

    // Signal 2: Keyword match in content (0.0-0.2)
    let item_lower = item.content.to_lowercase();
    let context_words: Vec<&str> = context_text
        .split_whitespace()
        .filter(|w| w.len() > 3)
        .collect();
    let mut word_matches = 0;
    for word in &context_words {
        if item_lower.contains(word) {
            word_matches += 1;
        }
    }
    if !context_words.is_empty() {
        score += 0.2 * (word_matches as f64 / context_words.len() as f64).min(1.0);
    }

    // Signal 3: Recency (0.0-0.2)
    if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&item.created_at) {
        let age_secs = (chrono::Utc::now() - created.with_timezone(&chrono::Utc))
            .num_seconds() as f64;
        let recency = (-age_secs / 3600.0).exp();
        score += 0.2 * recency;
    }

    // Signal 4: Cross-app affinity (0.0-0.05)
    if let (Some(ref source), Some(ref target)) = (&item.source_app, &screen.app_name) {
        if source != target {
            score += 0.05;
        }
    }

    // Signal 5: Pin/star boost (0.0-0.15)
    if item.is_pinned {
        score += 0.1;
    }
    if item.is_starred {
        score += 0.05;
    }

    score.min(1.0)
}
