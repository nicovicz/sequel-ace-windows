//! Wrapper tipis di atas `libmysqlclient` (via `mysqlclient-sys`).
//!
//! Ini menggantikan crate `mysql` (implementasi ulang protokol di Rust) dengan
//! library C yang sama persis seperti yang dibungkus macOS app. Kontraknya
//! meniru `Frameworks/SPMySQLFramework/Source/SPMySQLConnection.*` supaya
//! perilaku wire, auth, TLS, escaping, dan klasifikasi error ikut sama.
//!
//! Rujukan (doc `13-mysql-framework.md`):
//! - siklus koneksi & opsi handshake: §2
//! - eksekusi query + klasifikasi error: §3
//! - konversi tipe/escaping: §4
//!
//! Catatan portabilitas: `mysqlclient-sys` 0.5 hanya mendukung MySQL 5.7–9.5 dan
//! MariaDB 3.1/3.3/3.4. Target parity adalah MySQL **8.4.11** (bundled client
//! macOS app, `MySQL Client Libraries/include/mysql_version.h:12,24`).

// Permukaan wrapper ini sengaja memetakan seluruh API SPMySQLConnection yang
// relevan; sebagian belum dipanggil oleh UI port (mis. ping/keep-alive, deteksi
// SSL, thread id). `dead_code` dimatikan per-modul agar sinyal itu tidak jadi
// bising, bukan untuk menyembunyikan kode mati di luar wrapper.
#![allow(dead_code)]

use mysqlclient_sys as ffi;
use std::ffi::{CStr, CString};
use std::fmt;
use std::os::raw::{c_char, c_uint, c_ulong, c_void};
use std::ptr::{self, NonNull};
use std::sync::Once;

/// Flag klien default macOS app: compression + interactive + multi-results.
/// `SPMySQLConstants.h:79-81` (`SPMySQLClientFlagCompression`=32,
/// `...Interactive`=1024, `...MultiResults`=1<<17). Binding tidak mengekspor
/// konstanta ini, jadi nilainya dikutip langsung dari sumber.
const CLIENT_COMPRESS: c_ulong = 32;
const CLIENT_INTERACTIVE: c_ulong = 1024;
const CLIENT_MULTI_RESULTS: c_ulong = 1 << 17;

/// Charset transport default (`SPMySQLConnection.m:382-383`: utf8mb4).
const DEFAULT_CHARSET: &str = "utf8mb4";

/// Plugin auth yang dipaksa macOS app (`SPMySQLConnection.m:929`, issue #141).
const FORCED_DEFAULT_AUTH: &str = "mysql_native_password";

/// `BINARY_FLAG` dari `mysql.h`; menentukan string vs blob (doc 13 §4).
const BINARY_FLAG: u32 = 128;

/// Flag `NOT_NULL_FLAG`, dipakai untuk metadata kolom.
const NOT_NULL_FLAG: u32 = 1;

/// `PRI_KEY_FLAG`.
const PRI_KEY_FLAG: u32 = 2;

/// Batas tampilan satu sel di grid. Ini pilihan port (bukan perilaku macOS
/// app), dipakai bersama oleh content view dan SQL editor.
pub const MAX_CELL_BYTES: usize = 8192;

static LIBRARY_INIT: Once = Once::new();

/// Kesalahan ber-kode dari klien: meniru `lastErrorID`/`lastErrorMessage`/
/// `lastSqlstate` (`SPMySQLConnection.m:680-703`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodedError {
    pub id: u32,
    pub message: String,
    pub sqlstate: String,
}

impl CodedError {
    fn from_message(message: impl Into<String>) -> Self {
        Self {
            id: 0,
            message: message.into(),
            sqlstate: String::new(),
        }
    }
}

impl fmt::Display for CodedError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Sama seperti macOS app yang menampilkan `mysql_error` apa adanya.
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CodedError {}

/// Klasifikasi error koneksi, meniru `+isErrorIDConnectionError:`
/// (`Querying & Preparation.m:708-731`): 2001-2007, 2009, 2012, 2013, 2027,
/// 2032, 2047, 2048, 2050, 2055.
pub fn is_connection_error(id: u32) -> bool {
    matches!(
        id,
        2001..=2007 | 2009 | 2012 | 2013 | 2027 | 2032 | 2047 | 2048 | 2050 | 2055
    )
}

