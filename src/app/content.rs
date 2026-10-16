use super::grid::{COPY_CELL_BYTES, grid_view};
use super::{MainView, SequelAceApp};
use crate::database::{
    ColumnMeta, CopyKind, EditPlan, FilterSpec, MAX_CELL_BYTES, Request, default_row,
    duplicate_row, format_cell,
};
use eframe::egui;

/// Sheet edit satu sel: teks baru + flag `Set NULL` (mac: field editor).
pub(super) struct EditSheetState {
    pub plan: EditPlan,
    pub column: usize,
    pub text: String,
    pub set_null: bool,
    pub is_raw: bool,
    pub is_bit: bool,
    /// Blob-as-text toggle (mac: `SPDisplayBinaryDataAsHex`, doc 07 §1).
    pub show_as_text: bool,
}

/// Prompt filter 1-2 argumen, meniru sheet mac; `column` dari nama kolom.
pub(super) struct FilterPrompt {
    pub column: String,
    pub group: String,
    pub menu_label: String,
    pub values: Vec<String>,
    pub case_sensitive: bool,
}

impl SequelAceApp {
    pub(super) fn open_content(&mut self) {
        if self.selected_object.is_none() || self.busy {
            return;
        }
        self.main_view = MainView::Content;
        self.selected_content_cell = None;
        if self.content_page.is_none() {
            self.request_content_page(0);
        }
    }

    fn request_content_page(&mut self, page: u64) {
        if self.busy || self.selected_database.is_none() {
            return;
        }
        if let Some(entry) = self.selected_object.clone() {
            self.send(Request::LoadContent { entry, page });
        }
    }

    /// Copy semua baris tersembunyi pagi-ma, seperti "Copy Displayed Rows"
    /// `(SPCopyTable rowsAsTabStringWithHeaders:onlySelectedRows:… `doc 07 §4`.
    fn copy_content(&mut self, kind: CopyKind) {
        let Some(page) = self.content_page.as_ref() else {
            return;
        };
        let columns_meta = page.columns.clone();
        let rows_raw = page.rows.clone();
        let table = self
            .selected_object
            .as_ref()
            .map(|entry| entry.name.clone());
        self.copy_snapshot(columns_meta, rows_raw, table, kind);
    }

    /// Copy satu baris terpilih; tetap lewat worker supaya copy-as-SQL
    /// memakai escaping session.
    fn copy_content_row(&mut self, row: usize, kind: CopyKind) {
        let Some(page) = self.content_page.as_ref() else {
            return;
        };
        let Some(row_values) = page.rows.get(row).cloned() else {
            return;
        };
        let columns_meta = page.columns.clone();
        let table = self
            .selected_object
            .as_ref()
            .map(|entry| entry.name.clone());
        self.copy_snapshot(columns_meta, vec![row_values], table, kind);
    }

