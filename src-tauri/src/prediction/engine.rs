// ContextPaste — Prediction Engine
// Ranks clipboard items by: pin boost, chain boost, frequency, recency, type match, source affinity.
//
// score = pin_boost * 100
//       + chain_boost * 50
//       + frequency_score * 0.6    // paste_count * 10, capped at 100 — most important signal
//       + recency_score * 0.2      // e^(-t/3600) decay, τ=1hour
//       + type_match_score * 0.2
//       + source_affinity * 0.1

use std::collections::HashMap;

use crate::storage::database::DbPool;
use crate::storage::models::{ClipItem, RankedItem};
use crate::storage::queries;

/// Context data preloaded for a target app to avoid repeated DB queries per item.
struct TargetContext {
    /// Normalized scores (0-100) for each content type historically pasted into the target app.
    type_scores: HashMap<String, f64>,
    /// The target app name, used for source affinity lookups.
    target_app: String,
}

impl TargetContext {
    fn load(db: &DbPool, target_app: &str) -> Result<Self, String> {
        let type_scores = queries::get_type_match_scores(db, target_app)?;
        Ok(Self {
            type_scores,
            target_app: target_app.to_string(),
        })
    }

    /// Get the type match score for a given content type (0.0 to 100.0).
    fn type_match(&self, content_type: &str) -> f64 {
        self.type_scores.get(content_type).copied().unwrap_or(0.0)
    }
}

/// Rank items for the quick paste overlay.
/// When target_app is provided, type match and source affinity scores are included.
/// Uses paste sequence tracking: if you just pasted item A, item B (which you usually
/// paste after A) gets boosted to the top.
pub fn get_predictions(
    db: &DbPool,
    limit: u32,
    target_app: Option<&str>,
) -> Result<Vec<RankedItem>, String> {
    // Get recent items (fetch more than needed, then rank and trim)
    let items = queries::get_recent_items(db, limit * 3, 0)?;

    // Preload target context if we know the target app
    let context = match target_app {
        Some(app) if !app.is_empty() => Some(TargetContext::load(db, app)?),
        _ => None,
    };

    // Find what was LAST pasted — so we can boost the "next in sequence" item
    let next_in_sequence = find_next_in_sequence(db);

    // Preload source affinity scores for all unique source apps in one pass
    let source_affinities: HashMap<String, f64> = match &context {
        Some(ctx) => {
            let mut affinities = HashMap::new();
            for item in &items {
                if let Some(ref src) = item.source_app {
                    if !affinities.contains_key(src) {
                        let score =
                            queries::get_source_affinity(db, src, &ctx.target_app)
                                .unwrap_or(0.0);
                        affinities.insert(src.clone(), score);
                    }
                }
            }
            affinities
        }
        None => HashMap::new(),
    };

    let now = chrono::Utc::now();
    let mut ranked: Vec<RankedItem> = items
        .into_iter()
        .map(|item| {
            let (mut score, mut reason) =
                compute_score(&item, &now, context.as_ref(), &source_affinities);

            // Sequence boost: if this item usually follows the last-pasted item, boost it to top
            if let Some(ref next_id) = next_in_sequence {
                if item.id == *next_id {
                    score += 200.0; // Higher than pin boost (100) — sequence is the strongest signal
                    reason = "next_in_sequence".to_string();
                }
            }

            RankedItem {
                item,
                score,
                reason,
            }
        })
        .collect();

    // Sort by score descending
    ranked.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    ranked.truncate(limit as usize);

    Ok(ranked)
}

