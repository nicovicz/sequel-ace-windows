//! Filter "by selected value" — port dari `Resources/Plists/ContentFilters.plist`
//! + `SPTableFilterParser.m`.
//!
//! Group `bit`/`number`/`string`/`date`/`spatial` diambil persis dari plist
//! (MenuLabel/Clause/NumberOfArguments/SuppressLeadingFieldPlaceholder).
//! Pemetaan tipe kolom ke group mengikuti `SACellFilterOperator`
//! (doc `audit/content-helper-swift-cluster.md` §3) yang menerima `typegrouping`
//! dari `SPTableData.m:1443-1477`.
//!
//! Template token dikartauji `SPTableFilterParser.m:78-140`:
//! `$CURRENT_FIELD` → name kolom (backtick-quoted), `$BINARY ` → `BINARY `
//! hanya bila case-sensitive, `${}` → argument (escaping dari caller, karena
//! parser mac adalah regex-escaper; di port, setiap argumen di-escape oleh
//! session agar `NO_BACKSLASH_ESCAPES` ikut benar — doc 13 §4).

use super::client::ColumnMeta;
use super::metadata::quote_identifier;

/// Satu entri op filter, dikutip verbatim dari `ContentFilters.plist`.
pub struct FilterDef {
    pub menu_label: &'static str,
    pub clause: &'static str,
    pub number_of_arguments: u32,
    /// `SuppressLeadingFieldPlaceholder` di plist.
    pub suppress_leading_field: bool,
}

const fn def(
    menu_label: &'static str,
    clause: &'static str,
    number_of_arguments: u32,
    suppress_leading_field: bool,
) -> FilterDef {
    FilterDef {
        menu_label,
        clause,
        number_of_arguments,
        suppress_leading_field,
    }
}

const BIT: &[FilterDef] = &[
    def("=", "= CAST('${}' AS DECIMAL(65,30))", 1, false),
    def("≠", "!= CAST('${}' AS DECIMAL(65,30))", 1, false),
    def(">", "> CAST('${}' AS DECIMAL(65,30))", 1, false),
    def("<", "< CAST('${}' AS DECIMAL(65,30))", 1, false),
    def("≥", ">= CAST('${}' AS DECIMAL(65,30))", 1, false),
    def("≤", "<= CAST('${}' AS DECIMAL(65,30))", 1, false),
    def("IN", "IN (${})", 1, false),
    def(
        "NOT IN __ OR NULL",
        "($CURRENT_FIELD NOT IN (${}) OR $CURRENT_FIELD IS NULL)",
        1,
        true,
    ),
    def("LIKE", "LIKE '${}'", 1, false),
    def(
        "BETWEEN",
        "BETWEEN CAST('${}' AS DECIMAL(65,30)) AND CAST('${}' AS DECIMAL(65,30))",
        2,
        false,
    ),
    def("IS NULL", "IS NULL", 0, false),
    def("IS NOT NULL", "IS NOT NULL", 0, false),
];

const NUMBER: &[FilterDef] = &[
    def("=", "= '${}'", 1, false),
    def("≠", "!= '${}'", 1, false),
    def(">", "> '${}'", 1, false),
    def("<", "< '${}'", 1, false),
    def("≥", ">= '${}'", 1, false),
    def("≤", "<= '${}'", 1, false),
    def("IN", "IN (${})", 1, false),
    def(
        "NOT IN __ OR NULL",
        "($CURRENT_FIELD NOT IN (${}) OR $CURRENT_FIELD IS NULL)",
        1,
        true,
    ),
    def("LIKE", "LIKE '${}'", 1, false),
    def("BETWEEN", "BETWEEN '${}' AND '${}'", 2, false),
    def("IS NULL", "IS NULL", 0, false),
    def("IS NOT NULL", "IS NOT NULL", 0, false),
];