/// Parameter TLS, meniru `mysql_ssl_set(...)` + `MYSQL_OPT_SSL_MODE`
/// (`SPMySQLConnection.m:1016-1067`).
#[derive(Debug, Clone, Default)]
pub struct SslParams {
    pub key: Option<String>,
    pub cert: Option<String>,
    pub ca: Option<String>,
    pub ca_path: Option<String>,
    pub cipher: Option<String>,
    /// `SACleartextAuthPolicy requiresTLSWithCleartextPluginEnabled:sslRequested:`
    /// true → `SSL_MODE_REQUIRED`, else `SSL_MODE_PREFERRED`.
    pub require_tls: bool,
}

/// Parameter koneksi. Default meniru `SPMySQLConnection.m:344-441`.
#[derive(Debug, Clone)]
pub struct ConnectParams {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: Option<String>,
    /// `timeout` default 30 s (`SPMySQLConnection.m:372`).
    pub connect_timeout_secs: u64,
    /// `MYSQL_ENABLE_CLEARTEXT_PLUGIN` (issue #368).
    pub enable_cleartext_plugin: bool,
    /// `MYSQL_OPT_GET_SERVER_PUBLIC_KEY`.
    pub request_server_public_key: bool,
    /// `MYSQL_OPT_LOCAL_INFILE` (issue #245).
    pub allow_local_infile: bool,
    pub ssl: Option<SslParams>,
    pub socket_path: Option<String>,
}

impl ConnectParams {
    pub fn new(host: impl Into<String>, port: u16, user: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port,
            user: user.into(),
            password: String::new(),
            database: None,
            connect_timeout_secs: 30,
            enable_cleartext_plugin: false,
            request_server_public_key: false,
            allow_local_infile: false,
            ssl: None,
            socket_path: None,
        }
    }
}

/// Metadata satu kolom hasil, dipetakan dari `MYSQL_FIELD`
/// (`Field Definitions.m:343-438`).
#[derive(Debug, Clone)]
pub struct ColumnMeta {
    pub name: String,
    pub org_name: String,
    pub table: String,
    pub field_type: ffi::enum_field_types,
    pub flags: u32,
    pub charsetnr: u32,
    /// `MYSQL_FIELD.length` — binding-nya `c_ulong` yang 32-bit di Windows x64
    /// (LLP64) dan 64-bit di macOS (LP64); cast selalu ke u64 agar konstan.
    pub length: u64,
    pub decimals: u32,
}

/// `AUTO_INCREMENT_FLAG` dari `mysql.h` (16384); menandai kolom auto-increment
/// pada metadata hasil — dipakai `Copy as SQL INSERT (no auto_inc)` (doc 07 §4).
const AUTO_INCREMENT_FLAG: u32 = 16384;

impl ColumnMeta {
    pub fn is_binary(&self) -> bool {
        self.flags & BINARY_FLAG != 0
    }

    pub fn is_not_null(&self) -> bool {
        self.flags & NOT_NULL_FLAG != 0
    }

    pub fn is_primary_key(&self) -> bool {
        self.flags & PRI_KEY_FLAG != 0
    }

    pub fn is_auto_increment(&self) -> bool {
        self.flags & AUTO_INCREMENT_FLAG != 0
    }

    /// Kolom numerik yang dulisit *tanpa* quote pada copy-as-SQL, mengikuti
    /// `kColType == 0` dari doc 07 §4 (`SPFieldTypeClassifier`).
    pub fn is_unquoted_numeric(&self) -> bool {
        use ffi::enum_field_types::*;
        matches!(
            self.field_type,
            MYSQL_TYPE_DECIMAL
                | MYSQL_TYPE_TINY
                | MYSQL_TYPE_SHORT
                | MYSQL_TYPE_LONG
                | MYSQL_TYPE_FLOAT
                | MYSQL_TYPE_DOUBLE
                | MYSQL_TYPE_NULL
                | MYSQL_TYPE_LONGLONG
                | MYSQL_TYPE_INT24
                | MYSQL_TYPE_YEAR
                | MYSQL_TYPE_NEWDECIMAL
        )
    }

    /// Keluarga blob / geometry yang dicopy sebagai `X'<hex>'` (doc 07 §4).
    pub fn is_blob_family(&self) -> bool {
        use ffi::enum_field_types::*;
        matches!(
            self.field_type,
            MYSQL_TYPE_BLOB
                | MYSQL_TYPE_TINY_BLOB
                | MYSQL_TYPE_MEDIUM_BLOB
                | MYSQL_TYPE_LONG_BLOB
                | MYSQL_TYPE_GEOMETRY
        )
    }

