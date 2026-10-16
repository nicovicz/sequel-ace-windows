//! Bundle Editor UI (fase 15) — port `SPBundleEditorController`.
//!
//! Tree: daftar scope + bundles. Edit ke: name/scope/command/input/output/
//! trigger/keyEquivalent/description/author/disabled. Save/hapus bundle,
//! test-run bundle terhadap konteks saat ini.

use super::SequelAceApp;
use crate::database::bundle::{
    BUNDLE_INPUTS, BUNDLE_OUTPUTS, BUNDLE_SCOPES, BUNDLE_TRIGGERS, BundleContext, BundleEntry,
    delete_bundle, load_bundles, run_bundle_command, save_bundle,
};
use eframe::egui;

/// Drop-down pilih satu nilai dari daftar string — egui ComboBox yang disederhanakan.
fn combo_box(ui: &mut egui::Ui, options: &[&str], value: &mut String, id: &str) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(value.as_str())
        .show_ui(ui, |ui| {
            for option in options {
                ui.selectable_value(value, (*option).to_owned(), (*option).to_owned());
            }
        });
}

impl SequelAceApp {
    /// Tampilkan Bundle Editor window (modal-look). Flag open di-take dulu agar
    /// tidak double-borrow saat closure memutasi self.
    pub(super) fn bundle_editor(&mut self, ui: &mut egui::Ui) {
        let mut open = self.show_bundle_editor;
        egui::Window::new("Bundle Editor")
            .open(&mut open)
            .resizable(true)
            .default_width(720.0)
            .default_height(520.0)
            .show(ui, |ui| {
                self.bundle_editor_inner(ui);
            });
        self.show_bundle_editor = open;
        if !self.show_bundle_editor {
            self.bundle_error = None;
            self.bundle_output.clear();
        }
    }