/// Re-rank predictions based on screen context text (from OCR near cursor).
/// This is called AFTER OCR completes and matches the screen text against items.
///
/// Example: screen shows "GitHub Username:" → ranks items containing "username"
/// or items of type PlainText that have been pasted when similar text was on screen.
pub fn get_context_predictions(
    db: &DbPool,
    screen_text: &str,
    limit: u32,
) -> Result<Vec<RankedItem>, String> {
    let items = queries::get_recent_items(db, limit * 3, 0)?;
    let screen_lower = screen_text.to_lowercase();

    // Keyword to content type mapping
    let keyword_types: &[(&[&str], &str)] = &[
        (&["username", "user name", "user", "login", "account"], "PlainText"),
        (&["password", "passwd", "secret", "token", "access token", "personal access", "api key", "apikey"], "Credential"),
        (&["url", "link", "endpoint", "http", "https", "webhook"], "Url"),
        (&["email", "e-mail", "mail"], "Email"),
        (&["ip", "address", "host", "server"], "IpAddress"),
        (&["json", "payload", "body"], "Json"),
        (&["sql", "query", "select", "database"], "Sql"),
        (&["command", "cmd", "run", "execute", "terminal", "shell", "bash"], "ShellCommand"),
        (&["path", "file", "directory", "folder"], "FilePath"),
        (&["connection", "connect", "dsn", "jdbc", "database url"], "ConnectionString"),
    ];

    // Find which content types the screen text suggests
    let mut inferred_types: Vec<&str> = Vec::new();
    for (keywords, content_type) in keyword_types {
        for kw in *keywords {
            if screen_lower.contains(kw) {
                inferred_types.push(content_type);
                break;
            }
        }
    }

    let now = chrono::Utc::now();
    let next_in_seq = find_next_in_sequence(db);

    let mut ranked: Vec<RankedItem> = items
        .into_iter()
        .map(|item| {
            let mut score = 0.0_f64;
            let mut reason = "default".to_string();

            // Sequence boost (strongest signal)
            if let Some(ref next_id) = next_in_seq {
                if item.id == *next_id {
                    score += 200.0;
                    reason = "next_in_sequence".to_string();
                }
            }

            // Context type match (very strong) — screen says "username" and item is PlainText
            let type_str = item.content_type.as_str();
            if inferred_types.contains(&type_str) {
                score += 150.0;
                reason = format!("screen_match:{}", type_str);
            }

            // Direct content match — screen text contains words from the item
            let item_lower = item.content.to_lowercase();
            let screen_words: Vec<&str> = screen_lower.split_whitespace()
                .filter(|w| w.len() > 2)
                .collect();
            let mut word_matches = 0;
            for word in &screen_words {
                if item_lower.contains(word) {
                    word_matches += 1;
                }
            }
            if word_matches > 0 && !screen_words.is_empty() {
                let match_ratio = word_matches as f64 / screen_words.len() as f64;
                score += match_ratio * 80.0;
                if reason == "default" {
                    reason = "content_match".to_string();
                }
            }

            // Frequency boost
            let freq_score = (item.paste_count as f64 * 10.0).min(100.0);
            score += freq_score * 0.4;

            // Recency boost (smaller)
            let created = chrono::NaiveDateTime::parse_from_str(&item.created_at, "%Y-%m-%d %H:%M:%S")
                .unwrap_or_else(|_| now.naive_utc());
            let age_secs = (now.naive_utc() - created).num_seconds().max(0) as f64;
            let recency = (-age_secs / 3600.0).exp() * 100.0;
            score += recency * 0.1;

            // Pin boost
            if item.is_pinned {
                score += 100.0;
            }

            RankedItem { item, score, reason }
        })
        .collect();

    ranked.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    ranked.truncate(limit as usize);

    if let Some(first) = ranked.first() {
        log::info!("Context prediction: top={} score={:.1} reason={}",
            first.item.content.chars().take(30).collect::<String>(),
            first.score, first.reason);
    }

    Ok(ranked)
}

/// Find the item that should be pasted NEXT based on paste sequence history.
/// Looks at the most recent paste event, then finds which item was most frequently
/// pasted immediately after that same item in the past.
///
/// Example: If you always paste "mthamil107" then "ghp_token", after pasting
/// "mthamil107" this function returns the ID of "ghp_token".
fn find_next_in_sequence(db: &DbPool) -> Option<String> {
    let conn = db.lock().ok()?;

    // Get the item that was just pasted (most recent paste event)
    let last_pasted_id: String = conn.query_row(
        "SELECT item_id FROM paste_events ORDER BY pasted_at DESC LIMIT 1",
        [],
        |row| row.get(0),
    ).ok()?;

    // Find which item was most frequently pasted RIGHT AFTER this item
    // Uses rowid to find the immediately next paste event after each occurrence
    let next_id: Option<String> = conn.query_row(
        "SELECT p2.item_id
         FROM paste_events p1
         JOIN paste_events p2 ON p2.rowid = (
             SELECT MIN(p3.rowid) FROM paste_events p3 WHERE p3.rowid > p1.rowid
         )
         WHERE p1.item_id = ?1
         AND p2.item_id != ?1
         GROUP BY p2.item_id
         ORDER BY COUNT(*) DESC
         LIMIT 1",
        rusqlite::params![last_pasted_id],
        |row| row.get(0),
    ).ok();

    if let Some(ref id) = next_id {
        log::info!("Sequence prediction: after {} → next should be {}", last_pasted_id, id);
    }

    next_id
}