    /// Kolom biner mentah (BLOB/GEOMETRY, atau VARBINARY/BINARY — yang
    /// memakai `MYSQL_TYPE_*` lain dengan `BINARY_FLAG`). Keduanya tampil hex.
    pub fn is_raw_binary(&self) -> bool {
        use ffi::enum_field_types::*;
        self.is_blob_family()
            || ((self.flags & BINARY_FLAG != 0)
                && matches!(self.field_type, MYSQL_TYPE_VAR_STRING | MYSQL_TYPE_STRING))
    }
}

/// Hasil satu pernyataan. `rows` kosong untuk pernyataan non-SELECT.
#[derive(Debug, Clone, Default)]
pub struct ResultSet {
    pub columns: Vec<ColumnMeta>,
    pub rows: Vec<Vec<Option<Vec<u8>>>>,
    pub affected_rows: u64,
    pub insert_id: u64,
}

impl ResultSet {
    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|column| column.name == name)
    }

    pub fn cell(&self, row: usize, column: &str) -> Option<&[u8]> {
        let index = self.column_index(column)?;
        self.rows.get(row)?.get(index)?.as_deref()
    }

    /// Akses posisional, untuk hasil yang nama kolomnya dinamis
    /// (mis. `SHOW FULL TABLES` → `Tables_in_<db>`).
    pub fn cell_at(&self, row: usize, column: usize) -> Option<&[u8]> {
        self.rows.get(row)?.get(column)?.as_deref()
    }

    pub fn text_at(&self, row: usize, column: usize) -> String {
        self.cell_at(row, column)
            .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
            .unwrap_or_default()
    }

    /// Nilai kolom sebagai teks; sel hilang/NULL menjadi string kosong.
    /// Dipakai untuk metadata (`SHOW ...`) yang selalu berupa teks.
    pub fn cell_text(&self, row: usize, column: &str) -> String {
        self.cell(row, column)
            .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
            .unwrap_or_default()
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }
}

/// Inisialisasi library sekali per proses (`+initialize` memanggil
/// `mysql_library_init`, `SPMySQLConnection.m:144-156`).
fn init_library() {
    LIBRARY_INIT.call_once(|| unsafe {
        // Nama binding untuk `mysql_library_init`.
        ffi::mysql_server_init(0, ptr::null_mut(), ptr::null_mut());
    });
}

/// Salinan `MYSQL_RES` yang selalu dibebaskan.
struct RawResult(NonNull<ffi::MYSQL_RES>);

impl Drop for RawResult {
    fn drop(&mut self) {
        unsafe { ffi::mysql_free_result(self.0.as_ptr()) };
    }
}

/// Koneksi `MYSQL*`. Tidak `Send`/`Sync` secara default (menyimpan `NonNull`),
/// jadi ia harus dibuat dan dipakai di thread yang sama — sama seperti
/// `SPMySQLConnection` yang dikunci ke thread pemiliknya.
pub struct Conn {
    handle: NonNull<ffi::MYSQL>,
    connected: bool,
}

