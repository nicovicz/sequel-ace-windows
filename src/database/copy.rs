//! Aturan copy `SPCopyTable`, meniru `07-content-edit-copy.md` §4.
//!
//! Ini fungsi murni atas snapshot byte. Bagian yang memerlukan escaping menerima
//! *penyuntikan* (closure) supaya aturannya bisa diuji; pemanggil nyata
//! (`worker.rs`) menutupnya dengan `Conn` agar escaping mengikuti session —
//! sama seperti macOS app.

use super::client::{ColumnMeta, Conn, format_cell};
use super::models::{CopyKind, CopySnapshot};
use mysqlclient_sys as ffi;

/// Chunking copy-as-SQL: blok `INSERT INTO …` baru saat >250000 char
/// (doc 07 §4, "Chunking: blok `INSERT INTO…` baru saat >250000 char").
const INSERT_CHUNK_CHARS: usize = 250_000;

/// Escaper yang disuntikkan ke `rows_as_sql_inserts`.
pub type Escaper<'a> = &'a dyn Fn(&ColumnMeta, Option<&[u8]>) -> String;

/// Escaper nyata di atas `Conn`: blob/geometry → `X'<hex>'`, lainnya quoted.
pub fn session_escaper(conn: &Conn) -> impl Fn(&ColumnMeta, Option<&[u8]>) -> String + '_ {
    move |column, cell| match (cell, column.is_raw_binary()) {
        (Some(bytes), true) => conn.escape_data_hex(bytes),
        (Some(bytes), false) => {
            let text = String::from_utf8_lossy(bytes);
            conn.escape_string(&text, true).unwrap_or_default()
        }
        _ => "NULL".to_owned(),
    }
}

/// Bangun teks copy sesuai `kind`.
///
/// `SqlInsert*` memakai `escape` untuk literal tiap sel. Kolom auto_increment
/// dikeluarkan pada `SqlInsertNoAutoInc` (doc 07 §4, tag 2004); kolom generated
/// tidak dikeluarkan di sini karena flag `MYSQL_FIELD` tidak mengeksposnya —
/// itu fase 3 (`SHOW COLUMNS FROM` Extra).
pub fn render(snapshot: &CopySnapshot, kind: CopyKind, escape: Escaper<'_>) -> String {
    match kind {
        CopyKind::Tab => rows_as_tab(snapshot, false),
        CopyKind::TabWithHeaders => rows_as_tab(snapshot, true),
        CopyKind::Csv => rows_as_csv(snapshot),
        CopyKind::SqlInsert | CopyKind::SqlInsertNoAutoInc => {
            rows_as_sql_inserts(snapshot, escape, kind == CopyKind::SqlInsertNoAutoInc)
        }
    }
}

fn rows_as_tab(snapshot: &CopySnapshot, with_headers: bool) -> String {
    let mut out = String::new();
    if with_headers {
        for (index, column) in snapshot.columns.iter().enumerate() {
            if index > 0 {
                out.push('\t');
            }
            out.push_str(&column.name);
        }
        out.push('\n');
    }
    for row in &snapshot.rows {
        for (index, column) in snapshot.columns.iter().enumerate() {
            if index > 0 {
                out.push('\t');
            }
            // Nilai tab-copy mengikuti `displayStringForRow:column:` (doc 07 §4):
            // hex `0x…` untuk blob, bit-string untuk BIT, tanpa pemangkasan.
            out.push_str(&format_cell(
                Some(column),
                row.get(index).and_then(Option::as_deref),
                usize::MAX,
            ));
        }
        // Tab akhir dibuang; `"\n"` menggantikannya (doc 07 §4).
        out.push('\n');
    }
    out
}

