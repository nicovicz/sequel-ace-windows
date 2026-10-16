//! Derive SQL edit — port `07-content-edit-copy.md` §2-3.
//!
//! Fungsi murni atas `EditPlan`; escaping disuntikkan (pola `copy.rs`) supaya
//! test tidak butuh server. Pemanggil nyata (`worker.rs`) menutupnya dengan
//! `Conn` agar escaping mengikuti session — sama seperti macOS app.

use super::client::ColumnMeta;
use mysqlclient_sys as ffi;

/// Escaper yang disuntikkan; body-only (tanpa quote pembungkus) karena template
/// SQL sudah memuat pasangan quote (`'${}'`).
pub type Escaper<'a> = &'a dyn Fn(&ColumnMeta, Option<&[u8]>) -> String;

/// Satu rencana edit: tabel + kolom + baris lama/baru.
///
/// `pk_indexes` kosong berarti tabel tanpa PK usable — WHERE memakai semua
/// kolom + `LIMIT 1` (doc 07 §2, `argumentForRow:`).
#[derive(Debug, Clone)]
pub struct EditPlan {
    pub table: String,
    pub columns: Vec<ColumnMeta>,
    /// Kolom generated (Extra berisi `GENERATED`/`VIRTUAL`/`STORED`) — tidak
    /// boleh diedit (doc 07 §2, `deriveQueryString` 3085-3088).
    pub generated_columns: Vec<String>,
    pub old_row: Vec<Option<Vec<u8>>>,
    pub new_row: Vec<Option<Vec<u8>>>,
    /// Semua baris untuk delete-multi (`old_rows[row][col]`).
    pub old_rows: Vec<Vec<Option<Vec<u8>>>>,
    pub pk_indexes: Vec<usize>,
}

/// Jenis aksi edit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditKind {
    Insert,
    Update,
    Delete,
}

/// Sel yang dipakai menuliskan nilai SET: baris **baru** untuk UPDATE.
/// (WHERE identifikasi selalu memakai baris **lama** — mac `row < 0` →
/// `oldRow`, doc 07 §2.)
fn new_cell_at(plan: &EditPlan, index: usize) -> Option<&[u8]> {
    plan.new_row.get(index).and_then(Option::as_deref)
}

/// Identifikasi baris (doc 07 §2, `argumentForRow:`): per kolom PK,
/// `col IS NULL` untuk NULL, `col = <literal>` untuk lainnya; bila tak ada PK
/// usable, semua kolom dipakai + `LIMIT 1`. Selalu memakai **baris lama**,
/// karena WHERE harus mengenali baris sebelum diubah (mac `row < 0` →
/// `oldRow`).
fn row_identification(plan: &EditPlan, escape: Escaper<'_>) -> Result<String, String> {
    let sources: Vec<(usize, &ColumnMeta)> = if plan.pk_indexes.is_empty() {
        plan.columns.iter().enumerate().collect()
    } else {
        plan.pk_indexes
            .iter()
            .filter_map(|index| plan.columns.get(*index).map(|column| (*index, column)))
            .collect()
    };

    let mut parts = Vec::with_capacity(sources.len());
    for (index, column) in &sources {
        let Some(bytes) = plan.old_row.get(*index).and_then(Option::as_deref) else {
            // NULL (port memuat halaman penuh; tidak ada "not loaded") →
            // `IS NULL` (doc 07 §2, `isNSNull` → `"col IS NULL"`).
            parts.push(format!("{} IS NULL", Conn::quote_identifier(&column.name)));
            continue;
        };
        let literal = value_literal(column, bytes, escape);
        parts.push(format!(
            "{} = {}",
            Conn::quote_identifier(&column.name),
            literal
        ));
    }

    let mut out = parts.join(" AND ");
    if plan.pk_indexes.is_empty() {
        out.push_str(" LIMIT 1");
    }
    Ok(out)
}