impl Conn {
    /// Buka koneksi, meniru urutan `_makeRawMySQLConnectionWithEncoding:`
    /// (`SPMySQLConnection.m:895-1121`).
    pub fn connect(params: &ConnectParams) -> Result<Self, CodedError> {
        init_library();
        // Setiap thread yang menyentuh klien harus memanggil ini
        // (`-validateThreadSetup`, `SPMySQLConnection.m:1688-1704`).
        unsafe {
            ffi::mysql_thread_init();
        }

        let handle = unsafe { ffi::mysql_init(ptr::null_mut()) };
        let handle = NonNull::new(handle)
            .ok_or_else(|| CodedError::from_message("Could not initialise a MySQL connection."))?;

        let mut conn = Self {
            handle,
            connected: false,
        };

        // Jangan biarkan library menyambung ulang sendiri; retry ditangani di
        // lapisan kita (`MYSQL_OPT_RECONNECT=FALSE`, `SPMySQLConnection.m:909-910`).
        conn.set_option_bool(ffi::mysql_option::MYSQL_OPT_RECONNECT, false);
        conn.set_option_uint(
            ffi::mysql_option::MYSQL_OPT_CONNECT_TIMEOUT,
            params.connect_timeout_secs as c_uint,
        );
        // Port TCP; macOS memakai socket, Windows memakai TCP.
        conn.set_option_uint(
            ffi::mysql_option::MYSQL_OPT_PROTOCOL,
            ffi::mysql_protocol_type::MYSQL_PROTOCOL_TCP as c_uint,
        );
        conn.set_option_str(ffi::mysql_option::MYSQL_SET_CHARSET_NAME, DEFAULT_CHARSET);
        // Paksa plugin auth default (issue #141).
        conn.set_option_str(ffi::mysql_option::MYSQL_DEFAULT_AUTH, FORCED_DEFAULT_AUTH);

        if params.enable_cleartext_plugin {
            conn.set_option_bool(ffi::mysql_option::MYSQL_ENABLE_CLEARTEXT_PLUGIN, true);
        }
        if params.request_server_public_key {
            conn.set_option_bool(ffi::mysql_option::MYSQL_OPT_GET_SERVER_PUBLIC_KEY, true);
        }
        conn.set_option_uint(
            ffi::mysql_option::MYSQL_OPT_LOCAL_INFILE,
            u32::from(params.allow_local_infile),
        );

        if let Some(ssl) = &params.ssl {
            conn.configure_ssl(ssl);
        }

        let host = cstring(&params.host)?;
        let user = cstring(&params.user)?;
        let password = cstring(&params.password)?;
        let database = match &params.database {
            Some(name) => Some(cstring(name)?),
            None => None,
        };

        let socket_path = match &params.socket_path {
            Some(path) => CString::new(path.as_str()).ok(),
            None => None,
        };
        let (protocol, socket_ptr) = match &socket_path {
            Some(path) => (
                ffi::mysql_protocol_type::MYSQL_PROTOCOL_SOCKET,
                path.as_ptr(),
            ),
            None => (ffi::mysql_protocol_type::MYSQL_PROTOCOL_TCP, ptr::null()),
        };
        conn.set_option_uint(ffi::mysql_option::MYSQL_OPT_PROTOCOL, protocol as c_uint);
        let result = unsafe {
            ffi::mysql_real_connect(
                conn.handle.as_ptr(),
                host.as_ptr(),
                user.as_ptr(),
                password.as_ptr(),
                database.as_ref().map_or(ptr::null(), |name| name.as_ptr()),
                c_uint::from(params.port),
                socket_ptr,
                CLIENT_COMPRESS | CLIENT_INTERACTIVE | CLIENT_MULTI_RESULTS,
            )
        };

        if result.is_null() {
            return Err(conn.last_error());
        }

        conn.connected = true;
        Ok(conn)
    }

    fn configure_ssl(&mut self, ssl: &SslParams) {
        let key = optional_cstring(ssl.key.as_deref());
        let cert = optional_cstring(ssl.cert.as_deref());
        let ca = optional_cstring(ssl.ca.as_deref());
        let ca_path = optional_cstring(ssl.ca_path.as_deref());
        let cipher = optional_cstring(ssl.cipher.as_deref());
        unsafe {
            ffi::mysql_ssl_set(
                self.handle.as_ptr(),
                as_ptr(&key),
                as_ptr(&cert),
                as_ptr(&ca),
                as_ptr(&ca_path),
                as_ptr(&cipher),
            );
        }
        let mode = if ssl.require_tls {
            ffi::mysql_ssl_mode::SSL_MODE_REQUIRED
        } else {
            ffi::mysql_ssl_mode::SSL_MODE_PREFERRED
        };
        let mode: c_uint = mode as c_uint;
        unsafe {
            ffi::mysql_options(
                self.handle.as_ptr(),
                ffi::mysql_option::MYSQL_OPT_SSL_MODE,
                &mode as *const c_uint as *const c_void,
            );
        }
    }

    fn set_option_str(&mut self, option: ffi::mysql_option, value: &str) {
        if let Ok(value) = CString::new(value) {
            unsafe {
                ffi::mysql_options(
                    self.handle.as_ptr(),
                    option,
                    value.as_ptr() as *const c_void,
                );
            }
        }
    }

    fn set_option_uint(&mut self, option: ffi::mysql_option, value: c_uint) {
        unsafe {
            ffi::mysql_options(
                self.handle.as_ptr(),
                option,
                &value as *const c_uint as *const c_void,
            );
        }
    }

    fn set_option_bool(&mut self, option: ffi::mysql_option, value: bool) {
        unsafe {
            ffi::mysql_options(
                self.handle.as_ptr(),
                option,
                &value as *const bool as *const c_void,
            );
        }
    }

