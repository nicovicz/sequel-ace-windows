use super::SequelAceApp;
use super::grid::{COPY_CELL_BYTES, grid_view};
use crate::database::{CopyKind, MAX_CELL_BYTES, format_cell};
use eframe::egui;

impl SequelAceApp {
    pub(super) fn query_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("SQL Query");
        ui.add(
            egui::TextEdit::multiline(&mut self.query_text)
                .desired_rows(8)
                .desired_width(f32::INFINITY)
                .hint_text("Enter SQL here"),
        );
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    !self.busy && !self.query_text.trim().is_empty(),
                    egui::Button::new("Run SQL"),
                )
                .clicked()
            {
                self.run_sql();
            }
            // Query history (persisted, doc 08b §1).
            egui::ComboBox::from_id_salt("query_history")
                .selected_text("Recent queries")
                .show_ui(ui, |ui| {
                    if ui.button("Clear History").clicked() {
                        self.clear_query_history();
                    }
                    ui.separator();
                    let history = self.query_history.clone();
                    for sql in history {
                        let label = sql.lines().next().unwrap_or("");
                        if ui.selectable_label(false, label).clicked() {
                            self.query_text = sql;
                        }
                    }
                });
            // Query favorites (doc 08b §1, `SPQueryFavoriteManager`).
            let favorites = self.query_favorites.clone();
            egui::ComboBox::from_id_salt("query_favorites")
                .selected_text("Query Favorites")
                .show_ui(ui, |ui| {
                    if favorites.is_empty() {
                        ui.label("No favorites saved");
                    }
                    for favorite in &favorites {
                        ui.horizontal(|ui| {
                            if ui.selectable_label(false, &favorite.name).clicked() {
                                self.query_text = favorite.sql.clone();
                            }
                            if ui.button("×").clicked() {
                                self.delete_query_favorite(&favorite.id);
                            }
                        });
                    }
                });
            if ui.button("Save Query Favorite").clicked() {
                let name = format!("Query {}", self.query_favorites.len() + 1);
                self.save_query_favorite(name);
            }
            if self.busy {
                ui.spinner();
            }
        });
        ui.separator();

        // Salin nilai yang diperlukan ke lokal supaya tidak menahan pinjaman
        // `self` saat memanggil `self.copy_snapshot` (aturan pinjaman borrow).
        let state = match self.query_output.as_ref() {
            Some(output) => {
                let set = output.sets.get(self.selected_result_set);
                (
                    Some(output.elapsed_ms),
                    output.sets_truncated,
                    output.sets.len(),
                    set.map(|set| {
                        (
                            set.columns.clone(),
                            set.rows.clone(),
                            set.truncated,
                            set.affected_rows,
                        )
                    }),
                )
            }
            None => (None, false, 0, None),
        };
        let Some(elapsed_ms) = state.0 else {
            ui.label("Run a query to see results");
            return;
        };
        let (sets_truncated, sets_len) = (state.1, state.2);
        let Some((columns_meta, rows_raw, set_truncated, affected_rows)) = state.3 else {
            return;
        };

        ui.horizontal(|ui| {
            ui.label("Duration (ms)");
            ui.label(elapsed_ms.to_string());
        });
        if sets_truncated {
            ui.label("Only the first 20 result sets are shown");
        }
        if sets_len > 1 {
            ui.horizontal_wrapped(|ui| {
                for index in 0..sets_len {
                    if ui
                        .selectable_label(
                            self.selected_result_set == index,
                            format!("Result {}", index + 1),
                        )
                        .clicked()
                    {
                        self.selected_result_set = index;
                        self.selected_query_cell = None;
                    }
                }
            });
        }
        if columns_meta.is_empty() {
            ui.horizontal(|ui| {
                ui.label("Rows affected");
                ui.label(affected_rows.to_string());
            });
            return;
        }
        let names: Vec<String> = columns_meta
            .iter()
            .map(|column| column.name.clone())
            .collect();
        let display: Vec<Vec<String>> = rows_raw
            .iter()
            .map(|row| {
                row.iter()
                    .enumerate()
                    .map(|(index, cell)| {
                        format_cell(columns_meta.get(index), cell.as_deref(), MAX_CELL_BYTES)
                    })
                    .collect()
            })
            .collect();

        ui.horizontal(|ui| {
            ui.label("Rows shown");
            ui.label(display.len().to_string());
            if set_truncated {
                ui.label("Limited to first 1000 rows");
            }
            if ui.button("Export CSV").clicked() {
                self.export_csv();
            }
            for (title, kind) in [
                ("Copy Displayed Rows", CopyKind::Tab),
                (
                    "Copy Displayed Rows with Column Names",
                    CopyKind::TabWithHeaders,
                ),
                ("Copy Displayed Rows as CSV", CopyKind::Csv),
            ] {
                if ui.button(title).clicked() {
                    self.copy_snapshot(columns_meta.clone(), rows_raw.clone(), None, kind);
                }
            }
        });

        let mut pending: Option<(usize, CopyKind)> = None;
        {
            let columns_meta = &columns_meta;
            let rows_raw = &rows_raw;
            let mut menu = |ui: &mut egui::Ui, row: usize, column: usize| {
                if ui.button("Copy Cell Value").clicked() {
                    let text = format_cell(
                        columns_meta.get(column),
                        rows_raw
                            .get(row)
                            .and_then(|row| row.get(column))
                            .and_then(Option::as_deref),
                        COPY_CELL_BYTES,
                    );
                    ui.ctx().copy_text(text);
                }
                ui.separator();
                for (title, kind) in [
                    ("Copy Selected Row", CopyKind::Tab),
                    (
                        "Copy Selected Row With Column Names",
                        CopyKind::TabWithHeaders,
                    ),
                    ("Copy Selected Row as CSV", CopyKind::Csv),
                ] {
                    if ui.button(title).clicked() {
                        pending = Some((row, kind));
                    }
                }
            };
            grid_view(
                ui,
                "query_results",
                &names,
                &display,
                &mut self.selected_query_cell,
                Some(&mut menu),
            );
        }
        let Some((row, kind)) = pending else {
            return;
        };
        let Some(row_values) = rows_raw.get(row).cloned() else {
            return;
        };
        self.copy_snapshot(columns_meta.clone(), vec![row_values], None, kind);
    }
}
