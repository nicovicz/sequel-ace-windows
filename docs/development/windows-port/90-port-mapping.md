# Peta port: macOS Sequel Ace → Rust `WindowsPort/`

Dokumen ini memetakan setiap subsistem macOS (yang dispesifikasikan di dokumen
01–15) ke kode Rust yang ada di `WindowsPort/src/`, dengan status apa adanya.
Status di sini adalah **status source**, bukan bukti parity runtime — lihat
`../windows-port-parity-plan.md` untuk standar bukti.

Baseline macOS: commit `02c910932`. Rust: `WindowsPort/src/`
(eframe/egui + crate `mysql`, 13 file, ~2.341 baris).

## 1. Skala

| Sisi | Baris (perkiraan) |
| --- | --- |
| macOS `Source/` (.m + .h + .swift) | ~156.448 |
| macOS `Frameworks/SPMySQLFramework/` | ~25.967 |
| **macOS total** | **~182.415** |
| Rust `WindowsPort/src/` | ~2.341 |

Rasio ~1,3% baris. Angka ini hanya indikator kasar; yang menentukan adalah
cakupan perilaku per subsistem di tabel berikut.

## 2. Modul Rust yang ada

| File | Baris | Isi (dari source) |
| --- | --- | --- |
| `src/main.rs` | 18 | Entry `eframe::run_native("Sequel Ace")`, window 882×759 |
| `src/app.rs` | 399 | State `SequelAceApp`, loop Request/Response, tiga view (`Tables`/`Content`/`Query`), handler error/warning |
| `src/app/connection.rs` | 566 | Form koneksi (belum diaudit baris-per-baris di sini) |
| `src/app/browser.rs` | 151 | Daftar database + daftar tabel/view + filter nama + tombol Copy |
| `src/app/details.rs` | 215 | Panel detail objek: table info, kolom, index, FK, CREATE SQL; panel Server Info (variabel) |
| `src/app/query.rs` | 130 | Editor SQL, tombol Run, combo riwayat (in-memory), grid hasil multi-set, CSV |
| `src/app/content.rs` | 78 | View Content: Previous/Next/Refresh, Copy Page CSV, grid |
| `src/database.rs` | 50 | Re-export + unit test (default koneksi, partition, `SHOW FULL TABLES`, quoting) |
| `src/database/worker.rs` | 208 | Worker thread koneksi (belum diaudit) |
| `src/database/models.rs` | 200 | `ConnectionOptions`, `TableEntry`, `ColumnInfo`, `ObjectDetails`, `TableInfo`, `IndexInfo`, `ForeignKeyInfo`, `ServerInfo`, `QueryResultSet`, `QueryOutput`, `ContentPage`, `Request`, `Response` |
| `src/database/metadata.rs` | 160 | `SHOW FULL TABLES`, `SHOW FULL COLUMNS FROM`, `SHOW CREATE TABLE/VIEW`, `information_schema.TABLES`, `SHOW INDEX FROM`, `information_schema.KEY_COLUMN_USAGE`, `SHOW VARIABLES`; `quote_identifier`; `partition_databases` |
| `src/database/query.rs` | 105 | `execute_sql` (cap 1000 baris/set, 20 set, 8192 byte/sel); `display_value` (NULL/number/date/time/hex) |
| `src/database/content.rs` | 61 | `SELECT * FROM db.tbl LIMIT 1001 OFFSET <page*1000>`, fallback halaman-akhir-kosong |

## 3. Peta subsistem → target Rust

Status: **awal** = ada implementasi terbatas; **sebagian** = ada beberapa bagian;
**belum** = tak ada.

