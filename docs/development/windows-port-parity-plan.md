# Windows port: parity ledger

This is the working checklist for a gradual Windows port of Sequel Ace. The
repository source at the chosen commit is the behavior baseline. Compare each
Windows milestone with a macOS build from that **same commit**; an installed
older release is useful for historical behavior, but cannot establish current
feature parity. Record the commit and both application versions with every
comparison.

The [source-derived behavior map](windows-port-behavior-map.md) now records
the concrete behavior and known Rust differences for every implemented area.
Read and extend that map before adding a feature. A compiled executable is a
delivery artifact, not a parity result; no current Rust feature has passed a
same-commit macOS/Windows runtime comparison.

## Evidence rule

For each feature, record (1) its source entry point and data flow, (2) visible
states and error cases observed in the matching macOS build, (3) Windows
implementation and tests, and (4) remaining differences. Use REA for focused
inspection of a shipped binary or passive runtime observation when source and
ordinary testing leave a specific behavior unclear. A binary string or menu
entry alone does not prove that a feature works. Do not treat a 5.4.0 binary as
the reference for 6.0.3 source behavior.

## First milestone: connect and browse

| Behavior to match | Source of truth | Parity checks |
| --- | --- | --- |
| Connection types and fields | `Source/Model/SAConnectionInfo.swift`, `Source/Controllers/MainViewControllers/ConnectionView/SAConnectionFormView.swift` | TCP/IP, local socket, SSH tunnel, AWS IAM, Vault; field defaults, validation, saved favorites, password handling |
| Connection setup | `Source/Other/Utility/SAConnectionService.swift`, `Frameworks/SPMySQLFramework/Source/SPMySQLConnection.m` | Host and port defaults, TLS and client authentication, timeout, cancel, server errors, optional initial database, character set and time zone |
| Database list | `Source/Controllers/MainViewControllers/SPDatabaseDocument.m`, `Source/Controllers/MainViewControllers/SADatabaseListManager.swift` | Refresh, current selection, system/user grouping, empty list, insufficient privileges, connection loss |
| Table and object list | `Source/Controllers/SubviewControllers/SPTablesList.m` | Tables, views and routines; ordering, refresh, selection and server errors |
| Connection persistence | `Source/Controllers/DataControllers/SPFavoritesController.m`, `Source/Controllers/MainViewControllers/ConnectionView/SAFavoriteItem.swift`, `Source/Other/Keychain/SAKeychain.swift` | Favorites and passwords; existing `.spf` and preferences formats remain readable |

The first usable Windows build should connect to a test MySQL/MariaDB server,
list databases, select one, and show its tables and views. Its first parity
review must also include failed credentials, unavailable server, empty schema,
and a user without `SHOW DATABASES` privilege. Advanced authentication and SSH
remain tracked in this milestone even if delivered in later increments.

The initial Rust slice is now in `WindowsPort/`: TCP/IP connection, optional
initial and manual database selection, database/table filtering and name
copying, table/view column and CREATE SQL inspection, basic server information,
SQL execution with bounded result grids, session-only query history,
displayed-result CSV copy, table info/index/foreign-key inspection,
process-list shortcut and server-variable view, a read-only 1000-row paginated
Content grid, refresh and disconnect. It compiles
on macOS; the existing source-level unit tests were last run before the SQL
slice was added. Windows 11 execution, live-server scenarios, and parity against a
matching macOS build remain unverified. An x86-64 Windows GUI PE was
cross-compiled from macOS with Rust 1.93.1 and MinGW-w64; compilation alone
does not establish Windows runtime or database protocol parity.
The Content default is 1000 in the source, but its limit is configurable and
can be disabled. The 1000-row SQL display cap in the Rust port is independent
and is not an original query-editor default. The invented Preview Rows
shortcut has been removed after the source audit.

## Remaining feature groups

The [source-based feature inventory](windows-port-feature-inventory.md) lists
each group, the small features feasible from the current Rust baseline, and
their dependencies. Inventory each group down to individual actions and edge
cases before calling it complete. The menu in `Source/Interfaces/MainMenu.xib`
is a useful index, but controller code and a matching macOS run determine
actual behavior.

