//! Bundle (plugin shell) engine — port `SPBundleManager` + `SPBundleCommandRunner`.
//!
//! Port audit `12b-bundles-mcp-help.md` §1 + `bundle-help-cluster.md`.
//!
//! Model mac memakai shell `/bin/bash` + env-vars `SP_*`. Port Windows/Unix ini
//! memakai shell platform-native (bash di Unix, `cmd /c` di Windows) dengan
//! env `SP_*` yang sama persis.
//!
//! Search path bundle:
//! 1. `{app_data}/Bundles/*.saBundle|*.spBundle` (doc 12b §1.1)
//! 2. Bundles datasheet ditempatkan user otomatis.

use std::collections::BTreeMap;
use std::path::PathBuf;

/// Bundle list key plist, port `SPConstants.m:435-461`.
#[derive(Debug, Clone, Default)]
pub struct BundleEntry {
    /// Key `name`.
    pub name: String,
    /// Key `scope`: `editor`, `datatable`, `inputfield`, `general`.
    pub scope: String,
    /// Key `command`: shell script body (atau `#!/bin/bash` shebang).
    pub command: String,
    /// Key `input`: `none`, `selectedtext`, `entirecontent`, dll.
    pub input: String,
    /// Key `input_fallback`
    pub input_fallback: String,
    /// Key `output`: `none`, `insertastext`, `replaceselection`, `showashtml`, dll.
    pub output: String,
    /// Key `trigger`: `none`, `databasechanged`, `tablechanged`, `tablerowchanged`.
    pub trigger: String,
    /// Key `keyEquivalent`: karakter terakhir = tombol, `^@~$` = modifiers.
    pub key_equivalent: String,
    /// Key `description`.
    pub description: String,
    /// Key `author`.
    pub author: String,
    /// Key `disabled`.
    pub disabled: bool,
    /// Path bundle directory (.saBundle/.spBundle).
    pub path: PathBuf,
}

/// Nilai enum input yang didukung (doc 12b §1.2, `SPConstants.m:407-422`).
pub const BUNDLE_INPUTS: &[&str] = &[
    "none",
    "selectedtext",
    "entirecontent",
    "currentword",
    "currentquery",
    "currentline",
    "selectedtablerowsastab",
    "selectedtablerowsascsv",
    "selectedtablerowsassqlinsert",
    "tablerowsastab",
    "tablerowsascsv",
    "tablerowsassqlinsert",
];

/// Nilai enum output yang didukung (doc 12b §1.2, `SPConstants.m:423-430`).
pub const BUNDLE_OUTPUTS: &[&str] = &[
    "none",
    "insertastext",
    "insertassnippet",
    "replaceselection",
    "replacecontent",
    "showastexttooltip",
    "showashtmltooltip",
    "showashtml",
];

/// Nilai enum trigger yang didukung (doc 12b §1.2, `SPConstants.m:431-434`).
pub const BUNDLE_TRIGGERS: &[&str] =
    &["none", "databasechanged", "tablechanged", "tablerowchanged"];

/// Nilai enum scope yang didukung (doc 12b §1.2, `SPConstants.m:403-406`).
pub const BUNDLE_SCOPES: &[&str] = &["editor", "datatable", "inputfield", "general"];