/// Literal per sel (doc 07 §2, `deriveQueryString`): NULL → `NULL`; numeric
/// unquoted; bit → `b'…'`; blob/geometry → `X'…'`; lainnya escape session.
fn value_literal(column: &ColumnMeta, bytes: &[u8], escape: Escaper<'_>) -> String {
    if column.field_type == ffi::enum_field_types::MYSQL_TYPE_BIT {
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

/// `UPDATE <t> SET <col> = <val>, … WHERE <identifikasi>`; hanya kolom yang
/// berubah (doc 07 §2, `deriveQueryString` 3034).
pub fn update_sql(plan: &EditPlan, escape: Escaper<'_>) -> Result<String, String> {
    let where_clause = row_identification(plan, escape)?;
    if where_clause.is_empty() {
        return Err("WHERE clause is empty; row cannot be identified.".to_owned());
    }

    let mut assignments = Vec::new();
    for (index, column) in plan.columns.iter().enumerate() {
        if plan.generated_columns.contains(&column.name) {
            continue;
        }
        let old = plan.old_row.get(index).and_then(Option::as_deref);
        let new = new_cell_at(plan, index);
        if old == new {
            continue;
        }
        let literal = match new {
            None => "NULL".to_owned(),
            Some(bytes) => value_literal(column, bytes, escape),
        };
        assignments.push(format!(
            "{} = {}",
            Conn::quote_identifier(&column.name),
            literal
        ));
    }

    if assignments.is_empty() {
        return Err("No column has changed; nothing to update.".to_owned());
    }

    Ok(format!(
        "UPDATE {} SET {} WHERE {}",
        Conn::quote_identifier(&plan.table),
        assignments.join(", "),
        where_clause
    ))
}

/// `DELETE FROM <t> WHERE <identifikasi>` (doc 07 §3).
pub fn delete_sql(plan: &EditPlan, escape: Escaper<'_>) -> Result<String, String> {
    let where_clause = row_identification(plan, escape)?;
    if where_clause.is_empty() {
        return Err("WHERE clause is empty; row cannot be identified.".to_owned());
    }
    Ok(format!(
        "DELETE FROM {} WHERE {}",
        Conn::quote_identifier(&plan.table),
        where_clause
    ))
}

/// `INSERT INTO <t> (col…) VALUES (…)` untuk baris baru (doc 07 §2, 3097-3098).
/// Semua kolom non-generated ikut; auto_increment di-skip (server mengisi).
pub fn insert_sql(plan: &EditPlan, escape: Escaper<'_>) -> Result<String, String> {
    let mut columns = Vec::new();
    let mut literals = Vec::new();
    for (index, column) in plan.columns.iter().enumerate() {
        if plan.generated_columns.contains(&column.name) {
            continue;
        }
        if column.is_auto_increment() {
            continue;
        }
        columns.push(Conn::quote_identifier(&column.name));
        let literal = match plan.new_row.get(index).and_then(Option::as_deref) {
            None => "NULL".to_owned(),
            Some(bytes) => value_literal(column, bytes, escape),
        };
        literals.push(literal);
    }
    if columns.is_empty() {
        return Err("No editable columns for INSERT.".to_owned());
    }
    Ok(format!(
        "INSERT INTO {} ({}) VALUES ({})",
        Conn::quote_identifier(&plan.table),
        columns.join(", "),
        literals.join(", ")
    ))
}

/// Baris default untuk `addRow:` (doc 07 §3): nilai kosong untuk nullable,
/// `"0"` untuk numeric/bit NOT NULL, `NULL` untuk auto_increment.
pub fn default_row(columns: &[ColumnMeta]) -> Vec<Option<Vec<u8>>> {
    columns
        .iter()
        .map(|column| {
            if column.is_auto_increment() || column.is_not_null() {
                None // server mengisi; NULL untuk nullable
            } else if column.is_unquoted_numeric() {
                Some(b"0".to_vec())
            } else {
                None
            }
        })
        .collect()
}

/// Baris duplikat (doc 07 §3): salin `source`, auto_increment → NULL.
pub fn duplicate_row(columns: &[ColumnMeta], source: &[Option<Vec<u8>>]) -> Vec<Option<Vec<u8>>> {
    columns
        .iter()
        .enumerate()
        .map(|(index, column)| {
            if column.is_auto_increment() {
                return None; // mac: auto_increment → NSNull
            }
            source.get(index).cloned().unwrap_or_default()
        })
        .collect()
}

/// `DELETE FROM <t>` — menghapus seluruh baris (doc 07 §3, 2286).
pub fn delete_all_sql(table: &str) -> String {
    format!("DELETE FROM {}", Conn::quote_identifier(table))
}

/// `DELETE FROM <t> WHERE <pk> IN (v1, v2, …)` untuk PK tunggal, atau
/// `DELETE FROM <t> WHERE (a AND b) OR (c AND d)` untuk PK majemuk
/// (doc 07 §3, 2403-2497). Baris diambil dari `old_rows`.
pub fn delete_multi_sql(
    plan: &EditPlan,
    selected_rows: &[usize],
    escape: Escaper<'_>,
) -> Result<String, String> {
    if selected_rows.is_empty() {
        return Err("No rows selected for deletion.".to_owned());
    }

    // Kumpulkan literal per baris terpilih, hanya dari kolom PK (atau semua
    // kolom bila tanpa PK usable).
    let sources: Vec<usize> = if plan.pk_indexes.is_empty() {
        (0..plan.columns.len()).collect()
    } else {
        plan.pk_indexes.clone()
    };

    let mut rows: Vec<String> = Vec::with_capacity(selected_rows.len());
    for &row_index in selected_rows {
        let Some(row) = plan.old_rows.get(row_index) else {
            continue;
        };
        let mut parts = Vec::with_capacity(sources.len());
        for &col_index in &sources {
            let Some(column) = plan.columns.get(col_index) else {
                continue;
            };
            let Some(bytes) = row.get(col_index).and_then(Option::as_deref) else {
                parts.push(format!("{} IS NULL", Conn::quote_identifier(&column.name)));
                continue;
            };
            let literal = value_literal(column, bytes, escape);
            parts.push(format!(
                "{} = {}",
                Conn::quote_identifier(&column.name),
                literal
            ));
        }
        if parts.is_empty() {
            continue;
        }
        rows.push(parts.join(" AND "));
    }

    if rows.is_empty() {
        return Err("No valid rows to delete.".to_owned());
    }

    let is_multi_pk = plan.pk_indexes.len() > 1;
    let has_pk = !plan.pk_indexes.is_empty();

    if has_pk && !is_multi_pk {
        // PK tunggal: `DELETE FROM t WHERE pk IN (v1, v2, …)` (doc 07 §3, 2403).
        // Hanya nilai PK yang masuk IN, bukan kondisi lengkap.
        let pk_index = plan.pk_indexes[0];
        let values: Vec<String> = selected_rows
            .iter()
            .filter_map(|&row_index| {
                plan.old_rows
                    .get(row_index)?
                    .get(pk_index)?
                    .as_deref()
                    .map(|bytes| value_literal(&plan.columns[pk_index], bytes, escape))
            })
            .collect();
        let pk_name = plan
            .columns
            .get(pk_index)
            .map(|column| Conn::quote_identifier(&column.name))
            .unwrap_or_default();
        return Ok(format!(
            "DELETE FROM {} WHERE {pk_name} IN ({})",
            Conn::quote_identifier(&plan.table),
            values.join(", ")
        ));
    }

    // PK majemuk atau tanpa PK: kondisi lengkap per baris, join OR
    // (doc 07 §3, 2462-2497).
    let mut row_conditions: Vec<String> = Vec::with_capacity(selected_rows.len());
    for &row_index in selected_rows {
        let Some(row) = plan.old_rows.get(row_index) else {
            continue;
        };
        let mut parts = Vec::with_capacity(sources_len(plan));
        for &col_index in &sources_indexes(plan) {
            let Some(column) = plan.columns.get(col_index) else {
                continue;
            };
            let Some(bytes) = row.get(col_index).and_then(Option::as_deref) else {
                parts.push(format!("{} IS NULL", Conn::quote_identifier(&column.name)));
                continue;
            };
            let literal = value_literal(column, bytes, escape);
            parts.push(format!(
                "{} = {}",
                Conn::quote_identifier(&column.name),
                literal
            ));
        }
        if !parts.is_empty() {
            row_conditions.push(format!("({})", parts.join(" AND ")));
        }
    }

    let grouped = row_conditions.join(" OR ");
    Ok(format!(
        "DELETE FROM {} WHERE {grouped}",
        Conn::quote_identifier(&plan.table)
    ))
}

fn sources_len(plan: &EditPlan) -> usize {
    if plan.pk_indexes.is_empty() {
        plan.columns.len()
    } else {
        plan.pk_indexes.len()
    }
}

fn sources_indexes(plan: &EditPlan) -> Vec<usize> {
    if plan.pk_indexes.is_empty() {
        (0..plan.columns.len()).collect()
    } else {
        plan.pk_indexes.clone()
    }
}

/// Teks dialog konfirmasi (doc 07 §6, `SPTableContent.m:2214`).
pub const CONFIRM_DELETE_SINGLE: &str = "Are you sure you want to delete the selected row from this table? This action cannot be undone.";

/// Peringatan bila `affected_rows == 0` (mac `2906`).
pub const WARNING_NO_ROWS_AFFECTED: &str = "/* WARNING: No rows have been affected */\n";

/// Pesan generated column (mac `4865`).
pub fn generated_column_error(column: &str) -> String {
    format!("The current field \"{column}\" is a generated column and therefore cannot be edited.")
}

use super::client::Conn;

#[cfg(test)]
mod tests {
    use super::*;
    use ffi::enum_field_types as t;

    /// Escaper palsu: string → `'E(…)'`, blob → `X'…'` — fokus pada aturan.
    fn fake_escape(column: &ColumnMeta, cell: Option<&[u8]>) -> String {
        let Some(bytes) = cell else {
            return "NULL".to_owned();
        };
        if column.is_raw_binary() {
            let mut hex = String::new();
            for byte in bytes {
                hex.push_str(&format!("{byte:02X}"));
            }
            format!("X'{hex}'")
        } else {
            format!("'{}'", String::from_utf8_lossy(bytes))
        }
    }

    fn column(name: &str, ty: t, flags: u32, length: u64, charsetnr: u32) -> ColumnMeta {
        ColumnMeta {
            name: name.to_owned(),
            org_name: name.to_owned(),
            table: "orders".to_owned(),
            field_type: ty,
            flags,
            charsetnr,
            length,
            decimals: 0,
        }
    }

    const AUTO_INCREMENT_FLAG: u32 = 16384;

    fn plan(
        columns: Vec<ColumnMeta>,
        generated: &[&str],
        pk: &[usize],
        old_row: Vec<Option<Vec<u8>>>,
        new_row: Vec<Option<Vec<u8>>>,
    ) -> EditPlan {
        EditPlan {
            table: "orders".to_owned(),
            columns,
            generated_columns: generated.iter().map(|name| (*name).to_owned()).collect(),
            old_row,
            new_row,
            old_rows: vec![],
            pk_indexes: pk.to_vec(),
        }
    }

    #[test]
    fn update_includes_only_changed_columns() {
        let columns = vec![
            column("id", t::MYSQL_TYPE_LONG, AUTO_INCREMENT_FLAG, 0, 63),
            column("name", t::MYSQL_TYPE_VAR_STRING, 0, 64, 33),
        ];
        let plan = plan(
            columns,
            &[],
            &[0],
            vec![Some(b"1".to_vec()), Some(b"steel".to_vec())],
            vec![Some(b"1".to_vec()), Some(b"iron".to_vec())],
        );
        let sql = update_sql(&plan, &fake_escape).unwrap();
        assert_eq!(sql, "UPDATE `orders` SET `name` = 'iron' WHERE `id` = 1");
    }

    #[test]
    fn update_null_assignment_uses_null_literal() {
        let columns = vec![
            column("id", t::MYSQL_TYPE_LONG, AUTO_INCREMENT_FLAG, 0, 63),
            column("note", t::MYSQL_TYPE_VAR_STRING, 0, 64, 33),
        ];
        let plan = plan(
            columns,
            &[],
            &[0],
            vec![Some(b"1".to_vec()), Some(b"hello".to_vec())],
            vec![Some(b"1".to_vec()), None],
        );
        let sql = update_sql(&plan, &fake_escape).unwrap();
        assert_eq!(sql, "UPDATE `orders` SET `note` = NULL WHERE `id` = 1");
    }

    #[test]
    fn update_without_pk_uses_all_columns_and_limit_1() {
        let columns = vec![column("name", t::MYSQL_TYPE_VAR_STRING, 0, 64, 33)];
        let plan = plan(
            columns,
            &[],
            &[],
            vec![Some(b"steel".to_vec())],
            vec![Some(b"iron".to_vec())],
        );
        let sql = update_sql(&plan, &fake_escape).unwrap();
        assert_eq!(
            sql,
            "UPDATE `orders` SET `name` = 'iron' WHERE `name` = 'steel' LIMIT 1"
        );
    }

    #[test]
    fn update_skips_generated_columns() {
        let columns = vec![
            column("id", t::MYSQL_TYPE_LONG, AUTO_INCREMENT_FLAG, 0, 63),
            column("base", t::MYSQL_TYPE_VAR_STRING, 0, 64, 33),
            column("derived", t::MYSQL_TYPE_VAR_STRING, 0, 64, 33),
        ];
        let plan = plan(
            columns,
            &["derived"],
            &[0],
            vec![
                Some(b"1".to_vec()),
                Some(b"a".to_vec()),
                Some(b"a!".to_vec()),
            ],
            vec![
                Some(b"1".to_vec()),
                Some(b"b".to_vec()),
                Some(b"a!".to_vec()),
            ],
        );
        let sql = update_sql(&plan, &fake_escape).unwrap();
        assert!(!sql.contains("derived"), "generated column bocor: {sql}");
        assert_eq!(sql, "UPDATE `orders` SET `base` = 'b' WHERE `id` = 1");
    }

    #[test]
    fn update_bit_uses_b_literal() {
        let columns = vec![
            column("id", t::MYSQL_TYPE_LONG, AUTO_INCREMENT_FLAG, 0, 63),
            column("flag", t::MYSQL_TYPE_BIT, 0, 8, 63),
        ];
        let plan = plan(
            columns,
            &[],
            &[0],
            vec![Some(b"1".to_vec()), Some(vec![0b0000_0001u8])],
            vec![Some(b"1".to_vec()), Some(vec![0b0000_0101u8])],
        );
        let sql = update_sql(&plan, &fake_escape).unwrap();
        assert_eq!(
            sql,
            "UPDATE `orders` SET `flag` = b'00000101' WHERE `id` = 1"
        );
    }

    #[test]
    fn update_blob_uses_hex_escaper() {
        let columns = vec![
            column("id", t::MYSQL_TYPE_LONG, AUTO_INCREMENT_FLAG, 0, 63),
            column("data", t::MYSQL_TYPE_BLOB, 128, 32, 63),
        ];
        let plan = plan(
            columns,
            &[],
            &[0],
            vec![Some(b"1".to_vec()), Some(vec![0xDE, 0xAD])],
            vec![Some(b"1".to_vec()), Some(vec![0xBE, 0xEF])],
        );
        let sql = update_sql(&plan, &fake_escape).unwrap();
        assert_eq!(sql, "UPDATE `orders` SET `data` = X'BEEF' WHERE `id` = 1");
    }

    #[test]
    fn update_uses_is_null_for_null_pk() {
        let columns = vec![
            column("id", t::MYSQL_TYPE_LONG, AUTO_INCREMENT_FLAG, 0, 63),
            column("name", t::MYSQL_TYPE_VAR_STRING, 0, 64, 33),
        ];
        let plan = plan(
            columns,
            &[],
            &[0],
            vec![None, Some(b"steel".to_vec())],
            vec![None, Some(b"iron".to_vec())],
        );
        let sql = update_sql(&plan, &fake_escape).unwrap();
        assert_eq!(
            sql,
            "UPDATE `orders` SET `name` = 'iron' WHERE `id` IS NULL"
        );
    }

    #[test]
    fn update_without_change_is_rejected() {
        let columns = vec![column("name", t::MYSQL_TYPE_VAR_STRING, 0, 64, 33)];
        let plan = plan(
            columns,
            &[],
            &[],
            vec![Some(b"steel".to_vec())],
            vec![Some(b"steel".to_vec())],
        );
        let error = update_sql(&plan, &fake_escape).unwrap_err();
        assert!(error.contains("No column has changed"));
    }

    #[test]
    fn delete_uses_pk_and_old_row() {
        let columns = vec![
            column("id", t::MYSQL_TYPE_LONG, AUTO_INCREMENT_FLAG, 0, 63),
            column("name", t::MYSQL_TYPE_VAR_STRING, 0, 64, 33),
        ];
        let plan = plan(
            columns,
            &[],
            &[0],
            vec![Some(b"1".to_vec()), Some(b"steel".to_vec())],
            vec![Some(b"1".to_vec()), Some(b"steel".to_vec())],
        );
        let sql = delete_sql(&plan, &fake_escape).unwrap();
        assert_eq!(sql, "DELETE FROM `orders` WHERE `id` = 1");
    }

    #[test]
    fn delete_multi_pk_uses_all_pk_columns() {
        let columns = vec![
            column("a", t::MYSQL_TYPE_LONG, 0, 0, 63),
            column("b", t::MYSQL_TYPE_VAR_STRING, 0, 64, 33),
            column("c", t::MYSQL_TYPE_LONG, 0, 0, 63),
        ];
        let plan = plan(
            columns,
            &[],
            &[0, 1],
            vec![
                Some(b"1".to_vec()),
                Some(b"x".to_vec()),
                Some(b"9".to_vec()),
            ],
            vec![
                Some(b"1".to_vec()),
                Some(b"x".to_vec()),
                Some(b"9".to_vec()),
            ],
        );
        let sql = delete_sql(&plan, &fake_escape).unwrap();
        assert_eq!(sql, "DELETE FROM `orders` WHERE `a` = 1 AND `b` = 'x'");
    }
}

/// Uji hidup UPDATE/DELETE terhadap server nyata; `#[ignore]`.
/// Jalankan dengan `SA_PROBE_* cargo test --locked -- --ignored`.
#[cfg(test)]
mod live_tests {
    use super::*;
    use crate::database::client::Conn;
    use ffi::enum_field_types as t;

    const TABLE: &str = "sa_probe_edit";

    fn connection() -> Option<Conn> {
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
        Some(Conn::connect(&params).expect("connect to probe server"))
    }

    fn meta(name: &str, ty: t, charsetnr: u32, flags: u32) -> ColumnMeta {
        ColumnMeta {
            name: name.to_owned(),
            org_name: name.to_owned(),
            table: TABLE.to_owned(),
            field_type: ty,
            flags,
            charsetnr,
            length: 64,
            decimals: 0,
        }
    }

    fn escaper(conn: &Conn) -> impl Fn(&ColumnMeta, Option<&[u8]>) -> String + '_ {
        move |column, cell| {
            let Some(bytes) = cell else {
                return "NULL".to_owned();
            };
            if column.is_raw_binary() {
                conn.escape_data_hex(bytes)
            } else {
                let text = String::from_utf8_lossy(bytes);
                conn.escape_string(&text, true).unwrap_or_default()
            }
        }
    }

    #[test]
    #[ignore = "uji hidup, butuh server dan SA_PROBE_*"]
    fn update_and_delete_follow_the_session() {
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
            "CREATE TABLE sa_probe_edit (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(64))",
        )
        .expect("create probe table");
        for name in ["original", "second"] {
            let seeded = conn.escape_string(name, true).expect("escape seed");
            conn.execute(&format!(
                "INSERT INTO {} (name) VALUES ({seeded})",
                Conn::quote_identifier(TABLE)
            ))
            .expect("seed probe table");
        }

        // UPDATE
        let plan = EditPlan {
            table: TABLE.to_owned(),
            columns: vec![
                meta("id", t::MYSQL_TYPE_LONG, 63, 16384),
                meta("name", t::MYSQL_TYPE_VAR_STRING, 33, 0),
            ],
            generated_columns: vec![],
            old_row: vec![Some(b"1".to_vec()), Some(b"original".to_vec())],
            new_row: vec![Some(b"1".to_vec()), Some(b"renamed".to_vec())],
            old_rows: vec![],
            pk_indexes: vec![0],
        };
        let sql = update_sql(&plan, &escaper(&conn)).expect("update sql");
        assert_eq!(
            sql,
            "UPDATE `sa_probe_edit` SET `name` = 'renamed' WHERE `id` = 1"
        );
        conn.execute(&sql).expect("run update");
        let set = conn
            .query("SELECT name FROM sa_probe_edit WHERE id = 1")
            .expect("read back after update");
        assert_eq!(set.text_at(0, 0), "renamed");

        // DELETE
        let plan = EditPlan {
            table: TABLE.to_owned(),
            columns: vec![
                meta("id", t::MYSQL_TYPE_LONG, 63, 16384),
                meta("name", t::MYSQL_TYPE_VAR_STRING, 33, 0),
            ],
            generated_columns: vec![],
            old_row: vec![Some(b"1".to_vec()), Some(b"renamed".to_vec())],
            new_row: vec![Some(b"1".to_vec()), Some(b"renamed".to_vec())],
            old_rows: vec![],
            pk_indexes: vec![0],
        };
        let sql = delete_sql(&plan, &escaper(&conn)).expect("delete sql");
        assert_eq!(sql, "DELETE FROM `sa_probe_edit` WHERE `id` = 1");
        conn.execute(&sql).expect("run delete");
        let set = conn
            .query("SELECT COUNT(1) FROM sa_probe_edit")
            .expect("read back after delete");
        assert_eq!(set.text_at(0, 0), "1");

        conn.execute(&format!("DROP DATABASE IF EXISTS {TABLE}_db"))
            .ok();
    }
}