/// CSV sesuai `rowsAsCsvStringWithHeaders:` (doc 07 §4): header dibungkus quote
/// dengan `"` internal di-double; nilai dibungkus quote **tanpa** escape quote
/// internal ("description dibungkus quote tanpa escape quote internal"); NULL
/// ikut dibungkus quote; `"\n"` per baris.
fn rows_as_csv(snapshot: &CopySnapshot) -> String {
    let mut out = String::new();
    for (index, column) in snapshot.columns.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push('"');
        out.push_str(&column.name.replace('"', "\"\""));
        out.push('"');
    }
    out.push_str("\r\n");
    for row in &snapshot.rows {
        for (index, column) in snapshot.columns.iter().enumerate() {
            if index > 0 {
                out.push(',');
            }
            let value = format_cell(
                Some(column),
                row.get(index).and_then(Option::as_deref),
                usize::MAX,
            );
            out.push('"');
            out.push_str(&value);
            out.push('"');
        }
        out.push_str("\r\n");
    }
    out
}

fn insert_preamble(table: &str, columns: &[&ColumnMeta]) -> String {
    let all = columns
        .iter()
        .map(|column| Conn::quote_identifier(&column.name))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "INSERT INTO {} ({all})\nVALUES\n",
        Conn::quote_identifier(table)
    )
}

fn rows_as_sql_inserts(
    snapshot: &CopySnapshot,
    escape: Escaper<'_>,
    skip_auto_increment: bool,
) -> String {
    let Some(table) = snapshot.table.as_deref() else {
        return String::new();
    };

    // Simpan indeks asli supaya sel dimap ke kolom yang tepat saat kolom
    // auto_increment di-skip (doc 07 §4).
    let kept: Vec<(usize, &ColumnMeta)> = snapshot
        .columns
        .iter()
        .enumerate()
        .filter(|(_, column)| !(skip_auto_increment && column.is_auto_increment()))
        .collect();

    let mut out = insert_preamble(
        table,
        &kept.iter().map(|(_, column)| *column).collect::<Vec<_>>(),
    );
    let mut chunk_chars = 0usize;
    for row in &snapshot.rows {
        out.push('\t');
        out.push('(');
        let mut line_chars = 0usize;
        for (count, (index, column)) in kept.iter().enumerate() {
            if count > 0 {
                out.push_str(", ");
            }
            let literal = literal_for(column, row.get(*index).and_then(Option::as_deref), escape);
            out.push_str(&literal);
            line_chars += literal.chars().count();
        }
        chunk_chars += line_chars;
        out.push_str("),\n");
        // Blok `INSERT INTO…` baru saat sisa markah mencapai batas char.
        if chunk_chars >= INSERT_CHUNK_CHARS {
            out.truncate(out.len() - ",\n".len());
            out.push_str(";\n");
            out.push_str(&insert_preamble(
                table,
                &kept.iter().map(|(_, column)| *column).collect::<Vec<_>>(),
            ));
            chunk_chars = 0;
        }
    }
    // `",\n"` akhir dipangkas, lalu `";\n"` final (doc 07 §4).
    if out.ends_with(",\n") {
        out.truncate(out.len() - ",\n".len());
    }
    out.push_str(";\n");
    out
}

/// Literal per sel mengikuti aturan doc 07 §4: NULL → `NULL` unquoted;
/// numeric unquoted; bit → `b'…'`; blob/geometry → `X'<hex>'`; lainnya quoted.
pub fn literal_for(column: &ColumnMeta, cell: Option<&[u8]>, escape: Escaper<'_>) -> String {
    let Some(bytes) = cell else {
        return "NULL".to_owned();
    };

    if column.field_type == ffi::enum_field_types::MYSQL_TYPE_BIT {
        // Lebar bit mengikuti doc 13 §4: MIN(bytes*8, field.length).
        let width = (bytes.len() * 8).min(column.length as usize);
        let mut bits = String::with_capacity(width + 3);
        bits.push_str("b'");
        for index in 0..width {
            let byte = bytes[index / 8];
            let shift = 7 - (index % 8);
            bits.push(if byte >> shift & 1 == 1 { '1' } else { '0' });
        }
        bits.push('\'');
        return bits;
    }

    if column.is_raw_binary() {
        return escape(column, Some(bytes));
    }

    if column.is_unquoted_numeric() {
        String::from_utf8_lossy(bytes).into_owned()
    } else {
        escape(column, Some(bytes))
    }
}

