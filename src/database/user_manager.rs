//! User management — SQL generation untuk CREATE/DROP/GRANT/REVOKE.
//!
//! Port `SPUserManager.m` (doc `audit/SPUserManager.md`).
//! SQL yang dibangun: `CREATE USER`, `DROP USER`, `GRANT`, `REVOKE`,
//! `SHOW GRANTS FOR`, `SELECT User, Host FROM mysql.user`.

use super::metadata::quote_identifier;

/// Parameter untuk `CREATE USER`.
#[derive(Debug, Clone)]
pub struct CreateUserParams {
    pub user: String,
    pub host: String,
    pub password: Option<String>,
    /// `IDENTIFIED WITH plugin` (mac: `mysql_native_password` default).
    pub auth_plugin: Option<String>,
}

/// Parameter untuk `GRANT`/`REVOKE`.
#[derive(Debug, Clone)]
pub struct PrivilegeSpec {
    /// `ALL PRIVILEGES`, `SELECT`, `INSERT`, `UPDATE`, `DELETE`, dll.
    pub privileges: Vec<String>,
    /// Database scope (`*` = global, atau nama database).
    pub database: String,
    /// Table scope (`*` = seluruh tabel di database).
    pub user: String,
    pub host: String,
    pub with_grant_option: bool,
}

fn user_host(user: &str, host: &str) -> String {
    format!("'{}'@'{}'", escape_string(user), escape_string(host))
}

fn escape_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

/// `SELECT User, Host FROM mysql.user ORDER BY User, Host` — baca daftar user.
pub fn list_users_sql() -> String {
    "SELECT User, Host FROM mysql.user WHERE User != '' ORDER BY User, Host".to_owned()
}

/// `CREATE USER 'user'@'host' IDENTIFIED BY 'password'` (doc 07, mac `SPUserManager`).
pub fn create_user_sql(params: &CreateUserParams) -> String {
    let mut sql = format!("CREATE USER {}", user_host(&params.user, &params.host));
    if let Some(password) = &params.password
        && !password.is_empty()
    {
        sql.push_str(&format!(" IDENTIFIED BY '{}'", escape_string(password)));
    }
    if let Some(plugin) = &params.auth_plugin {
        sql.push_str(&format!(" IDENTIFIED WITH {plugin}"));
    }
    sql
}

/// `DROP USER 'user'@'host'`.
pub fn drop_user_sql(user: &str, host: &str) -> String {
    format!("DROP USER {}", user_host(user, host))
}

/// `ALTER USER 'user'@'host' IDENTIFIED BY 'new_password'`.
#[allow(dead_code)] // dipakai user via SQL editor
pub fn change_password_sql(user: &str, host: &str, new_password: &str) -> String {
    format!(
        "ALTER USER {} IDENTIFIED BY '{}'",
        user_host(user, host),
        escape_string(new_password)
    )
}

/// `GRANT <privileges> ON <db>.<table> TO <user>` (doc 07, mac `doApply:`).
pub fn grant_sql(spec: &PrivilegeSpec) -> String {
    let privileges = if spec.privileges.is_empty() {
        "USAGE".to_owned()
    } else {
        spec.privileges.join(", ")
    };
    let mut sql = format!(
        "GRANT {privileges} ON {}.* TO {}",
        scope_database(&spec.database),
        user_host(&spec.user, &spec.host)
    );
    if spec.with_grant_option {
        sql.push_str(" WITH GRANT OPTION");
    }
    sql
}

/// `REVOKE <privileges> ON <db>.* FROM <user>`.
#[allow(dead_code)] // dipakai user via SQL editor
pub fn revoke_sql(spec: &PrivilegeSpec) -> String {
    let privileges = if spec.privileges.is_empty() {
        "ALL PRIVILEGES".to_owned()
    } else {
        spec.privileges.join(", ")
    };
    format!(
        "REVOKE {privileges} ON {}.* FROM {}",
        scope_database(&spec.database),
        user_host(&spec.user, &spec.host)
    )
}

fn scope_database(database: &str) -> String {
    if database == "*" || database.is_empty() {
        "*".to_owned()
    } else {
        quote_identifier(database)
    }
}

