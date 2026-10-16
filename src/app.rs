mod browser;
mod bundle_editor;
mod connection;
mod content;
mod details;
mod favorites;
mod grid;
mod import_export;
mod query;
mod query_store;
mod structure;

use self::content::{EditSheetState, FilterPrompt};
use self::favorites::{Favorite, FavoritesStore};
use self::query_store::{QueryFavorite, QueryStore};
use crate::database::keychain as kc;
use crate::database::{
    ColumnMeta, ConnectionOptions, ContentPage, CopyKind, CopySnapshot, DatabaseWorker, EditPlan,
    FilterSpec, ObjectDetails, QueryOutput, Request, Response, ServerInfo, TableEntry,
};
use eframe::egui;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq)]
enum MainView {
    Tables,
    Structure,
    Content,
    Query,
    Users,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ConnectionMethod {
    TcpIp,
    Socket,
    Ssh,
}

pub struct SequelAceApp {
    worker: DatabaseWorker,
    connection_method: ConnectionMethod,
    connection_name: String,
    favorite_filter: String,
    socket_path: String,
    ssh_host: String,
    ssh_user: String,
    ssh_password: String,
    ssh_port: String,
    ssh_key_path: String,
    host: String,
    port: String,
    user: String,
    password: String,
    show_password: bool,
    show_ssh_password: bool,
    time_zone: String,
    initial_database: String,
    connected: bool,
    busy: bool,
    databases: Vec<String>,
    database_filter: String,
    database_name: String,
    selected_database: Option<String>,
    tables: Vec<TableEntry>,
    table_filter: String,
    selected_object: Option<TableEntry>,
    object_details: Option<ObjectDetails>,
    content_page: Option<ContentPage>,
    content_page_index: u64,
    server_info: Option<ServerInfo>,
    show_server_info: bool,
    show_all_variables: bool,
    show_bundle_editor: bool,
    main_view: MainView,
    query_text: String,
    query_output: Option<QueryOutput>,
    query_history: Vec<String>,
    selected_result_set: usize,
    selected_content_cell: Option<(usize, usize)>,
    selected_content_rows: Vec<usize>,
    selected_query_cell: Option<(usize, usize)>,
    content_filter: Option<FilterSpec>,
    edit_sheet: Option<EditSheetState>,
    confirm_delete: Option<EditPlan>,
    // Table Structure editor (fase 12)
    structure_add_column: Option<ColumnDefinitionState>,
    confirm_drop_column: Option<String>,
    // User Manager (fase 12)
    user_list: Vec<(String, String)>,
    user_new_name: String,
    user_new_host: String,
    user_new_password: String,
    user_confirm_drop: Option<(String, String)>,
    // Table/Database DDL (fase 14)
    rename_table_dialog: bool,
    rename_table_old: String,
    rename_table_new: String,
    confirm_drop_table: bool,
    confirm_drop_db: bool,
    // Bundle editor (fase 15)
    bundles: Vec<crate::database::bundle::BundleEntry>,
    bundle_selected: Option<usize>,
    bundle_confirm_delete: Option<usize>,
    bundle_error: Option<String>,
    bundle_output: String,
    confirm_delete_all: bool,
    #[allow(dead_code)] // dipakai di edit_overlays delete-all dialog
    reset_auto_increment: bool,
    confirm_delete_multi: Option<(EditPlan, Vec<usize>)>,
    filter_prompt: Option<FilterPrompt>,
    generated_columns: Option<Vec<String>>,
    favorites: Vec<Favorite>,
    favorites_store: FavoritesStore,
    query_favorites: Vec<QueryFavorite>,
    query_store: QueryStore,
    allow_local_infile: bool,
    enable_cleartext_plugin: bool,
    require_ssl: bool,
    request_server_public_key: bool,
    error: Option<String>,
    warning: Option<String>,
    copied: Option<String>,
}

use self::structure::ColumnDefinitionState;

impl SequelAceApp {
    pub fn new() -> Self {
        Self {
            worker: DatabaseWorker::start(),
            connection_method: ConnectionMethod::TcpIp,
            connection_name: String::new(),
            favorite_filter: String::new(),
            socket_path: String::new(),
            ssh_host: String::new(),
            ssh_user: String::new(),
            ssh_password: String::new(),
            ssh_port: String::new(),
            ssh_key_path: String::new(),
            host: String::new(),
            port: String::new(),
            user: String::new(),
            password: String::new(),
            show_password: false,
            show_ssh_password: false,
            time_zone: String::new(),
            initial_database: String::new(),
            connected: false,
            busy: false,
            databases: Vec::new(),
            database_filter: String::new(),
            database_name: String::new(),
            selected_database: None,
            tables: Vec::new(),
            table_filter: String::new(),
            selected_object: None,
            object_details: None,
            content_page: None,
            content_page_index: 0,
            server_info: None,
            show_server_info: false,
            show_all_variables: false,
            show_bundle_editor: false,
            main_view: MainView::Tables,
            query_text: String::new(),
            query_output: None,
            query_history: QueryStore::new(dirs::data_dir().unwrap_or_default())
                .load_history()
                .into_iter()
                .map(|entry| entry.query)
                .collect(),
            selected_result_set: 0,
            selected_content_cell: None,
            selected_content_rows: Vec::new(),
            selected_query_cell: None,
            content_filter: None,
            edit_sheet: None,
            confirm_delete: None,
            structure_add_column: None,
            confirm_drop_column: None,
            user_list: Vec::new(),
            user_new_name: String::new(),
            user_new_host: "localhost".to_owned(),
            user_new_password: String::new(),
            user_confirm_drop: None,
            rename_table_dialog: false,
            rename_table_old: String::new(),
            rename_table_new: String::new(),
            confirm_drop_table: false,
            confirm_drop_db: false,
            bundles: Vec::new(),
            bundle_selected: None,
            bundle_confirm_delete: None,
            bundle_error: None,
            bundle_output: String::new(),
            confirm_delete_all: false,
            reset_auto_increment: false,
            confirm_delete_multi: None,
            filter_prompt: None,
            generated_columns: None,
            favorites: FavoritesStore::new(dirs::data_dir().unwrap_or_default()).load(),
            favorites_store: FavoritesStore::new(dirs::data_dir().unwrap_or_default()),
            query_favorites: QueryStore::new(dirs::data_dir().unwrap_or_default()).load_favorites(),
            query_store: QueryStore::new(dirs::data_dir().unwrap_or_default()),
            allow_local_infile: false,
            enable_cleartext_plugin: false,
            require_ssl: false,
            request_server_public_key: false,
            error: None,
            warning: None,
            copied: None,
        }
    }

