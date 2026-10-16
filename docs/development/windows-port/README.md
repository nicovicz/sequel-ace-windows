# Spesifikasi source Sequel Ace untuk port Windows (Rust)

Pohon dokumen ini adalah **spesifikasi struktural tingkat source** dari aplikasi
macOS Sequel Ace pada checkout ini, dibuat sebagai dasar port 1:1 ke Rust di
`WindowsPort/`. Baseline commit: `02c910932` (versi 6.0.3, build 20116).

Aturan keras yang dipakai di seluruh dokumen:

1. **Setiap klaim perilaku harus punya sitasi `file:line`.** Tidak ada asumsi.
   Kalau source tidak menentukan sesuatu, ditulis `UNKNOWN` secara eksplisit,
   bukan diisi dengan tebakan.
2. **Source code adalah satu-satunya fakta.** Binary rilis, memori, atau
   kebiasaan versi lain tidak dipakai sebagai bukti perilaku source.
3. **SQL dan string yang dibentuk aplikasi dikutip apa adanya** (termasuk format
   string dan urutan klausa), karena port harus menghasilkan byte yang sama.
4. **Status runtime terpisah dari status source.** Dokumen di sini memetakan
   *apa yang source lakukan*. Apakah EXE Windows benar-benar setara hanya
   dibuktikan dengan uji runtime terhadap build macOS dari commit yang sama —
   lihat ledger di `../windows-port-parity-plan.md`.

## Peta dokumen

| Dok | Isi | Sumber fakta |
| --- | --- | --- |
| [01-source-inventory.md](./01-source-inventory.md) | Inventaris mekanis 529 file: baris, bahasa, lapisan | filesystem |
| [02-runtime-architecture.md](./02-runtime-architecture.md) | Startup, `SPAppController`, window/tab, seluruh menu, `.spf`, preferensi | `SPAppController`, `SPWindowController`, `TabManager`, `MainMenu.xib` |
| [03-koneksi-favorites-keychain.md](./03-koneksi-favorites-keychain.md) | State machine koneksi, 5 tipe koneksi, favorites, keychain, `mysql://` | `SPConnectionController`, `SAConnectionInfo*`, `SAKeychain*` |
| [04-database-document.md](./04-database-document.md) | Dokumen per-koneksi, subview, task/thread, sesi `.spf` | `SPDatabaseDocument` |
| [05-daftar-objek-navigator.md](./05-daftar-objek-navigator.md) | SQL daftar tabel/view/routine, popup database, pinned, navigator | `SPTablesList`, `SADatabaseListManager`, `SPNavigatorController` |
| [06-content-load-filter.md](./06-content-load-filter.md) | SELECT/LIMIT, pagination, filter, sortir, count, streaming | `SPTableContent`, `SPRuleFilterController`, `SPFilterTableController` |
| [07-content-edit-copy.md](./07-content-edit-copy.md) | Aturan sel, commit edit, INSERT/UPDATE/DELETE, copy | `SPTableContent`, `SPCopyTable`, `SPDataCellFormatter` |
| [08-sql-editor-eksekusi.md](./08-sql-editor-eksekusi.md) | Eksekusi query, `SPSQLParser`, EXPLAIN, editor (highlight/completion/snippet) | `SPCustomQuery`, `SPSQLParser`, `SPTextView` |
| [08b-query-history-favorites.md](./08b-query-history-favorites.md) | Riwayat query + favorites, skema SQLite, display format | `SPQueryController`, `SQLiteHistoryManager`, `SAQueryFavoriteStore` |
| [09-import.md](./09-import.md) | Import SQL/CSV, field mapper, encoding/Unicode | `SPDataImport`, `SPFieldMapperController`, `SACSVImportStreamDecoder` |
| [09b-export-print.md](./09b-export-print.md) | Export SQL/CSV/XML/Dot, token nama, printing | `SPExportController`, `Exporters/*`, `SAPrintUtility` |
| [10-struktur-index-field-editor.md](./10-struktur-index-field-editor.md) | Tab Structure: baca kolom, semua `ALTER TABLE`, index, auto-increment, sheet field editor + PHP-serialize | `SPTableStructure`, `SPIndexesController`, `SPFieldEditorController` |
| [11-relasi-trigger-info-metadata.md](./11-relasi-trigger-info-metadata.md) | Relasi/foreign key, trigger, Table Info, cache metadata `SPTableData`, capability server | `SPTableRelations`, `SPTableTriggers`, `SPTableInfo`, `SPExtendedTableInfo`, `SPTableData`, `SPServerSupport` |
| [12-server-tools.md](./12-server-tools.md) | User manager (grant/revoke), process list, server variables, go-to-database | `SPUserManager`, `SPProcessListController`, `SPServerVariablesController` |
| [12b-bundles-mcp-help.md](./12b-bundles-mcp-help.md) | Bundle shell + editor, server MCP + guard read-only, help viewer | `SPBundleManager`, `SPMCPServer`, `SAHelpViewer*` |
| [13-mysql-framework.md](./13-mysql-framework.md) | Framework wire-protocol MySQL: koneksi, result, streaming, escaping, capability | `Frameworks/SPMySQLFramework` |
| [14-integrasi.md](./14-integrasi.md) | SSH tunnel + IPC, AWS IAM/SSO, Vault, secure bookmark, updater, analytics, drag pasteboard | `SPSSHTunnel`, `AWS/*`, `Vault/*`, `SecureBookmark*` |
| [15-views-utilitas.md](./15-views-utilitas.md) | View/cell/control AppKit kustom, parser/formatter/utilitas, third-party | `Source/Views`, `Source/Other`, `Source/ThirdParty` |
| [90-port-mapping.md](./90-port-mapping.md) | Peta tiap subsistem macOS → modul Rust `WindowsPort/` + celah | source Rust |
| [audit/README.md](./audit/README.md) | **Audit baris-per-baris** 14 file raksasa (≥1750 baris, ~50k baris): setiap method + rentang baris + tujuan, plus inventaris ivar/pragma dan ringkasan bug/hazard | source |