/// `SHOW GRANTS FOR 'user'@'host'` — baca privilege yang ada.
#[allow(dead_code)] // dipakai user via SQL editor
pub fn show_grants_sql(user: &str, host: &str) -> String {
    format!("SHOW GRANTS FOR {}", user_host(user, host))
}

/// `FLUSH PRIVILEGES` (mac `doApply:` memanggil ini setelah save).
#[allow(dead_code)] // dipakai user via SQL editor
pub fn flush_privileges_sql() -> String {
    "FLUSH PRIVILEGES".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_users_sql_format() {
        assert_eq!(
            list_users_sql(),
            "SELECT User, Host FROM mysql.user WHERE User != '' ORDER BY User, Host"
        );
    }

    #[test]
    fn create_user_with_password() {
        let params = CreateUserParams {
            user: "app_user".to_owned(),
            host: "localhost".to_owned(),
            password: Some("secret123".to_owned()),
            auth_plugin: None,
        };
        let sql = create_user_sql(&params);
        assert_eq!(
            sql,
            "CREATE USER 'app_user'@'localhost' IDENTIFIED BY 'secret123'"
        );
    }

    #[test]
    fn create_user_escapes_password() {
        let params = CreateUserParams {
            user: "app".to_owned(),
            host: "%".to_owned(),
            password: Some("pa\\ss'word".to_owned()),
            auth_plugin: None,
        };
        let sql = create_user_sql(&params);
        assert_eq!(sql, "CREATE USER 'app'@'%' IDENTIFIED BY 'pa\\\\ss\\'word'");
    }

    #[test]
    fn drop_user_format() {
        let sql = drop_user_sql("old_user", "localhost");
        assert_eq!(sql, "DROP USER 'old_user'@'localhost'");
    }

    #[test]
    fn grant_all_privileges() {
        let spec = PrivilegeSpec {
            privileges: vec!["ALL PRIVILEGES".to_owned()],
            database: "*".to_owned(),
            user: "admin".to_owned(),
            host: "localhost".to_owned(),
            with_grant_option: true,
        };
        let sql = grant_sql(&spec);
        assert_eq!(
            sql,
            "GRANT ALL PRIVILEGES ON *.* TO 'admin'@'localhost' WITH GRANT OPTION"
        );
    }

    #[test]
    fn grant_select_insert_on_database() {
        let spec = PrivilegeSpec {
            privileges: vec!["SELECT".to_owned(), "INSERT".to_owned()],
            database: "mydb".to_owned(),
            user: "reader".to_owned(),
            host: "%".to_owned(),
            with_grant_option: false,
        };
        let sql = grant_sql(&spec);
        assert_eq!(sql, "GRANT SELECT, INSERT ON `mydb`.* TO 'reader'@'%'");
    }

    #[test]
    fn revoke_privileges() {
        let spec = PrivilegeSpec {
            privileges: vec!["ALL PRIVILEGES".to_owned()],
            database: "mydb".to_owned(),
            user: "old_user".to_owned(),
            host: "localhost".to_owned(),
            with_grant_option: false,
        };
        let sql = revoke_sql(&spec);
        assert_eq!(
            sql,
            "REVOKE ALL PRIVILEGES ON `mydb`.* FROM 'old_user'@'localhost'"
        );
    }

    #[test]
    fn show_grants_format() {
        assert_eq!(
            show_grants_sql("root", "localhost"),
            "SHOW GRANTS FOR 'root'@'localhost'"
        );
    }

    #[test]
    fn change_password_format() {
        let sql = change_password_sql("user", "host", "new_pass");
        assert_eq!(sql, "ALTER USER 'user'@'host' IDENTIFIED BY 'new_pass'");
    }

    #[test]
    fn flush_privileges_format() {
        assert_eq!(flush_privileges_sql(), "FLUSH PRIVILEGES");
    }

    #[test]
    fn user_host_escapes_quotes() {
        let result = user_host("it's", "host\\name");
        assert_eq!(result, "'it\\'s'@'host\\\\name'");
    }
}