| Group | Principal source locations | Status |
| --- | --- | --- |
| Table structure, content, relations, triggers and info | `Source/Controllers/MainViewControllers/TableStructure/`, `TableContent/`, `TableRelations/`, `TableTriggers/` | Group inventoried; detailed parity pending |
| SQL editor, execution, results and query history/favorites | `Source/Controllers/MainViewControllers/SPCustomQuery.m`, `Source/Controllers/SubviewControllers/SPQueryController.m`, `Source/Controllers/Other/SQLiteHistoryManager.swift` | Group inventoried; detailed parity pending |
| Database and table creation, changes, copy and rename | `Source/Other/DatabaseActions/`, `Source/Controllers/SubviewControllers/SPTablesList.m` | Group inventoried; detailed parity pending |
| Import and export formats and options | `Source/Controllers/DataImport/`, `Source/Controllers/DataExport/` | Group inventoried; detailed parity pending |
| Session files, favorites and preferences | `Source/Controllers/MainViewControllers/SPDatabaseDocument.m`, `Source/Controllers/Preferences/`, `Source/Controllers/DataControllers/SPFavoritesController.m` | Group inventoried; detailed parity pending |
| Bundles, printing, help, notifications and other integrations | `Source/Controllers/BundleSupport/`, `Source/Other/`, `Source/Views/` | Group inventoried; detailed parity pending |

## Delivery sequence

1. Fix a source commit and produce a matching macOS reference build. Capture
   connection and database-list states without storing credentials in evidence.
2. Choose a Windows UI and database-client stack. Keep platform-specific UI,
   credential storage, sockets and SSH behind interfaces; preserve the existing
   macOS application and its persisted formats.
3. Deliver TCP/IP connection, database list and table list with real-server
   integration tests and a side-by-side parity review.
4. Extend connection parity (TLS, sockets, SSH, favorites, AWS IAM, Vault), then
   SQL editing/results, table editing, import/export and remaining groups.
5. Mark a feature complete only when its behavior, errors, keyboard actions,
   persistence and relevant compatibility fixtures match the reference.

## Windows implementation

The user selected Rust for the Windows port on 2026-10-14, overriding the
repository's Swift-only rule for this separate `WindowsPort/` project. The
existing macOS application remains under the normal Swift modernization rule.
The first runnable slice uses `eframe`/`egui`, following PhotoCraft's broad
engine-plus-UI structure, with the pure Rust `mysql` crate for TCP/IP access.
This is a delivery choice for the first slice, not evidence that this driver
matches every `libmysqlclient` authentication, TLS, and encoding behavior.

The existing `SPMySQLFramework` calls `libmysqlclient` but wraps it in Cocoa
objects, so it cannot be compiled unchanged for Windows. Keep the Rust driver
behind a small interface and compare it with the original MySQL client on real
servers before expanding connection types. Replace or supplement it if full
parity requires `libmysqlclient` bindings.

The first slice passes only when a Windows executable can connect to a real
test server, show databases and tables, report authentication/network errors,
and close cleanly. Run the same scenarios against the matching macOS build.
The macOS source currently cannot be built in this environment without Xcode.

Technology references (checked 2026-10-14):

- [PhotoCraft architecture](https://github.com/storytold/photocraft)
- [egui and eframe](https://github.com/emilk/egui)
- [Rust MySQL client](https://docs.rs/mysql/latest/mysql/)

## Current baseline note

`/Applications/Sequel Ace.app` is still version 5.4.0 (`CFBundleVersion`
20109). The user has placed a 6.0.3 (20116) copy at
`~/Downloads/Sequel Ace.app`; use that exact path for release comparison.
The checked-out repository is commit `02c910932`;
`Resources/Plists/Info.plist` declares 6.0.3 and the Xcode project declares
`CURRENT_PROJECT_VERSION` 20116. Neither app copy alone is an exact parity
oracle for this checkout.
The official 6.0.3 (20116) release tag is `production/6.0.3-20116` at
`08fc8596`, 24 commits behind this checkout. Use that tag's source as the
6.0.3 release reference and audit later commits
separately.
This macOS environment currently has only Xcode Command Line Tools selected
and no `/Applications/Xcode.app`; `xcodebuild` cannot produce the matching
reference app until full Xcode is available.