/// Katalog variabel shell `SP_*` (doc 12b §1.5, `SPConstants.m:473-521`).
pub fn shell_environment(context: &BundleContext) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    env.insert("SP_CURRENT_HOST".to_owned(), context.host.clone());
    env.insert("SP_CURRENT_PORT".to_owned(), context.port.clone());
    env.insert("SP_CURRENT_USER".to_owned(), context.user.clone());
    env.insert(
        "SP_SELECTED_DATABASE".to_owned(),
        context.selected_database.clone(),
    );
    env.insert(
        "SP_SELECTED_TABLE".to_owned(),
        context.selected_table.clone(),
    );
    env.insert(
        "SP_SELECTED_TABLES".to_owned(),
        context.selected_tables.join(","),
    );
    env.insert(
        "SP_SELECTED_ROW_INDICES".to_owned(),
        context
            .selected_row_indices
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(","),
    );
    env.insert("SP_CURRENT_WORD".to_owned(), context.current_word.clone());
    env.insert("SP_CURRENT_LINE".to_owned(), context.current_line.clone());
    env.insert("SP_CURRENT_QUERY".to_owned(), context.current_query.clone());
    env.insert("SP_SELECTED_TEXT".to_owned(), context.selected_text.clone());
    env.insert(
        "SP_USED_QUERY_FOR_TABLE".to_owned(),
        context.used_query_for_table.clone(),
    );
    env.insert("SP_RDBMS_TYPE".to_owned(), context.rdbms_type.clone());
    env.insert("SP_RDBMS_VERSION".to_owned(), context.rdbms_version.clone());
    env.insert(
        "SP_BUNDLE_SCOPE".to_owned(),
        context.bundle_scope.clone().unwrap_or_default(),
    );
    env.insert(
        "SP_CURRENT_EDITED_TABLE".to_owned(),
        context.current_edited_table.clone(),
    );
    env.insert(
        "SP_CURRENT_EDITED_COLUMN_NAME".to_owned(),
        context.current_edited_column_name.clone(),
    );
    // All-databases list.
    env.insert(
        "SP_ALL_DATABASES".to_owned(),
        context.all_databases.join(" "),
    );
    env.insert("SP_ALL_TABLES".to_owned(), context.all_tables.join(" "));
    env.insert("SP_ALL_VIEWS".to_owned(), context.all_views.join(" "));
    env.insert(
        "SP_ALL_FUNCTIONS".to_owned(),
        context.all_functions.join(" "),
    );
    env.insert(
        "SP_ALL_PROCEDURES".to_owned(),
        context.all_procedures.join(" "),
    );
    env
}

/// Konteks eksekusi bundle — mengisi variabel `SP_*`.
#[derive(Debug, Clone, Default)]
pub struct BundleContext {
    pub host: String,
    pub port: String,
    pub user: String,
    pub selected_database: String,
    pub selected_table: String,
    pub selected_tables: Vec<String>,
    pub selected_row_indices: Vec<usize>,
    pub current_word: String,
    pub current_line: String,
    pub current_query: String,
    pub selected_text: String,
    pub used_query_for_table: String,
    pub rdbms_type: String,
    pub rdbms_version: String,
    pub bundle_scope: Option<String>,
    pub current_edited_table: String,
    pub current_edited_column_name: String,
    pub all_databases: Vec<String>,
    pub all_tables: Vec<String>,
    pub all_views: Vec<String>,
    pub all_functions: Vec<String>,
    pub all_procedures: Vec<String>,
    /// Teks input yang ditulis ke file `SP_BUNDLE_INPUT` (Input Field/Data Table).
    pub bundle_input: Option<String>,
}

/// Hasil eksekusi bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleExecutionResult {
    /// Exit status — 200-208 = redirect action (mac `SPConstants.h:703-714`).
    pub exit_code: i32,
    /// Isi `SP_BUNDLE_OUTPUT` (stdout).
    pub output: String,
    /// Isi stderr (untuk alert/error display).
    pub stderr: String,
}