    fn send(&mut self, request: Request) {
        self.error = None;
        match self.worker.requests.send(request) {
            Ok(()) => self.busy = true,
            Err(_) => self.error = Some("Database worker stopped unexpectedly.".to_owned()),
        }
    }

    fn connect(&mut self) {
        if self.connection_method != ConnectionMethod::TcpIp {
            self.error = Some("Socket and SSH connections are not implemented yet.".to_owned());
            return;
        }
        match ConnectionOptions::with_port(
            &self.host,
            &self.port,
            &self.user,
            &self.password,
            &self.initial_database,
        ) {
            Ok(mut options) => {
                options.time_zone = (!self.time_zone.is_empty()).then(|| self.time_zone.clone());
                options.allow_local_infile = self.allow_local_infile;
                options.enable_cleartext_plugin = self.enable_cleartext_plugin;
                options.request_server_public_key = self.request_server_public_key;
                options.require_tls = self.require_ssl;
                self.show_password = false;
                self.show_ssh_password = false;
                self.send(Request::Connect(options));
            }
            Err(error) => self.error = Some(error),
        }
    }

    /// Menyalin snapshot (sel/baris/baris-lengkap) via worker thread, karena
    /// copy-as-SQL harus menanyakan `Conn` untuk escaping (doc 07 §4).
    fn copy_rows(&mut self, snapshot: CopySnapshot, kind: CopyKind) {
        if self.busy {
            return;
        }
        self.send(Request::CopyRows(snapshot, kind));
    }

    /// Terapkan / bersihkan filter konten lewat worker (doc 06 §2: ganti tab,
    /// ganti database dan setiap LoadContent memakai filter terbaru).
    fn send_filter(&mut self, filter: Option<FilterSpec>) {
        let Some(entry) = self.selected_object.clone() else {
            self.error = Some("Select a table to filter.".to_owned());
            return;
        };
        self.content_filter = filter.clone();
        self.selected_content_cell = None;
        self.send(Request::SetContentFilter { entry, filter });
    }

    /// Lengan menolong `copy_rows` dengan data yang di-borrow dari lokals.
    fn copy_snapshot(
        &mut self,
        columns: Vec<ColumnMeta>,
        rows: Vec<Vec<Option<Vec<u8>>>>,
        table: Option<String>,
        kind: CopyKind,
    ) {
        self.copy_rows(
            CopySnapshot {
                table,
                columns,
                rows,
            },
            kind,
        );
    }