#[cfg(test)]
mod insert_tests {
    use super::*;
    use ffi::enum_field_types as t;

    const BINARY_FLAG: u32 = 128;
    const AUTO_INCREMENT_FLAG: u32 = 16384;

    fn fake_escape(column: &ColumnMeta, cell: Option<&[u8]>) -> String {
        let Some(bytes) = cell else {
            return "NULL".to_owned();
        };
        if column.is_raw_binary() {
            let mut hex = String::new();
            for byte in bytes {
                hex.push_str(&format!("{byte:02X}"));
            }
            format!("X'{hex}'")
        } else {
            format!("'{}'", String::from_utf8_lossy(bytes))
        }
    }

    fn column(name: &str, ty: t, flags: u32, charsetnr: u32) -> ColumnMeta {
        ColumnMeta {
            name: name.to_owned(),
            org_name: name.to_owned(),
            table: "orders".to_owned(),
            field_type: ty,
            flags,
            charsetnr,
            length: 64,
            decimals: 0,
        }
    }

    fn plan(
        columns: Vec<ColumnMeta>,
        generated: &[&str],
        new_row: Vec<Option<Vec<u8>>>,
    ) -> EditPlan {
        EditPlan {
            table: "orders".to_owned(),
            columns,
            generated_columns: generated.iter().map(|name| (*name).to_owned()).collect(),
            old_row: vec![],
            new_row,
            old_rows: vec![],
            pk_indexes: vec![],
        }
    }

