pub mod bundle;
mod client;
mod content;
mod copy;
mod edit;
mod filter;
#[allow(dead_code)]
pub mod keychain;
mod metadata;
mod models;
mod query;
mod ssh_tunnel;
pub mod table_structure;
pub mod user_manager;
mod worker;

pub use client::{ColumnMeta, MAX_CELL_BYTES, format_cell};
pub use edit::{
    CONFIRM_DELETE_SINGLE, EditKind, EditPlan, WARNING_NO_ROWS_AFFECTED, default_row,
    duplicate_row, generated_column_error,
};
pub use filter::{FilterSpec, menu_entries};
pub use metadata::quote_identifier;

pub use metadata::partition_databases;
pub use models::{
    ConnectionOptions, ContentPage, CopyKind, CopySnapshot, EntryKind, ObjectDetails, QueryOutput,
    Request, Response, ServerInfo, TableEntry,
};
pub use ssh_tunnel::detect_ssh_agent;

pub use worker::DatabaseWorker;

#[cfg(test)]
mod tests {
    use super::{ConnectionOptions, EntryKind, TableEntry, partition_databases};

    #[test]
    fn connection_defaults_match_mac_connection_service() {
        let options = ConnectionOptions::with_port(" ", "", "root", "", " my_db ").unwrap();
        assert_eq!(options.host, "127.0.0.1");
        assert_eq!(options.port, 3306);
        assert_eq!(options.initial_database.as_deref(), Some("my_db"));
        assert!(ConnectionOptions::with_port("localhost", "0", "root", "", "").is_err());
        assert!(ConnectionOptions::with_port("localhost", "65536", "root", "", "").is_err());
    }

    #[test]
    fn database_partition_preserves_server_order_and_exact_system_names() {
        let databases = ["shop", "mysql", "Sys", "sys", "inventory"].map(str::to_owned);
        let (system, user) = partition_databases(&databases);
        assert_eq!(system, ["mysql", "sys"]);
        assert_eq!(user, ["shop", "Sys", "inventory"]);
    }

    #[test]
    fn full_tables_marks_views_without_changing_names() {
        let view = TableEntry::from_full_tables_row("a view".into(), "VIEW".into());
        let table = TableEntry::from_full_tables_row("a table".into(), "BASE TABLE".into());
        assert_eq!(view.kind, EntryKind::View);
        assert_eq!(table.kind, EntryKind::Table);
        assert_eq!(view.name, "a view");
    }

    #[test]
    fn object_names_are_quoted_as_mysql_identifiers() {
        assert_eq!(super::metadata::quote_identifier("a`b"), "`a``b`");
        assert_eq!(super::metadata::quote_identifier("one.two"), "`one.two`");
    }
}

#[cfg(test)]
mod copy_tests {
    use super::*;
    use crate::database::client::ColumnMeta;
    use crate::database::copy as copy_rules;
    use mysqlclient_sys as ffi;

    fn column(name: &str, ty: ffi::enum_field_types, flags: u32) -> ColumnMeta {
        ColumnMeta {
            name: name.to_owned(),
            org_name: name.to_owned(),
            table: "orders".to_owned(),
            field_type: ty,
            flags,
            charsetnr: 63,
            length: 8,
            decimals: 0,
        }
    }

    const AUTO_INCREMENT_FLAG: u32 = 16384;

    /// Escaper palsu yang menandai pilihan aturan: string → `'Q(…)'` (dengan
    /// quote seperti `escapeAndQuoteString:`), blob/geometry → `X'<hex>'`;
    /// jadi test fokus pada aturan, bukan escaping nyata.
    fn fake_escape(column: &ColumnMeta, cell: Option<&[u8]>) -> String {
        let Some(bytes) = cell else {
            return "NULL".to_owned();
        };
        if column.is_blob_family() {
            let mut hex = String::new();
            for byte in bytes {
                hex.push_str(&format!("{byte:02X}"));
            }
            format!("X'{hex}'")
        } else {
            format!("'{}'", String::from_utf8_lossy(bytes))
        }
    }

    fn snapshot(columns: Vec<ColumnMeta>, rows: Vec<Vec<Option<Vec<u8>>>>) -> CopySnapshot {
        CopySnapshot {
            table: Some("orders".to_owned()),
            columns,
            rows,
        }
    }

    #[test]
    fn tab_copy_uses_null_string_and_keeps_visible_text() {
        let columns = vec![column(
            "name",
            ffi::enum_field_types::MYSQL_TYPE_VAR_STRING,
            0,
        )];
        let rows = vec![vec![Some(b"steel".to_vec())], vec![None]];
        let out = copy_rules::render(&snapshot(columns, rows), CopyKind::Tab, &fake_escape);
        assert_eq!(out, "steel\nNULL\n");
    }