fn compute_score(
    item: &ClipItem,
    now: &chrono::DateTime<chrono::Utc>,
    context: Option<&TargetContext>,
    source_affinities: &HashMap<String, f64>,
) -> (f64, String) {
    let mut score = 0.0_f64;
    let mut top_reason = "recency";
    let mut top_component = 0.0_f64;

    // Pin boost
    if item.is_pinned {
        score += 100.0;
        top_reason = "pinned";
        top_component = 100.0;
    }

    // Frequency score — items pasted many times get a STRONG boost
    // 1 paste = 10, 3 pastes = 30, 5+ pastes = 50+
    let freq_score = (item.paste_count as f64 * 10.0).min(100.0);
    let freq_contrib = freq_score * 0.6;
    score += freq_contrib;
    if freq_contrib > top_component && !item.is_pinned {
        top_component = freq_contrib;
        top_reason = "frequency";
    }

    // Recency score: e^(-t/3600) where t is seconds since creation
    let created = chrono::NaiveDateTime::parse_from_str(&item.created_at, "%Y-%m-%d %H:%M:%S")
        .unwrap_or_else(|_| now.naive_utc());
    let age_seconds = (now.naive_utc() - created).num_seconds().max(0) as f64;
    let recency_score = (-age_seconds / 3600.0).exp() * 100.0;
    let recency_contrib = recency_score * 0.2;
    score += recency_contrib;
    if recency_contrib > top_component && !item.is_pinned {
        top_component = recency_contrib;
        top_reason = "recency";
    }

    // Starred items get a small boost
    if item.is_starred {
        score += 10.0;
    }

    // Type match score (0.2 weight) — only when target context is available
    if let Some(ctx) = context {
        let type_match = ctx.type_match(item.content_type.as_str());
        let type_contrib = type_match * 0.2;
        score += type_contrib;
        if type_contrib > top_component && !item.is_pinned {
            top_component = type_contrib;
            top_reason = "type_match";
        }

        // Source affinity score (0.1 weight)
        if let Some(ref src) = item.source_app {
            let affinity = source_affinities.get(src).copied().unwrap_or(0.0);
            let affinity_contrib = affinity * 0.1;
            score += affinity_contrib;
            if affinity_contrib > top_component && !item.is_pinned {
                // top_component not updated here as this is the final scoring component
                top_reason = "source_affinity";
            }
        }
    }

    (score, top_reason.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database::init_test_db;
    use crate::storage::models::ContentType;
    use crate::storage::queries::{compute_hash, insert_clip_item, update_prediction_stat};

    fn make_item(content_type: ContentType, pinned: bool, paste_count: i64, created_at: &str) -> ClipItem {
        make_item_with_source(content_type, pinned, paste_count, created_at, None)
    }

    fn make_item_with_source(
        content_type: ContentType,
        pinned: bool,
        paste_count: i64,
        created_at: &str,
        source_app: Option<&str>,
    ) -> ClipItem {
        ClipItem {
            id: uuid::Uuid::new_v4().to_string(),
            content: "test".to_string(),
            content_type,
            content_hash: compute_hash(&uuid::Uuid::new_v4().to_string()),
            content_length: 4,
            is_credential: false,
            credential_type: None,
            source_app: source_app.map(|s| s.to_string()),
            source_window_title: None,
            is_pinned: pinned,
            is_starred: false,
            expires_at: None,
            created_at: created_at.to_string(),
            last_pasted_at: None,
            paste_count,
            tags: None,
        }
    }

    #[test]
    fn test_pinned_higher_score() {
        let now = chrono::Utc::now();
        let ts = now.format("%Y-%m-%d %H:%M:%S").to_string();
        let (pinned_score, _) = compute_score(
            &make_item(ContentType::PlainText, true, 0, &ts),
            &now,
            None,
            &HashMap::new(),
        );
        let (normal_score, _) = compute_score(
            &make_item(ContentType::PlainText, false, 0, &ts),
            &now,
            None,
            &HashMap::new(),
        );
        assert!(pinned_score > normal_score);
    }

    #[test]
    fn test_recent_higher_than_old() {
        let now = chrono::Utc::now();
        let recent = now.format("%Y-%m-%d %H:%M:%S").to_string();
        let old = (now - chrono::Duration::hours(2))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let (recent_score, _) = compute_score(
            &make_item(ContentType::PlainText, false, 0, &recent),
            &now,
            None,
            &HashMap::new(),
        );
        let (old_score, _) = compute_score(
            &make_item(ContentType::PlainText, false, 0, &old),
            &now,
            None,
            &HashMap::new(),
        );
        assert!(recent_score > old_score);
    }

    #[test]
    fn test_frequency_boost() {
        let now = chrono::Utc::now();
        let ts = now.format("%Y-%m-%d %H:%M:%S").to_string();
        let (high_freq, _) = compute_score(
            &make_item(ContentType::PlainText, false, 50, &ts),
            &now,
            None,
            &HashMap::new(),
        );
        let (low_freq, _) = compute_score(
            &make_item(ContentType::PlainText, false, 0, &ts),
            &now,
            None,
            &HashMap::new(),
        );
        assert!(high_freq > low_freq);
    }

    #[test]
    fn test_type_match_boosts_matching_content_type() {
        let now = chrono::Utc::now();
        let ts = now.format("%Y-%m-%d %H:%M:%S").to_string();

        // Simulate a target context where Url is the top type (score=100)
        let mut type_scores = HashMap::new();
        type_scores.insert("Url".to_string(), 100.0);
        type_scores.insert("PlainText".to_string(), 10.0);

        let ctx = TargetContext {
            type_scores,
            target_app: "VSCode".to_string(),
        };

        let url_item = make_item(ContentType::Url, false, 0, &ts);
        let plain_item = make_item(ContentType::PlainText, false, 0, &ts);

        let (url_score, _) =
            compute_score(&url_item, &now, Some(&ctx), &HashMap::new());
        let (plain_score, _) =
            compute_score(&plain_item, &now, Some(&ctx), &HashMap::new());

        // Url should score higher because of type_match boost (0.2 * 100 = 20 vs 0.2 * 10 = 2)
        assert!(
            url_score > plain_score,
            "Url score ({}) should be higher than PlainText score ({})",
            url_score,
            plain_score
        );

        // Verify the difference is approximately the type_match delta:
        // (100 * 0.2) - (10 * 0.2) = 18.0
        let diff = url_score - plain_score;
        assert!(
            (diff - 18.0).abs() < 0.01,
            "Score difference ({}) should be ~18.0 (type_match delta)",
            diff
        );
    }

    #[test]
    fn test_source_affinity_boosts_correlated_source() {
        let now = chrono::Utc::now();
        let ts = now.format("%Y-%m-%d %H:%M:%S").to_string();

        let ctx = TargetContext {
            type_scores: HashMap::new(),
            target_app: "VSCode".to_string(),
        };

        let chrome_item =
            make_item_with_source(ContentType::PlainText, false, 0, &ts, Some("Chrome"));
        let notepad_item =
            make_item_with_source(ContentType::PlainText, false, 0, &ts, Some("Notepad"));

        // Chrome has high affinity, Notepad has none
        let mut affinities = HashMap::new();
        affinities.insert("Chrome".to_string(), 80.0);
        affinities.insert("Notepad".to_string(), 5.0);

        let (chrome_score, _) =
            compute_score(&chrome_item, &now, Some(&ctx), &affinities);
        let (notepad_score, _) =
            compute_score(&notepad_item, &now, Some(&ctx), &affinities);

        assert!(
            chrome_score > notepad_score,
            "Chrome-sourced item ({}) should rank higher than Notepad-sourced item ({})",
            chrome_score,
            notepad_score
        );
    }

    #[test]
    fn test_no_target_app_still_works() {
        // Without context, scoring should not crash and should use only
        // pin + frequency + recency + starred
        let now = chrono::Utc::now();
        let ts = now.format("%Y-%m-%d %H:%M:%S").to_string();

        let item = make_item(ContentType::Url, false, 10, &ts);
        let (score, reason) = compute_score(&item, &now, None, &HashMap::new());

        assert!(score > 0.0);
        // Without context, reason should be frequency or recency
        assert!(reason == "frequency" || reason == "recency");
    }

    #[test]
    fn test_full_ranking_order_with_mixed_items() {
        let db = init_test_db().expect("DB init failed");
        let now = chrono::Utc::now();
        let ts = now.format("%Y-%m-%d %H:%M:%S").to_string();
        let old_ts = (now - chrono::Duration::hours(3))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        // Seed prediction stats: Url is the top type pasted into VSCode
        for _ in 0..20 {
            update_prediction_stat(&db, "Url", Some("Chrome"), "VSCode")
                .expect("stat failed");
        }
        for _ in 0..5 {
            update_prediction_stat(&db, "Code", Some("Terminal"), "VSCode")
                .expect("stat failed");
        }

        // Insert items with different characteristics
        // Item A: pinned PlainText, old
        let mut a = make_item(ContentType::PlainText, true, 0, &old_ts);
        a.content = "pinned item".to_string();
        a.content_hash = compute_hash("pinned item");
        insert_clip_item(&db, &a).expect("insert failed");

        // Item B: Url (matching type), recent, from Chrome (high source affinity)
        let mut b = make_item_with_source(ContentType::Url, false, 5, &ts, Some("Chrome"));
        b.content = "https://example.com".to_string();
        b.content_hash = compute_hash("https://example.com");
        insert_clip_item(&db, &b).expect("insert failed");

        // Item C: Code (lesser matching type), recent, from Terminal
        let mut c = make_item_with_source(ContentType::Code, false, 5, &ts, Some("Terminal"));
        c.content = "fn main() {}".to_string();
        c.content_hash = compute_hash("fn main() {}");
        insert_clip_item(&db, &c).expect("insert failed");

        // Item D: PlainText (no type match), old, low frequency
        let mut d = make_item(ContentType::PlainText, false, 0, &old_ts);
        d.content = "plain old text".to_string();
        d.content_hash = compute_hash("plain old text");
        insert_clip_item(&db, &d).expect("insert failed");

        let ranked = get_predictions(&db, 10, Some("VSCode")).expect("predictions failed");

        assert!(ranked.len() >= 4);

        // Pinned item should be first
        assert_eq!(ranked[0].item.id, a.id, "Pinned item should rank first");
        assert_eq!(ranked[0].reason, "pinned");

        // Url+Chrome item should rank above plain old text
        let b_pos = ranked.iter().position(|r| r.item.id == b.id).unwrap();
        let d_pos = ranked.iter().position(|r| r.item.id == d.id).unwrap();
        assert!(
            b_pos < d_pos,
            "Url item from Chrome (pos {}) should rank above plain old text (pos {})",
            b_pos,
            d_pos
        );

        // Code item from Terminal should rank above plain old text too
        let c_pos = ranked.iter().position(|r| r.item.id == c.id).unwrap();
        assert!(
            c_pos < d_pos,
            "Code item (pos {}) should rank above plain old text (pos {})",
            c_pos,
            d_pos
        );
    }

    #[test]
    fn test_get_predictions_without_target_app() {
        let db = init_test_db().expect("DB init failed");
        let now = chrono::Utc::now();
        let ts = now.format("%Y-%m-%d %H:%M:%S").to_string();

        let mut item = make_item(ContentType::PlainText, false, 0, &ts);
        item.content = "no context item".to_string();
        item.content_hash = compute_hash("no context item");
        insert_clip_item(&db, &item).expect("insert failed");

        // Should work fine with None
        let ranked = get_predictions(&db, 10, None).expect("predictions failed");
        assert!(!ranked.is_empty());
        assert_eq!(ranked[0].item.id, item.id);

        // Should also work with empty string
        let ranked2 = get_predictions(&db, 10, Some("")).expect("predictions failed");
        assert!(!ranked2.is_empty());
    }
}