    #[test]
    fn insert_skips_auto_increment() {
        let columns = vec![
            column("id", t::MYSQL_TYPE_LONG, AUTO_INCREMENT_FLAG, 63),
            column("name", t::MYSQL_TYPE_VAR_STRING, 0, 33),
        ];
        let plan = plan(columns, &[], vec![None, Some(b"steel".to_vec())]);
        let sql = insert_sql(&plan, &fake_escape).unwrap();
        assert_eq!(sql, "INSERT INTO `orders` (`name`) VALUES ('steel')");
    }

    #[test]
    fn insert_uses_null_for_none() {
        let columns = vec![
            column("name", t::MYSQL_TYPE_VAR_STRING, 0, 33),
            column("note", t::MYSQL_TYPE_VAR_STRING, 0, 33),
        ];
        let plan = plan(columns, &[], vec![Some(b"a".to_vec()), None]);
        let sql = insert_sql(&plan, &fake_escape).unwrap();
        assert_eq!(
            sql,
            "INSERT INTO `orders` (`name`, `note`) VALUES ('a', NULL)"
        );
    }

    #[test]
    fn insert_numeric_unquoted() {
        let columns = vec![
            column("qty", t::MYSQL_TYPE_LONG, 0, 63),
            column("price", t::MYSQL_TYPE_DOUBLE, 0, 63),
        ];
        let plan = plan(
            columns,
            &[],
            vec![Some(b"5".to_vec()), Some(b"19.99".to_vec())],
        );
        let sql = insert_sql(&plan, &fake_escape).unwrap();
        assert_eq!(
            sql,
            "INSERT INTO `orders` (`qty`, `price`) VALUES (5, 19.99)"
        );
    }