    fn last_error(&self) -> CodedError {
        unsafe {
            CodedError {
                id: ffi::mysql_errno(self.handle.as_ptr()) as u32,
                message: cstr_to_string(ffi::mysql_error(self.handle.as_ptr())),
                sqlstate: cstr_to_string(ffi::mysql_sqlstate(self.handle.as_ptr())),
            }
        }
    }

    /// Jalankan pernyataan tanpa mengambil hasil (untuk DML/DDL).
    pub fn execute(&mut self, sql: &str) -> Result<(), CodedError> {
        let sql = cstring(sql)?;
        let status = unsafe {
            ffi::mysql_real_query(
                self.handle.as_ptr(),
                sql.as_ptr(),
                sql.as_bytes().len() as c_ulong,
            )
        };
        if status != 0 {
            return Err(self.last_error());
        }
        Ok(())
    }

    /// Jalankan pernyataan dan kumpulkan hasil lengkap.
    ///
    /// Memakai `mysql_store_result` seperti `SPMySQLResultAsResult`
    /// (`Querying & Preparation.m:543-569`). Streaming `mysql_use_result`
    /// belum dipakai di port ini.
    pub fn query(&mut self, sql: &str) -> Result<ResultSet, CodedError> {
        self.execute(sql)?;
        self.collect_result()
    }

    /// Ambil hasil setelah `execute`, termasuk `affected_rows`/`insert_id`
    /// (`SPMySQLConnection.m:488, 582-584`).
    pub fn collect_result(&mut self) -> Result<ResultSet, CodedError> {
        let handle = self.handle.as_ptr();
        let insert_id = unsafe { ffi::mysql_insert_id(handle) };
        let field_count = unsafe { ffi::mysql_field_count(handle) };

        if field_count == 0 {
            return Ok(ResultSet {
                columns: Vec::new(),
                rows: Vec::new(),
                affected_rows: unsafe { ffi::mysql_affected_rows(handle) },
                insert_id,
            });
        }

        let raw = unsafe { ffi::mysql_store_result(handle) };
        let raw = match NonNull::new(raw) {
            Some(raw) => RawResult(raw),
            None => return Err(self.last_error()),
        };

        let result = raw.0.as_ptr();
        let field_count = unsafe { ffi::mysql_num_fields(result) } as usize;
        let columns = self.read_columns(result, field_count);
        let mut rows = Vec::new();
        loop {
            let row = unsafe { ffi::mysql_fetch_row(result) };
            if row.is_null() {
                break;
            }
            let lengths = unsafe { ffi::mysql_fetch_lengths(result) };
            let mut cells = Vec::with_capacity(field_count);
            for index in 0..field_count {
                let cell = unsafe { *row.add(index) };
                if cell.is_null() || lengths.is_null() {
                    cells.push(None);
                    continue;
                }
                let length = unsafe { *lengths.add(index) } as usize;
                let bytes = unsafe { std::slice::from_raw_parts(cell as *const u8, length) };
                cells.push(Some(bytes.to_vec()));
            }
            rows.push(cells);
        }

        // Setelah `mysql_fetch_row` selesai, errno != 0 menandakan kegagalan
        // di tengah pengambilan.
        let error_id = unsafe { ffi::mysql_errno(handle) };
        if error_id != 0 {
            return Err(self.last_error());
        }

        Ok(ResultSet {
            columns,
            rows,
            affected_rows: unsafe { ffi::mysql_affected_rows(handle) },
            insert_id,
        })
    }

    fn read_columns(&self, result: *mut ffi::MYSQL_RES, count: usize) -> Vec<ColumnMeta> {
        let fields = unsafe { ffi::mysql_fetch_fields(result) };
        if fields.is_null() {
            return Vec::new();
        }
        (0..count)
            .map(|index| {
                let field = unsafe { &*fields.add(index) };
                ColumnMeta {
                    name: field_bytes(field.name, field.name_length),
                    org_name: field_bytes(field.org_name, field.org_name_length),
                    table: field_bytes(field.table, field.table_length),
                    field_type: field.type_,
                    flags: field.flags,
                    charsetnr: field.charsetnr,
                    length: field.length as u64,
                    decimals: field.decimals,
                }
            })
            .collect()
    }

