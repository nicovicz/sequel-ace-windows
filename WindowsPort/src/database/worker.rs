use super::client::{Conn, ConnectParams};
use super::content::load_content_page;
use super::copy::{render, session_escaper};
use super::edit::{EditKind, delete_multi_sql, delete_sql, insert_sql, update_sql};
use super::filter::{FilterSpec, where_clause};
use super::metadata::{
    current_database, list_databases, load_object_details, load_server_info, load_tables,
    primary_key_indexes,
};
use super::models::{CopyKind, Request, Response};
use super::query::execute_sql;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

pub struct DatabaseWorker {
    pub requests: Sender<Request>,
    pub responses: Receiver<Response>,
}

impl DatabaseWorker {
    pub fn start() -> Self {
        let (requests, request_receiver) = mpsc::channel();
        let (response_sender, responses) = mpsc::channel();
        thread::spawn(move || run_worker(request_receiver, response_sender));
        Self {
            requests,
            responses,
        }
    }
}

fn run_worker(requests: Receiver<Request>, responses: Sender<Response>) {
    // `Conn` tidak `Send`: ia dibuat dan dipakai hanya di thread ini, sama
    // seperti `SPMySQLConnection` yang dikunci ke thread pemiliknya.
    let mut connection: Option<Conn> = None;
    let mut selected_database: Option<String> = None;
    // Filter "by selected value" aktif (`doc 06-content-load-filter.md`, fase 2).
    let mut content_filter: Option<FilterSpec> = None;

    while let Ok(request) = requests.recv() {
        let response = match request {
            Request::Connect(options) => {
                connection = None;
                selected_database = None;
                let initial_database = options.initial_database.clone();
                let time_zone = options.time_zone.clone();

                // SSH tunnel (doc 03 + SPSSHTunnel.m): bila ssh_host ada,
                // buka tunnel dulu, lalu connect MySQL ke 127.0.0.1:tunnel_port.
                let (connect_host, connect_port, _tunnel) = if let Some(ssh_host) =
                    &options.ssh_host
                {
                    match super::ssh_tunnel::SshTunnel::open(&super::ssh_tunnel::SshTunnelParams {
                        ssh_host: ssh_host.clone(),
                        ssh_port: options.ssh_port.unwrap_or(22),
                        ssh_user: options.ssh_user.clone().unwrap_or_default(),
                        ssh_key_path: options.ssh_key_path.clone(),
                        remote_host: options.host.clone(),
                        remote_port: options.port,
                        strict_host_key_checking: false,
                        agent_forwarding: false,
                        identities_only: options.ssh_key_path.is_some(),
                    }) {
                        Ok(t) => ("127.0.0.1".to_owned(), t.local_port, Some(t)),
                        Err(error) => {
                            connection = None;
                            selected_database = None;
                            if responses
                                .send(Response::Failed(format!("SSH tunnel failed: {error}")))
                                .is_err()
                            {
                                break;
                            }
                            continue;
                        }
                    }
                } else {
                    (options.host.clone(), options.port, None)
                };

                let mut params = ConnectParams::new(connect_host, connect_port, options.user);
                params.password = options.password;
                params.database = initial_database.clone();
                params.allow_local_infile = options.allow_local_infile;
                params.enable_cleartext_plugin = options.enable_cleartext_plugin;
                params.request_server_public_key = options.request_server_public_key;
                // doc 13 §2: SSL_MODE_REQUIRED bila cleartext harus TLS atau user
                // menyetel Require SSL, else SSL_MODE_PREFERRED.
                params.ssl = (options.require_tls || options.enable_cleartext_plugin).then(|| {
                    super::client::SslParams {
                        require_tls: true,
                        ..Default::default()
                    }
                });

                match Conn::connect(&params) {
                    Ok(mut conn) => {
                        if let Some(zone) = time_zone {
                            let zone = if zone == "UTC" {
                                "+00:00".to_owned()
                            } else {
                                zone
                            };
                            let statement = conn
                                .quote_literal(&zone)
                                .map(|literal| format!("SET SESSION time_zone = {literal}"));
                            let result = match statement {
                                Ok(statement) => conn.execute(&statement),
                                Err(error) => Err(error),
                            };
                            if let Err(error) = result {
                                if responses
                                    .send(Response::Failed(format!(
                                        "Could not set time zone: {error}"
                                    )))
                                    .is_err()
                                {
                                    break;
                                }
                                continue;
                            }
                        }

                        let database_result = list_databases(&mut conn);
                        let (tables, selection_error) = if let Some(database) = initial_database {
                            match conn.select_db(&database) {
                                Ok(()) => {
                                    selected_database = Some(database);
                                    match load_tables(&mut conn) {
                                        Ok(tables) => (tables, None),
                                        Err(error) => (Vec::new(), Some(error)),
                                    }
                                }
                                Err(error) => (Vec::new(), Some(error.to_string())),
                            }
                        } else {
                            (Vec::new(), None)
                        };
                        connection = Some(conn);
                        match database_result {
                            Ok(databases) => Response::Connected {
                                databases,
                                warning: None,
                                selected_database: selected_database.clone(),
                                tables,
                                selection_error,
                            },
                            Err(error) => Response::Connected {
                                databases: Vec::new(),
                                warning: Some(error),
                                selected_database: selected_database.clone(),
                                tables,
                                selection_error,
                            },
                        }
                    }
                    Err(error) => Response::Failed(error.to_string()),
                }
            }
            Request::RefreshDatabases => match connection.as_mut() {
                Some(conn) => match list_databases(conn) {
                    Ok(databases) => Response::Databases(databases),
                    Err(error) => Response::Failed(error),
                },
                None => Response::Disconnected,
            },
            Request::SelectDatabase(database) => match connection.as_mut() {
                Some(conn) => match conn.select_db(&database) {
                    Ok(()) => {
                        selected_database = Some(database.clone());
                        match load_tables(conn) {
                            Ok(entries) => Response::Tables { database, entries },
                            Err(error) => Response::SelectionFailed {
                                database,
                                error,
                                selected: true,
                            },
                        }
                    }
                    Err(error) => Response::SelectionFailed {
                        database,
                        error: error.to_string(),
                        selected: false,
                    },
                },
                None => Response::Disconnected,
            },
            Request::RefreshTables => match (connection.as_mut(), selected_database.as_ref()) {
                (Some(conn), Some(database)) => match load_tables(conn) {
                    Ok(entries) => Response::Tables {
                        database: database.clone(),
                        entries,
                    },
                    Err(error) => Response::Failed(error),
                },
                _ => Response::Disconnected,
            },
            Request::LoadObject(entry) => match (connection.as_mut(), selected_database.as_ref()) {
                (Some(conn), Some(database)) => Response::ObjectDetails {
                    database: database.clone(),
                    details: load_object_details(conn, database, &entry),
                    entry,
                },
                _ => Response::Disconnected,
            },
            Request::LoadContent { entry, page } => {
                match (connection.as_mut(), selected_database.as_ref()) {
                    (Some(conn), Some(database)) => Response::ContentPage {
                        database: database.clone(),
                        result: load_content_page(
                            conn,
                            database,
                            &entry.name,
                            page,
                            content_filter.as_ref(),
                        ),
                        entry,
                    },
                    _ => Response::Disconnected,
                }
            }
            Request::SetContentFilter { entry, filter } => {
                match (connection.as_mut(), selected_database.as_ref()) {
                    (Some(conn), Some(database)) => {
                        // Reset filter dengan None; mac men-reset ke halaman 1
                        // saat filter diterapkan (doc 06 §2).
                        let mut filter_error: Option<String> = None;
                        let next_filter = match filter.as_ref() {
                            Some(spec) => match where_clause(spec, &filter_escape(conn)) {
                                Ok(_) => Some(spec.clone()),
                                Err(error) => {
                                    filter_error = Some(error);
                                    None
                                }
                            },
                            None => None,
                        };
                        content_filter = next_filter;
                        if let Some(error) = filter_error {
                            Response::Failed(error)
                        } else {
                            Response::ContentPage {
                                database: database.clone(),
                                result: load_content_page(
                                    conn,
                                    database,
                                    &entry.name,
                                    0,
                                    content_filter.as_ref(),
                                ),
                                entry,
                            }
                        }
                    }
                    _ => Response::Disconnected,
                }
            }
            Request::DeleteAllRows { table } => match connection.as_mut() {
                Some(conn) => {
                    let sql = super::edit::delete_all_sql(&table);
                    match conn.execute(&sql) {
                        Ok(()) => Response::EditApplied {
                            affected_rows: conn.last_affected_rows(),
                            sql,
                        },
                        Err(error) => Response::Failed(error.to_string()),
                    }
                }
                None => Response::Disconnected,
            },
            Request::DeleteMultiRows { mut plan, rows } => match connection.as_mut() {
                Some(conn) => {
                    if plan.pk_indexes.is_empty()
                        && let Ok(pk) = primary_key_indexes(
                            conn,
                            selected_database.as_ref().unwrap_or(&String::new()),
                            &plan.table,
                        )
                    {
                        plan.pk_indexes = pk;
                    }
                    let sql = {
                        let escaper = session_escaper(conn);
                        delete_multi_sql(&plan, &rows, &escaper)
                    };
                    match sql {
                        Ok(sql) => match conn.execute(&sql) {
                            Ok(()) => Response::EditApplied {
                                affected_rows: conn.last_affected_rows(),
                                sql,
                            },
                            Err(error) => Response::Failed(error.to_string()),
                        },
                        Err(error) => Response::Failed(error),
                    }
                }
                None => Response::Disconnected,
            },
            Request::LoadServerInfo => match connection.as_mut() {
                Some(conn) => Response::ServerInfo(load_server_info(conn)),
                None => Response::Disconnected,
            },
            Request::ExecuteEdit(mut plan, kind) => match connection.as_mut() {
                // DeleteAll tidak butuh PK; cukup table name.
                Some(conn) => {
                    // PK di-resolve di sini (bukan UI) supaya UI tidak
                    // menyentuh koneksi. Bila `pk_indexes` kosong dan tabel
                    // punya PK, isi dengan posisi kolom PK pada result set.
                    if plan.pk_indexes.is_empty()
                        && let Ok(pk) = primary_key_indexes(
                            conn,
                            selected_database.as_ref().unwrap_or(&String::new()),
                            &plan.table,
                        )
                    {
                        plan.pk_indexes = pk;
                    }
                    let sql = match kind {
                        EditKind::Insert => insert_sql(&plan, &session_escaper(conn)),
                        EditKind::Update => update_sql(&plan, &session_escaper(conn)),
                        EditKind::Delete => delete_sql(&plan, &session_escaper(conn)),
                    };
                    match sql {
                        Ok(sql_string) => match conn.execute(&sql_string) {
                            Ok(()) => Response::EditApplied {
                                affected_rows: conn.last_affected_rows(),
                                sql: sql_string,
                            },
                            Err(error) => Response::Failed(error.to_string()),
                        },
                        Err(error) => Response::Failed(error),
                    }
                }
                None => Response::Disconnected,
            },
            Request::CopyRows(snapshot, kind) => {
                match connection.as_mut() {
                    // Copy-as-SQL perlu koneksi agar escaping mengikuti session
                    // (doc 07 §4); tab/CSV tidak menyentuh koneksi.
                    _conn
                        if matches!(kind, CopyKind::SqlInsert | CopyKind::SqlInsertNoAutoInc)
                            && snapshot.table.is_none() =>
                    {
                        Response::Failed(
                            "Copy as SQL INSERT needs a table in the Content view.".to_owned(),
                        )
                    }
                    Some(conn) => {
                        Response::CopyResult(render(&snapshot, kind, &session_escaper(conn)))
                    }
                    None => Response::Disconnected,
                }
            }
            Request::ExecuteSql(sql) => match connection.as_mut() {
                Some(conn) => match execute_sql(conn, &sql) {
                    Ok(output) => {
                        let mut warning = None;
                        let mut database = selected_database.clone();
                        let mut database_changed = false;
                        let mut tables = None;
                        match current_database(conn) {
                            Ok(current) => {
                                if current != selected_database {
                                    database_changed = true;
                                    database = current.clone();
                                    selected_database = current;
                                    if selected_database.is_some() {
                                        match load_tables(conn) {
                                            Ok(entries) => tables = Some(entries),
                                            Err(error) => warning = Some(error),
                                        }
                                    }
                                }
                            }
                            Err(error) => warning = Some(error),
                        }
                        Response::QueryExecuted {
                            output,
                            database,
                            database_changed,
                            tables,
                            warning,
                        }
                    }
                    Err(error) => Response::Failed(error),
                },
                None => Response::Disconnected,
            },
            Request::Disconnect => {
                connection = None;
                selected_database = None;
                Response::Disconnected
            }
        };
        if responses.send(response).is_err() {
            break;
        }
    }
}

/// Escaper body-only untuk argumen filter (escaping session).
fn filter_escape(conn: &Conn) -> impl Fn(&str) -> String + '_ {
    move |value| {
        conn.escape_string(value, false)
            .unwrap_or_else(|| value.to_owned())
    }
}