    fn bundle_editor_inner(&mut self, ui: &mut egui::Ui) {
        // Reload button.
        ui.horizontal(|ui| {
            if ui.button("Reload Bundles").clicked() {
                self.bundles = load_bundles();
                self.bundle_selected = self
                    .bundles
                    .first()
                    .map(|_| 0)
                    .filter(|_| !self.bundles.is_empty());
            }
            // New bundle.
            if ui.button("New Bundle").clicked() {
                let new_entry = BundleEntry {
                    name: "New Bundle".to_owned(),
                    scope: "general".to_owned(),
                    command: String::new(),
                    input: "none".to_owned(),
                    input_fallback: "none".to_owned(),
                    output: "showastexttooltip".to_owned(),
                    trigger: "none".to_owned(),
                    key_equivalent: String::new(),
                    description: String::new(),
                    author: String::new(),
                    disabled: false,
                    // Path di-set saat save pertama.
                    path: crate::database::bundle::bundle_directory().join("New Bundle.saBundle"),
                };
                if let Ok(()) = save_bundle(&new_entry) {
                    self.bundles = load_bundles();
                    if let Some(index) = self
                        .bundles
                        .iter()
                        .position(|bundle| bundle.name == new_entry.name)
                    {
                        self.bundle_selected = Some(index);
                    }
                }
            }
            // Delete bundle.
            let delete_enabled = self.bundle_selected.is_some();
            if ui
                .add_enabled(delete_enabled, egui::Button::new("Delete"))
                .clicked()
            {
                self.bundle_confirm_delete = self.bundle_selected.take();
            }
        });
        ui.separator();

        // Split: list kiri + form kanan.
        ui.columns(2, |columns| {
            // Kiri: bundle list by scope.
            columns[0].heading("BUNDLES");
            if columns[0].button("Reload").clicked() {
                self.bundles = load_bundles();
            }
            egui::ScrollArea::vertical().show(&mut columns[0], |ui| {
                for scope in BUNDLE_SCOPES {
                    ui.strong(*scope);
                    for (index, bundle) in self.bundles.iter().enumerate() {
                        if bundle.scope != *scope {
                            continue;
                        }
                        let selected = self.bundle_selected == Some(index);
                        let label = if bundle.disabled {
                            format!("✗ {}", bundle.name)
                        } else {
                            bundle.name.clone()
                        };
                        if ui.selectable_label(selected, label).clicked() {
                            self.bundle_selected = Some(index);
                        }
                    }
                }
            });

            // Kanan: form editor. Clone-based agar tidak double-borrow saat
            // memanggil self.save/test (bundle ditangani lokal lalu di-apply).
            let Some(index) = self.bundle_selected else {
                columns[1].label("Select a bundle");
                return;
            };
            let Some(original) = self.bundles.get(index) else {
                return;
            };
            let mut bundle = original.clone();
            self.bundle_form(&mut columns[1], &mut bundle, index);
        });

        // Konfirmasi delete.
        if let Some(index) = self.bundle_confirm_delete.take() {
            let mut confirmed = false;
            egui::Window::new("Delete Bundle")
                .collapsible(false)
                .resizable(false)
                .show(ui, |ui| {
                    let name = self
                        .bundles
                        .get(index)
                        .map(|bundle| bundle.name.clone())
                        .unwrap_or_default();
                    ui.label(format!("Are you sure you want to delete bundle '{name}'?"));
                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() {
                            confirmed = true;
                        }
                        if ui.button("Cancel").clicked() {
                            self.bundle_confirm_delete = None;
                        }
                    });
                });
            if confirmed {
                if let Some(entry) = self.bundles.get(index) {
                    let entry_clone = entry.clone();
                    let _ = delete_bundle(&entry_clone);
                }
                self.bundles = load_bundles();
                self.bundle_selected = (!self.bundles.is_empty()).then_some(0);
            }
        }
    }

    /// Form edit satu bundle. `index` dipakai untuk tulis kembali ke list.
    fn bundle_form(&mut self, ui: &mut egui::Ui, bundle: &mut BundleEntry, index: usize) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("bundle_form_grid").show(ui, |ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut bundle.name);
                ui.end_row();

                ui.label("Scope:");
                combo_box(ui, BUNDLE_SCOPES, &mut bundle.scope, "bundle_scope");
                ui.end_row();

                ui.label("Trigger:");
                combo_box(ui, BUNDLE_TRIGGERS, &mut bundle.trigger, "bundle_trigger");
                ui.end_row();

                ui.label("Input:");
                combo_box(ui, BUNDLE_INPUTS, &mut bundle.input, "bundle_input");
                ui.end_row();

                ui.label("Input Fallback:");
                combo_box(
                    ui,
                    BUNDLE_INPUTS,
                    &mut bundle.input_fallback,
                    "bundle_input_fallback",
                );
                ui.end_row();

                ui.label("Output:");
                combo_box(ui, BUNDLE_OUTPUTS, &mut bundle.output, "bundle_output");
                ui.end_row();

                ui.label("Key Equivalent:");
                // Format mac: `^@~$` + char.
                ui.text_edit_singleline(&mut bundle.key_equivalent);
                ui.end_row();

                ui.label("Description:");
                ui.text_edit_singleline(&mut bundle.description);
                ui.end_row();

                ui.label("Author:");
                ui.text_edit_singleline(&mut bundle.author);
                ui.end_row();

                ui.label("Disabled:");
                ui.checkbox(&mut bundle.disabled, "");
                ui.end_row();
            });

            ui.separator();
            ui.label("Command:");
            ui.add_sized(
                ui.available_size_before_wrap(),
                egui::TextEdit::multiline(&mut bundle.command).code_editor(),
            );

            ui.separator();
            ui.horizontal(|ui| {
                let can_save = !bundle.name.trim().is_empty() && !bundle.scope.trim().is_empty();
                if ui
                    .add_enabled(can_save, egui::Button::new("Save"))
                    .clicked()
                {
                    if let Err(error) = save_bundle(bundle) {
                        self.bundle_error = Some(error);
                    } else {
                        self.bundles = load_bundles();
                        self.bundle_selected = Some(index).filter(|i| i < &self.bundles.len());
                    }
                }
                // Test run.
                if ui.button("Test Run").clicked() {
                    let context = self.bundle_context();
                    match run_bundle_command(bundle, &context) {
                        Ok(result) => {
                            self.bundle_output =
                                format!("exit: {}\n\n{}", result.exit_code, result.output);
                        }
                        Err(error) => self.bundle_error = Some(error),
                    }
                }
            });
            if !self.bundle_output.is_empty() {
                ui.separator();
                ui.strong("Output");
                ui.monospace(self.bundle_output.clone());
            }
            if let Some(error) = &self.bundle_error {
                ui.colored_label(egui::Color32::RED, error);
            }
        });
    }

    /// Kumpulkan konteks eksekusi dari state app saat ini.
    fn bundle_context(&self) -> BundleContext {
        BundleContext {
            host: self.host.clone(),
            port: self.port.clone(),
            user: self.user.clone(),
            selected_database: self.selected_database.clone().unwrap_or_default(),
            selected_table: self
                .selected_object
                .as_ref()
                .map(|entry| entry.name.clone())
                .unwrap_or_default(),
            selected_tables: self
                .selected_object
                .as_ref()
                .map(|entry| vec![entry.name.clone()])
                .unwrap_or_default(),
            selected_row_indices: self.selected_content_rows.clone(),
            current_word: String::new(),
            current_line: String::new(),
            current_query: self.query_text.clone(),
            selected_text: String::new(),
            used_query_for_table: String::new(),
            rdbms_type: "MySQL".to_owned(),
            rdbms_version: self
                .server_info
                .as_ref()
                .map(|info| info.version.clone())
                .unwrap_or_default(),
            bundle_scope: None,
            current_edited_table: String::new(),
            current_edited_column_name: String::new(),
            all_databases: self.databases.clone(),
            all_tables: self.tables.iter().map(|entry| entry.name.clone()).collect(),
            all_views: Vec::new(),
            all_functions: Vec::new(),
            all_procedures: Vec::new(),
            bundle_input: None,
        }
    }
}
