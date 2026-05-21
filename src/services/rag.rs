use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use calamine::{open_workbook_auto_from_rs, Reader};
use serde::{Deserialize, Serialize};

const RAG_DIR: &str = "context";
const INDEX_FILE: &str = "context/index.json";
const INFO_FILE: &str = "context/info.json";

const CHUNK_TARGET_CHARS: usize = 500;
const TOP_K_DEFAULT: usize = 5;
const BM25_K1: f64 = 1.5;
const BM25_B: f64 = 0.75;

#[derive(Default, Serialize, Deserialize, Clone)]
struct Index {
    chunks: Vec<String>,
    doc_lengths: Vec<u32>,
    avgdl: f64,
    postings: BTreeMap<String, Vec<(u32, u32)>>,
}

#[derive(Default, Serialize, Deserialize, Clone)]
struct Info {
    filename: String,
    n_chunks: usize,
}

fn ensure_dir() {
    let _ = fs::create_dir_all(RAG_DIR);
}

fn cache() -> &'static Mutex<Option<Index>> {
    static C: OnceLock<Mutex<Option<Index>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(None))
}

fn load_index() -> Option<Index> {
    if let Ok(g) = cache().lock()
        && let Some(i) = g.as_ref()
    {
        return Some(i.clone());
    }
    let raw = fs::read_to_string(INDEX_FILE).ok()?;
    let idx: Index = serde_json::from_str(&raw).ok()?;
    if let Ok(mut g) = cache().lock() {
        *g = Some(idx.clone());
    }
    Some(idx)
}

pub fn info() -> Option<(String, usize)> {
    let raw = fs::read_to_string(INFO_FILE).ok()?;
    let info: Info = serde_json::from_str(&raw).ok()?;
    Some((info.filename, info.n_chunks))
}

pub fn clear() {
    let _ = fs::remove_file(INDEX_FILE);
    let _ = fs::remove_file(INFO_FILE);
    if let Ok(mut g) = cache().lock() {
        *g = None;
    }
}

pub fn extract_text(filename: &str, bytes: &[u8]) -> Result<String, String> {
    let ext = Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "txt" | "md" | "csv" | "tsv" | "json" => String::from_utf8(bytes.to_vec())
            .map_err(|e| format!("not valid UTF-8 text: {}", e)),
        "xlsx" | "xls" | "ods" | "xlsb" => extract_spreadsheet(bytes),
        "" => Err("missing file extension".into()),
        other => Err(format!(
            "unsupported file type '.{}'. Use .txt, .md, .csv, .tsv, .json, or .xlsx. \
             For .docx/.pdf, export to .txt or .csv first. For SQLite, use the Database URL feature.",
            other
        )),
    }
}

fn extract_spreadsheet(bytes: &[u8]) -> Result<String, String> {
    let cursor = Cursor::new(bytes.to_vec());
    let mut workbook =
        open_workbook_auto_from_rs(cursor).map_err(|e| format!("spreadsheet open failed: {}", e))?;
    let mut out = String::new();
    let names: Vec<String> = workbook.sheet_names().to_vec();
    for name in names {
        let range = match workbook.worksheet_range(&name) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if range.is_empty() {
            continue;
        }
        out.push_str(&format!("# Sheet: {}\n", name));
        for row in range.rows() {
            let cells: Vec<String> = row.iter().map(|c| c.to_string()).collect();
            out.push_str(&cells.join("\t"));
            out.push('\n');
        }
        out.push('\n');
    }
    if out.is_empty() {
        return Err("spreadsheet has no readable sheets".into());
    }
    Ok(out)
}

fn tokenize(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 2)
        .map(|s| s.to_string())
        .collect()
}

