use std::collections::BTreeMap;
use std::sync::OnceLock;
use std::time::Duration;

use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{Column, Row};
use tokio::sync::Mutex;
use tokio::time::timeout;

const CONNECT_TIMEOUT_SECS: u64 = 5;
const QUERY_TIMEOUT_SECS: u64 = 5;
const MAX_CONNECTIONS: u32 = 2;
const MAX_ROWS: usize = 50;
const MAX_RESULT_CHARS: usize = 2000;
const MAX_SCHEMA_CHARS: usize = 4000;
const MAX_SQL_LEN: usize = 4000;

fn pool_cache() -> &'static Mutex<Option<(String, PgPool)>> {
    static C: OnceLock<Mutex<Option<(String, PgPool)>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(None))
}

fn schema_cache() -> &'static Mutex<Option<(String, String)>> {
    static C: OnceLock<Mutex<Option<(String, String)>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(None))
}

async fn get_pool(url: &str) -> Result<PgPool, String> {
    let mut guard = pool_cache().lock().await;
    if let Some((u, p)) = guard.as_ref() {
        if u == url {
            return Ok(p.clone());
        }
    }
    let pool = PgPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .acquire_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .connect(url)
        .await
        .map_err(|e| format!("connection failed: {}", e))?;
    *guard = Some((url.to_string(), pool.clone()));
    Ok(pool)
}

fn strip_sql_comments(sql: &str) -> String {
    let mut out = String::with_capacity(sql.len());
    let bytes = sql.as_bytes();
    let mut i = 0;
    let mut in_single = false;
    let mut in_double = false;
    while i < bytes.len() {
        let b = bytes[i];
        if !in_single && !in_double && i + 1 < bytes.len() && b == b'-' && bytes[i + 1] == b'-' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if !in_single && !in_double && i + 1 < bytes.len() && b == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            continue;
        }
        if b == b'\'' && !in_double {
            in_single = !in_single;
        } else if b == b'"' && !in_single {
            in_double = !in_double;
        }
        out.push(b as char);
        i += 1;
    }
    out
}

pub fn validate_query(sql: &str) -> Result<(), String> {
    if sql.len() > MAX_SQL_LEN {
        return Err(format!("query exceeds {} chars", MAX_SQL_LEN));
    }
    let stripped = strip_sql_comments(sql);
    let trimmed = stripped.trim().trim_end_matches(';').trim();
    if trimmed.is_empty() {
        return Err("empty query".into());
    }
    if trimmed.contains(';') {
        return Err("multiple statements not allowed".into());
    }
    let first_word_end = trimmed
        .find(|c: char| c.is_whitespace())
        .unwrap_or(trimmed.len());
    let first = trimmed[..first_word_end].to_lowercase();
    if first != "select" && first != "with" {
        return Err("only SELECT and WITH queries are allowed (read-only)".into());
    }
    Ok(())
}

fn cell_to_string(row: &sqlx::postgres::PgRow, idx: usize) -> String {
    if let Ok(Some(v)) = row.try_get::<Option<String>, _>(idx) {
        return v;
    }
    if let Ok(Some(v)) = row.try_get::<Option<i64>, _>(idx) {
        return v.to_string();
    }
    if let Ok(Some(v)) = row.try_get::<Option<i32>, _>(idx) {
        return v.to_string();
    }
    if let Ok(Some(v)) = row.try_get::<Option<f64>, _>(idx) {
        return v.to_string();
    }
    if let Ok(Some(v)) = row.try_get::<Option<bool>, _>(idx) {
        return v.to_string();
    }
    if let Ok(Some(v)) = row.try_get::<Option<i16>, _>(idx) {
        return v.to_string();
    }
    if let Ok(None::<String>) = row.try_get::<Option<String>, _>(idx) {
        return "null".into();
    }
    "<unsupported type — cast to ::text in SQL>".into()
}

fn truncate_chars(mut s: String, max: usize) -> String {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    s.truncate(end);
    s.push_str("\n... (output truncated)");
    s
}

pub async fn execute_query(url: &str, sql: &str) -> String {
    if let Err(e) = validate_query(sql) {
        return format!("Error: {}", e);
    }
    let pool = match get_pool(url).await {
        Ok(p) => p,
        Err(e) => return format!("Error: {}", e),
    };

    let fut = sqlx::query(sql).fetch_all(&pool);
    let rows = match timeout(Duration::from_secs(QUERY_TIMEOUT_SECS), fut).await {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => return format!("Error: {}", e),
        Err(_) => return format!("Error: query timed out after {}s", QUERY_TIMEOUT_SECS),
    };

    if rows.is_empty() {
        return "(no rows)".into();
    }

    let cols: Vec<String> = rows[0]
        .columns()
        .iter()
        .map(|c| c.name().to_string())
        .collect();

    let mut out = String::new();
    out.push_str(&cols.join(" | "));
    out.push('\n');

    let total = rows.len();
    for (i, row) in rows.iter().enumerate() {
        if i >= MAX_ROWS {
            out.push_str(&format!("... ({} more rows truncated)\n", total - MAX_ROWS));
            break;
        }
        let cells: Vec<String> = (0..cols.len()).map(|c| cell_to_string(row, c)).collect();
        out.push_str(&cells.join(" | "));
        out.push('\n');
    }

    truncate_chars(out, MAX_RESULT_CHARS)
}

pub async fn get_or_fetch_schema(url: &str) -> String {
    {
        let guard = schema_cache().lock().await;
        if let Some((u, s)) = guard.as_ref() {
            if u == url {
                return s.clone();
            }
        }
    }
    let s = fetch_schema(url).await;
    let mut guard = schema_cache().lock().await;
    *guard = Some((url.to_string(), s.clone()));
    s
}

async fn fetch_schema(url: &str) -> String {
    let pool = match get_pool(url).await {
        Ok(p) => p,
        Err(e) => return format!("(schema unavailable: {})", e),
    };
    let sql = "SELECT table_name, column_name, data_type \
               FROM information_schema.columns \
               WHERE table_schema = 'public' \
               ORDER BY table_name, ordinal_position \
               LIMIT 500";
    let fut = sqlx::query(sql).fetch_all(&pool);
    let rows = match timeout(Duration::from_secs(QUERY_TIMEOUT_SECS), fut).await {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => return format!("(schema fetch failed: {})", e),
        Err(_) => return "(schema fetch timed out)".into(),
    };
    let mut by_table: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for row in &rows {
        let t: String = row.try_get(0).unwrap_or_default();
        let c: String = row.try_get(1).unwrap_or_default();
        let ty: String = row.try_get(2).unwrap_or_default();
        by_table.entry(t).or_default().push(format!("{} {}", c, ty));
    }
    if by_table.is_empty() {
        return "(no tables found in 'public' schema)".into();
    }
    let mut out = String::new();
    for (table, cols) in &by_table {
        out.push_str(&format!("- {}({})\n", table, cols.join(", ")));
    }
    truncate_chars(out, MAX_SCHEMA_CHARS)
}
