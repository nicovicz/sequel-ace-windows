//! Table Structure editor + User Manager UI (fase 12).
//!
//! Structure: tampilkan kolom, Add/Drop/Modify Column via `table_structure.rs`.
//! User Manager: list/create/drop user via `user_manager.rs`.

use super::{MainView, SequelAceApp};
use crate::database::Request;
use crate::database::table_structure::{
    ColumnDefinition, add_column_sql, change_column_sql, drop_column_sql, set_auto_increment_sql,
};
use crate::database::user_manager::{
    PrivilegeSpec, create_user_sql, drop_user_sql, flush_privileges_sql, grant_sql, list_users_sql,
};
use eframe::egui;

/// State untuk form Add/Edit Column.
pub struct ColumnDefinitionState {
    pub name: String,
    pub data_type: String,
    pub length: String,
    pub nullable: bool,
    pub default_value: String,
    pub auto_increment: bool,
    pub after_column: String,
    pub is_edit: bool,
    pub old_name: Option<String>,
}

/// Parse `VARCHAR(255)` → (`VARCHAR`, `255`); `INT` → (`INT`, `""`).
fn parse_column_type(data_type: &str) -> (String, String) {
    let trimmed = data_type.trim();
    if let Some(paren_start) = trimmed.find('(') {
        let base = trimmed[..paren_start].to_owned();
        let length = trimmed[paren_start + 1..trimmed.len().saturating_sub(1)].to_owned();
        (base, length)
    } else {
        (trimmed.to_owned(), String::new())
    }
}