    /// Jalankan pernyataan dan kumpulkan **semua** result set.
    ///
    /// Koneksi dibuka dengan `CLIENT_MULTI_RESULTS`, dan macOS app menguras
    /// sisa hasil tiap kali koneksi di-unlock
    /// (`Locking.m:91-102`, `Querying & Preparation.m:807-827`).
    pub fn query_multi(&mut self, sql: &str) -> Result<Vec<ResultSet>, CodedError> {
        self.execute(sql)?;
        let mut sets = Vec::new();
        loop {
            sets.push(self.collect_result()?);
            if !self.more_results() {
                break;
            }
            let status = unsafe { ffi::mysql_next_result(self.handle.as_ptr()) };
            if status != 0 {
                return Err(self.last_error());
            }
        }
        Ok(sets)
    }

    pub fn more_results(&self) -> bool {
        unsafe { ffi::mysql_more_results(self.handle.as_ptr()) }
    }

    /// `escapeString:includingQuotes:YES` sebagai `Result`.
    pub fn quote_literal(&self, value: &str) -> Result<String, CodedError> {
        self.escape_string(value, true).ok_or_else(|| {
            CodedError::from_message(format!(
                "The value could not be escaped for character set '{DEFAULT_CHARSET}'."
            ))
        })
    }

    /// Pindah ke database lain (`mysql_select_db`).
    pub fn select_db(&mut self, name: &str) -> Result<(), CodedError> {
        let name = cstring(name)?;
        let status = unsafe { ffi::mysql_select_db(self.handle.as_ptr(), name.as_ptr()) };
        if status != 0 {
            return Err(self.last_error());
        }
        Ok(())
    }

    /// Ping; `Ping & KeepAlive.m:273` memakai `mysql_ping`.
    pub fn ping(&mut self) -> Result<(), CodedError> {
        let status = unsafe { ffi::mysql_ping(self.handle.as_ptr()) };
        if status != 0 {
            return Err(self.last_error());
        }
        Ok(())
    }

    /// Escape byte untuk charset koneksi.
    ///
    /// Meniru `SAConnectionEscaper.swift:57-68`: pakai
    /// `mysql_real_escape_string_quote` supaya mode `NO_BACKSLASH_ESCAPES`
    /// (quote digandakan) ikut benar.
    pub fn escape_bytes(&self, value: &[u8]) -> Option<Vec<u8>> {
        // Buffer terburuk: 2n + 1 byte.
        let mut buffer = vec![0u8; value.len() * 2 + 1];
        let written = unsafe {
            ffi::mysql_real_escape_string_quote(
                self.handle.as_ptr(),
                buffer.as_mut_ptr() as *mut c_char,
                value.as_ptr() as *const c_char,
                value.len() as c_ulong,
                b'\'' as c_char,
            )
        };
        if written == c_ulong::MAX {
            return None;
        }
        buffer.truncate(written as usize);
        Some(buffer)
    }

    /// `escapeString:includingQuotes:` (`Querying & Preparation.m:75-177`).
    /// Mengembalikan `None` bila nilai tidak bisa di-escape untuk charset ini.
    pub fn escape_string(&self, value: &str, including_quotes: bool) -> Option<String> {
        let escaped = self.escape_bytes(value.as_bytes())?;
        let escaped = String::from_utf8_lossy(&escaped).into_owned();
        if including_quotes {
            Some(format!("'{escaped}'"))
        } else {
            Some(escaped)
        }
    }

    /// `escapeData:includingQuotes:YES` → `X'<hex>'` via `mysql_hex_string`
    /// (`Querying & Preparation.m:196-230`).
    pub fn escape_data_hex(&self, value: &[u8]) -> String {
        let mut buffer = vec![0u8; value.len() * 2 + 1];
        let written = unsafe {
            ffi::mysql_hex_string(
                buffer.as_mut_ptr() as *mut c_char,
                value.as_ptr() as *const c_char,
                value.len() as c_ulong,
            )
        };
        buffer.truncate(written as usize);
        format!("X'{}'", String::from_utf8_lossy(&buffer))
    }

    /// Quote identifier gaya macOS (`mySQLBacktickQuotedString`,
    /// `SPMySQLStringAdditions.m:41-54`): gandakan backtick.
    pub fn quote_identifier(name: &str) -> String {
        format!("`{}`", name.replace('`', "``"))
    }

    pub fn server_info(&self) -> String {
        unsafe { cstr_to_string(ffi::mysql_get_server_info(self.handle.as_ptr())) }
    }

    /// `serverVersionNumber` (`SPMySQLConnection.m:844-846`).
    pub fn server_version(&self) -> u64 {
        unsafe { ffi::mysql_get_server_version(self.handle.as_ptr()) as u64 }
    }