    /// Simpan query saat ini sebagai query favorite.
    fn save_query_favorite(&mut self, name: String) {
        let sql = self.query_text.trim().to_owned();
        if sql.is_empty() {
            return;
        }
        let favorite = QueryFavorite {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            sql,
        };
        self.query_favorites.push(favorite);
        if let Err(error) = self.query_store.save_favorites(&self.query_favorites) {
            self.error = Some(error);
        }
    }

    /// Hapus query favorite by id.
    fn delete_query_favorite(&mut self, id: &str) {
        self.query_favorites.retain(|favorite| favorite.id != id);
        if let Err(error) = self.query_store.save_favorites(&self.query_favorites) {
            self.error = Some(error);
        }
    }

    /// Clear query history (mac: `deleteQueryHistory`).
    fn clear_query_history(&mut self) {
        self.query_history.clear();
        self.query_store.clear_history();
    }

    /// Muat favorite ke form koneksi (doc 03, `updateFavoriteSelection:`).
    fn load_favorite(&mut self, index: usize) {
        let Some(favorite) = self.favorites.get(index) else {
            return;
        };
        self.connection_name = favorite.name.clone();
        self.host = favorite.host.clone();
        self.port = favorite.port.to_string();
        self.user = favorite.user.clone();
        self.password = kc::read_mysql_password(&favorite.name, &favorite.id)
            .ok()
            .flatten()
            .unwrap_or_else(|| favorite.password.clone());
        self.initial_database = favorite.database.clone();
        // SSH password dari keychain
        if let Some(ssh_password) = kc::read_ssh_password(&favorite.name, &favorite.id)
            .ok()
            .flatten()
        {
            self.ssh_password = ssh_password;
        }
        self.connection_method = match favorite.connection_type.as_str() {
            "socket" => ConnectionMethod::Socket,
            "ssh" => ConnectionMethod::Ssh,
            _ => ConnectionMethod::TcpIp,
        };
    }

    /// Simpan form koneksi saat ini sebagai favorite baru.
    #[allow(clippy::collapsible_if)] // let-chain belum stabil di stable Rust
    fn save_favorite(&mut self) {
        let name = if self.connection_name.is_empty() {
            format!("{}:{}", self.host, self.port)
        } else {
            self.connection_name.clone()
        };
        let favorite = Favorite::new(
            name,
            &self.host,
            self.port.parse().unwrap_or(3306),
            &self.user,
        );
        let favorite_id = favorite.id.clone();
        let favorite_name = favorite.name.clone();
        self.favorites.push(favorite);
        if let Err(error) = self.favorites_store.save(&self.favorites) {
            self.error = Some(error);
        }
        // Simpan password ke keychain (Windows Credential Manager / macOS Keychain).
        if !self.password.is_empty() {
            if let Err(error) =
                kc::save_mysql_password(&favorite_name, &favorite_id, &self.password)
            {
                self.error = Some(error);
            }
        }
        if !self.ssh_password.is_empty() {
            if let Err(error) =
                kc::save_ssh_password(&favorite_name, &favorite_id, &self.ssh_password)
            {
                self.error = Some(error);
            }
        }
    }

    /// Hapus favorite by index + password dari keychain.
    fn delete_favorite(&mut self, index: usize) {
        if index < self.favorites.len() {
            let favorite = self.favorites.remove(index);
            let _ = kc::delete_ssh_password(&favorite.name, &favorite.id);
            let _ = kc::delete_mysql_password(&favorite.name, &favorite.id);
            if let Err(error) = self.favorites_store.save(&self.favorites) {
                self.error = Some(error);
            }
        }
    }

    /// Jalankan teks SQL yang berasal dari tombol (mis. `SHOW PROCESSLIST`).
    fn run_sql(&mut self) {
        let sql = self.query_text.trim().to_owned();
        if sql.is_empty() || self.busy {
            return;
        }
        self.query_history.retain(|previous| previous != &sql);
        self.query_history.insert(0, sql.clone());
        self.query_history.truncate(self.query_store.max_history);
        self.query_store.add_history(&sql);
        self.query_output = None;
        self.selected_result_set = 0;
        self.send(Request::ExecuteSql(sql));
    }

    fn clear_object_selection(&mut self) {
        self.selected_object = None;
        self.object_details = None;
        self.content_page = None;
        self.content_page_index = 0;
    }