    #[test]
    fn insert_blob_hex() {
        let columns = vec![
            column("name", t::MYSQL_TYPE_VAR_STRING, 0, 33),
            column("data", t::MYSQL_TYPE_BLOB, BINARY_FLAG, 63),
        ];
        let plan = plan(
            columns,
            &[],
            vec![Some(b"x".to_vec()), Some(vec![0xDE, 0xAD])],
        );
        let sql = insert_sql(&plan, &fake_escape).unwrap();
        assert_eq!(
            sql,
            "INSERT INTO `orders` (`name`, `data`) VALUES ('x', X'DEAD')"
        );
    }

    #[test]
    fn insert_skips_generated() {
        let columns = vec![
            column("name", t::MYSQL_TYPE_VAR_STRING, 0, 33),
            column("computed", t::MYSQL_TYPE_VAR_STRING, 0, 33),
        ];
        let plan = plan(
            columns,
            &["computed"],
            vec![Some(b"a".to_vec()), Some(b"ignored".to_vec())],
        );
        let sql = insert_sql(&plan, &fake_escape).unwrap();
        assert_eq!(sql, "INSERT INTO `orders` (`name`) VALUES ('a')");
    }

    #[test]
    fn default_row_numeric_gets_zero_nullable_gets_null() {
        let columns = vec![
            column("id", t::MYSQL_TYPE_LONG, AUTO_INCREMENT_FLAG, 63),
            column("qty", t::MYSQL_TYPE_LONG, 0, 63),
            column("name", t::MYSQL_TYPE_VAR_STRING, 0, 33),
        ];
        let row = default_row(&columns);
        assert_eq!(row[0], None); // auto_increment → NULL
        assert_eq!(row[1].as_deref(), Some(b"0".as_slice())); // numeric → "0"
        assert_eq!(row[2], None); // nullable string → NULL
    }