    /// `connectedWithSSL = mysql_get_ssl_cipher(...) != NULL`
    /// (`SPMySQLConnection.m:849`).
    pub fn is_connected_via_ssl(&self) -> bool {
        unsafe { !ffi::mysql_get_ssl_cipher(self.handle.as_ptr()).is_null() }
    }

    pub fn ssl_cipher(&self) -> Option<String> {
        unsafe {
            let cipher = ffi::mysql_get_ssl_cipher(self.handle.as_ptr());
            (!cipher.is_null()).then(|| cstr_to_string(cipher))
        }
    }

    /// `mysqlConnectionThreadId` (`SPMySQLConnection.m:830-835`).
    pub fn thread_id(&self) -> u64 {
        unsafe { ffi::mysql_thread_id(self.handle.as_ptr()) as u64 }
    }

    pub fn warning_count(&self) -> u32 {
        unsafe { ffi::mysql_warning_count(self.handle.as_ptr()) as u32 }
    }

    /// `mysql_affected_rows` dari pernyataan terakhir
    /// (`SPMySQLConnection.m:488`, `rowsAffectedByLastQuery:`).
    pub fn last_affected_rows(&self) -> u64 {
        unsafe { ffi::mysql_affected_rows(self.handle.as_ptr()) }
    }

    /// `mysql_field_count` pada koneksi — apakah pernyataan terakhir punya baris.
    pub fn field_count(&self) -> u32 {
        unsafe { ffi::mysql_field_count(self.handle.as_ptr()) as u32 }
    }
}

impl Drop for Conn {
    fn drop(&mut self) {
        if self.connected {
            unsafe { ffi::mysql_close(self.handle.as_ptr()) };
        }
        unsafe { ffi::mysql_thread_end() };
    }
}

/// Tampilan satu sel untuk grid, mengikuti konversi tipe doc 13 §4:
/// NULL → `NSNull` (di sini literal `NULL`), angka/tanggal → teks apa adanya,
/// BIT → string biner, blob/geometry → byte.
pub fn format_cell(meta: Option<&ColumnMeta>, cell: Option<&[u8]>, max_bytes: usize) -> String {
    use ffi::enum_field_types as field;

    let Some(bytes) = cell else {
        return "NULL".to_owned();
    };
    let truncated = bytes.len() > max_bytes;

    // Blob/geometry ditampilkan hex sesuai reglenya di tab-copy/INSERT
    // (`07-content-edit-copy.md` §4); tentukan dari metadata, bukan dari hasil
    // coba-decode, supaya byte `0x84 0x12` yang valid UTF-8 tak jadi karakter.
    if let Some(meta) = meta
        && meta.is_raw_binary()
    {
        let mut hex = String::with_capacity(bytes.len() * 2 + 2);
        hex.push_str("0x");
        for byte in bytes {
            hex.push_str(&format!("{byte:02X}"));
        }
        return if truncated { format!("{hex}…") } else { hex };
    }

    // BIT bukan teks di protokol teks: byte mentah, ditampilkan sebagai bit
    // string dengan panjang MIN(bytes*8, field.length) (doc 13 §4).
    if let Some(meta) = meta
        && meta.field_type == field::MYSQL_TYPE_BIT
    {
        let width = (bytes.len() * 8).min(meta.length as usize);
        if width == 0 {
            return String::new();
        }
        let mut bits = String::with_capacity(width);
        for index in 0..width {
            let byte = bytes[index / 8];
            let shift = 7 - (index % 8);
            bits.push(if byte >> shift & 1 == 1 { '1' } else { '0' });
        }
        return bits;
    }

    let truncated = bytes.len() > max_bytes;
    let prefix = &bytes[..bytes.len().min(max_bytes)];
    let mut text = match std::str::from_utf8(prefix) {
        Ok(text) => text.to_owned(),
        Err(_) => {
            let mut hex = String::with_capacity(prefix.len() * 2 + 2);
            hex.push_str("0x");
            for byte in prefix {
                hex.push_str(&format!("{byte:02X}"));
            }
            hex
        }
    };
    if truncated {
        text.push('…');
    }
    text
}

fn cstring(value: &str) -> Result<CString, CodedError> {
    CString::new(value).map_err(|_| {
        CodedError::from_message("Connection value contains a NUL byte and cannot be used.")
    })
}

fn optional_cstring(value: Option<&str>) -> Option<CString> {
    value.and_then(|value| CString::new(value).ok())
}

fn as_ptr(value: &Option<CString>) -> *const c_char {
    value.as_ref().map_or(ptr::null(), |value| value.as_ptr())
}