    fn poll_responses(&mut self) {
        while let Ok(response) = self.worker.responses.try_recv() {
            self.busy = false;
            match response {
                Response::Connected {
                    databases,
                    warning,
                    selected_database,
                    tables,
                    selection_error,
                } => {
                    self.connected = true;
                    self.content_filter = None;
                    self.edit_sheet = None;
                    self.confirm_delete = None;
                    self.filter_prompt = None;
                    self.databases = databases;
                    self.database_name = selected_database.clone().unwrap_or_default();
                    self.selected_database = selected_database;
                    self.tables = tables;
                    self.clear_object_selection();
                    self.server_info = None;
                    self.show_server_info = false;
                    self.main_view = MainView::Tables;
                    self.query_output = None;
                    self.warning = warning;
                    self.error = selection_error;
                }
                Response::Databases(databases) => {
                    if self
                        .selected_database
                        .as_ref()
                        .is_some_and(|selected| !databases.iter().any(|name| name == selected))
                    {
                        self.selected_database = None;
                        self.tables.clear();
                        self.clear_object_selection();
                    }
                    self.databases = databases;
                    self.warning = None;
                }
                Response::Tables { database, entries } => {
                    self.content_filter = None;
                    self.edit_sheet = None;
                    self.confirm_delete = None;
                    self.filter_prompt = None;
                    self.generated_columns = None;
                    self.database_name = database.clone();
                    self.selected_database = Some(database);
                    self.tables = entries;
                    self.clear_object_selection();
                    self.show_server_info = false;
                    self.main_view = MainView::Tables;
                }
                Response::SelectionFailed {
                    database,
                    error,
                    selected,
                } => {
                    if selected {
                        self.database_name = database.clone();
                        self.selected_database = Some(database);
                        self.tables.clear();
                        self.clear_object_selection();
                    }
                    self.error = Some(error);
                }
                Response::ObjectDetails {
                    database,
                    entry,
                    details,
                } => {
                    if self.selected_database.as_deref() == Some(database.as_str())
                        && self.selected_object.as_ref() == Some(&entry)
                    {
                        match details {
                            Ok(details) => {
                                let generated: Vec<String> = details
                                    .columns
                                    .iter()
                                    .filter(|column| {
                                        column.extra.contains("GENERATED")
                                            || column.extra.contains("VIRTUAL")
                                            || column.extra.contains("STORED")
                                    })
                                    .map(|column| column.name.clone())
                                    .collect();
                                self.generated_columns = Some(generated);
                                self.object_details = Some(details);
                            }
                            Err(error) => self.error = Some(error),
                        }
                    }
                }
                Response::ContentPage {
                    database,
                    entry,
                    result,
                } => {
                    if self.selected_database.as_deref() == Some(database.as_str())
                        && self.selected_object.as_ref() == Some(&entry)
                    {
                        match result {
                            Ok((page, content_page)) => {
                                self.content_page = Some(content_page);
                                self.content_page_index = page;
                            }
                            Err(error) => self.error = Some(error),
                        }
                    }
                }
                Response::ServerInfo(info) => match info {
                    Ok(info) => self.server_info = Some(info),
                    Err(error) => self.error = Some(error),
                },
                Response::QueryExecuted {
                    output,
                    database,
                    database_changed,
                    tables,
                    warning,
                } => {
                    self.selected_result_set = output.sets.len().saturating_sub(1);
                    self.query_output = Some(output);
                    self.warning = warning;
                    if database_changed {
                        self.database_name = database.clone().unwrap_or_default();
                        self.selected_database = database;
                        self.tables = tables.unwrap_or_default();
                        self.clear_object_selection();
                        self.show_server_info = false;
                    }
                }
                Response::Disconnected => {
                    self.connected = false;
                    self.content_filter = None;
                    self.edit_sheet = None;
                    self.confirm_delete = None;
                    self.filter_prompt = None;
                    self.databases.clear();
                    self.selected_database = None;
                    self.tables.clear();
                    self.clear_object_selection();
                    self.server_info = None;
                    self.show_server_info = false;
                    self.query_output = None;
                    self.warning = None;
                }
                Response::CopyResult(text) => {
                    self.copied = Some(text);
                }
                Response::EditApplied { affected_rows, sql } => {
                    if affected_rows == 0 {
                        self.warning = Some(crate::database::WARNING_NO_ROWS_AFFECTED.to_owned());
                    }
                    let _ = sql;
                    // Auto-refresh halaman aktif (mac me-refresh setelah edit).
                    if self.main_view == MainView::Content {
                        let page = self.content_page_index;
                        if let Some(entry) = self.selected_object.clone() {
                            self.send(Request::LoadContent { entry, page });
                        }
                    }
                }
                Response::Failed(error) => self.error = Some(error),
            }
        }
    }
}

impl eframe::App for SequelAceApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_responses();
        if let Some(text) = self.copied.take() {
            ui.ctx().copy_text(text);
        }
        if self.connected {
            self.edit_overlays(ui);
            // Drop Database confirmation (fase 14).
            if self.confirm_drop_db {
                let db = self.selected_database.clone().unwrap_or_default();
                let mut confirmed = false;
                egui::Window::new("Drop Database")
                    .collapsible(false)
                    .resizable(false)
                    .show(ui, |ui| {
                        ui.colored_label(
                            egui::Color32::RED,
                            format!(
                                "Are you sure you want to drop database '{db}'?\nAll tables and data will be lost. This action cannot be undone."
                            ),
                        );
                        ui.horizontal(|ui| {
                            if ui.button("Drop Database").clicked() {
                                confirmed = true;
                            }
                            if ui.button("Cancel").clicked() {
                                self.confirm_drop_db = false;
                            }
                        });
                    });
                if confirmed {
                    self.confirm_drop_db = false;
                    let sql = crate::database::table_structure::drop_database_sql(&db, false);
                    self.send(Request::ExecuteSql(sql));
                    self.send(Request::RefreshDatabases);
                }
            }
        }
        if self.busy {
            ui.ctx().request_repaint_after(Duration::from_millis(100));
        }