## Cara pakai untuk port

Setiap dokumen subsistem mengikuti pola yang sama supaya bisa langsung dipakai
sebagai kontrak implementasi:

- **Model/data** — tipe, field, dan artinya.
- **Query & SQL** — string persis yang dikirim ke server, dengan sitasi.
- **Alur** — urutan operasi dan state.
- **Error & string** — semua teks yang terlihat pengguna, dikutip persis.
- **UNKNOWN** — hal yang belum bisa dipastikan dari source, dengan file yang
  perlu ditelusuri lebih lanjut.

Urutan pengerjaan port mengikuti dependency di
`../windows-port-feature-inventory.md`. Dokumen di sini mengisi sisi "kontrak
source" untuk tiap baris ledger.

## Status pengerjaan

| Subsistem | Dokumen | Status |
| --- | --- | --- |
| Inventaris file | 01 | selesai |
| Arsitektur runtime (window/tab/menu) | 02 | selesai |
| Koneksi + favorites + keychain | 03 | selesai |
| Database document (mode, task, session) | 04 | selesai |
| Daftar database/objek + navigator | 05 | selesai |
| Content: load/pagination/filter | 06 | selesai |
| Content: edit/copy/format | 07 | selesai |
| SQL editor + eksekusi | 08 | selesai |
| Query history + favorites | 08b | selesai |
| Import | 09 | selesai |
| Export + print | 09b | selesai |
| Structure + Indexes + Field editor | 10 | selesai |
| Relations + Triggers + Table Info + metadata | 11 | selesai |
| Server tools (user/process/variables/goto) | 12 | selesai |
| Bundles + MCP + help viewer | 12b | selesai |
| Framework MySQL (SPMySQLFramework) | 13 | selesai |
| Integrasi (SSH/AWS/Vault/keychain/updater) | 14 | selesai |
| Views & komponen UI AppKit | 15 | selesai |
| Peta port macOS → modul Rust | 90 | selesai |

Seluruh subsistem utama `Source/` (±156k baris) + `Frameworks/SPMySQLFramework`
(±26k baris) sudah dipetakan dengan sitasi. Yang belum: audit baris-per-baris
untuk beberapa file raksasa (mis. `SPDatabaseDocument.m` 6.4k baris hanya
dirangkum per seksi), dan pembuktian runtime (di luar cakupan dokumen ini).
