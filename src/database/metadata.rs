use super::client::Conn;
use super::models::{
    ColumnInfo, EntryKind, ForeignKeyInfo, IndexInfo, ObjectDetails, ServerInfo, TableEntry,
    TableInfo,
};

/// `mySQLBacktickQuotedString` (`SPMySQLStringAdditions.m:41-54`).
pub fn quote_identifier(identifier: &str) -> String {
    Conn::quote_identifier(identifier)
}

pub(super) fn list_databases(conn: &mut Conn) -> Result<Vec<String>, String> {
    let result = conn.query("SHOW DATABASES").map_err(|e| e.to_string())?;
    Ok((0..result.row_count())
        .map(|row| result.text_at(row, 0))
        .collect())
}

pub(super) fn current_database(conn: &mut Conn) -> Result<Option<String>, String> {
    let result = conn.query("SELECT DATABASE()").map_err(|e| e.to_string())?;
    Ok(result
        .cell_at(0, 0)
        .map(|bytes| String::from_utf8_lossy(bytes).into_owned()))
}

pub(super) fn load_tables(conn: &mut Conn) -> Result<Vec<TableEntry>, String> {
    let result = conn.query("SHOW FULL TABLES").map_err(|e| e.to_string())?;
    // Nama kolom pertama dinamis (`Tables_in_<db>`), jadi dibaca posisional.
    Ok((0..result.row_count())
        .map(|row| TableEntry::from_full_tables_row(result.text_at(row, 0), result.text_at(row, 1)))
        .collect())
}

pub(super) fn load_object_details(
    conn: &mut Conn,
    database: &str,
    entry: &TableEntry,
) -> Result<ObjectDetails, String> {
    // `SHOW FULL COLUMNS FROM tbl FROM db` — bentuk yang dipakai macOS app
    // (doc 13 §2, `SPDatabaseStructure.m:318`).
    let columns_result = conn
        .query(&format!(
            "SHOW FULL COLUMNS FROM {} FROM {}",
            quote_identifier(&entry.name),
            quote_identifier(database)
        ))
        .map_err(|e| e.to_string())?;
    let columns = (0..columns_result.row_count())
        .map(|row| ColumnInfo {
            name: columns_result.cell_text(row, "Field"),
            data_type: columns_result.cell_text(row, "Type"),
            nullable: columns_result.cell_text(row, "Null"),
            key: columns_result.cell_text(row, "Key"),
            default_value: columns_result
                .cell(row, "Default")
                .map(|bytes| String::from_utf8_lossy(bytes).into_owned()),
            extra: columns_result.cell_text(row, "Extra"),
        })
        .collect();

    let kind = match entry.kind {
        EntryKind::Table => "TABLE",
        EntryKind::View => "VIEW",
    };
    let create_result = conn
        .query(&format!(
            "SHOW CREATE {kind} {}",
            quote_identifier(&entry.name)
        ))
        .map_err(|e| e.to_string())?;
    let create_sql = create_result
        .cell_at(0, 1)
        .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
        .ok_or_else(|| "Server returned no CREATE statement.".to_owned())?;

    let (table_info, indexes, foreign_keys) = if entry.kind == EntryKind::Table {
        (
            Some(load_table_info(conn, database, &entry.name)),
            Some(load_indexes(conn, &entry.name)),
            Some(load_foreign_keys(conn, database, &entry.name)),
        )
    } else {
        (None, None, None)
    };

    Ok(ObjectDetails {
        columns,
        create_sql,
        table_info,
        indexes,
        foreign_keys,
    })
}

fn load_table_info(conn: &mut Conn, database: &str, table: &str) -> Result<TableInfo, String> {
    // macOS app membaca information_schema, bukan SHOW TABLE STATUS
    // (`SPTableInfo.m:132-349` menerima baris statusnya).
    let sql = format!(
        "SELECT ENGINE, TABLE_ROWS, DATA_LENGTH, INDEX_LENGTH, TABLE_COLLATION, TABLE_COMMENT \
         FROM information_schema.TABLES WHERE TABLE_SCHEMA = {} AND TABLE_NAME = {}",
        conn.quote_literal(database).map_err(|e| e.to_string())?,
        conn.quote_literal(table).map_err(|e| e.to_string())?
    );
    let result = conn.query(&sql).map_err(|e| e.to_string())?;
    if result.row_count() == 0 {
        return Err("Server returned no table information.".to_owned());
    }
    Ok(TableInfo {
        engine: result.cell_text(0, "ENGINE"),
        estimated_rows: result.cell_text(0, "TABLE_ROWS"),
        data_length: result.cell_text(0, "DATA_LENGTH"),
        index_length: result.cell_text(0, "INDEX_LENGTH"),
        collation: result.cell_text(0, "TABLE_COLLATION"),
        comment: result.cell_text(0, "TABLE_COMMENT"),
    })
}