const STRING: &[FilterDef] = &[
    def("=", "= $BINARY '${}'", 1, false),
    def("≠", "!= $BINARY '${}'", 1, false),
    def("LIKE", "LIKE $BINARY '${}'", 1, false),
    def("NOT LIKE", "NOT LIKE $BINARY '${}'", 1, false),
    def("contains", "LIKE $BINARY '%${}%'", 1, false),
    def("does not contain", "NOT LIKE $BINARY '%${}%'", 1, false),
    def("starts with", "LIKE $BINARY '${}%'", 1, false),
    def("does not start with", "NOT LIKE $BINARY '${}%'", 1, false),
    def("ends with", "LIKE $BINARY '%${}'", 1, false),
    def("does not end with", "NOT LIKE $BINARY '%${}'", 1, false),
    def("matches RegExp", "REGEXP $BINARY '${}'", 1, false),
    def(
        "does not match RegExp",
        "NOT REGEXP $BINARY '${}'",
        1,
        false,
    ),
    def("IN", "IN (${})", 1, false),
    def(
        "NOT IN __ OR NULL",
        "($CURRENT_FIELD NOT IN (${}) OR $CURRENT_FIELD IS NULL)",
        1,
        true,
    ),
    def("BETWEEN", "BETWEEN $BINARY '${}' AND '${}'", 2, false),
    def("IS NULL", "IS NULL", 0, false),
    def("IS NOT NULL", "IS NOT NULL", 0, false),
    def("is empty", "LIKE ''", 0, false),
    def("is not empty", "NOT LIKE ''", 0, false),
    def("length =", "CHAR_LENGTH($CURRENT_FIELD) = ${}", 1, true),
    def("length >", "CHAR_LENGTH($CURRENT_FIELD) > ${}", 1, true),
    def("length <", "CHAR_LENGTH($CURRENT_FIELD) < ${}", 1, true),
    def("= (Hex String)", "= $BINARY UNHEX('${}')", 1, false),
];

const DATE: &[FilterDef] = &[
    def("=", "= '${}'", 1, false),
    def("≠", "!= '${}'", 1, false),
    def("is after", "> '${}'", 1, false),
    def("is before", "< '${}'", 1, false),
    def("is after or equal to", ">= '${}'", 1, false),
    def("is before or equal to", "<= '${}'", 1, false),
    def("BETWEEN", "BETWEEN '${}' AND '${}'", 2, false),
    def("IS NULL", "IS NULL", 0, false),
    def("IS NOT NULL", "IS NOT NULL", 0, false),
];

const SPATIAL: &[FilterDef] = &[
    def(
        "contains",
        "MBRContains(ST_GeomFromText('${}'),$CURRENT_FIELD)",
        1,
        true,
    ),
    def(
        "within",
        "MBRWithin(ST_GeomFromText('${}'),$CURRENT_FIELD)",
        1,
        true,
    ),
    def(
        "disjoint",
        "MBRDisjoint(ST_GeomFromText('${}'),$CURRENT_FIELD)",
        1,
        true,
    ),
    def(
        "equal",
        "MBREqual(ST_GeomFromText('${}'),$CURRENT_FIELD)",
        1,
        true,
    ),
    def(
        "intersects",
        "MBRIntersects(ST_GeomFromText('${}'),$CURRENT_FIELD)",
        1,
        true,
    ),
    def(
        "overlaps",
        "MBROverlaps(ST_GeomFromText('${}'),$CURRENT_FIELD)",
        1,
        true,
    ),
    def(
        "touches",
        "MBRTouches(ST_GeomFromText('${}'),$CURRENT_FIELD)",
        1,
        true,
    ),
    def("IS NULL", "IS NULL", 0, false),
    def("IS NOT NULL", "IS NOT NULL", 0, false),
];

/// Definisi filter untuk satu group (`bit`, `number`, `string`, `date`, `spatial`).
pub fn group_definitions(group: &str) -> Option<&'static [FilterDef]> {
    match group {
        "bit" => Some(BIT),
        "number" => Some(NUMBER),
        "string" => Some(STRING),
        "date" => Some(DATE),
        "spatial" => Some(SPATIAL),
        _ => None,
    }
}