    #[test]
    fn tab_with_headers_adds_column_names_first() {
        let columns = vec![
            column("name", ffi::enum_field_types::MYSQL_TYPE_VAR_STRING, 0),
            column("qty", ffi::enum_field_types::MYSQL_TYPE_LONG, 0),
        ];
        let rows = vec![vec![Some(b"steel".to_vec()), Some(b"2".to_vec())]];
        let out = copy_rules::render(
            &snapshot(columns, rows),
            CopyKind::TabWithHeaders,
            &fake_escape,
        );
        assert_eq!(out, "name\tqty\nsteel\t2\n");
    }

    #[test]
    fn csv_doubles_header_quotes_but_not_value_quotes() {
        // Doc 07 §4: header `"` internal → `""`; nilai dibungkus quote TANPA
        // escape internal.
        let columns = vec![
            column("a\"b", ffi::enum_field_types::MYSQL_TYPE_VAR_STRING, 0),
            column("qty", ffi::enum_field_types::MYSQL_TYPE_LONG, 0),
        ];
        let rows = vec![vec![Some(b"say \"hi\"".to_vec()), None]];
        let out = copy_rules::render(&snapshot(columns, rows), CopyKind::Csv, &fake_escape);
        // Nilai dibungkus quote TANPA escape internal (doc 07 §4); hanya header
        // yang di-double `""`.
        assert_eq!(out, "\"a\"\"b\",\"qty\"\r\n\"say \"hi\"\",\"NULL\"\r\n");
    }

    #[test]
    fn sql_insert_uses_unquoted_numeric_and_quoted_text() {
        let columns = vec![
            column("id", ffi::enum_field_types::MYSQL_TYPE_LONG, 0),
            column("name", ffi::enum_field_types::MYSQL_TYPE_VAR_STRING, 0),
        ];
        let rows = vec![vec![Some(b"7".to_vec()), Some(b"steel".to_vec())]];
        let out = copy_rules::render(&snapshot(columns, rows), CopyKind::SqlInsert, &fake_escape);
        assert_eq!(
            out,
            "INSERT INTO `orders` (`id`, `name`)\nVALUES\n\t(7, 'steel');\n"
        );
    }

    #[test]
    fn sql_insert_bit_uses_b_literal() {
        let columns = vec![column("flag", ffi::enum_field_types::MYSQL_TYPE_BIT, 0)];
        let rows = vec![vec![Some(vec![0b0101_0101u8])]];
        let out = copy_rules::render(&snapshot(columns, rows), CopyKind::SqlInsert, &fake_escape);
        assert_eq!(
            out,
            "INSERT INTO `orders` (`flag`)\nVALUES\n\t(b'01010101');\n"
        );
    }

    #[test]
    fn sql_insert_blob_goes_to_hex_escaper() {
        let columns = vec![column("data", ffi::enum_field_types::MYSQL_TYPE_BLOB, 0)];
        let rows = vec![vec![Some(vec![0xDE, 0xAD])]];
        let out = copy_rules::render(&snapshot(columns, rows), CopyKind::SqlInsert, &fake_escape);
        assert_eq!(out, "INSERT INTO `orders` (`data`)\nVALUES\n\t(X'DEAD');\n");
    }

    #[test]
    fn sql_insert_no_auto_inc_skips_auto_increment_column() {
        let columns = vec![
            column(
                "id",
                ffi::enum_field_types::MYSQL_TYPE_LONG,
                AUTO_INCREMENT_FLAG,
            ),
            column("name", ffi::enum_field_types::MYSQL_TYPE_VAR_STRING, 0),
        ];
        let rows = vec![vec![Some(b"7".to_vec()), Some(b"steel".to_vec())]];
        let out = copy_rules::render(
            &snapshot(columns, rows),
            CopyKind::SqlInsertNoAutoInc,
            &fake_escape,
        );
        assert_eq!(out, "INSERT INTO `orders` (`name`)\nVALUES\n\t('steel');\n");
    }

    #[test]
    fn sql_insert_without_table_context_is_empty() {
        let mut snapshot = snapshot(
            vec![column("id", ffi::enum_field_types::MYSQL_TYPE_LONG, 0)],
            vec![vec![Some(b"7".to_vec())]],
        );
        snapshot.table = None;
        assert_eq!(
            copy_rules::render(&snapshot, CopyKind::SqlInsert, &fake_escape),
            ""
        );
    }
}