| macOS (dok) | Sumber macOS utama | Target Rust | Status |
| --- | --- | --- | --- |
| Arsitektur runtime (02) | `SPAppController`, `SPWindowController`, `TabManager`, `MainMenu.xib` | `src/main.rs`, `src/app.rs` (single-window egui) | sebagian (tanpa tab/window/menu model) |
| Koneksi & favorites (03) | `SPConnectionController`, `SAConnectionInfo`, `SAConnectionService`, `SPFavoritesController`, `SAKeychain*` | `src/app/connection.rs`, `src/database/worker.rs`, `ConnectionOptions` | sebagian (TCP/IP saja; tanpa favorites/keychain/SSH/AWS/Vault/SSL) |
| Database document (04) | `SPDatabaseDocument` | `src/app.rs` (state + view) | awal (tanpa task model, sesi `.spf`, mode penuh) |
| Daftar DB/objek + navigator (05) | `SPTablesList`, `SADatabaseListManager`, `SPNavigatorController`, `SQLitePinnedTableManager`, `SPTableHistoryManager` | `src/app/browser.rs`, `src/database/metadata.rs` | sebagian (tanpa routine, pinned, history, navigator, aksi tabel) |
| Content load/filter (06) | `SPTableContent` (load), `SPRuleFilterController`, `SPFilterTableController` | `src/app/content.rs`, `src/database/content.rs` | awal (tanpa filter/sort/count/preferensi limit) |
| Content edit/copy (07) | `SPTableContent` (edit), `SPCopyTable`, `SPFieldEditorController` | `src/app/query.rs` (`result_as_csv`) | belum (tanpa edit/insert/delete/duplicate, COPY as SQL/tab) |
| Struktur+index+field editor (10) | `SPTableStructure`, `SPIndexesController`, `SPFieldEditorController`, `SAPHPSerializedValue` | `src/app/details.rs` (hanya tampil) | awal (read-only; tanpa ALTER/index/editor/PHP) |
| Relasi+trigger+info+metadata (11) | `SPTableRelations`, `SPTableTriggers`, `SPTableInfo`, `SPTableData`, `SPServerSupport` | `src/app/details.rs`, `src/database/metadata.rs` | sebagian (FK + table-info read-only; tanpa trigger, tanpa ADD/DROP FK, tanpa capability) |
| SQL editor+eksekusi (08) | `SPCustomQuery`, `SPSQLParser`, `SPTextView` | `src/app/query.rs`, `src/database/query.rs` | awal (tanpa pemisahan statement/DELIMITER, pilihan/caret, EXPLAIN, cancel, highlight, autocompletion, snippet) |
| Query history+favorites (08b) | `SPQueryController`, `SQLiteHistoryManager`, `SAQueryFavoriteStore`, `SPQueryFavoriteManager` | `src/app/query.rs` (`query_history`, 20 entri, in-memory) | bagian terkecil (tanpa persistensi SQLite, tanpa favorites, tanpa tab trigger) |
| Import (09) | `SPDataImport`, `SPFieldMapperController`, `SACSVImportStreamDecoder`, `SPCSVParser` | — | belum |
| Export+print (09b) | `SPExportController`, `Exporters/*`, `SAPrintUtility` | `result_as_csv` (copy clipboard saja) | belum (tanpa SQL/XML/Dot, tanpa file, tanpa print) |
| Server tools (12) | `SPUserManager`, `SPProcessListController`, `SPServerVariablesController`, `SPGotoDatabaseController` | `src/app/details.rs` (variabel + shortcut `SHOW PROCESSLIST`) | sebagian kecil (tanpa user manager, kill, filter proses, goto database) |
| Bundles+MCP+help (12b) | `SPBundleManager`, `SPMCPServer`, `SAHelpViewer*` | — | belum |
| Framework MySQL (13) | `SPMySQLFramework` (libmysqlclient 8.4.11) | crate `mysql` (`minimal-rust`) di `database/worker.rs` | awal (driver berbeda; tanpa streaming store, tanpa keeper/cancel/reconnect, escaping/encoding belum parity) |
| Integrasi (14) | SSH tunnel, AWS, Vault, secure bookmark, updater, analytics, drag pasteboard | — | belum |
| Views+utilitas (15) | `Source/Views/**`, `Other/{Parsing,Formatters,Extensions,Utility,CategoryAdditions}`, `ThirdParty` | egui bawaan + helper lokal | sebagian kecil (tanpa widget kustom; parser CSV/JSON/SQL belum) |

## 4. Celah per lapisan (yang belum ada padanannya)

