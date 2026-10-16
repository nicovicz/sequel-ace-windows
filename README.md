# Sequel Ace Windows port (Rust)

A native Windows port of Sequel Ace built in Rust with an `eframe`/`egui` UI.
It links the **same `libmysqlclient` C library the macOS app bundles** via
`mysqlclient-sys`, so the port shares Sequel Ace's wire protocol, auth
plugins, TLS behavior, escaping, and error text instead of reimplementing
them.

Features are ported per phase against the macOS source trees — behavior
contract comes from the audit documents in
[`docs/development/windows-port/audit/`](../docs/development/windows-port/audit/)
(`SPTableContent`, `SPCopyTable`, `SPTableFilterParser`, `SPSSHTunnel`,
`SAKeychain`, `SPBundleManager`, `SPUserManager`, `SPTableStructure`, …).

## Feature inventory (phases 1–15)

| Area | What works |
| --- | --- |
| Connection | TCP/IP; Unix socket; SSH tunnel (key file / password / SSH agent); `SSH_ASKPASS` handled by a dedicated helper binary so SSH passwords never touch disk; favorite connections saved as JSON with passwords in the OS credential store (Windows Credential Manager / macOS Keychain); `Allow LOCAL_DATA_INFILE`, `Enable Cleartext plugin`, `Require SSL`, `Get Public Key` checkboxes drive the same `MYSQL_OPT_*` options as macOS |
| Browser | Database list (system/user split), table/view list with filter, per-object columns, indexes, foreign keys, `SHOW CREATE` text, engine/row-count/size details |
| Content | Paged reads (1000 rows/page like the macOS default), edit cell (with hex/`X'…'` validation for binary), add/duplicate/delete rows (single, multi-select, all), `Filter by Selected Value` via the 65-operator `ContentFilters.plist` operator catalog, blob-as-text display |
| Copy | `SPCopyTable` rules: Tab, Tab+Headers, CSV, SQL INSERT, SQL INSERT (no auto_inc) on both Content and Query grids, session-escaped (`NO_BACKSLASH_ESCAPES` doubling quotes like macOS) |
| Structure | Add / edit / drop column, rename table, drop table, reset `AUTO_INCREMENT`; create / drop database (`utf8mb4`) |
| SQL editor | Multi-statement execution with multiple result sets (up to 1000 rows shown per set), query history (persisted, deduped, capped) and query favorites (persisted) |
| User Manager | List users (`mysql.user`), create user (+ initial grant), drop user, `FLUSH PRIVILEGES` |
| Import / Export | CSV export, SQL INSERT export, SQL file import (file dialogs) |
| Bundles | `.saBundle`/`.spBundle` plugin shell scripts with the macOS `SP_*` environment variables (24 vars: `SP_CURRENT_HOST`, `SP_SELECTED_TABLE`, `SP_ALL_DATABASES`, …), scope/input/output/trigger categories, key equivalents, a Bundle Editor for create/edit/delete/test-run |
| Server tools | Server info, all variables, process list in SQL results |

Not yet ported: AWS IAM / Vault auth plugins, security-scoped bookmarks
(mac-only concept), and the TLS certificate file pickers.

## Build on Windows

The Windows build targets the **MSVC x64** toolchain only — MySQL ships its
Windows client library as MSVC, and vcpkg's `libmysql` port declares `!mingw`.
`build-windows.sh` documents why macOS→Windows cross-compilation is not
supported instead of attempting one.

Development happens **locally**: there is no CI-based build gate. The optional
`.github/workflows/windows_port.yml` wraps the same script if a runner is ever
wanted.

### Locally (the supported path)

