use super::{MainView, SequelAceApp};
use eframe::egui;

impl SequelAceApp {
    pub(super) fn details_panel(&mut self, ui: &mut egui::Ui) {
        if self.show_server_info {
            ui.heading("Server Info");
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!self.busy, egui::Button::new("Process List"))
                    .clicked()
                {
                    self.query_text = "SHOW PROCESSLIST".to_owned();
                    self.main_view = MainView::Query;
                    self.run_sql();
                }
                if ui
                    .button(if self.show_all_variables {
                        "Basic Variables"
                    } else {
                        "All Variables"
                    })
                    .clicked()
                {
                    self.show_all_variables = !self.show_all_variables;
                }
            });
            if let Some(info) = &self.server_info {
                ui.horizontal(|ui| {
                    ui.label("Version");
                    ui.label(&info.version);
                });
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("server_variables")
                        .striped(true)
                        .show(ui, |ui| {
                            for (name, value) in &info.variables {
                                if !self.show_all_variables
                                    && ![
                                        "version_comment",
                                        "character_set_server",
                                        "collation_server",
                                        "sql_mode",
                                        "time_zone",
                                    ]
                                    .contains(&name.as_str())
                                {
                                    continue;
                                }
                                ui.label(name);
                                ui.label(value);
                                ui.end_row();
                            }
                        });
                });
            } else if self.busy {
                ui.spinner();
            }
            return;
        }
        let Some(entry) = self.selected_object.clone() else {
            return;
        };
        ui.heading(&entry.name);
        if ui
            .add_enabled(!self.busy, egui::Button::new("Browse Rows"))
            .clicked()
        {
            self.open_content();
        }
        if let Some(details) = &self.object_details {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(table_info) = &details.table_info {
                    ui.strong("Table Info");
                    match table_info {
                        Ok(info) => {
                            egui::Grid::new("table_info").striped(true).show(ui, |ui| {
                                for (label, value) in [
                                    ("Engine", &info.engine),
                                    ("Estimated Rows", &info.estimated_rows),
                                    ("Data Length (bytes)", &info.data_length),
                                    ("Index Length (bytes)", &info.index_length),
                                    ("Collation", &info.collation),
                                    ("Comment", &info.comment),
                                ] {
                                    ui.label(label);
                                    ui.label(value);
                                    ui.end_row();
                                }
                            });
                        }
                        Err(error) => {
                            ui.colored_label(egui::Color32::RED, error);
                        }
                    }
                    ui.separator();
                }
                ui.horizontal(|ui| {
                    ui.strong("Columns");
                    if ui.button("Copy CREATE SQL").clicked() {
                        ui.ctx().copy_text(details.create_sql.clone());
                    }
                });
                egui::ScrollArea::horizontal()
                    .max_height(250.0)
                    .show(ui, |ui| {
                        egui::Grid::new("object_columns")
                            .striped(true)
                            .show(ui, |ui| {
                                for heading in ["Name", "Type", "Null", "Key", "Default", "Extra"] {
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
                                    ui.end_row();
                                }
                            });
                    });
                if let Some(indexes) = &details.indexes {
                    ui.separator();
                    ui.strong("Indexes");
                    match indexes {
                        Ok(indexes) if indexes.is_empty() => {
                            ui.label("No indexes");
                        }
                        Ok(indexes) => {
                            egui::ScrollArea::horizontal().show(ui, |ui| {
                                egui::Grid::new("object_indexes")
                                    .striped(true)
                                    .show(ui, |ui| {
                                        for heading in
                                            ["Name", "Column", "Position", "Unique", "Type"]
                                        {
                                            ui.strong(heading);
                                        }
                                        ui.end_row();
                                        for index in indexes {
                                            ui.label(&index.name);
                                            ui.label(&index.column);
                                            ui.label(&index.sequence);
                                            ui.label(if index.unique { "Yes" } else { "No" });
                                            ui.label(&index.index_type);
                                            ui.end_row();
                                        }
                                    });
                            });
                        }
                        Err(error) => {
                            ui.colored_label(egui::Color32::RED, error);
                        }
                    }
                }
                if let Some(foreign_keys) = &details.foreign_keys {
                    ui.separator();
                    ui.strong("Foreign Keys");
                    match foreign_keys {
                        Ok(foreign_keys) if foreign_keys.is_empty() => {
                            ui.label("No foreign keys");
                        }
                        Ok(foreign_keys) => {
                            egui::ScrollArea::horizontal().show(ui, |ui| {
                                egui::Grid::new("object_foreign_keys").striped(true).show(
                                    ui,
                                    |ui| {
                                        for heading in [
                                            "Name",
                                            "Column",
                                            "Referenced Database",
                                            "Referenced Table",
                                            "Referenced Column",
                                        ] {
                                            ui.strong(heading);
                                        }
                                        ui.end_row();
                                        for key in foreign_keys {
                                            ui.label(&key.name);
                                            ui.label(&key.column);
                                            ui.label(&key.referenced_schema);
                                            ui.label(&key.referenced_table);
                                            ui.label(&key.referenced_column);
                                            ui.end_row();
                                        }
                                    },
                                );
                            });
                        }
                        Err(error) => {
                            ui.colored_label(egui::Color32::RED, error);
                        }
                    }
                }
                ui.separator();
                ui.strong("CREATE SQL");
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    let mut sql = details.create_sql.clone();
                    ui.add(
                        egui::TextEdit::multiline(&mut sql)
                            .interactive(false)
                            .desired_rows(12)
                            .desired_width(f32::INFINITY),
                    );
                });
            });
        } else if self.busy {
            ui.spinner();
        }
    }
}