fn chunk_text(text: &str) -> Vec<String> {
    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();
    for raw_line in text.lines() {
        let line = raw_line.trim_end();
        if current.len() + line.len() + 1 > CHUNK_TARGET_CHARS && !current.is_empty() {
            chunks.push(current.trim().to_string());
            current.clear();
        }
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(line);
        // Force-split if a single logical block grows too large.
        while current.len() > CHUNK_TARGET_CHARS * 2 {
            let slice_end = CHUNK_TARGET_CHARS.min(current.len());
            let break_at = current[..slice_end]
                .rfind(|c: char| c.is_whitespace())
                .unwrap_or(slice_end);
            let mut end = break_at;
            while !current.is_char_boundary(end) {
                end -= 1;
            }
            chunks.push(current[..end].trim().to_string());
            current = current[end..].trim_start().to_string();
        }
    }
    if !current.trim().is_empty() {
        chunks.push(current.trim().to_string());
    }
    chunks.retain(|c| !c.is_empty());
    chunks
}

pub fn build_index(filename: &str, text: &str) -> Result<usize, String> {
    ensure_dir();
    let chunks = chunk_text(text);
    if chunks.is_empty() {
        return Err("no extractable text".into());
    }

    let mut doc_lengths: Vec<u32> = Vec::with_capacity(chunks.len());
    let mut postings: BTreeMap<String, Vec<(u32, u32)>> = BTreeMap::new();

    for (idx, chunk) in chunks.iter().enumerate() {
        let tokens = tokenize(chunk);
        doc_lengths.push(tokens.len() as u32);
        let mut term_counts: BTreeMap<String, u32> = BTreeMap::new();
        for t in tokens {
            *term_counts.entry(t).or_insert(0) += 1;
        }
        for (term, tf) in term_counts {
            postings.entry(term).or_default().push((idx as u32, tf));
        }
    }

    let total_tokens: u32 = doc_lengths.iter().sum();
    let avgdl = if doc_lengths.is_empty() {
        0.0
    } else {
        total_tokens as f64 / doc_lengths.len() as f64
    };

    let index = Index {
        chunks: chunks.clone(),
        doc_lengths,
        avgdl,
        postings,
    };
    let n_chunks = chunks.len();
    let index_json = serde_json::to_string(&index).map_err(|e| e.to_string())?;
    fs::write(INDEX_FILE, index_json).map_err(|e| e.to_string())?;

    let info = Info {
        filename: filename.to_string(),
        n_chunks,
    };
    let info_json = serde_json::to_string(&info).map_err(|e| e.to_string())?;
    fs::write(INFO_FILE, info_json).map_err(|e| e.to_string())?;

    if let Ok(mut g) = cache().lock() {
        *g = Some(index);
    }
    Ok(n_chunks)
}

pub fn retrieve(query: &str, top_k: usize) -> Vec<String> {
    let index = match load_index() {
        Some(i) => i,
        None => return Vec::new(),
    };
    if index.chunks.is_empty() || index.avgdl == 0.0 {
        return Vec::new();
    }
    let q_tokens = tokenize(query);
    if q_tokens.is_empty() {
        return Vec::new();
    }

    let n_docs = index.chunks.len() as f64;
    let mut scores: BTreeMap<u32, f64> = BTreeMap::new();
    let mut seen: HashSet<String> = HashSet::new();

    for term in q_tokens {
        if !seen.insert(term.clone()) {
            continue;
        }
        let Some(postings) = index.postings.get(&term) else {
            continue;
        };
        let df = postings.len() as f64;
        let idf = ((n_docs - df + 0.5) / (df + 0.5) + 1.0).ln();
        for (chunk_idx, tf) in postings {
            let dl = index.doc_lengths[*chunk_idx as usize] as f64;
            let tf_f = *tf as f64;
            let numerator = tf_f * (BM25_K1 + 1.0);
            let denominator = tf_f + BM25_K1 * (1.0 - BM25_B + BM25_B * dl / index.avgdl);
            let score = idf * (numerator / denominator);
            *scores.entry(*chunk_idx).or_insert(0.0) += score;
        }
    }

    if scores.is_empty() {
        return Vec::new();
    }

    let mut sorted: Vec<(u32, f64)> = scores.into_iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    sorted.truncate(top_k);
    sorted
        .into_iter()
        .map(|(idx, _)| index.chunks[idx as usize].clone())
        .collect()
}

pub fn retrieve_default(query: &str) -> Vec<String> {
    retrieve(query, TOP_K_DEFAULT)
}