Requires [rustup](https://rustup.rs/) plus the MSVC C++ build tools Rust
needs, and `libmysqlclient` in one of two layouts:

**vcpkg (recommended).** Install once:

```powershell
vcpkg install libmysql:x64-windows
$env:VCPKG_ROOT = '<vcpkg installation directory>'
```

then build everything, run the checks, and assemble the deployable folder:

```powershell
.\build-windows.ps1
```

The script installs the client library through vcpkg if it is missing, reads
the installed client version from `mysql_version.h`, runs
`fmt --check` → `test` → `clippy -D warnings` → release build, and writes a
`dist\` folder:

```
dist\
├── SequelAce-Windows-x64.exe   main GUI
├── sequel-ace-askpass.exe      SSH_ASKPASS helper (SSH password auth)
└── libmysql.dll                (+ any OpenSSL/zlib/zstd/SSL DLLs it needs)
```

All three pieces of the artifact folder must stay in one directory — the
`.exe`es resolve `libmysql.dll` next to themselves, and the main app finds
`sequel-ace-askpass.exe` next to itself for SSH password authentication.

**An already installed MySQL client.** Most setups need none of this: the
script auto-detects an extracted `mysql-8.4.x-winx64.zip` under `C:\mysql*\`
or an MSI-installed server under `C:\Program Files\MySQL\`. It reads the
client version from `mysql_version.h` and sets the `MYSQLCLIENT_*` variables
itself — a plain `.\build-windows.ps1` is enough. Only set the variables
explicitly when the install lives somewhere unusual:

```powershell
$env:MYSQLCLIENT_LIB_DIR     = 'C:\Program Files\MySQL\<version>\lib'
$env:MYSQLCLIENT_INCLUDE_DIR = 'C:\Program Files\MySQL\<version>\include'
$env:MYSQLCLIENT_VERSION     = '8.4.11'
.\build-windows.ps1
```

On a Windows **ARM64** host (UTM/Parallels on Apple Silicon) the script also
sets `CARGO_BUILD_TARGET=x86_64-pc-windows-msvc` automatically and
cross-compiles with the native ARM64 compiler; the resulting binaries are
plain x64 that run under the OS's Prism emulation.

Either way the script verifies that both binaries exist and refuses to build
an incomplete artifact.

### Driver constraints

`mysqlclient-sys` 0.5 supports MySQL 5.7–9.5 and MariaDB 3.1/3.3/3.4 with
pre-generated bindings per version, so no bindgen runs during the build. The
 dynamic `x64-windows` triplet gives the classical layout — `libmysql.dll`
 plus the `libmysql.lib` import library, exactly what `MYSQLCLIENT_LIBNAME`
 defaults to. The parity target is the bundled **MySQL 8.4.11** client
 (`Frameworks/libmysqlclient/`); other versions compile and run but change
 cipher defaults, auth plugins, and error text.

## Development on macOS

The full app runs natively on macOS with `cargo run --locked` from
`WindowsPort/`; the Windows-only part is the final MSVC compile. For the test
environment export the client library paths (ServBay 8.0 layout shown):

```sh
export MYSQLCLIENT_LIB_DIR=/Applications/ServBay/package/mysql/8.0/8.0.46/lib
export MYSQLCLIENT_INCLUDE_DIR=/Applications/ServBay/package/mysql/8.0/8.0.46/include
export MYSQLCLIENT_VERSION=8.0.46
export DYLD_LIBRARY_PATH="$MYSQLCLIENT_LIB_DIR"   # loader path for tests/run
cargo test --locked
cargo run --locked
```

Three live-server tests are `#[ignore]`d and run against a real MySQL server:

```sh
SA_PROBE_HOST=127.0.0.1 SA_PROBE_PORT=3306 SA_PROBE_USER=root \
SA_PROBE_PASSWORD=root cargo test --locked -- --ignored
```

Coding conventions (no new Objective-C is added here — this folder is its own
Rust crate; the macOS rules stay in `AGENTS.md` at the repo root):

```sh
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

## Architecture

```
WindowsPort/src/
├── main.rs                    entry point (Windows subsystem + eframe boot)
├── bin/sequel-ace-askpass.rs  second [[bin]]: echoes SSH_ASKPASS_PASSWORD
├── app.rs                     application state, response dispatch, toolbar
├── app/                       screens
│   ├── connection.rs          connection form (TCP/Socket/SSH tabs)
│   ├── browser.rs             database + table sidebar lists
│   ├── content.rs             Content grid, filter prompts, edit sheet
│   ├── grid.rs                shared egui data-grid with selection/context menu
│   ├── query.rs               SQL editor + results
│   ├── structure.rs           Structure tab (ALTER/RENAME/DROP) + User Manager
│   ├── bundle_editor.rs       Bundle Editor window
│   ├── favorites.rs           connection favorites (JSON)
│   ├── import_export.rs       CSV/SQL export + SQL import
│   └── query_store.rs         query history + favorites persistence
└── database.rs + database/    the data layer (worker thread + SQL generation)
    ├── worker.rs              one background thread owning the connection
    ├── client.rs              `mysqlclient-sys` wrapper mirroring SPMySQLConnection
    ├── metadata.rs            SHOW-based schema reads
    ├── content.rs             paged content reads
    ├── edit.rs                UPDATE/DELETE/INSERT plan builder
    ├── filter.rs              ContentFilters.plist operator catalog
    ├── copy.rs                SPCopyTable copy formats
    ├── table_structure.rs     ALTER TABLE / CREATE/DROP DATABASE SQL
    ├── user_manager.rs        CREATE/GRANT/REVOKE/DROP USER SQL
    ├── bundle.rs              bundle loading, SP_* environment, command runner
    ├── ssh_tunnel.rs          ssh process management
    ├── keychain.rs            OS credential store + SSH_ASKPASS plumbing
    └── query.rs               multi-statement query execution
```

The UI thread never touches the connection directly — everything flows
through `Request` values into the worker and `Response` values back, so data
grid behavior stays responsive.

## Parity verification workflow

Each ported phase follows the same rhythm documented in the audits:

1. Read the macOS source and record the behavior contract under
   `docs/development/windows-port/audit/`.
2. Implement the SQL generation / data step in `src/database/` with
   unit tests asserting exact SQL strings against the macOS reference
   (`quote_identifier`, session-aware escaping, `NO_BACKSLASH_ESCAPES`).
3. Wire that behavior in `src/app/` with the egui screen.
4. Verify against a live server using `--ignored` probe tests, then repeat the
   same SQL on macOS against the same server and diff the results.

Known deviations from the macOS app (deliberate or pending) live in
[`windows-port-behavior-map.md`](../docs/development/windows-port-behavior-map.md);
broader plans and status are tracked in
[`windows-port-parity-plan.md`](../docs/development/windows-port-parity-plan.md)
and [`windows-port-feature-inventory.md`](../docs/development/windows-port-feature-inventory.md).