    #[test]
    fn duplicate_row_clears_auto_increment() {
        let columns = vec![
            column("id", t::MYSQL_TYPE_LONG, AUTO_INCREMENT_FLAG, 63),
            column("name", t::MYSQL_TYPE_VAR_STRING, 0, 33),
        ];
        let source = vec![Some(b"1".to_vec()), Some(b"steel".to_vec())];
        let row = duplicate_row(&columns, &source);
        assert_eq!(row[0], None); // auto_increment → NULL
        assert_eq!(row[1].as_deref(), Some(b"steel".as_slice())); // sisanya disalin
    }
}

#[cfg(test)]
mod delete_multi_tests {
    use super::*;
    use ffi::enum_field_types as t;

    const AUTO_INCREMENT_FLAG: u32 = 16384;
    fn fake_escape(_column: &ColumnMeta, cell: Option<&[u8]>) -> String {
        let Some(bytes) = cell else {
            return "NULL".to_owned();
        };
        String::from_utf8_lossy(bytes).into_owned()
    }

    fn column(name: &str, ty: t, flags: u32) -> ColumnMeta {
        ColumnMeta {
            name: name.to_owned(),
            org_name: name.to_owned(),
            table: "orders".to_owned(),
            field_type: ty,
            flags,
            charsetnr: 63,
            length: 64,
            decimals: 0,
        }
    }

