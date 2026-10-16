use super::client::Conn;
use super::models::{QueryOutput, QueryResultSet};
use std::time::Instant;

// Batas tampilan khusus port; tampilan query macOS men-streaming tanpa batas ini.
const MAX_RESULT_ROWS: usize = 1000;
const MAX_RESULT_SETS: usize = 20;

/// Jalankan SQL dan kembalikan semua result set.
///
/// Memakai `mysql_real_query` + `mysql_store_result` + `mysql_next_result`,
/// jalur yang sama seperti `SPMySQLConnection` (doc 13 §3). Setiap set
/// di-materialisasi penuh di memori (macOS memakai `mysql_use_result`
/// untuk streaming pada mode low-memory); batas tampilan di bawah hanya
/// memotong apa yang dikirim ke UI, bukan yang dibaca dari server.
pub(super) fn execute_sql(conn: &mut Conn, sql: &str) -> Result<QueryOutput, String> {
    let started = Instant::now();
    let sets = conn.query_multi(sql).map_err(|error| error.to_string())?;

    let sets_truncated = sets.len() > MAX_RESULT_SETS;
    let mut output = Vec::with_capacity(sets.len().min(MAX_RESULT_SETS));
    for set in sets.into_iter().take(MAX_RESULT_SETS) {
        let truncated = set.rows.len() > MAX_RESULT_ROWS;
        let mut rows = set.rows;
        rows.truncate(MAX_RESULT_ROWS);
        output.push(QueryResultSet {
            columns: set.columns,
            rows,
            affected_rows: set.affected_rows,
            truncated,
        });
    }

    Ok(QueryOutput {
        sets: output,
        sets_truncated,
        elapsed_ms: started.elapsed().as_millis(),
    })
}