        if self.error.is_some() || self.warning.is_some() {
            egui::Panel::bottom("messages").show(ui, |ui| {
                if let Some(error) = &self.error {
                    ui.colored_label(egui::Color32::RED, error);
                }
                if let Some(warning) = &self.warning {
                    ui.label(warning);
                }
                if ui.button("Dismiss").clicked() {
                    self.error = None;
                    self.warning = None;
                }
            });
        }

        if !self.connected {
            self.connection_window(ui);
        } else {
            egui::Panel::top("connection_bar").show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Connected");
                    ui.label(format!("{}:{}", self.host, self.port));
                    if ui
                        .add_enabled(!self.busy, egui::Button::new("Server Info"))
                        .clicked()
                    {
                        self.show_server_info = true;
                        self.clear_object_selection();
                        self.server_info = None;
                        self.show_all_variables = false;
                        self.send(Request::LoadServerInfo);
                        self.main_view = MainView::Tables;
                    }
                    if ui
                        .selectable_label(self.main_view == MainView::Tables, "Tables")
                        .clicked()
                    {
                        self.main_view = MainView::Tables;
                    }
                    if ui
                        .add_enabled(
                            self.selected_object.is_some() && !self.busy,
                            egui::Button::selectable(
                                self.main_view == MainView::Content,
                                "Content",
                            ),
                        )
                        .clicked()
                    {
                        self.open_content();
                    }
                    if ui
                        .selectable_label(self.main_view == MainView::Structure, "Structure")
                        .clicked()
                    {
                        self.main_view = MainView::Structure;
                    }
                    if ui
                        .selectable_label(self.main_view == MainView::Query, "SQL Query")
                        .clicked()
                    {
                        self.main_view = MainView::Query;
                    }
                    if ui
                        .selectable_label(self.main_view == MainView::Users, "Users")
                        .clicked()
                    {
                        self.main_view = MainView::Users;
                        self.send(Request::ExecuteSql(
                            "SELECT User, Host FROM mysql.user WHERE User != '' ORDER BY User, Host"
                                .to_owned(),
                        ));
                    }
                    if ui.button("Bundles").clicked() {
                        self.show_bundle_editor = true;
                        self.bundles = crate::database::bundle::load_bundles();
                    }
                    if self.busy {
                        ui.spinner();
                    }
                    if ui
                        .add_enabled(!self.busy, egui::Button::new("Disconnect"))
                        .clicked()
                    {
                        self.send(Request::Disconnect);
                    }
                });
            });
            egui::Panel::left("databases")
                .min_size(230.0)
                .show(ui, |ui| self.database_list(ui));
            if self.main_view == MainView::Tables
                && (self.show_server_info || self.selected_object.is_some())
            {
                egui::Panel::right("details")
                    .min_size(430.0)
                    .show(ui, |ui| self.details_panel(ui));
            }
            egui::CentralPanel::default().show(ui, |ui| match self.main_view {
                MainView::Tables => self.tables_list(ui),
                MainView::Structure => self.structure_view(ui),
                MainView::Content => self.content_view(ui),
                MainView::Query => self.query_view(ui),
                MainView::Users => self.users_view(ui),
            });
            if self.show_bundle_editor {
                self.bundle_editor(ui);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::browser::matches_filter;

    #[test]
    fn filters_names_without_changing_their_display_spelling() {
        assert!(matches_filter("Orders_2026", " orders "));
        assert!(matches_filter("Crème", "CRÈ"));
        assert!(!matches_filter("customers", "orders"));
    }
}