/// `typegrouping` kolom (mac: `SPTableData.m:1443-1477`), tapi dari
/// `MYSQL_FIELD` bawaan protokol (doc 13 §4) alih-alih teks `SHOW COLUMNS`.
pub fn typegrouping(meta: &ColumnMeta) -> &'static str {
    use ffi::enum_field_types as t;
    use mysqlclient_sys as ffi;
    // charset 63 = binary collation; TEXT ikut kolom BLOB dengan charset teks.
    let is_binary_charset = meta.charsetnr == 63;
    match meta.field_type {
        t::MYSQL_TYPE_BIT => "bit",
        t::MYSQL_TYPE_TINY
        | t::MYSQL_TYPE_SHORT
        | t::MYSQL_TYPE_LONG
        | t::MYSQL_TYPE_LONGLONG
        | t::MYSQL_TYPE_INT24 => "integer",
        t::MYSQL_TYPE_FLOAT
        | t::MYSQL_TYPE_DOUBLE
        | t::MYSQL_TYPE_DECIMAL
        | t::MYSQL_TYPE_NEWDECIMAL => "float",
        t::MYSQL_TYPE_DATE
        | t::MYSQL_TYPE_TIME
        | t::MYSQL_TYPE_DATETIME
        | t::MYSQL_TYPE_TIMESTAMP
        | t::MYSQL_TYPE_NEWDATE
        | t::MYSQL_TYPE_YEAR => "date",
        // CHAR/VARCHAR; MySQL 8+ merek `BINARY`/`VARBINARY` dengan flag binary.
        t::MYSQL_TYPE_STRING | t::MYSQL_TYPE_VAR_STRING
            if meta.flags & 128 != 0 && is_binary_charset =>
        {
            "binary"
        }
        t::MYSQL_TYPE_STRING | t::MYSQL_TYPE_VAR_STRING => "string",
        t::MYSQL_TYPE_ENUM | t::MYSQL_TYPE_SET => "enum",
        t::MYSQL_TYPE_JSON => "textdata",
        t::MYSQL_TYPE_TINY_BLOB
        | t::MYSQL_TYPE_MEDIUM_BLOB
        | t::MYSQL_TYPE_LONG_BLOB
        | t::MYSQL_TYPE_BLOB => {
            if is_binary_charset {
                "blobdata"
            } else {
                "textdata"
            }
        }
        t::MYSQL_TYPE_GEOMETRY => "geometry",
        _ => "blobdata",
    }
}

/// Group filter yang boleh dipilih untuk kolom ini; `None` = kolom tak punya
/// menu filter (`SACellFilterOperator.ruleFilterGroup(forTypeGrouping:)`).
/// `bit` menggunakan group definisi `bit`, lainnya mengikuti group tipe.
pub fn filter_group(meta: &ColumnMeta) -> Option<&'static str> {
    match typegrouping(meta) {
        "bit" => Some("bit"),
        "integer" | "float" => Some("number"),
        "string" | "binary" | "textdata" | "blobdata" | "enum" => Some("string"),
        "date" => Some("date"),
        "geometry" => Some("spatial"),
        _ => None,
    }
}

/// Spesifik filta yang disimpan di state UI/worker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterSpec {
    pub column: String,
    /// group dari plist (`bit`/`number`/`string`/`date`/`spatial`).
    pub group: String,
    pub menu_label: String,
    pub values: Vec<String>,
    /// `$BINARY ` diikut-kan bila `true` (mac: Shift saat invoke).
    pub case_sensitive: bool,
}

impl FilterSpec {
    pub fn from_def(
        column: impl Into<String>,
        group: impl Into<String>,
        chosen: &FilterDef,
        values: Vec<String>,
        case_sensitive: bool,
    ) -> Self {
        Self {
            column: column.into(),
            group: group.into(),
            menu_label: chosen.menu_label.to_owned(),
            values,
            case_sensitive,
        }
    }
}

/// Ganti token literal (mis. `$CURRENT_FIELD`, `$BINARY `) di clause; token
/// diawali backslash dianggap teks pengguna dan dibiarkan
/// (`SPTableFilterParser.m:85-89`).
fn replace_token(clause: &str, token: &str, replacement: &str) -> String {
    let bytes = clause.as_bytes();
    let mut out = String::with_capacity(clause.len() + replacement.len());
    let mut cursor = 0usize;
    while let Some(offset) = clause[cursor..].find(token) {
        let start = cursor + offset;
        let escaped = start > 0 && bytes[start - 1] == b'\\';
        out.push_str(&clause[cursor..start]);
        if escaped {
            out.push_str(token);
        } else {
            out.push_str(replacement);
        }
        cursor = start + token.len();
    }
    out.push_str(&clause[cursor..]);
    out
}

