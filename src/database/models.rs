// Sequel Ace's enabled-by-default LimitResultsValue in PreferenceDefaults.plist.
pub const DEFAULT_CONTENT_PAGE_SIZE: usize = 1000;

use super::client::ColumnMeta;
use super::edit::{EditKind, EditPlan};
use super::filter::FilterSpec;

#[derive(Clone)]
pub struct ConnectionOptions {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub initial_database: Option<String>,
    pub time_zone: Option<String>,
    pub allow_local_infile: bool,
    pub enable_cleartext_plugin: bool,
    pub request_server_public_key: bool,
    pub require_tls: bool,
    #[allow(dead_code)] // dibaca client.rs ConnectParams, bukan di models
    pub socket_path: Option<String>,
    // SSH tunnel (doc 03, SSH tab)
    pub ssh_host: Option<String>,
    pub ssh_port: Option<u16>,
    pub ssh_user: Option<String>,
    #[allow(dead_code)] // dipakai nanti untuk SSH_ASKPASS (mac: SequeLAceTunnelAssistant)
    pub ssh_password: Option<String>,
    pub ssh_key_path: Option<String>,
}

impl ConnectionOptions {
    pub fn with_port(
        host: &str,
        port: &str,
        user: &str,
        password: &str,
        initial_database: &str,
    ) -> Result<Self, String> {
        let port = if port.trim().is_empty() {
            3306
        } else {
            port.trim()
                .parse::<u16>()
                .ok()
                .filter(|port| *port > 0)
                .ok_or_else(|| "Enter a port between 1 and 65535.".to_owned())?
        };
        let host = if host.trim().is_empty() {
            "127.0.0.1"
        } else {
            host.trim()
        };
        Ok(Self {
            host: host.to_owned(),
            port,
            user: user.to_owned(),
            password: password.to_owned(),
            initial_database: nonempty_trimmed(initial_database),
            time_zone: None,
            allow_local_infile: false,
            enable_cleartext_plugin: false,
            request_server_public_key: false,
            require_tls: false,
            socket_path: None,
            ssh_host: None,
            ssh_port: None,
            ssh_user: None,
            ssh_password: None,
            ssh_key_path: None,
        })
    }
}

fn nonempty_trimmed(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Table,
    View,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableEntry {
    pub name: String,
    pub kind: EntryKind,
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: String,
    pub key: String,
    pub default_value: Option<String>,
    pub extra: String,
}

#[derive(Debug, Clone)]
pub struct ObjectDetails {
    pub columns: Vec<ColumnInfo>,
    pub create_sql: String,
    pub table_info: Option<Result<TableInfo, String>>,
    pub indexes: Option<Result<Vec<IndexInfo>, String>>,
    pub foreign_keys: Option<Result<Vec<ForeignKeyInfo>, String>>,
}

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub engine: String,
    pub estimated_rows: String,
    pub data_length: String,
    pub index_length: String,
    pub collation: String,
    pub comment: String,
}

#[derive(Debug, Clone)]
pub struct IndexInfo {
    pub name: String,
    pub column: String,
    pub sequence: String,
    pub unique: bool,
    pub index_type: String,
}

#[derive(Debug, Clone)]
pub struct ForeignKeyInfo {
    pub name: String,
    pub column: String,
    pub referenced_schema: String,
    pub referenced_table: String,
    pub referenced_column: String,
}

#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub version: String,
    pub variables: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct QueryResultSet {
    pub columns: Vec<ColumnMeta>,
    pub rows: Vec<Vec<Option<Vec<u8>>>>,
    pub affected_rows: u64,
    pub truncated: bool,
}

#[derive(Debug, Clone)]
pub struct QueryOutput {
    pub sets: Vec<QueryResultSet>,
    pub sets_truncated: bool,
    pub elapsed_ms: u128,
}

#[derive(Debug, Clone)]
pub struct ContentPage {
    pub columns: Vec<ColumnMeta>,
    pub rows: Vec<Vec<Option<Vec<u8>>>>,
    pub has_next: bool,
}

#[derive(Debug, Clone)]
pub struct CopySnapshot {
    pub table: Option<String>,
    pub columns: Vec<ColumnMeta>,
    pub rows: Vec<Vec<Option<Vec<u8>>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyKind {
    Tab,
    TabWithHeaders,
    Csv,
    SqlInsert,
    SqlInsertNoAutoInc,
}

impl TableEntry {
    pub(super) fn from_full_tables_row(name: String, table_type: String) -> Self {
        let kind = if table_type.eq_ignore_ascii_case("VIEW") {
            EntryKind::View
        } else {
            EntryKind::Table
        };
        Self { name, kind }
    }
}

pub enum Request {
    Connect(ConnectionOptions),
    RefreshDatabases,
    SelectDatabase(String),
    RefreshTables,
    LoadObject(TableEntry),
    LoadContent {
        entry: TableEntry,
        page: u64,
    },
    SetContentFilter {
        entry: TableEntry,
        filter: Option<FilterSpec>,
    },
    LoadServerInfo,
    ExecuteSql(String),
    CopyRows(CopySnapshot, CopyKind),
    ExecuteEdit(EditPlan, EditKind),
    DeleteAllRows {
        table: String,
    },
    DeleteMultiRows {
        plan: EditPlan,
        rows: Vec<usize>,
    },
    Disconnect,
}

pub enum Response {
    Connected {
        databases: Vec<String>,
        warning: Option<String>,
        selected_database: Option<String>,
        tables: Vec<TableEntry>,
        selection_error: Option<String>,
    },
    Databases(Vec<String>),
    Tables {
        database: String,
        entries: Vec<TableEntry>,
    },
    SelectionFailed {
        database: String,
        error: String,
        selected: bool,
    },
    ObjectDetails {
        database: String,
        entry: TableEntry,
        details: Result<ObjectDetails, String>,
    },
    ContentPage {
        database: String,
        entry: TableEntry,
        result: Result<(u64, ContentPage), String>,
    },
    ServerInfo(Result<ServerInfo, String>),
    QueryExecuted {
        output: QueryOutput,
        database: Option<String>,
        database_changed: bool,
        tables: Option<Vec<TableEntry>>,
        warning: Option<String>,
    },
    CopyResult(String),
    EditApplied {
        affected_rows: u64,
        sql: String,
    },
    Disconnected,
    Failed(String),
}