fn load_indexes(conn: &mut Conn, table: &str) -> Result<Vec<IndexInfo>, String> {
    let result = conn
        .query(&format!("SHOW INDEX FROM {}", quote_identifier(table)))
        .map_err(|e| e.to_string())?;
    Ok((0..result.row_count())
        .map(|row| IndexInfo {
            name: result.cell_text(row, "Key_name"),
            column: result.cell_text(row, "Column_name"),
            sequence: result.cell_text(row, "Seq_in_index"),
            unique: result.cell_text(row, "Non_unique") == "0",
            index_type: result.cell_text(row, "Index_type"),
        })
        .collect())
}

fn load_foreign_keys(
    conn: &mut Conn,
    database: &str,
    table: &str,
) -> Result<Vec<ForeignKeyInfo>, String> {
    let sql = format!(
        "SELECT CONSTRAINT_NAME, COLUMN_NAME, REFERENCED_TABLE_SCHEMA, \
         REFERENCED_TABLE_NAME, REFERENCED_COLUMN_NAME \
         FROM information_schema.KEY_COLUMN_USAGE \
         WHERE TABLE_SCHEMA = {} AND TABLE_NAME = {} AND REFERENCED_TABLE_NAME IS NOT NULL \
         ORDER BY CONSTRAINT_NAME, ORDINAL_POSITION",
        conn.quote_literal(database).map_err(|e| e.to_string())?,
        conn.quote_literal(table).map_err(|e| e.to_string())?
    );
    let result = conn.query(&sql).map_err(|e| e.to_string())?;
    Ok((0..result.row_count())
        .map(|row| ForeignKeyInfo {
            name: result.cell_text(row, "CONSTRAINT_NAME"),
            column: result.cell_text(row, "COLUMN_NAME"),
            referenced_schema: result.cell_text(row, "REFERENCED_TABLE_SCHEMA"),
            referenced_table: result.cell_text(row, "REFERENCED_TABLE_NAME"),
            referenced_column: result.cell_text(row, "REFERENCED_COLUMN_NAME"),
        })
        .collect())
}

pub(super) fn primary_key_indexes(
    conn: &mut Conn,
    database: &str,
    table: &str,
) -> Result<Vec<usize>, String> {
    let result = conn
        .query(&format!(
            "SHOW COLUMNS FROM {} FROM {}",
            quote_identifier(table),
            quote_identifier(database)
        ))
        .map_err(|e| e.to_string())?;
    Ok((0..result.row_count())
        .filter(|&row| result.cell_text(row, "Key") == "PRI")
        .collect())
}

pub(super) fn load_server_info(conn: &mut Conn) -> Result<ServerInfo, String> {
    let version_result = conn.query("SELECT VERSION()").map_err(|e| e.to_string())?;
    let version = version_result.text_at(0, 0);
    let variables_result = conn.query("SHOW VARIABLES").map_err(|e| e.to_string())?;
    let variables = (0..variables_result.row_count())
        .map(|row| {
            (
                variables_result.text_at(row, 0),
                variables_result.text_at(row, 1),
            )
        })
        .collect();
    Ok(ServerInfo { version, variables })
}

const SYSTEM_DATABASES: &[&str] = &["mysql", "information_schema", "performance_schema", "sys"];

pub fn partition_databases(databases: &[String]) -> (Vec<&str>, Vec<&str>) {
    let mut system = Vec::new();
    let mut user = Vec::new();
    for name in databases {
        if SYSTEM_DATABASES.contains(&name.as_str()) {
            system.push(name.as_str());
        } else {
            user.push(name.as_str());
        }
    }
    (system, user)
}