    pub(super) fn content_view(&mut self, ui: &mut egui::Ui) {
        let Some(entry) = self.selected_object.clone() else {
            ui.label("Select a table or view");
            return;
        };
        ui.heading(&entry.name);
        let has_next = self.content_page.as_ref().is_some_and(|page| page.has_next);
        let page_index = self.content_page_index;
        ui.horizontal(|ui| {
            if ui
                .add_enabled(page_index > 0 && !self.busy, egui::Button::new("Previous"))
                .clicked()
            {
                self.request_content_page(page_index - 1);
            }
            ui.label("Page");
            ui.label((page_index + 1).to_string());
            if ui
                .add_enabled(
                    has_next && !self.busy && page_index < u64::MAX,
                    egui::Button::new("Next"),
                )
                .clicked()
            {
                self.request_content_page(page_index + 1);
            }
            if ui
                .add_enabled(!self.busy, egui::Button::new("Refresh Rows"))
                .clicked()
            {
                self.request_content_page(page_index);
            }
            // Add Row (doc 07 §3): bangun baris default, buka sheet INSERT.
            if ui
                .add_enabled(!self.busy, egui::Button::new("Add Row"))
                .clicked()
            {
                self.start_add_row();
            }
            // Export SQL INSERT (doc 09b) + Import SQL file (doc 09).
            if ui.button("Export SQL").clicked() {
                self.export_sql_inserts();
            }
            if ui.button("Import SQL").clicked() {
                self.import_sql_file();
            }
            // Delete All Rows (doc 07 §3, 2286): konfirmasi + DELETE FROM t.
            if ui
                .add_enabled(
                    !self.busy
                        && self
                            .content_page
                            .as_ref()
                            .is_some_and(|page| !page.rows.is_empty()),
                    egui::Button::new("Delete All Rows"),
                )
                .clicked()
            {
                self.confirm_delete_all = true;
            }
            // Delete Selected Rows (multi-baris, doc 07 §3, 2403-2497).
            if ui
                .add_enabled(
                    !self.busy && self.selected_content_rows.len() > 1,
                    egui::Button::new(format!(
                        "Delete Selected Rows ({})",
                        self.selected_content_rows.len()
                    )),
                )
                .clicked()
            {
                let Some(page) = self.content_page.as_ref() else {
                    return;
                };
                let Some(entry) = self.selected_object.as_ref() else {
                    return;
                };
                let generated = self.generated_columns.as_deref().unwrap_or(&[]);
                let plan = EditPlan {
                    table: entry.name.clone(),
                    columns: page.columns.clone(),
                    generated_columns: generated.to_vec(),
                    old_row: vec![],
                    new_row: vec![],
                    old_rows: page.rows.clone(),
                    pk_indexes: Vec::new(),
                };
                let rows = self.selected_content_rows.clone();
                self.confirm_delete_multi = Some((plan, rows));
            }
            if self.busy {
                ui.spinner();
            }
        });
        ui.separator();
        let Some(page) = self.content_page.as_ref() else {
            return;
        };
        let columns_meta: Vec<ColumnMeta> = page.columns.clone();
        let rows_raw = page.rows.clone();
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
            for (title, kind) in [
                ("Copy Displayed Rows", CopyKind::Tab),
                (
                    "Copy Displayed Rows with Column Names",
                    CopyKind::TabWithHeaders,
                ),
                ("Copy Displayed Rows as CSV", CopyKind::Csv),
                ("Copy Displayed Rows as SQL INSERT", CopyKind::SqlInsert),
                (
                    "Copy Displayed Rows as SQL INSERT (no auto_inc)",
                    CopyKind::SqlInsertNoAutoInc,
                ),
            ] {
                if ui.button(title).clicked() {
                    self.copy_content(kind);
                }
            }
        });
        if display.is_empty() {
            ui.label("No rows");
        }

        let mut pending: Option<(usize, CopyKind)> = None;
        let mut pending_filter: Option<FilterSpec> = None;
        let mut pending_clear_filter = false;
        let mut pending_edit: Option<(usize, usize)> = None;
        let mut pending_delete: Option<usize> = None;
        let mut pending_duplicate: Option<usize> = None;
        let mut pending_filter_prompt: Option<FilterPrompt> = None;
        let generated = self.generated_columns.clone().unwrap_or_default();
        {
            let columns_meta = &columns_meta;
            let rows_raw = &rows_raw;
            let mut menu = |ui: &mut egui::Ui, row: usize, column: usize| {
                let Some(meta) = columns_meta.get(column) else {
                    return;
                };
                let Some(cell) = rows_raw
                    .get(row)
                    .and_then(|row| row.get(column))
                    .and_then(Option::as_deref)
                    .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
                else {
                    return;
                };

                if ui.button("Copy Cell Value").clicked() {
                    ui.ctx().copy_text(cell.clone());
                }
                ui.separator();
                for (title, kind) in [
                    ("Copy Selected Row", CopyKind::Tab),
                    (
                        "Copy Selected Row With Column Names",
                        CopyKind::TabWithHeaders,
                    ),
                    ("Copy Selected Row as CSV", CopyKind::Csv),
                    ("Copy Selected Row as SQL INSERT", CopyKind::SqlInsert),
                    (
                        "Copy Selected Row as SQL INSERT (no auto_inc)",
                        CopyKind::SqlInsertNoAutoInc,
                    ),
                ] {
                    if ui.button(title).clicked() {
                        pending = Some((row, kind));
                    }
                }

                // Edit sel (fase 3): generated column ditolak (doc 07 §2).
                ui.separator();
                let is_generated = generated.contains(&meta.name);
                if ui
                    .add_enabled(!self.busy && !is_generated, egui::Button::new("Edit Cell"))
                    .on_disabled_hover_text(if is_generated {
                        crate::database::generated_column_error(&meta.name)
                    } else {
                        String::new()
                    })
                    .clicked()
                {
                    pending_edit = Some((row, column));
                }
                // Delete baris (fase 3): konfirmasi mac-style (doc 07 §6).
                if ui
                    .add_enabled(!self.busy, egui::Button::new("Delete Selected Row"))
                    .clicked()
                {
                    pending_delete = Some(row);
                }
                // Multi-row select toggle (ctrl-click alternatif untuk touch).
                if ui.button("Toggle Row Selection").clicked() {
                    if self.selected_content_rows.contains(&row) {
                        self.selected_content_rows.retain(|&r| r != row);
                    } else {
                        self.selected_content_rows.push(row);
                    }
                }
                // Duplicate row (fase 4): salin + auto_increment NULL (doc 07 §3).
                if ui
                    .add_enabled(!self.busy, egui::Button::new("Duplicate Row"))
                    .clicked()
                {
                    pending_duplicate = Some(row);
                }

                // Filter "by selected value" — fase 2/3 (doc 06 §3).
                if let Some((group, defs)) = crate::database::menu_entries(meta) {
                    ui.separator();
                    ui.strong("Filter by Selected Value");
                    let is_null = cell == "NULL";
                    let bit_string =
                        meta.field_type == mysqlclient_sys::enum_field_types::MYSQL_TYPE_BIT;
                    for definition in defs {
                        let forced_null =
                            is_null || (cell.is_empty() && definition.number_of_arguments > 0);
                        let label = if forced_null && definition.number_of_arguments > 0 {
                            continue; // NULL cell hanya boleh IS NULL / IS NOT NULL
                        } else {
                            definition.menu_label
                        };
                        if ui
                            .add_enabled(!self.busy, egui::Button::new(label))
                            .clicked()
                        {
                            // bit: bit-string → decimal (audit content-helper).
                            let value = if bit_string && definition.number_of_arguments > 0 {
                                u64::from_str_radix(&cell, 2)
                                    .map(|value| value.to_string())
                                    .unwrap_or(cell.clone())
                            } else {
                                cell.clone()
                            };
                            match definition.number_of_arguments {
                                0 => {
                                    pending_filter = Some(FilterSpec::from_def(
                                        meta.name.clone(),
                                        group,
                                        definition,
                                        vec![],
                                        false,
                                    ));
                                }
                                2 => {
                                    pending_filter_prompt = Some(FilterPrompt {
                                        column: meta.name.clone(),
                                        group: group.to_owned(),
                                        menu_label: definition.menu_label.to_owned(),
                                        values: vec![value.clone(), value],
                                        case_sensitive: false,
                                    });
                                }
                                _ => {
                                    pending_filter = Some(FilterSpec::from_def(
                                        meta.name.clone(),
                                        group,
                                        definition,
                                        vec![value],
                                        false,
                                    ));
                                }
                            }
                        }
                    }
                    ui.separator();
                    if ui
                        .add_enabled(!self.busy, egui::Button::new("Clear Filter"))
                        .clicked()
                    {
                        pending_clear_filter = true;
                    }
                }
            };
            grid_view(
                ui,
                "content_results",
                &names,
                &display,
                &mut self.selected_content_cell,
                Some(&mut menu),
            );
        }
        if let Some((row, kind)) = pending {
            self.copy_content_row(row, kind);
        }
        if let Some(row) = pending_duplicate {
            self.start_duplicate_row(row);
        }
        if pending_clear_filter {
            self.send_filter(None);
        } else if let Some(filter) = pending_filter {
            self.send_filter(Some(filter));
        }
        if let Some((row, column)) = pending_edit {
            self.open_edit_sheet(row, column);
        }
        if let Some(row) = pending_delete {
            self.confirm_delete_row(row);
            self.selected_content_rows.clear();
        }
        if let Some(prompt) = pending_filter_prompt {
            self.filter_prompt = Some(prompt);
        }
    }

    /// Sheet edit: bangun EditPlan dari baris terpilih + kolom yang diklik.
    fn open_edit_sheet(&mut self, row: usize, column: usize) {
        let Some(page) = self.content_page.as_ref() else {
            return;
        };
        let Some(entry) = self.selected_object.as_ref() else {
            return;
        };
        let Some(meta) = page.columns.get(column) else {
            return;
        };
        let generated = self.generated_columns.as_deref().unwrap_or(&[]);
        if generated.contains(&meta.name) {
            self.error = Some(crate::database::generated_column_error(&meta.name));
            return;
        }
        // Buka koneksi PK di thread worker; di UI kita pakai data yang sudah ada.
        let plan = EditPlan {
            table: entry.name.clone(),
            columns: page.columns.clone(),
            generated_columns: generated.to_vec(),
            old_row: page.rows.get(row).cloned().unwrap_or_default(),
            new_row: page.rows.get(row).cloned().unwrap_or_default(),
            old_rows: page.rows.clone(),
            pk_indexes: Vec::new(), // diisi worker lewat `primary_key_indexes`
        };
        let text = crate::database::format_cell(
            Some(meta),
            plan.old_row.get(column).and_then(Option::as_deref),
            COPY_CELL_BYTES,
        );
        let is_raw = meta.is_raw_binary();
        let is_bit = meta.field_type == mysqlclient_sys::enum_field_types::MYSQL_TYPE_BIT;
        self.edit_sheet = Some(EditSheetState {
            plan,
            column,
            text,
            set_null: false,
            is_raw,
            is_bit,
            show_as_text: false,
        });
    }

    /// Add Row (doc 07 §3): bangun baris default, buka sheet INSERT.
    fn start_add_row(&mut self) {
        let Some(page) = self.content_page.as_ref() else {
            return;
        };
        let Some(entry) = self.selected_object.as_ref() else {
            return;
        };
        let generated = self.generated_columns.as_deref().unwrap_or(&[]);
        let columns = page.columns.clone();
        let new_row = default_row(&columns);
        let text = String::new();
        let plan = EditPlan {
            table: entry.name.clone(),
            columns,
            generated_columns: generated.to_vec(),
            old_row: vec![None; new_row.len()],
            new_row,
            old_rows: page.rows.clone(),
            pk_indexes: vec![],
        };
        self.edit_sheet = Some(EditSheetState {
            plan,
            column: usize::MAX, // penanda INSERT
            text,
            set_null: false,
            is_raw: false,
            is_bit: false,
            show_as_text: false,
        });
    }

    /// Duplicate Row (doc 07 §3): salin baris terpilih, auto_increment → NULL.
    fn start_duplicate_row(&mut self, row: usize) {
        let Some(page) = self.content_page.as_ref() else {
            return;
        };
        let Some(entry) = self.selected_object.as_ref() else {
            return;
        };
        let Some(source) = page.rows.get(row).cloned() else {
            return;
        };
        let generated = self.generated_columns.as_deref().unwrap_or(&[]);
        let columns = page.columns.clone();
        let new_row = duplicate_row(&columns, &source);
        let plan = EditPlan {
            table: entry.name.clone(),
            columns,
            generated_columns: generated.to_vec(),
            old_row: vec![None; new_row.len()],
            new_row,
            old_rows: page.rows.clone(),
            pk_indexes: vec![],
        };
        self.edit_sheet = Some(EditSheetState {
            plan,
            column: usize::MAX,
            text: String::new(),
            set_null: false,
            is_raw: false,
            is_bit: false,
            show_as_text: false,
        });
    }

    /// Konfirmasi delete: bangun EditPlan lalu tampilkan dialog mac-style.
    fn confirm_delete_row(&mut self, row: usize) {
        let Some(page) = self.content_page.as_ref() else {
            return;
        };
        let Some(entry) = self.selected_object.as_ref() else {
            return;
        };
        let generated = self.generated_columns.as_deref().unwrap_or(&[]);
        self.confirm_delete = Some(EditPlan {
            table: entry.name.clone(),
            columns: page.columns.clone(),
            generated_columns: generated.to_vec(),
            old_row: page.rows.get(row).cloned().unwrap_or_default(),
            new_row: page.rows.get(row).cloned().unwrap_or_default(),
            old_rows: page.rows.clone(),
            pk_indexes: Vec::new(),
        });
    }
}