    fn plan(
        columns: Vec<ColumnMeta>,
        pk: &[usize],
        old_rows: Vec<Vec<Option<Vec<u8>>>>,
    ) -> EditPlan {
        EditPlan {
            table: "orders".to_owned(),
            columns,
            generated_columns: vec![],
            old_row: vec![],
            new_row: vec![],
            old_rows,
            pk_indexes: pk.to_vec(),
        }
    }

    #[test]
    fn delete_all_returns_simple_sql() {
        let sql = delete_all_sql("orders");
        assert_eq!(sql, "DELETE FROM `orders`");
    }

    #[test]
    fn delete_multi_single_pk_uses_in_clause() {
        let columns = vec![
            column("id", t::MYSQL_TYPE_LONG, AUTO_INCREMENT_FLAG),
            column("name", t::MYSQL_TYPE_VAR_STRING, 0),
        ];
        let old_rows = vec![
            vec![Some(b"1".to_vec()), Some(b"steel".to_vec())],
            vec![Some(b"2".to_vec()), Some(b"iron".to_vec())],
            vec![Some(b"3".to_vec()), Some(b"copper".to_vec())],
        ];
        let plan = plan(columns, &[0], old_rows);
        let selected = vec![0, 2];
        let sql = delete_multi_sql(&plan, &selected, &fake_escape).unwrap();
        // PK tunggal: mac memakai `IN` (doc 07 §3, 2403), bukan grup kolom.
        assert_eq!(sql, "DELETE FROM `orders` WHERE `id` IN (1, 3)");
    }

    #[test]
    fn delete_multi_without_pk_uses_grouped_conditions() {
        let columns = vec![column("name", t::MYSQL_TYPE_VAR_STRING, 0)];
        let old_rows = vec![vec![Some(b"steel".to_vec())]];
        let plan = plan(columns, &[], old_rows);
        let selected = vec![0];
        let sql = delete_multi_sql(&plan, &selected, &fake_escape).unwrap();
        // Tanpa PK: kondisi lengkap per baris, join OR (doc 07 §3, 2462).
        // Tanpa LIMIT 1 karena multi-row delete.
        assert_eq!(sql, "DELETE FROM `orders` WHERE (`name` = steel)");
    }

    #[test]
    fn delete_multi_empty_selection_is_error() {
        let columns = vec![column("id", t::MYSQL_TYPE_LONG, 0)];
        let plan = plan(columns, &[0], vec![]);
        let error = delete_multi_sql(&plan, &[], &fake_escape).unwrap_err();
        assert!(error.contains("No rows selected"));
    }
}