/// Uji hidup terhadap server nyata, digerbangi variabel lingkungan supaya
/// `cargo test` biasa tetap tidak menyentuh jaringan:
///
/// ```sh
/// export SA_PROBE_HOST=127.0.0.1 SA_PROBE_PORT=3306 SA_PROBE_USER=root SA_PROBE_PASSWORD=…
/// cargo test --locked -- --ignored
/// ```
///
/// Memverifikasi copy-as-SQL memakai escaping session (bukan escaping murni
/// Rust, doc 13 §4) dan bahwa `NO_BACKSLASH_ESCAPES` menghasilkan `'O''Brien'`.
#[cfg(test)]
mod live_tests {
    use super::{render, session_escaper};
    use crate::database::models::{CopyKind, CopySnapshot};

    const TABLE: &str = "sa_probe_copy";

    fn connection() -> Option<crate::database::client::Conn> {
        let host = std::env::var("SA_PROBE_HOST").ok()?;
        let mut params = crate::database::client::ConnectParams::new(
            host,
            std::env::var("SA_PROBE_PORT")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(3306_u16),
            std::env::var("SA_PROBE_USER").unwrap_or_else(|_| "root".to_owned()),
        );
        params.password = std::env::var("SA_PROBE_PASSWORD").unwrap_or_default();
        Some(crate::database::client::Conn::connect(&params).expect("connect to probe server"))
    }

    /// `#[ignore]` supaya `cargo test` biasa tak menyentuh jaringan; jalankan
    /// dengan `cargo test --locked -- --ignored` setelah menyetel `SA_PROBE_*`.
    #[test]
    #[ignore = "uji hidup, butuh server dan SA_PROBE_*"]
    fn copy_rules_follow_the_session() {
        let Some(mut conn) = connection() else {
            eprintln!("SA_PROBE_* tidak diset; uji hidup dilewati.");
            return;
        };
        for statement in [
            format!("DROP DATABASE IF EXISTS {TABLE}_db"),
            format!("CREATE DATABASE {TABLE}_db"),
        ] {
            conn.execute(&statement).expect("probe db setup");
        }
        conn.select_db(&format!("{TABLE}_db"))
            .expect("select probe db");
        conn.execute(
            "CREATE TABLE sa_probe_copy (id INT AUTO_INCREMENT PRIMARY KEY, \
             name VARCHAR(64), flag BIT(8), raw VARBINARY(32))",
        )
        .expect("create probe table");
        let seed_name = conn.quote_literal("O'Brien").expect("escape probe value");
        conn.execute(&format!(
            "INSERT INTO {} (name, flag, raw) VALUES ({seed_name}, b'01010101', X'0F0F00')",
            crate::database::client::Conn::quote_identifier(TABLE),
        ))
        .expect("seed probe table");
        let set = conn
            .query("SELECT name, flag, raw FROM sa_probe_copy")
            .expect("select probe");
        let snapshot = CopySnapshot {
            table: Some(TABLE.to_owned()),
            columns: set.columns.clone(),
            rows: set.rows.clone(),
        };

        assert_eq!(
            render(&snapshot, CopyKind::Tab, &session_escaper(&conn)),
            "O'Brien\t01010101\t0x0F0F00\n"
        );
        assert_eq!(
            render(
                &snapshot,
                CopyKind::SqlInsertNoAutoInc,
                &session_escaper(&conn)
            ),
            "INSERT INTO `sa_probe_copy` (`name`, `flag`, `raw`)\n\
             VALUES\n\t('O\\'Brien', b'01010101', X'0F0F00');\n"
        );
        // NO_BACKSLASH_ESCAPES menggandakan quote, bukan backslash (doc 13 §4).
        conn.execute("SET SESSION sql_mode = 'NO_BACKSLASH_ESCAPES'")
            .expect("set sql_mode");
        assert_eq!(
            render(&snapshot, CopyKind::SqlInsert, &session_escaper(&conn)),
            "INSERT INTO `sa_probe_copy` (`name`, `flag`, `raw`)\n\
             VALUES\n\t('O''Brien', b'01010101', X'0F0F00');\n"
        );

        // teardown
        conn.execute(&format!("DROP DATABASE IF EXISTS {TABLE}_db"))
            .ok();
    }
}