fn cstr_to_string(value: *const c_char) -> String {
    if value.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(value) }
        .to_string_lossy()
        .into_owned()
}

fn field_bytes(pointer: *const c_char, length: c_uint) -> String {
    if pointer.is_null() {
        return String::new();
    }
    let length = length as usize;
    let bytes = unsafe { std::slice::from_raw_parts(pointer as *const u8, length) };
    String::from_utf8_lossy(bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn column(field_type: ffi::enum_field_types, length: u64) -> ColumnMeta {
        ColumnMeta {
            name: "c".to_owned(),
            org_name: "c".to_owned(),
            table: "t".to_owned(),
            field_type,
            flags: 0,
            charsetnr: 63,
            length,
            decimals: 0,
        }
    }

    #[test]
    fn connection_error_classification_matches_mac_connection_list() {
        // Querying & Preparation.m:708-731
        for id in [
            2001, 2002, 2003, 2004, 2005, 2006, 2007, 2009, 2012, 2013, 2027, 2032, 2047, 2048,
            2050, 2055,
        ] {
            assert!(is_connection_error(id), "{id} seharusnya error koneksi");
        }
        assert!(!is_connection_error(0));
        assert!(!is_connection_error(1062)); // duplicate entry
        assert!(!is_connection_error(1146)); // table doesn't exist
        assert!(!is_connection_error(1317)); // query interrupted
    }

    #[test]
    fn null_cell_renders_as_the_mac_null_placeholder() {
        assert_eq!(format_cell(None, None, MAX_CELL_BYTES), "NULL");
    }

    #[test]
    fn textual_cells_pass_through_unmodified() {
        let cell = b"hello".to_vec();
        assert_eq!(format_cell(None, Some(&cell), MAX_CELL_BYTES), "hello");
    }

    #[test]
    fn non_utf8_cells_fall_back_to_hex_like_the_port_does() {
        let cell = vec![0xDE, 0xAD, 0xBE, 0xEF];
        assert_eq!(format_cell(None, Some(&cell), MAX_CELL_BYTES), "0xDEADBEEF");
    }

    #[test]
    fn oversized_cells_are_truncated_and_marked() {
        let cell = vec![b'a'; 10];
        assert_eq!(format_cell(None, Some(&cell), 4), "aaaa…");
    }

    #[test]
    fn bit_columns_render_as_bit_strings_and_respect_field_length() {
        use ffi::enum_field_types as field;
        // MYSQL_TYPE_BIT bukan teks: 0x05 → "00000101" saat panjang field 8.
        let cell = vec![0x05u8];
        assert_eq!(
            format_cell(
                Some(&column(field::MYSQL_TYPE_BIT, 8)),
                Some(&cell),
                MAX_CELL_BYTES
            ),
            "00000101"
        );
        // Panjang field membatasi jumlah bit yang ditampilkan.
        assert_eq!(
            format_cell(
                Some(&column(field::MYSQL_TYPE_BIT, 4)),
                Some(&cell),
                MAX_CELL_BYTES
            ),
            "0000"
        );
    }

    #[test]
    fn empty_bit_cell_does_not_panic() {
        use ffi::enum_field_types as field;
        let cell: Vec<u8> = Vec::new();
        assert_eq!(
            format_cell(
                Some(&column(field::MYSQL_TYPE_BIT, 8)),
                Some(&cell),
                MAX_CELL_BYTES
            ),
            ""
        );
    }

    #[test]
    fn identifiers_are_backtick_quoted_by_doubling_backticks() {
        assert_eq!(Conn::quote_identifier("plain"), "`plain`");
        assert_eq!(Conn::quote_identifier("a`b"), "`a``b`");
        assert_eq!(Conn::quote_identifier("one.two"), "`one.two`");
    }

    #[test]
    fn result_set_column_lookup_is_by_name_and_position() {
        let set = ResultSet {
            columns: vec![column(ffi::enum_field_types::MYSQL_TYPE_VAR_STRING, 0)],
            rows: vec![vec![Some(b"value".to_vec())], vec![None]],
            affected_rows: 0,
            insert_id: 0,
        };
        assert_eq!(set.cell_text(0, "c"), "value");
        assert_eq!(set.text_at(0, 0), "value");
        assert_eq!(set.cell_text(1, "c"), "");
        assert!(set.cell(1, "c").is_none());
        assert_eq!(set.row_count(), 2);
    }
}