/// Jalankan command bundle. Port `SPBundleCommandRunner.runBashCommand:`
/// (doc 12b §1.4, `SPBundleCommandRunner.m:184-507`).
pub fn run_bundle_command(
    entry: &BundleEntry,
    context: &BundleContext,
) -> Result<BundleExecutionResult, String> {
    let temp_dir = std::env::temp_dir();
    // Counter atomik global: test berparalel dalam 1 process (pid sama),
    // jadi id dari pid saja bentrok. Mac memakai UUID per-invocation.
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let uuid = std::process::id() as u64 * 1_000_000
        + COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    // `.ps1` suffix: PowerShell -File menolak file tanpa ekstensi itu; pada
    // Unix `bash <file>` mengabaikan ekstensi, jadi satu nama untuk semua.
    let script_path = temp_dir.join(format!("SP_BUNDLE_SCRIPT_COMMAND_{uuid}.ps1"));
    let input_path = temp_dir.join(format!("SP_BUNDLE_INPUT_{uuid}"));
    let output_path = temp_dir.join(format!("SP_BUNDLE_OUTPUT_{uuid}"));

    // Tulis script body (document 12b §1.4: script mulai `#!` → simpan apa adanya,
    // else body ditulis + dijalankan dengan /bin/sh). Kita tulis apa adanya dan
    // shell platform-native di bawah.
    std::fs::write(&script_path, &entry.command)
        .map_err(|error| format!("Could not write bundle script: {error}"))?;

    // Tulis input bundle bila ada.
    if let Some(input) = &context.bundle_input {
        std::fs::write(&input_path, input)
            .map_err(|error| format!("Could not write bundle input: {error}"))?;
    }

    let env = shell_environment(context);

    // Jalankan dengan shell platform: `/bin/bash` (Unix) atau PowerShell
    // (Windows — shell native platform; script bundle di sisi Windows harus
    // ditulis PowerShell). Input bundle di-pipe ke stdin bila ada.
    #[cfg(windows)]
    let (exit_code, output) = {
        let mut cmd = std::process::Command::new("powershell");
        cmd.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"]);
        cmd.arg(&script_path);
        for (key, value) in &env {
            cmd.env(key, value);
        }
        if context.bundle_input.is_some() {
            let input_file = std::fs::File::open(&input_path)
                .map_err(|error| format!("Could not open bundle input: {error}"))?;
            cmd.stdin(std::process::Stdio::from(input_file));
        }
        let result = cmd
            .output()
            .map_err(|error| format!("Could not run bundle command: {error}"))?;
        (
            result.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&result.stdout).into_owned(),
        )
    };
    #[cfg(not(windows))]
    let (exit_code, output) = {
        // Redirect input/output lewat file temp (pola mac: SP_BUNDLE_INPUT/
        // SP_BUNDLE_OUTPUT); `none` tidak menulis input redirect — mirip mac.
        let redirect_input = match &context.bundle_input {
            Some(_) => format!(" < {}", input_path.to_string_lossy()),
            None => String::new(),
        };
        let mut cmd = std::process::Command::new("/bin/bash");
        cmd.arg("-c");
        cmd.arg(format!(
            "bash {} > {}{}",
            script_path.to_string_lossy(),
            output_path.to_string_lossy(),
            redirect_input,
        ));
        for (key, value) in &env {
            cmd.env(key, value);
        }
        // Nonaktifkan SHELL agar tidak salah (mac 632-633).
        cmd.env_remove("SHELL");
        let status = cmd
            .status()
            .map_err(|error| format!("Could not run bundle command: {error}"))?;
        let stdout = std::fs::read_to_string(&output_path).unwrap_or_default();
        (status.code().unwrap_or(-1), stdout)
    };

    // Cleanup temp files.
    let _ = std::fs::remove_file(&script_path);
    let _ = std::fs::remove_file(&input_path);
    let _ = std::fs::remove_file(&output_path);

    Ok(BundleExecutionResult {
        exit_code,
        output,
        stderr: String::new(),
    })
}

/// Search path default untuk bundle dirs — port `SPBundleManager.m:368-371`.
pub fn bundle_directory() -> PathBuf {
    #[cfg(target_os = "windows")]
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    #[cfg(not(target_os = "windows"))]
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("Sequel Ace").join("Bundles")
}

/// Muat semua bundles dari directories. Port `SPBundleManager.reloadBundles:`
/// (`SPBundleManager.m:343-718`) — file invalid di-skip.
pub fn load_bundles() -> Vec<BundleEntry> {
    let mut bundles = Vec::new();
    let dir = bundle_directory();
    if !dir.exists() {
        return bundles;
    }
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return bundles;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let is_bundle_dir = path.is_dir()
            && matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("saBundle") | Some("spBundle")
            );
        if !is_bundle_dir {
            continue;
        }
        if let Ok(loaded) = load_bundle(&path) {
            bundles.push(loaded);
        }
    }
    bundles.sort_by(|first, second| first.name.cmp(&second.name));
    bundles
}

/// Muat satu bundle `command.plist` (doc 12b §1.1: `PropertyListSerialization`).
fn load_bundle(path: &std::path::Path) -> Result<BundleEntry, String> {
    let plist_path = path.join("command.plist");
    let contents = std::fs::read_to_string(&plist_path)
        .map_err(|error| format!("Could not read command.plist: {error}"))?;
    let value: serde_json::Value = parse_plist(&contents).map_err(|error| {
        format!(
            "Could not parse command.plist ({}) as plist: {error}",
            plist_path.display()
        )
    })?;
    let object = value.as_object().ok_or("command.plist harus object")?;

    // Nama + scope wajib.
    let name = get_str(object, "name").ok_or("Missing key 'name'")?;
    let scope = get_str(object, "scope").ok_or("Missing key 'scope'")?;
    let command = get_str(object, "command").unwrap_or_default();

    // Disabled → skip (mac SPBundleManager.m:423-427: "disabled tidak didaftarkan").
    let disabled = get_bool(object, "disabled").unwrap_or(false);
    if disabled && !name.is_empty() && !scope.is_empty() {
        return Err("Bundle disabled".to_owned());
    }

    // Setelah name/scope valid, load sisanya.
    Ok(BundleEntry {
        name,
        scope,
        command,
        input: get_str(object, "input").unwrap_or_default(),
        input_fallback: get_str(object, "input_fallback").unwrap_or_default(),
        output: get_str(object, "output").unwrap_or_default(),
        trigger: get_str(object, "trigger").unwrap_or_default(),
        key_equivalent: get_str(object, "keyEquivalent").unwrap_or_default(),
        description: get_str(object, "description").unwrap_or_default(),
        author: get_str(object, "author").unwrap_or_default(),
        disabled: get_bool(object, "disabled").unwrap_or(false),
        path: path.to_path_buf(),
    })
}