/// Hitung sisa `${}` (dengan pencegahan escape tanda backslash).
fn placeholder_count(clause: &str) -> usize {
    let bytes = clause.as_bytes();
    let mut cursor = 0usize;
    let mut count = 0usize;
    while let Some(offset) = clause[cursor..].find("${}") {
        let start = cursor + offset;
        let escaped = start > 0 && bytes[start - 1] == b'\\';
        if !escaped {
            count += 1;
        }
        cursor = start + "${}".len();
    }
    count
}

/// Isi tiap `${}` dengan argumen berikutnya (`SPTableFilterParser.m:118-134`).
fn substitute_placeholders(clause: &str, arguments: &[&str]) -> String {
    let bytes = clause.as_bytes();
    let capacity = clause.len()
        + arguments
            .iter()
            .map(|argument| argument.len())
            .sum::<usize>();
    let mut out = String::with_capacity(capacity);
    let mut cursor = 0usize;
    let mut next = 0usize;
    while let Some(offset) = clause[cursor..].find("${}") {
        let start = cursor + offset;
        let escaped = start > 0 && bytes[start - 1] == b'\\';
        out.push_str(&clause[cursor..start]);
        if escaped {
            out.push_str("${}");
        } else {
            out.push_str(arguments.get(next).copied().unwrap_or(""));
            next += 1;
        }
        cursor = start + "${}".len();
    }
    out.push_str(&clause[cursor..]);
    out
}

/// Escaper yang disuntikkan untuk argumen (`${}`) — di port, argumen di-escape
/// oleh session (`Conn::escape_string`), sedangkan parser melakukannya manual.
pub type Escaper<'a> = &'a dyn Fn(&str) -> String;

/// Susun klausa WHERE (tanpa `WHERE`), mengikuti parser mac:
/// 1. `$BINARY ` → `BINARY ` bila case-sensitive, else dibuang.
/// 2. `$CURRENT_FIELD` → backtick-quoted nama kolom.
/// 3. `${}` → argumen di-escape; jumlah placeholder wajib cocok dengan
///    `NumberOfArguments` (mismasilnya mac: `NSBeep()` + nil).
/// 4. awalan `column ` kecuali `SuppressLeadingFieldPlaceholder`.
///
pub fn where_clause(spec: &FilterSpec, escape: Escaper<'_>) -> Result<String, String> {
    let Some(defs) = group_definitions(&spec.group) else {
        return Err(format!("Unknown filter group '{}'", spec.group));
    };
    let Some(definition) = defs.iter().find(|def| def.menu_label == spec.menu_label) else {
        return Err(format!(
            "Operator '{}' tidak ditemukan di group '{}'",
            spec.menu_label, spec.group
        ));
    };

    let quoted = quote_identifier(&spec.column);
    let mut clause = definition.clause.to_owned();
    clause = replace_token(
        &clause,
        "$BINARY ",
        if spec.case_sensitive { "BINARY " } else { "" },
    );
    clause = replace_token(&clause, "$CURRENT_FIELD", &quoted);

    let placeholders = placeholder_count(&clause);
    if placeholders != definition.number_of_arguments as usize {
        return Err(format!(
            "Definisi filter '{}' deklarasi {} argumen tapi clause memuat {} placeholder.",
            spec.menu_label, definition.number_of_arguments, placeholders
        ));
    }

    let arguments: Vec<String> = match definition.number_of_arguments {
        0 => Vec::new(),
        1 => vec![escape(
            spec.values.first().map(String::as_str).unwrap_or(""),
        )],
        _ => vec![
            escape(spec.values.first().map(String::as_str).unwrap_or("")),
            escape(spec.values.get(1).map(String::as_str).unwrap_or("")),
        ],
    };
    let argument_refs: Vec<&str> = arguments.iter().map(String::as_str).collect();
    clause = substitute_placeholders(&clause, &argument_refs);

    if definition.suppress_leading_field {
        return Ok(clause);
    }
    Ok(format!("{quoted} {clause}"))
}

