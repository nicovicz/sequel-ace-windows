//! Table structure editing — SQL generation untuk `ALTER TABLE`.
//!
//! Port `07-content-edit-copy.md` §2 (`deriveQueryString`) + audit
//! `SPTableStructure.md` (`addRowToDB` 944-1071, `_buildPartialColumnDefinitionString:` 1078-1301).
//!
//! Kolom definition mengikuti urutan mac:
//! `name type(length) [CHARSET] [COLLATE] [UNSIGNED] [ZEROFILL] [NOT NULL] [DEFAULT x] [AUTO_INCREMENT] [COMMENT 'x']`
//!
//! Escaping disuntikkan (pola `copy.rs`/`edit.rs`) supaya test tidak butuh server.

use super::metadata::quote_identifier;

/// Satu definisi kolom baru/ubah untuk `ALTER TABLE`.
#[derive(Debug, Clone, Default)]
pub struct ColumnDefinition {
    /// Nama kolom (baru, atau nama lama bila `old_name` diset untuk CHANGE).
    pub name: String,
    pub old_name: Option<String>,
    pub data_type: String,
    pub length: Option<String>,
    pub unsigned: bool,
    pub zerofill: bool,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub auto_increment: bool,
    pub comment: Option<String>,
    pub charset: Option<String>,
    pub collation: Option<String>,
    /// `AFTER <col>` atau `FIRST` untuk posisi.
    pub after_column: Option<String>,
}

impl ColumnDefinition {
    /// Bangun string definisi kolom (doc 07, `_buildPartialColumnDefinitionString:` 1078-1301).
    pub fn to_definition_string(&self) -> String {
        let mut parts = Vec::new();
        parts.push(quote_identifier(&self.name));
        let type_with_length = match &self.length {
            Some(length) => format!("{}({})", self.data_type, length),
            None => self.data_type.clone(),
        };
        parts.push(type_with_length);
        if let Some(charset) = &self.charset {
            parts.push(format!("CHARACTER SET {charset}"));
        }
        if let Some(collation) = &self.collation {
            parts.push(format!("COLLATE {collation}"));
        }
        if self.unsigned {
            parts.push("UNSIGNED".to_owned());
        }
        if self.zerofill {
            parts.push("ZEROFILL".to_owned());
        }
        if !self.nullable {
            parts.push("NOT NULL".to_owned());
        } else {
            parts.push("NULL".to_owned());
        }
        match &self.default_value {
            Some(value) if !value.is_empty() => {
                parts.push(format!("DEFAULT {}", value));
            }
            _ => {
                if self.nullable {
                    parts.push("DEFAULT NULL".to_owned());
                }
            }
        }
        if self.auto_increment {
            parts.push("AUTO_INCREMENT".to_owned());
        }
        if let Some(comment) = &self.comment {
            let escaped = comment.replace('\'', "''");
            parts.push(format!("COMMENT '{escaped}'"));
        }
        parts.join(" ")
    }
}

/// `ALTER TABLE <t> ADD COLUMN <def>` (doc 07, `addRowToDB` 970-979).
pub fn add_column_sql(table: &str, definition: &ColumnDefinition) -> String {
    let mut sql = format!(
        "ALTER TABLE {} ADD COLUMN {}",
        quote_identifier(table),
        definition.to_definition_string()
    );
    if let Some(after) = &definition.after_column {
        sql.push_str(&format!(" AFTER {}", quote_identifier(after)));
    }
    sql
}

/// `ALTER TABLE <t> DROP COLUMN <name>` (doc 07, `_removeFieldAndForeignKey:`).
pub fn drop_column_sql(table: &str, column_name: &str) -> String {
    format!(
        "ALTER TABLE {} DROP COLUMN {}",
        quote_identifier(table),
        quote_identifier(column_name)
    )
}

/// `ALTER TABLE <t> CHANGE <old> <def>` (rename + modify).
pub fn change_column_sql(table: &str, definition: &ColumnDefinition) -> String {
    let old_name = definition.old_name.as_deref().unwrap_or(&definition.name);
    let mut sql = format!(
        "ALTER TABLE {} CHANGE {} {}",
        quote_identifier(table),
        quote_identifier(old_name),
        definition.to_definition_string()
    );
    if let Some(after) = &definition.after_column {
        sql.push_str(&format!(" AFTER {}", quote_identifier(after)));
    }
    sql
}

/// `ALTER TABLE <t> AUTO_INCREMENT = <n>` (doc 07, `setAutoIncrementTo:` 837-866).
#[allow(dead_code)]
pub fn set_auto_increment_sql(table: &str, value: u64) -> String {
    format!(
        "ALTER TABLE {} AUTO_INCREMENT = {}",
        quote_identifier(table),
        value
    )
}

/// `DROP TABLE [IF EXISTS] <t>`.
#[allow(dead_code)]
pub fn drop_table_sql(table: &str, if_exists: bool) -> String {
    let if_exists = if if_exists { "IF EXISTS " } else { "" };
    format!("DROP TABLE {if_exists}{}", quote_identifier(table))
}