/// Simpan bundle ke directory (mac `saveBundle:atPath:`, doc 12b §1.7).
pub fn save_bundle(entry: &BundleEntry) -> Result<(), String> {
    let dir = if entry.path.exists() {
        entry.path.clone()
    } else {
        bundle_directory().join(format!("{}.saBundle", entry.name.replace(['/', '\\'], "_")))
    };
    std::fs::create_dir_all(&dir)
        .map_err(|error| format!("Could not create bundle dir: {error}"))?;
    let plist_path = dir.join("command.plist");
    let content = build_plist(entry);
    std::fs::write(&plist_path, content)
        .map_err(|error| format!("Could not write command.plist: {error}"))
}

/// Hapus bundle directory.
pub fn delete_bundle(entry: &BundleEntry) -> Result<(), String> {
    if entry.path.exists() {
        std::fs::remove_dir_all(&entry.path)
            .map_err(|error| format!("Could not delete bundle: {error}"))
    } else {
        Ok(())
    }
}

/// Get nested keys dari plist object.
fn get_str(object: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<String> {
    object
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
}

/// Get bool dari plist object.
fn get_bool(object: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<bool> {
    object.get(key).and_then(serde_json::Value::as_bool)
}

/// Parse plist sederhana (XML plist) → JSON Value. Support `<dict>`, `<string>`,
/// `<integer>`, `<real>`, `<true/>`, `<false/>`.
fn parse_plist(contents: &str) -> Result<serde_json::Value, String> {
    let lines = contents.lines();
    let mut json = String::from("{");
    let mut in_dict = false;
    let mut key: Option<String> = None;
    for line in lines {
        let trimmed = line.trim();
        if trimmed == "<dict>" {
            in_dict = true;
        } else if trimmed == "</dict>" {
            in_dict = false;
        } else if in_dict {
            if let Some(key_start) = trimmed.strip_prefix("<key>") {
                if let Some(key_end) = key_start.strip_suffix("</key>") {
                    key = Some(key_end.to_owned());
                    json.push_str(&format!("\"{key_end}\":"));
                }
            } else if key.is_some() {
                if let Some(value) = trimmed
                    .strip_prefix("<string>")
                    .and_then(|rest| rest.strip_suffix("</string>"))
                {
                    json.push_str(&format!("\"{value}\","));
                } else {
                    // Integer / real / true / false.
                    let numeric = trimmed
                        .strip_prefix("<integer>")
                        .and_then(|rest| rest.strip_suffix("</integer>"));
                    if let Some(numeric) = numeric {
                        json.push_str(&format!("{numeric},"));
                    } else if trimmed == "<true/>" {
                        json.push_str("true,");
                    } else if trimmed == "<false/>" {
                        json.push_str("false,");
                    }
                }
                key = None;
            }
        }
    }
    // Hapus trailing comma.
    if json.ends_with(',') {
        json.pop();
    }
    json.push('}');
    serde_json::from_str(&json).map_err(|error| format!("Invalid plist: {error}"))
}

/// Bangun XML plist dari BundleEntry.
fn build_plist(entry: &BundleEntry) -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n");
    out.push_str("<plist version=\"1.0\">\n");
    out.push_str("<dict>\n");
    out.push_str(&plist_entry("name", &entry.name));
    out.push_str(&plist_entry("scope", &entry.scope));
    out.push_str(&plist_entry("command", &entry.command));
    out.push_str(&plist_entry("input", &entry.input));
    out.push_str(&plist_entry("input_fallback", &entry.input_fallback));
    out.push_str(&plist_entry("output", &entry.output));
    out.push_str(&plist_entry("trigger", &entry.trigger));
    out.push_str(&plist_entry("keyEquivalent", &entry.key_equivalent));
    out.push_str(&plist_entry("description", &entry.description));
    out.push_str(&plist_entry("author", &entry.author));
    if entry.disabled {
        out.push_str("\t<key>disabled</key>\n\t<true/>\n");
    }
    out.push_str("</dict>\n");
    out.push_str("</plist>\n");
    out
}

/// Satu key-value untuk plist builder.
fn plist_entry(key: &str, value: &str) -> String {
    format!("\t<key>{key}</key>\n\t<string>{value}</string>\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_entry() -> BundleEntry {
        BundleEntry {
            name: "Test Bundle".to_owned(),
            scope: "datatable".to_owned(),
            command: "echo SP_SELECTED_TABLE=$SP_SELECTED_TABLE".to_owned(),
            input: "none".to_owned(),
            input_fallback: "none".to_owned(),
            output: "showastexttooltip".to_owned(),
            trigger: "none".to_owned(),
            key_equivalent: "^t".to_owned(),
            description: "test".to_owned(),
            author: "tester".to_owned(),
            disabled: false,
            path: PathBuf::from("/tmp"),
        }
    }

    #[test]
    fn shell_environment_contains_basic_vars() {
        let context = BundleContext {
            host: "localhost".to_owned(),
            port: "3306".to_owned(),
            user: "root".to_owned(),
            selected_database: "mydb".to_owned(),
            selected_table: "tbl".to_owned(),
            rdbms_type: "MySQL".to_owned(),
            rdbms_version: "8.0".to_owned(),
            ..Default::default()
        };
        let env = shell_environment(&context);
        assert_eq!(env.get("SP_CURRENT_HOST").unwrap(), "localhost");
        assert_eq!(env.get("SP_CURRENT_PORT").unwrap(), "3306");
        assert_eq!(env.get("SP_CURRENT_USER").unwrap(), "root");
        assert_eq!(env.get("SP_SELECTED_DATABASE").unwrap(), "mydb");
        assert_eq!(env.get("SP_SELECTED_TABLE").unwrap(), "tbl");
        assert_eq!(env.get("SP_RDBMS_TYPE").unwrap(), "MySQL");
        assert_eq!(env.get("SP_RDBMS_VERSION").unwrap(), "8.0");
    }

    #[test]
    fn shell_environment_multi_tables() {
        let context = BundleContext {
            selected_tables: vec!["a".to_owned(), "b".to_owned()],
            selected_row_indices: vec![1, 3, 5],
            all_databases: vec!["db1".to_owned(), "db2".to_owned()],
            ..Default::default()
        };
        let env = shell_environment(&context);
        assert_eq!(env.get("SP_SELECTED_TABLES").unwrap(), "a,b");
        assert_eq!(env.get("SP_SELECTED_ROW_INDICES").unwrap(), "1,3,5");
        assert_eq!(env.get("SP_ALL_DATABASES").unwrap(), "db1 db2");
    }

    #[test]
    #[cfg(not(windows))]
    fn run_bundle_command_echoes_env() {
        let entry = test_entry();
        let context = BundleContext {
            selected_table: "mytable".to_owned(),
            ..Default::default()
        };
        let result = run_bundle_command(&entry, &context).unwrap();
        assert!(result.output.contains("SP_SELECTED_TABLE=mytable"));
    }

    #[test]
    fn run_bundle_command_exit_code() {
        let mut entry = test_entry();
        entry.command = "exit 201".to_owned();
        let result = run_bundle_command(&entry, &BundleContext::default()).unwrap();
        assert_eq!(result.exit_code, 201);
    }

    #[test]
    fn build_plist_round_trip() {
        let entry = test_entry();
        let plist = build_plist(&entry);
        let parsed = parse_plist(&plist).unwrap();
        assert_eq!(parsed.get("name").unwrap(), "Test Bundle");
        assert_eq!(parsed.get("scope").unwrap(), "datatable");
        assert_eq!(parsed.get("output").unwrap(), "showastexttooltip");
    }

    #[test]
    fn bundle_directory_name_pattern() {
        // Mac: `~/Library/Application Support/Sequel Ace/Bundles`
        // Windows: `%APPDATA%\Sequel Ace\Bundles`
        let dir = bundle_directory();
        assert!(
            dir.to_string_lossy().contains("Sequel Ace")
                && dir.to_string_lossy().ends_with("Bundles"),
            "dir = {}",
            dir.display()
        );
    }
}