/// Katalog untuk menu UI: (group, bölüm) yang tersedia untuk kolom ini.
pub fn menu_entries(meta: &ColumnMeta) -> Option<(&'static str, &'static [FilterDef])> {
    let group = filter_group(meta)?;
    Some((group, group_definitions(group)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::client::ColumnMeta;
    use mysqlclient_sys as ffi;

    fn meta(field_type: ffi::enum_field_types, charsetnr: u32, flags: u32) -> ColumnMeta {
        ColumnMeta {
            name: "name".to_owned(),
            org_name: "name".to_owned(),
            table: "orders".to_owned(),
            field_type,
            flags,
            charsetnr,
            length: 10,
            decimals: 0,
        }
    }

    /// Escaper seperti session (`Conn::escape_string`): *body only*,
    /// tanpa quote pembungkus — quote pembungkus ada di template.
    fn literal(value: &str) -> String {
        format!("E({value})")
    }

    fn spec(group: &str, menu_label: &str, values: &[&str], case_sensitive: bool) -> FilterSpec {
        FilterSpec {
            column: "name".to_owned(),
            group: group.to_owned(),
            menu_label: menu_label.to_owned(),
            values: values.iter().map(|value| (*value).to_owned()).collect(),
            case_sensitive,
        }
    }

    #[test]
    fn operator_catalog_matches_the_plist_groups() {
        assert_eq!(group_definitions("bit").unwrap().len(), 12);
        assert_eq!(group_definitions("number").unwrap().len(), 12);
        assert_eq!(group_definitions("string").unwrap().len(), 23);
        assert_eq!(group_definitions("date").unwrap().len(), 9);
        assert_eq!(group_definitions("spatial").unwrap().len(), 9);
        assert!(group_definitions("expression").is_none());
    }

    #[test]
    fn typegrouping_maps_the_mac_column_groups() {
        use ffi::enum_field_types as t;
        assert_eq!(typegrouping(&meta(t::MYSQL_TYPE_BIT, 63, 0)), "bit");
        assert_eq!(
            typegrouping(&meta(t::MYSQL_TYPE_LONGLONG, 63, 0)),
            "integer"
        );
        assert_eq!(typegrouping(&meta(t::MYSQL_TYPE_DOUBLE, 63, 0)), "float");
        assert_eq!(typegrouping(&meta(t::MYSQL_TYPE_DATETIME, 63, 0)), "date");
        assert_eq!(
            typegrouping(&meta(t::MYSQL_TYPE_VAR_STRING, 33, 0)),
            "string"
        );
        let binary = meta(t::MYSQL_TYPE_VAR_STRING, 63, 128);
        assert_eq!(typegrouping(&binary), "binary");
        assert_eq!(typegrouping(&meta(t::MYSQL_TYPE_ENUM, 33, 0)), "enum");
        assert_eq!(typegrouping(&meta(t::MYSQL_TYPE_JSON, 63, 0)), "textdata");
        assert_eq!(typegrouping(&meta(t::MYSQL_TYPE_BLOB, 63, 0)), "blobdata");
        assert_eq!(typegrouping(&meta(t::MYSQL_TYPE_BLOB, 33, 0)), "textdata");
        assert_eq!(
            typegrouping(&meta(t::MYSQL_TYPE_GEOMETRY, 63, 0)),
            "geometry"
        );
    }

    #[test]
    fn filter_group_maps_bit_strings_only() {
        use ffi::enum_field_types as t;
        assert_eq!(filter_group(&meta(t::MYSQL_TYPE_BIT, 63, 0)), Some("bit"));
        assert_eq!(
            filter_group(&meta(t::MYSQL_TYPE_SHORT, 63, 0)),
            Some("number")
        );
        assert_eq!(
            filter_group(&meta(t::MYSQL_TYPE_VAR_STRING, 33, 0)),
            Some("string")
        );
        assert_eq!(
            filter_group(&meta(t::MYSQL_TYPE_STRING, 63, 0)),
            Some("string")
        ); // binary
        assert_eq!(
            filter_group(&meta(t::MYSQL_TYPE_JSON, 63, 0)),
            Some("string")
        );
        assert_eq!(
            filter_group(&meta(t::MYSQL_TYPE_BLOB, 63, 0)),
            Some("string")
        ); // blobdata
        assert_eq!(
            filter_group(&meta(t::MYSQL_TYPE_GEOMETRY, 63, 0)),
            Some("spatial")
        );
    }

    #[test]
    fn clause_uses_session_escaping_without_double_quotes() {
        // Escape disuntikkan tanpa outer quotes; template membr memuatnya.
        let out = where_clause(&spec("number", "=", &["7"], false), &literal).unwrap();
        assert_eq!(out, "`name` = 'E(7)'");
    }

    #[test]
    fn bit_group_wraps_in_cast_decimal() {
        let out = where_clause(&spec("bit", "≠", &["5"], false), &literal).unwrap();
        assert_eq!(out, "`name` != CAST('E(5)' AS DECIMAL(65,30))");
    }

    #[test]
    fn string_contains_wraps_in_wildcards() {
        let out = where_clause(&spec("string", "contains", &["ali"], false), &literal).unwrap();
        assert_eq!(out, "`name` LIKE '%E(ali)%'");
    }

    #[test]
    fn binary_keyword_respects_case_sensitivity() {
        let out = where_clause(&spec("string", "=", &["a"], false), &literal).unwrap();
        assert_eq!(out, "`name` = 'E(a)'");
        let out = where_clause(&spec("string", "=", &["a"], true), &literal).unwrap();
        assert_eq!(out, "`name` = BINARY 'E(a)'");
    }
}

/// Uji hidup filter terhadap server nyata; `#[ignore]` supaya `cargo test`
/// biasa tak menyentuh jaringan. Jalankan:
/// `SA_PROBE_* … cargo test --locked -- --ignored`
///
/// Memastikan klausa yang disusun memilih 1 baris di server, serta
/// `NO_BACKSLASH_ESCAPES` menggandakan quote (doc 13 §4).
#[cfg(test)]
mod live_tests {
    use super::*;

    const TABLE: &str = "sa_probe_filter";

    fn connection() -> Option<crate::database::client::Conn> {
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
        Some(crate::database::client::Conn::connect(&params).expect("connect to probe server"))
    }

    fn filter_spec(menu_label: &str, value: &str) -> FilterSpec {
        FilterSpec {
            column: "name".to_owned(),
            group: "string".to_owned(),
            menu_label: menu_label.to_owned(),
            values: vec![value.to_owned()],
            case_sensitive: false,
        }
    }

    #[test]
    #[ignore = "uji hidup, butuh server dan SA_PROBE_*"]
    fn filter_clause_returns_only_the_matching_rows() {
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
        conn.execute("CREATE TABLE sa_probe_filter (name VARCHAR(64))")
            .expect("create probe table");
        // "= memilih satu baris saja.
        fn escape_value(conn: &crate::database::client::Conn, value: &str) -> String {
            conn.escape_string(value, false)
                .unwrap_or_else(|| value.to_owned())
        }
        for value in ["O'Brien", "plain"] {
            let seeded = escape_value(&conn, value);
            conn.execute(&format!(
                "INSERT INTO {} (name) VALUES (\'{}\')",
                crate::database::client::Conn::quote_identifier(TABLE),
                seeded
            ))
            .expect("seed probe table");
        }

        // \"= memilih satu baris saja.
        let clause = where_clause(&filter_spec("=", "O'Brien"), &|value| {
            escape_value(&conn, value)
        })
        .expect("where");
        let result = conn
            .query(&format!(
                "SELECT name FROM {} WHERE {}",
                crate::database::client::Conn::quote_identifier(TABLE),
                clause
            ))
            .expect("run filter");
        assert_eq!(result.row_count(), 1);

        // \"contains\" dengan wildcard.
        let clause = where_clause(&filter_spec("contains", "Brien"), &|value| {
            escape_value(&conn, value)
        })
        .expect("where");
        let result = conn
            .query(&format!(
                "SELECT name FROM {} WHERE {}",
                crate::database::client::Conn::quote_identifier(TABLE),
                clause
            ))
            .expect("run filter");
        assert_eq!(result.row_count(), 1);

        // NO_BACKSLASH_ESCAPES: quote digandakan, hasil tetap 1 baris
        // (doc 13 §4).
        conn.execute("SET SESSION sql_mode = 'NO_BACKSLASH_ESCAPES'")
            .expect("set sql_mode");
        let clause = where_clause(&filter_spec("=", "O'Brien"), &|value| {
            escape_value(&conn, value)
        })
        .expect("where");
        assert!(clause.contains("'O''Brien'"), "clause = {clause}");
        let result = conn
            .query(&format!(
                "SELECT name FROM {} WHERE {}",
                crate::database::client::Conn::quote_identifier(TABLE),
                clause
            ))
            .expect("run filter");
        assert_eq!(result.row_count(), 1);

        conn.execute(&format!("DROP DATABASE IF EXISTS {TABLE}_db"))
            .ok();
    }
}
