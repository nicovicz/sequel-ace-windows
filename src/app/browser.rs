use super::SequelAceApp;
use crate::database::table_structure::create_database_sql;
use crate::database::{EntryKind, Request, TableEntry, partition_databases};
use eframe::egui;

impl SequelAceApp {
    pub(super) fn database_list(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Database");
            if ui
                .add_enabled(!self.busy, egui::Button::new("Refresh Databases"))
                .clicked()
            {
                self.send(Request::RefreshDatabases);
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.database_name);
            if ui
                .add_enabled(
                    !self.busy && !self.database_name.trim().is_empty(),
                    egui::Button::new("Select Database"),
                )
                .clicked()
            {
                self.send(Request::SelectDatabase(
                    self.database_name.trim().to_owned(),
                ));
            }
        });
        ui.horizontal(|ui| {
            // Create Database (doc 07: create_database_sql).
            if ui
                .add_enabled(
                    !self.busy && !self.database_name.trim().is_empty(),
                    egui::Button::new("Create"),
                )
                .clicked()
            {
                let sql = create_database_sql(
                    self.database_name.trim(),
                    true,
                    Some("utf8mb4"),
                    Some("utf8mb4_unicode_ci"),
                );
                self.send(Request::ExecuteSql(sql));
                self.send(Request::RefreshDatabases);
            }
            // Drop Database (doc 07: drop_database_sql).
            if ui
                .add_enabled(
                    !self.busy
                        && self.selected_database.is_some()
                        && self.selected_database.as_deref().is_some_and(|db| {
                            !["mysql", "information_schema", "performance_schema", "sys"]
                                .contains(&db)
                        }),
                    egui::Button::new("Drop"),
                )
                .clicked()
            {
                self.confirm_drop_db = true;
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Filter Databases");
            ui.text_edit_singleline(&mut self.database_filter);
        });
        ui.separator();
        let (system, user) = partition_databases(&self.databases);
        let system: Vec<String> = system
            .into_iter()
            .filter(|name| matches_filter(name, &self.database_filter))
            .map(str::to_owned)
            .collect();
        let user: Vec<String> = user
            .into_iter()
            .filter(|name| matches_filter(name, &self.database_filter))
            .map(str::to_owned)
            .collect();
        egui::ScrollArea::vertical().show(ui, |ui| {
            if !system.is_empty() {
                ui.strong("System Databases");
                for database in &system {
                    self.database_button(ui, database);
                }
                ui.separator();
            }
            for database in &user {
                self.database_button(ui, database);
            }
            if self.databases.is_empty() {
                ui.label("No databases available");
            } else if system.is_empty() && user.is_empty() {
                ui.label("No matching databases");
            }
        });
    }

    fn database_button(&mut self, ui: &mut egui::Ui, database: &str) {
        let selected = self.selected_database.as_deref() == Some(database);
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!self.busy, egui::Button::selectable(selected, database))
                .clicked()
            {
                self.database_name = database.to_owned();
                self.send(Request::SelectDatabase(database.to_owned()));
            }
            if ui.small_button("Copy").clicked() {
                ui.ctx().copy_text(database.to_owned());
            }
        });
    }

    pub(super) fn tables_list(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(
                self.selected_database
                    .as_deref()
                    .unwrap_or("Choose Database..."),
            );
            if self.selected_database.is_some()
                && ui
                    .add_enabled(!self.busy, egui::Button::new("Refresh Tables"))
                    .clicked()
            {
                self.send(Request::RefreshTables);
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Filter Tables");
            ui.text_edit_singleline(&mut self.table_filter);
        });
        ui.separator();
        if self.selected_database.is_none() {
            ui.label("Select a database");
            return;
        }
        if self.tables.is_empty() {
            ui.label("No tables or views");
            return;
        }
        let filtered: Vec<TableEntry> = self
            .tables
            .iter()
            .filter(|entry| matches_filter(&entry.name, &self.table_filter))
            .cloned()
            .collect();
        if filtered.is_empty() {
            ui.label("No matching tables or views");
        }
        egui::ScrollArea::vertical().show(ui, |ui| {
            for entry in filtered {
                ui.horizontal(|ui| {
                    ui.label(match entry.kind {
                        EntryKind::Table => "Table",
                        EntryKind::View => "View",
                    });
                    let selected = self.selected_object.as_ref() == Some(&entry);
                    if ui
                        .add_enabled(!self.busy, egui::Button::selectable(selected, &entry.name))
                        .clicked()
                    {
                        self.selected_object = Some(entry.clone());
                        self.object_details = None;
                        self.content_page = None;
                        self.content_page_index = 0;
                        self.show_server_info = false;
                        self.send(Request::LoadObject(entry.clone()));
                    }
                    if ui.small_button("Copy").clicked() {
                        ui.ctx().copy_text(entry.name.clone());
                    }
                });
            }
        });
    }
}

pub(super) fn matches_filter(name: &str, filter: &str) -> bool {
    name.to_lowercase().contains(&filter.trim().to_lowercase())
}