- **UI shell**: tak ada model window/tab, menu, toolbar, preferensi, atau pane terpisah. Rust hanya satu window egui dengan tiga view.
- **Koneksi**: hanya TCP/IP dasar. Tanpa socket lokal, SSH tunnel, AWS IAM, Vault, TLS/SSL, zona waktu, encoding, kompresi, retry/cancel, favorites, keychain, `.spf`/`.spfs`, `mysql://`.
- **Eksplorasi objek**: tanpa procedure/function/event, header section, pinned table, history, navigator, filter regex server-order, aksi (add/drop/rename/duplicate/truncate), komentar tabel.
- **Content**: tanpa filter (sederhana/lanjutan/kustom), sortir, count, preferensi `LimitResults`, editing (insert/update/delete/duplicate), copy tab/INSERT, NULL/BLOB/geometry yang setara, streaming/cancel.
- **Struktur**: read-only; tanpa `ALTER TABLE`, index manager, auto-increment rule, charset/collation popup, field editor, PHP-serialize.
- **Relasi/trigger**: hanya FK read-only; tanpa add/drop FK, ON UPDATE/DELETE, trigger, capability MySQL 8.4, negative caching.
- **SQL editor**: tanpa `SPSQLParser` (pemisahan/DELIMITER), eksekusi pilihan/caret, EXPLAIN, cancel, syntax highlight, autocompletion, bracket, snippet, classifier destruktif.
- **History/favorites**: hanya 20 entri in-memory; tanpa SQLite (`QueryHistory`, `ColumnDisplayOverrides`), tanpa scope per-dokumen/global, tanpa favorites/tab trigger/import-export.
- **Import/Export/Print**: seluruhnya belum.
- **Server tools**: tanpa user manager (grant/revoke), process list penuh (full/filter/kill/auto-refresh), save/copy variabel, goto database.
- **Bundles/MCP/help**: seluruhnya belum.
- **Driver**: crate `mysql` bukan libmysqlclient; streaming result store, keep-alive/ping/reconnect, cancel via koneksi kedua, escaping charset-aware, dan pemetaan error belum parity (lihat dok 13).
- **Integrasi**: seluruhnya belum.
- **Pemetaan tipe/format**: `display_value` port memakai `"NULL"` literal dan hex `0x…`, sedangkan macOS memakai placeholder pref `SPNullValue` (default `"NULL"`) dan aturan `SPDisplayBinaryDataAsHex` — perlu disamakan.

## 5. Urutan kerja yang disarankan (dari dependency)

Mengikuti `../windows-port-feature-inventory.md` dan celah di atas:

1. **Driver**: putuskan apakah tetap `mysql` crate atau bind ke libmysqlclient; samakan escaping/charset/cancel/reconnect/streaming (dok 13) — fondasi semua fitur.
2. **Kontrak koneksi**: lengkapi opsi (SSL, zona waktu, encoding, timeout) dan pemetaan error (dok 03).
3. **Daftar objek**: routine, filter/normalisasi, pinned/history, navigator (dok 05).
4. **Content**: filter+sort+count+preferensi limit sebelum editing (dok 06), lalu editing+copy (dok 07).
5. **Struktur/relasi/trigger**: dari read-only ke aksi DDL (dok 10, 11).
6. **SQL editor**: parser + eksekusi pilihan/caret + cancel + EXPLAIN (dok 08), lalu history SQLite + favorites (dok 08b).
7. **Import/Export/Print** (dok 09, 09b).
8. **Server tools** (dok 12), **Bundles/MCP/help** (dok 12b), **Integrasi** (dok 14).

## 6. Catatan kejujuran

Peta ini memetakan *keberadaan* kode dan *celah source*, bukan bukti bahwa EXE
Windows berperilaku sama dengan macOS. Tidak ada baris di sini yang boleh
disebut "setara/parity" tanpa uji runtime pada build macOS dan Windows dari
commit yang sama beserta langkah reproduksi dan hasil kondisi normal/kosong/error
— persis standar di `../windows-port-parity-plan.md`.
