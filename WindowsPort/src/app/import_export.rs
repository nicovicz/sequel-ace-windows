//! Import (execute SQL file) + Export (CSV/SQL) — doc 09/09b.

use super::SequelAceApp;
use crate::database::CopySnapshot;
impl SequelAceApp {
    /// Export hasil query aktif ke file CSV.
    pub(super) fn export_csv(&mut self) {
        let Some(output) = self.query_output.as_ref() else {
            self.error = Some("No query results to export.".to_owned());
            return;
        };
        let Some(set) = output.sets.get(self.selected_result_set) else {
            return;
        };
        let Some(path) = rfd::FileDialog::new()
            .set_file_name("export.csv")
            .add_filter("CSV", &["csv"])
            .save_file()
        else {
            return;
        };
        let columns: Vec<String> = set.columns.iter().map(|c| c.name.clone()).collect();
        let rows: Vec<Vec<String>> = set
            .rows
            .iter()
            .map(|row| {
                row.iter()
                    .enumerate()
                    .map(|(i, cell)| {
                        crate::database::format_cell(
                            set.columns.get(i),
                            cell.as_deref(),
                            usize::MAX,
                        )
                    })
                    .collect()
            })
            .collect();
        let csv = csv_string(&columns, &rows);
        if let Err(error) = std::fs::write(&path, csv) {
            self.error = Some(format!("Could not write export file: {error}"));
        }
    }

    /// Export Content page ke file SQL INSERT.
    pub(super) fn export_sql_inserts(&mut self) {
        let Some(page) = self.content_page.as_ref() else {
            self.error = Some("No content to export.".to_owned());
            return;
        };
        let Some(entry) = self.selected_object.as_ref() else {
            return;
        };
        let Some(path) = rfd::FileDialog::new()
            .set_file_name(format!("{}.sql", entry.name))
            .add_filter("SQL", &["sql"])
            .save_file()
        else {
            return;
        };
        let snapshot = CopySnapshot {
            table: Some(entry.name.clone()),
            columns: page.columns.clone(),
            rows: page.rows.clone(),
        };
        // Copy-as-SQL INSERT (doc 07 §4, fase 1).
        // Worker handles escaping; here we use the display values as a proxy
        // since we can't access `Conn` from the UI thread.
        let columns: Vec<String> = snapshot.columns.iter().map(|c| c.name.clone()).collect();
        let mut sql = format!(
            "INSERT INTO {} ({})\nVALUES\n",
            entry.name,
            columns
                .iter()
                .map(|c| format!("`{c}`"))
                .collect::<Vec<_>>()
                .join(", ")
        );
        for row in &snapshot.rows {
            let values: Vec<String> = row
                .iter()
                .map(|cell| match cell {
                    None => "NULL".to_owned(),
                    Some(bytes) => {
                        let text = String::from_utf8_lossy(bytes);
                        format!("'{}'", text.replace('\\', "\\\\").replace('\'', "\\'"))
                    }
                })
                .collect();
            sql.push_str(&format!("\t({}),\n", values.join(", ")));
        }
        if sql.ends_with(",\n") {
            sql.truncate(sql.len() - 2);
        }
        sql.push_str(";\n");
        if let Err(error) = std::fs::write(&path, sql) {
            self.error = Some(format!("Could not write export file: {error}"));
        }
    }

    /// Import: buka file SQL dan jalankan di worker.
    pub(super) fn import_sql_file(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("SQL", &["sql"])
            .pick_file()
        else {
            return;
        };
        let Ok(sql) = std::fs::read_to_string(&path) else {
            self.error = Some(format!("Could not read file: {}", path.display()));
            return;
        };
        if sql.trim().is_empty() {
            self.error = Some("The selected SQL file is empty.".to_owned());
            return;
        }
        self.main_view = super::MainView::Query;
        self.query_text = sql;
        self.run_sql();
    }
}

fn csv_string(columns: &[String], rows: &[Vec<String>]) -> String {
    let mut out = String::new();
    for (index, column) in columns.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push('"');
        out.push_str(&column.replace('"', "\"\""));
        out.push('"');
    }
    out.push_str("\r\n");
    for row in rows {
        for (index, value) in row.iter().enumerate() {
            if index > 0 {
                out.push(',');
            }
            out.push('"');
            out.push_str(value);
            out.push('"');
        }
        out.push_str("\r\n");
    }
    out
}