/// `CREATE DATABASE [IF NOT EXISTS] <db> [CHARSET] [COLLATE]`.
#[allow(dead_code)]
pub fn create_database_sql(
    name: &str,
    if_not_exists: bool,
    charset: Option<&str>,
    collation: Option<&str>,
) -> String {
    let ine = if if_not_exists { "IF NOT EXISTS " } else { "" };
    let mut sql = format!("CREATE DATABASE {ine}{}", quote_identifier(name));
    if let Some(charset) = charset {
        sql.push_str(&format!(" CHARACTER SET {charset}"));
    }
    if let Some(collation) = collation {
        sql.push_str(&format!(" COLLATE {collation}"));
    }
    sql
}

/// `DROP DATABASE [IF EXISTS] <db>`.
#[allow(dead_code)]
pub fn drop_database_sql(name: &str, if_exists: bool) -> String {
    let ine = if if_exists { "IF EXISTS " } else { "" };
    format!("DROP DATABASE {ine}{}", quote_identifier(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_column_basic() {
        let def = ColumnDefinition {
            name: "email".to_owned(),
            data_type: "VARCHAR".to_owned(),
            length: Some("255".to_owned()),
            nullable: true,
            default_value: Some("NULL".to_owned()),
            ..Default::default()
        };
        let sql = add_column_sql("users", &def);
        assert_eq!(
            sql,
            "ALTER TABLE `users` ADD COLUMN `email` VARCHAR(255) NULL DEFAULT NULL"
        );
    }

    #[test]
    fn add_column_not_null_with_default() {
        let def = ColumnDefinition {
            name: "status".to_owned(),
            data_type: "ENUM".to_owned(),
            length: Some("'active','inactive'".to_owned()),
            nullable: false,
            default_value: Some("'active'".to_owned()),
            ..Default::default()
        };
        let sql = add_column_sql("users", &def);
        assert_eq!(
            sql,
            "ALTER TABLE `users` ADD COLUMN `status` ENUM('active','inactive') NOT NULL DEFAULT 'active'"
        );
    }

    #[test]
    fn add_column_auto_increment() {
        let def = ColumnDefinition {
            name: "id".to_owned(),
            data_type: "INT".to_owned(),
            length: Some("11".to_owned()),
            unsigned: true,
            nullable: false,
            auto_increment: true,
            ..Default::default()
        };
        let sql = add_column_sql("users", &def);
        assert_eq!(
            sql,
            "ALTER TABLE `users` ADD COLUMN `id` INT(11) UNSIGNED NOT NULL AUTO_INCREMENT"
        );
    }

    #[test]
    fn add_column_after() {
        let def = ColumnDefinition {
            name: "email".to_owned(),
            data_type: "VARCHAR".to_owned(),
            length: Some("255".to_owned()),
            nullable: true,
            after_column: Some("name".to_owned()),
            ..Default::default()
        };
        let sql = add_column_sql("users", &def);
        assert_eq!(
            sql,
            "ALTER TABLE `users` ADD COLUMN `email` VARCHAR(255) NULL DEFAULT NULL AFTER `name`"
        );
    }

    #[test]
    fn drop_column() {
        let sql = drop_column_sql("users", "old_field");
        assert_eq!(sql, "ALTER TABLE `users` DROP COLUMN `old_field`");
    }

    #[test]
    fn change_column_rename() {
        let def = ColumnDefinition {
            name: "new_name".to_owned(),
            old_name: Some("old_name".to_owned()),
            data_type: "VARCHAR".to_owned(),
            length: Some("128".to_owned()),
            nullable: true,
            ..Default::default()
        };
        let sql = change_column_sql("users", &def);
        assert_eq!(
            sql,
            "ALTER TABLE `users` CHANGE `old_name` `new_name` VARCHAR(128) NULL DEFAULT NULL"
        );
    }

    #[test]
    fn set_auto_increment() {
        let sql = set_auto_increment_sql("orders", 1000);
        assert_eq!(sql, "ALTER TABLE `orders` AUTO_INCREMENT = 1000");
    }

    #[test]
    fn drop_table_if_exists() {
        let sql = drop_table_sql("old_table", true);
        assert_eq!(sql, "DROP TABLE IF EXISTS `old_table`");
    }

    #[test]
    fn create_database_with_charset() {
        let sql = create_database_sql("mydb", true, Some("utf8mb4"), Some("utf8mb4_unicode_ci"));
        assert_eq!(
            sql,
            "CREATE DATABASE IF NOT EXISTS `mydb` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci"
        );
    }

    #[test]
    fn drop_database() {
        let sql = drop_database_sql("mydb", false);
        assert_eq!(sql, "DROP DATABASE `mydb`");
    }

    #[test]
    fn column_with_comment_and_charset() {
        let def = ColumnDefinition {
            name: "bio".to_owned(),
            data_type: "TEXT".to_owned(),
            nullable: true,
            charset: Some("utf8mb4".to_owned()),
            collation: Some("utf8mb4_unicode_ci".to_owned()),
            comment: Some("User's bio".to_owned()),
            ..Default::default()
        };
        let sql = add_column_sql("users", &def);
        assert_eq!(
            sql,
            "ALTER TABLE `users` ADD COLUMN `bio` TEXT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NULL DEFAULT NULL COMMENT 'User''s bio'"
        );
    }
}
