use super::client::{ColumnMeta, Conn};
use super::filter::FilterSpec;
use super::metadata::quote_identifier;
use super::models::{ContentPage, DEFAULT_CONTENT_PAGE_SIZE};

pub(super) fn load_content_page(
    conn: &mut Conn,
    database: &str,
    table: &str,
    page: u64,
    filter: Option<&FilterSpec>,
) -> Result<(u64, ContentPage), String> {
    let content = fetch_content_page(conn, database, table, page, filter)?;
    if page > 0 && content.rows.is_empty() {
        return Ok((0, fetch_content_page(conn, database, table, 0, filter)?));
    }
    Ok((page, content))
}

fn fetch_content_page(
    conn: &mut Conn,
    database: &str,
    table: &str,
    page: u64,
    filter: Option<&FilterSpec>,
) -> Result<ContentPage, String> {
    let offset = page
        .checked_mul(DEFAULT_CONTENT_PAGE_SIZE as u64)
        .ok_or_else(|| "Page number is too large.".to_owned())?;
    let sql = match filter {
        // `SELECT * FROM t WHERE <filter> LIMIT n OFFSET p` — doc 06 §1.
        Some(spec) => {
            let clause = super::filter::where_clause(spec, &filter_escape(conn))?;
            format!(
                "SELECT * FROM {}.{} WHERE {} LIMIT {} OFFSET {offset}",
                quote_identifier(database),
                quote_identifier(table),
                clause,
                DEFAULT_CONTENT_PAGE_SIZE + 1
            )
        }
        None => format!(
            "SELECT * FROM {}.{} LIMIT {} OFFSET {offset}",
            quote_identifier(database),
            quote_identifier(table),
            DEFAULT_CONTENT_PAGE_SIZE + 1
        ),
    };
    let result = conn.query(&sql).map_err(|error| error.to_string())?;
    let columns: Vec<ColumnMeta> = result.columns.clone();
    let has_next = result.rows.len() > DEFAULT_CONTENT_PAGE_SIZE;
    let mut rows = result.rows;
    rows.truncate(DEFAULT_CONTENT_PAGE_SIZE);
    Ok(ContentPage {
        columns,
        rows,
        has_next,
    })
}

/// Escaper body-only untuk argumen filter (`${}`) — memakai escaping session
/// agar `NO_BACKSLASH_ESCAPES` juga benar (doc 13 §4).
fn filter_escape(conn: &Conn) -> impl Fn(&str) -> String + '_ {
    move |value| conn.escape_string(value, false).unwrap_or(value.to_owned())
}