impl SequelAceApp {
    /// Tampilkan Structure tab: kolom + Add/Drop/Modify.
    pub(super) fn structure_view(&mut self, ui: &mut egui::Ui) {
        let Some(entry) = self.selected_object.clone() else {
            ui.label("Select a table to view its structure");
            return;
        };
        ui.heading(&entry.name);
        ui.horizontal(|ui| {
            if ui.button("Add Column").clicked() && self.structure_add_column.is_none() {
                self.structure_add_column = Some(ColumnDefinitionState {
                    name: String::new(),
                    data_type: "VARCHAR".to_owned(),
                    length: "255".to_owned(),
                    nullable: true,
                    default_value: String::new(),
                    auto_increment: false,
                    after_column: String::new(),
                    is_edit: false,
                    old_name: None,
                });
            }
            // Rename Table (doc 07 §2, ALTER TABLE RENAME).
            if ui.button("Rename Table").clicked() && !self.rename_table_dialog {
                self.rename_table_old = entry.name.clone();
                self.rename_table_new = entry.name.clone();
                self.rename_table_dialog = true;
            }
            // Drop Table (doc 07 §3).
            if ui.button("Drop Table").clicked() {
                self.confirm_drop_table = true;
            }
            if ui.button("Back").clicked() {
                self.main_view = MainView::Tables;
            }
        });
        ui.separator();

        // Tampilkan kolom dari object_details.
        let Some(details) = self.object_details.as_ref() else {
            ui.label("Loading columns…");
            return;
        };
        egui::ScrollArea::both().show(ui, |ui| {
            egui::Grid::new("structure_columns")
                .striped(true)
                .show(ui, |ui| {
                    for heading in ["Name", "Type", "Null", "Key", "Default", "Extra", ""] {
                        ui.strong(heading);
                    }
                    ui.end_row();
                    for column in &details.columns {
                        ui.label(&column.name);
                        ui.label(&column.data_type);
                        ui.label(&column.nullable);
                        ui.label(&column.key);
                        ui.label(column.default_value.as_deref().unwrap_or("NULL"));
                        ui.label(&column.extra);
                        // Edit + Drop button per kolom.
                        if ui
                            .add_enabled(!self.busy, egui::Button::new("Edit"))
                            .clicked()
                        {
                            // Parse tipe + length dari data_type string.
                            let (base_type, length) = parse_column_type(&column.data_type);
                            self.structure_add_column = Some(ColumnDefinitionState {
                                name: column.name.clone(),
                                data_type: base_type,
                                length,
                                nullable: column.nullable == "YES",
                                default_value: column
                                    .default_value
                                    .as_deref()
                                    .unwrap_or("")
                                    .to_owned(),
                                auto_increment: column.extra.contains("auto_increment"),
                                after_column: String::new(),
                                is_edit: true,
                                old_name: Some(column.name.clone()),
                            });
                        }
                        if ui
                            .add_enabled(!self.busy, egui::Button::new("Drop"))
                            .clicked()
                        {
                            self.confirm_drop_column = Some(column.name.clone());
                        }
                        ui.end_row();
                    }
                });
        });

        // Reset AUTO_INCREMENT.
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("AUTO_INCREMENT:");
            let mut value = String::new();
            if ui.text_edit_singleline(&mut value).lost_focus()
                && !value.is_empty()
                && ui.input(|input| input.key_pressed(egui::Key::Enter))
                && let Ok(n) = value.parse::<u64>()
            {
                let sql = set_auto_increment_sql(&entry.name, n);
                self.send(Request::ExecuteSql(sql));
            }
        });

        // Render Add Column form. `.take()` memindahkan ownership agar tidak
        // double-borrow saat memanggil `self.send()`.
        let Some(mut add_state) = self.structure_add_column.take() else {
            return;
        };
        {
            ui.separator();
            ui.heading(if add_state.is_edit {
                "Modify Column"
            } else {
                "Add Column"
            });
            egui::Grid::new("add_column_form").show(ui, |ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut add_state.name);
                ui.end_row();
                ui.label("Type:");
                ui.text_edit_singleline(&mut add_state.data_type);
                ui.end_row();
                ui.label("Length:");
                ui.text_edit_singleline(&mut add_state.length);
                ui.end_row();
                ui.label("Nullable:");
                ui.checkbox(&mut add_state.nullable, "");
                ui.end_row();
                ui.label("Default:");
                ui.text_edit_singleline(&mut add_state.default_value);
                ui.end_row();
                ui.label("Auto Increment:");
                ui.checkbox(&mut add_state.auto_increment, "");
                ui.end_row();
                ui.label("After:");
                ui.text_edit_singleline(&mut add_state.after_column);
                ui.end_row();
            });
            let mut apply = false;
            ui.horizontal(|ui| {
                if ui.button("Apply").clicked() {
                    apply = true;
                    let definition = ColumnDefinition {
                        name: add_state.name.clone(),
                        old_name: add_state.old_name.clone(),
                        data_type: add_state.data_type.clone(),
                        length: (!add_state.length.is_empty()).then(|| add_state.length.clone()),
                        unsigned: false,
                        zerofill: false,
                        nullable: add_state.nullable,
                        default_value: (!add_state.default_value.is_empty())
                            .then(|| format!("'{}'", add_state.default_value)),
                        auto_increment: add_state.auto_increment,
                        comment: None,
                        charset: None,
                        collation: None,
                        after_column: (!add_state.after_column.is_empty())
                            .then(|| add_state.after_column.clone()),
                    };
                    let sql = if add_state.is_edit {
                        change_column_sql(&entry.name, &definition)
                    } else {
                        add_column_sql(&entry.name, &definition)
                    };
                    self.send(Request::ExecuteSql(sql));
                    self.structure_add_column = None;
                }
                if ui.button("Cancel").clicked() {
                    self.structure_add_column = None;
                }
            });
        }

        // Rename Table dialog (ALTER TABLE RENAME).
        if self.rename_table_dialog {
            egui::Window::new("Rename Table")
                .collapsible(false)
                .resizable(false)
                .show(ui, |ui| {
                    ui.label(format!("Rename '{}' to:", self.rename_table_old));
                    ui.text_edit_singleline(&mut self.rename_table_new);
                    ui.horizontal(|ui| {
                        if ui.button("Rename").clicked() {
                            let sql = format!(
                                "ALTER TABLE {} RENAME TO {}",
                                crate::database::quote_identifier(&self.rename_table_old),
                                crate::database::quote_identifier(&self.rename_table_new),
                            );
                            self.send(Request::ExecuteSql(sql));
                            self.rename_table_dialog = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.rename_table_dialog = false;
                        }
                    });
                });
        }

        // Drop Table confirmation.
        if self.confirm_drop_table {
            egui::Window::new("Drop Table")
                .collapsible(false)
                .resizable(false)
                .show(ui, |ui| {
                    ui.colored_label(
                        egui::Color32::RED,
                        format!(
                            "Are you sure you want to drop table '{}'?\nThis action cannot be undone.",
                            entry.name
                        ),
                    );
                    ui.horizontal(|ui| {
                        if ui.button("Drop Table").clicked() {
                            let sql = crate::database::table_structure::drop_table_sql(
                                &entry.name,
                                false,
                            );
                            self.send(Request::ExecuteSql(sql));
                            self.confirm_drop_table = false;
                            self.main_view = MainView::Tables;
                        }
                        if ui.button("Cancel").clicked() {
                            self.confirm_drop_table = false;
                        }
                    });
                });
        }

        // Konfirmasi drop column.
        if let Some(column_name) = self.confirm_drop_column.take() {
            let mut confirmed = false;
            egui::Window::new("Drop Column")
                .collapsible(false)
                .resizable(false)
                .show(ui, |ui| {
                    ui.label(format!(
                        "Are you sure you want to drop column '{column_name}'?"
                    ));
                    ui.horizontal(|ui| {
                        if ui.button("Drop").clicked() {
                            confirmed = true;
                        }
                        if ui.button("Cancel").clicked() {
                            self.confirm_drop_column = None;
                        }
                    });
                });
            if confirmed {
                let sql = drop_column_sql(&entry.name, &column_name);
                self.send(Request::ExecuteSql(sql));
            }
        }
    }

    /// Tampilkan User Manager tab.
    pub(super) fn users_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("User Manager");
        ui.horizontal(|ui| {
            if ui.button("Refresh").clicked() {
                self.send(Request::ExecuteSql(list_users_sql()));
            }
            if ui.button("FLUSH PRIVILEGES").clicked() {
                self.send(Request::ExecuteSql(flush_privileges_sql()));
            }
        });
        ui.separator();

        // Daftar user dari hasil query terakhir.
        let Some(output) = self.query_output.as_ref() else {
            ui.label("Click Refresh to load users.");
            return;
        };
        let Some(set) = output.sets.first() else {
            return;
        };
        if set.columns.is_empty() || set.columns[0].name != "User" {
            ui.label("No user data available.");
            return;
        }

        // Simpan user list ke state agar bisa di-referensi.
        self.user_list = set
            .rows
            .iter()
            .map(|row| {
                let user = row
                    .first()
                    .map(|cell| String::from_utf8_lossy(cell.as_deref().unwrap_or(b"")))
                    .unwrap_or_default()
                    .into_owned();
                let host = row
                    .get(1)
                    .map(|cell| String::from_utf8_lossy(cell.as_deref().unwrap_or(b"")))
                    .unwrap_or_default()
                    .into_owned();
                (user, host)
            })
            .collect();

        // Tampilkan daftar user + tombol Drop.
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("user_list_grid")
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("User");
                    ui.strong("Host");
                    ui.strong("");
                    ui.end_row();
                    let user_list = self.user_list.clone();
                    for (user, host) in &user_list {
                        ui.label(user);
                        ui.label(host);
                        if ui
                            .add_enabled(!self.busy, egui::Button::new("Drop"))
                            .clicked()
                        {
                            self.user_confirm_drop = Some((user.clone(), host.clone()));
                        }
                        ui.end_row();
                    }
                });
        });

        // Konfirmasi drop user.
        if let Some((user, host)) = self.user_confirm_drop.take() {
            let mut confirmed = false;
            egui::Window::new("Drop User")
                .collapsible(false)
                .resizable(false)
                .show(ui, |ui| {
                    ui.label(format!(
                        "Are you sure you want to drop user '{user}'@'{host}'?"
                    ));
                    ui.horizontal(|ui| {
                        if ui.button("Drop").clicked() {
                            confirmed = true;
                        }
                        if ui.button("Cancel").clicked() {
                            self.user_confirm_drop = None;
                        }
                    });
                });
            if confirmed {
                let sql = drop_user_sql(&user, &host);
                self.send(Request::ExecuteSql(sql));
                self.send(Request::ExecuteSql(list_users_sql()));
            }
        }

        ui.separator();
        ui.heading("Create User");

        // Form create user.
        egui::Grid::new("create_user_form").show(ui, |ui| {
            ui.label("Username:");
            ui.text_edit_singleline(&mut self.user_new_name);
            ui.end_row();
            ui.label("Host:");
            ui.text_edit_singleline(&mut self.user_new_host);
            ui.end_row();
            ui.label("Password:");
            ui.text_edit_singleline(&mut self.user_new_password);
            ui.end_row();
        });
        ui.horizontal(|ui| {
            let can_create =
                !self.user_new_name.is_empty() && !self.user_new_host.is_empty() && !self.busy;
            if ui
                .add_enabled(can_create, egui::Button::new("Create User"))
                .clicked()
            {
                let params = crate::database::user_manager::CreateUserParams {
                    user: self.user_new_name.clone(),
                    host: self.user_new_host.clone(),
                    password: (!self.user_new_password.is_empty())
                        .then(|| self.user_new_password.clone()),
                    auth_plugin: None,
                };
                let sql = create_user_sql(&params);
                self.send(Request::ExecuteSql(sql));
                // Grant basic privileges.
                let spec = PrivilegeSpec {
                    privileges: vec!["ALL PRIVILEGES".to_owned()],
                    database: "*".to_owned(),
                    user: self.user_new_name.clone(),
                    host: self.user_new_host.clone(),
                    with_grant_option: false,
                };
                let grant = grant_sql(&spec);
                self.send(Request::ExecuteSql(grant));
                // Refresh list.
                self.send(Request::ExecuteSql(list_users_sql()));
                self.user_new_name.clear();
                self.user_new_password.clear();
            }
        });
    }
}
