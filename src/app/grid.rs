//! Grid hasil yang dapat dipilih, dengan menu konteks per sel.
//!
//! Dipakai oleh Content view dan hasil SQL Query. Seleksi
//! (`Option<(row, column)>`) hidup di `SequelAceApp`; snapshot yang dikirim ke
//! worker untuk menyalin baris dibangun di `app.rs`.
use eframe::egui;

/// Callback menu per sel (dipanggil saat sel diklik-kanan).
pub(super) type GridMenu<'a> = dyn FnMut(&mut egui::Ui, usize, usize) + 'a;

/// Batas byte sel untuk copy (tanpa pemangkasan), mengikuti aturan
/// `rowsAsTabStringWithHeaders:` (`07-content-edit-copy.md` §4) — sel tidak
/// dibungkas saat di-copy.
pub(super) const COPY_CELL_BYTES: usize = usize::MAX;

/// Tampilan grid dengan seleksi sel.
///
/// `menu` dipanggil saat sel diklik-kanan, memberi kesempatan menambah item
/// menu jangkauan khusus view (content menambah copy-as-SQL).
pub(super) fn grid_view(
    ui: &mut egui::Ui,
    id: &'static str,
    columns: &[String],
    rows: &[Vec<String>],
    selected: &mut Option<(usize, usize)>,
    mut menu: Option<&mut GridMenu<'_>>,
) {
    egui::ScrollArea::both().id_salt(id).show(ui, |ui| {
        egui::Grid::new((id, "grid")).striped(true).show(ui, |ui| {
            for column in columns {
                ui.strong(column);
            }
            ui.end_row();
            for (row_index, row) in rows.iter().enumerate() {
                for (column_index, value) in row.iter().enumerate() {
                    let is_selected = selected
                        .as_ref()
                        .is_some_and(|(r, c)| *r == row_index && *c == column_index);
                    let label = ui.selectable_label(is_selected, value);
                    if label.clicked() {
                        *selected = Some((row_index, column_index));
                    }
                    if label.secondary_clicked() {
                        *selected = Some((row_index, column_index));
                    }
                    if let Some(menu) = menu.as_mut() {
                        label.context_menu(|ui| menu(ui, row_index, column_index));
                    }
                }
                ui.end_row();
            }
        });
    });
}