impl SequelAceApp {
    /// Render edit sheet / delete confirm / filter prompt (dipanggil dari ui).
    pub(super) fn edit_overlays(&mut self, ui: &mut egui::Ui) {
        // 1. Edit sheet.
        if let Some(mut sheet) = self.edit_sheet.take() {
            let mut keep_open = true;
            let mut apply = false;
            let mut cancel = false;
            egui::Window::new(format!("Edit {} → {}", sheet.plan.table, {
                let Some(meta) = sheet.plan.columns.get(sheet.column) else {
                    self.edit_sheet = Some(sheet);
                    return;
                };
                meta.name.clone()
            }))
            .open(&mut keep_open)
            .collapsible(false)
            .resizable(false)
            .show(ui, |ui| {
                if sheet.is_bit {
                    ui.label("BIT field — only '0' and '1' are allowed.");
                } else if sheet.is_raw {
                    // Blob-as-text (mac: SPDisplayBinaryDataAsHex toggle, doc 07 §1).
                    ui.checkbox(&mut sheet.show_as_text, "Show as text (decode blob)");
                    if sheet.show_as_text {
                        ui.label("Binary field — decoded text (lossy):");
                    } else {
                        ui.label("Binary field — hex string (0x…) is used.");
                    }
                }
                ui.add_enabled(
                    !sheet.set_null,
                    egui::TextEdit::multiline(&mut sheet.text)
                        .desired_rows(4)
                        .desired_width(320.0),
                );
                ui.checkbox(&mut sheet.set_null, "Set NULL");
                ui.horizontal(|ui| {
                    if ui.button("Apply").clicked() {
                        apply = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });
            if apply {
                // Isi new_row[column] dari text / NULL.
                let Some(meta) = sheet.plan.columns.get(sheet.column) else {
                    return;
                };
                let value: Option<Vec<u8>> = if sheet.set_null {
                    None
                } else if sheet.is_bit {
                    // bit-string → byte (mac validasi hanya 0/1).
                    let bits = sheet.text.trim();
                    let width = meta.length as usize;
                    let mut bytes = vec![0u8; width.div_ceil(8)];
                    for (index, character) in bits.chars().take(width).enumerate() {
                        if character == '1' {
                            bytes[index / 8] |= 1 << (7 - (index % 8));
                        }
                    }
                    Some(bytes)
                } else if sheet.is_raw {
                    // hex `0x…` → byte
                    let hex = sheet
                        .text
                        .trim()
                        .trim_start_matches("0x")
                        .trim_start_matches("0X");
                    let decoded: Vec<u8> = (0..hex.len() / 2)
                        .filter_map(|index| {
                            u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).ok()
                        })
                        .collect();
                    Some(decoded)
                } else {
                    Some(sheet.text.clone().into_bytes())
                };
                sheet.plan.new_row = sheet.plan.old_row.clone();
                if let Some(cell) = sheet.plan.new_row.get_mut(sheet.column) {
                    *cell = value;
                }
                self.send(Request::ExecuteEdit(
                    sheet.plan,
                    if sheet.column == usize::MAX {
                        crate::database::EditKind::Insert
                    } else {
                        crate::database::EditKind::Update
                    },
                ));
            } else if !cancel {
                self.edit_sheet = Some(sheet);
            }
        }

        // 2. Delete confirmation.
        if let Some(plan) = self.confirm_delete.take() {
            let mut confirmed = false;
            egui::Window::new("Delete Selected Row")
                .collapsible(false)
                .resizable(false)
                .show(ui, |ui| {
                    ui.label(crate::database::CONFIRM_DELETE_SINGLE);
                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() {
                            confirmed = true;
                        }
                        if ui.button("Cancel").clicked() {
                            self.confirm_delete = None;
                        }
                    });
                });
            if confirmed {
                self.send(Request::ExecuteEdit(
                    plan,
                    crate::database::EditKind::Delete,
                ));
            }
        }

        // 2a. Delete All Rows confirmation (doc 07 §6, 2187-2208).
        if self.confirm_delete_all {
            let table = self
                .selected_object
                .as_ref()
                .map(|entry| entry.name.clone())
                .unwrap_or_default();
            let mut confirmed = false;
            egui::Window::new("Delete ALL ROWS IN TABLE")
                .collapsible(false)
                .resizable(false)
                .show(ui, |ui| {
                    ui.colored_label(
                        egui::Color32::RED,
                        format!(
                            "Are you sure you want to delete ALL ROWS IN TABLE '{table}'?\nThis action cannot be undone."
                        ),
                    );
                    ui.checkbox(
                        &mut self.reset_auto_increment,
                        "Reset AUTO_INCREMENT after deletion\n(only for Delete ALL ROWS IN TABLE)?",
                    );
                    ui.horizontal(|ui| {
                        if ui.button("Delete All").clicked() {
                            confirmed = true;
                        }
                        if ui.button("Cancel").clicked() {
                            self.confirm_delete_all = false;
                        }
                    });
                });
            if confirmed {
                self.confirm_delete_all = false;
                self.send(Request::DeleteAllRows { table });
            }
        }

        // 2b. Delete Multi Rows confirmation (doc 07 §6, 196).
        if let Some((plan, rows)) = self.confirm_delete_multi.take() {
            let mut confirmed = false;
            egui::Window::new("Delete Selected Rows")
                .collapsible(false)
                .resizable(false)
                .show(ui, |ui| {
                    ui.label(format!(
                        "Are you sure you want to delete the selected {} rows from this table? This action cannot be undone.",
                        rows.len()
                    ));
                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() {
                            confirmed = true;
                        }
                        if ui.button("Cancel").clicked() {
                            self.confirm_delete_multi = None;
                        }
                    });
                });
            if confirmed {
                self.send(Request::DeleteMultiRows { plan, rows });
            }
        }

        // 3. Filter prompt (1-2 argumen).
        if let Some(mut prompt) = self.filter_prompt.take() {
            let mut apply = false;
            egui::Window::new(format!("Filter: {}", prompt.menu_label))
                .collapsible(false)
                .resizable(false)
                .show(ui, |ui| {
                    ui.label(format!("Column: {}", prompt.column));
                    match prompt.values.len() {
                        0 => {}
                        1 => {
                            ui.label("Value:");
                            ui.text_edit_singleline(&mut prompt.values[0]);
                        }
                        _ => {
                            ui.label("First value:");
                            ui.text_edit_singleline(&mut prompt.values[0]);
                            ui.label("Second value:");
                            ui.text_edit_singleline(&mut prompt.values[1]);
                        }
                    }
                    ui.checkbox(&mut prompt.case_sensitive, "Case-sensitive (BINARY)");
                    ui.horizontal(|ui| {
                        if ui.button("Apply").clicked() {
                            apply = true;
                        }
                        if ui.button("Cancel").clicked() {
                            self.filter_prompt = None;
                        }
                    });
                });
            if apply {
                let Some(entry) = self.selected_object.clone() else {
                    return;
                };
                self.content_filter = Some(FilterSpec {
                    column: prompt.column.clone(),
                    group: prompt.group.clone(),
                    menu_label: prompt.menu_label.clone(),
                    values: prompt.values.clone(),
                    case_sensitive: prompt.case_sensitive,
                });
                self.selected_content_cell = None;
                self.send(Request::SetContentFilter {
                    entry,
                    filter: self.content_filter.clone(),
                });
            } else if self.filter_prompt.is_none() {
                // Cancel sudah ditangani di atas.
            } else {
                self.filter_prompt = Some(prompt);
            }
        }
    }
}
