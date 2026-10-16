//! Connection geometry follows ConnectionView.xib in logical points.
//! Native window chrome and the proportional font remain platform substitutions.
use super::{ConnectionMethod, SequelAceApp};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Stroke, Vec2};

const BACKGROUND: Color32 = Color32::from_rgb(237, 237, 237);
const TEXT: Color32 = Color32::from_rgb(30, 30, 30);
const BORDER: Color32 = Color32::from_rgb(190, 190, 190);
const BLUE: Color32 = Color32::from_rgb(0, 122, 255);

// Keep context access and texture upload in separate lock scopes.
fn icon(ui: &egui::Ui, bounds: Rect, name: &str) {
    let bytes: &[u8] = match name {
        "structure" => include_bytes!("../../assets/connection/structure.png"),
        "content" => include_bytes!("../../assets/connection/content.png"),
        "relations" => include_bytes!("../../assets/connection/relations.png"),
        "triggers" => include_bytes!("../../assets/connection/triggers.png"),
        "info" => include_bytes!("../../assets/connection/info.png"),
        "query" => include_bytes!("../../assets/connection/query.png"),
        "users" => include_bytes!("../../assets/connection/users.png"),
        "console" => include_bytes!("../../assets/connection/console.png"),
        "quick-connect" => include_bytes!("../../assets/connection/quick-connect.png"),
        "search" => include_bytes!("../../assets/connection/search.png"),
        "eye" => include_bytes!("../../assets/connection/eye.png"),
        "eye-slash" => include_bytes!("../../assets/connection/eye-slash.png"),
        _ => return,
    };
    let id = egui::Id::new(("connection_asset", name));
    let cached = ui
        .ctx()
        .data(|data| data.get_temp::<egui::TextureHandle>(id));
    let texture = cached.unwrap_or_else(|| {
        let decoded = image::load_from_memory(bytes)
            .expect("embedded connection icon")
            .to_rgba8();
        let pixels = egui::ColorImage::from_rgba_unmultiplied(
            [decoded.width() as usize, decoded.height() as usize],
            decoded.as_raw(),
        );
        let texture = ui
            .ctx()
            .load_texture(name, pixels, egui::TextureOptions::LINEAR);
        ui.ctx()
            .data_mut(|data| data.insert_temp(id, texture.clone()));
        texture
    });
    ui.painter().image(
        texture.id(),
        bounds,
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        Color32::WHITE,
    );
}

fn toolbar_symbol(ui: &egui::Ui, c: Pos2, index: usize) {
    if index == 6 {
        let stroke = Stroke::new(1.5, Color32::from_gray(85));
        ui.painter().rect_stroke(
            Rect::from_center_size(c, Vec2::new(52.0, 24.0)),
            4,
            stroke,
            egui::StrokeKind::Inside,
        );
        ui.painter().line_segment(
            [c + Vec2::new(0.0, -11.0), c + Vec2::new(0.0, 11.0)],
            stroke,
        );
        for sign in [-1.0, 1.0] {
            ui.painter().line_segment(
                [
                    c + Vec2::new(sign * 10.0, -5.0),
                    c + Vec2::new(sign * 16.0, 0.0),
                ],
                stroke,
            );
            ui.painter().line_segment(
                [
                    c + Vec2::new(sign * 16.0, 0.0),
                    c + Vec2::new(sign * 10.0, 5.0),
                ],
                stroke,
            );
        }
    } else {
        let name = [
            "structure",
            "content",
            "relations",
            "triggers",
            "info",
            "query",
            "",
            "users",
            "console",
        ][index];
        icon(ui, Rect::from_center_size(c, Vec2::splat(28.0)), name);
    }
}

fn rect(origin: Pos2, x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::from_min_size(origin + Vec2::new(x, y), Vec2::new(w, h))
}

fn text(ui: &egui::Ui, at: Pos2, align: Align2, value: &str, color: Color32) {
    ui.painter()
        .text(at, align, value, FontId::proportional(13.0), color);
}

impl SequelAceApp {
    pub(super) fn connection_window(&mut self, ui: &mut egui::Ui) {
        ui.scope(|ui| {
            // Scope the entire palette, including text edits: a light frame with
            // inherited dark-theme text was unreadable in the previous screen.
            ui.set_visuals(egui::Visuals::light());
            ui.visuals_mut().override_text_color = Some(TEXT);
            ui.visuals_mut().weak_text_color = Some(Color32::from_gray(95));
            ui.visuals_mut().disabled_alpha = 0.72;
            ui.style_mut().override_font_id = Some(FontId::proportional(13.0));
            let bounds = ui.available_rect_before_wrap();
            ui.painter().rect_filled(bounds, 0, BACKGROUND);
            let o = bounds.min;
            self.connection_toolbar(ui, o, bounds.width());
            let sidebar = rect(o, 0.0, 66.0, 200.0, bounds.height() - 66.0);
            ui.painter()
                .rect_filled(sidebar, 0, Color32::from_rgb(229, 232, 236));
            ui.painter().line_segment(
                [sidebar.right_top(), sidebar.right_bottom()],
                Stroke::new(1.0, BORDER),
            );
            ui.put(
                rect(sidebar.min, 6.0, 8.0, 188.0, 22.0),
                egui::TextEdit::singleline(&mut self.favorite_filter)
                    .hint_text("Search favorites")
                    .margin(Vec2::new(25.0, 3.0)),
            );
            icon(ui, rect(sidebar.min, 10.0, 11.0, 16.0, 16.0), "search");
            let selected = rect(sidebar.min, 0.0, 39.0, 199.0, 26.0);
            ui.painter()
                .rect_filled(selected, 0, Color32::from_rgb(206, 217, 233));
            icon(
                ui,
                Rect::from_center_size(
                    selected.left_center() + Vec2::new(12.0, 0.0),
                    Vec2::splat(18.0),
                ),
                "quick-connect",
            );
            text(
                ui,
                selected.left_center() + Vec2::new(24.0, 0.0),
                Align2::LEFT_CENTER,
                "Quick Connect",
                TEXT,
            );
            text(
                ui,
                sidebar.min + Vec2::new(10.0, 86.0),
                Align2::LEFT_CENTER,
                "Favorites",
                Color32::GRAY,
            );
            // Daftar favorites (doc 03): klik untuk memuat ke form.
            let favorites = self.favorites.clone();
            for (index, favorite) in favorites.iter().enumerate() {
                let y = 106.0 + index as f32 * 26.0;
                if y + 26.0 > sidebar.height() - 30.0 {
                    break; // sidebar overflow
                }
                let item = rect(sidebar.min, 0.0, y, 199.0, 24.0);
                let response = ui.put(
                    item,
                    egui::Button::new(egui::RichText::new(&favorite.name).color(TEXT).size(12.0))
                        .fill(Color32::TRANSPARENT)
                        .frame(false),
                );
                if response.clicked() {
                    self.load_favorite(index);
                }
                if response.secondary_clicked() {
                    self.delete_favorite(index);
                }
            }
            let bottom = sidebar.bottom() - 25.0;
            ui.painter()
                .hline(sidebar.x_range(), bottom, Stroke::new(1.0, BORDER));
            for (x, label) in [(5.0, "…"), (45.0, "+"), (80.0, "−")] {
                ui.put(
                    Rect::from_min_size(Pos2::new(o.x + x, bottom), Vec2::new(25.0, 23.0)),
                    egui::Button::new(label)
                        .sense(egui::Sense::hover())
                        .frame(false),
                );
            }
            let content = Rect::from_min_max(o + Vec2::new(201.0, 66.0), bounds.max);
            ui.scope_builder(egui::UiBuilder::new().max_rect(content), |ui| {
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let size = ui.available_size();
                        let height = if self.connection_method == ConnectionMethod::Ssh {
                            690.0
                        } else {
                            600.0
                        };
                        let (canvas, _) = ui.allocate_exact_size(
                            Vec2::new(size.x.max(470.0), size.y.max(height + 40.0)),
                            egui::Sense::hover(),
                        );
                        let origin = Pos2::new(
                            canvas.center().x - 223.0,
                            canvas.top() + ((canvas.height() - height) * 0.5).max(20.0),
                        );
                        self.connection_form(ui, origin, height);
                    });
            });
        });
    }

    fn connection_toolbar(&self, ui: &mut egui::Ui, o: Pos2, width: f32) {
        ui.painter().rect_filled(
            rect(o, 0.0, 0.0, width, 66.0),
            0,
            Color32::from_rgb(246, 246, 246),
        );
        ui.painter()
            .hline(o.x..=o.x + width, o.y + 65.0, Stroke::new(1.0, BORDER));
        ui.put(
            rect(o, 8.0, 20.0, 180.0, 24.0),
            egui::Button::new("Choose Database…").sense(egui::Sense::hover()),
        );
        // Disabled toolbar actions retain their geometry on the connection screen.
        let names = [
            "Structure",
            "Content",
            "Relations",
            "Triggers",
            "Table Info",
            "Query",
            "Table History",
            "Users",
            "Console",
        ];
        let slot = ((width - 205.0) / names.len() as f32).max(60.0);
        for (i, name) in names.iter().enumerate() {
            let center = o + Vec2::new(205.0 + slot * (i as f32 + 0.5), 24.0);
            toolbar_symbol(ui, center, i);
            text(
                ui,
                center + Vec2::new(0.0, 26.0),
                Align2::CENTER_CENTER,
                name,
                Color32::from_gray(85),
            );
        }
    }

    fn connection_form(&mut self, ui: &mut egui::Ui, o: Pos2, height: f32) {
        let panel = rect(o, 3.0, 12.0, 440.0, height - 65.0);
        ui.painter()
            .rect_filled(panel, 6, Color32::from_rgb(244, 244, 244));
        ui.painter()
            .rect_stroke(panel, 6, Stroke::new(1.0, BORDER), egui::StrokeKind::Inside);
        // Explicit horizontal centering; horizontal_centered only aligns on Y.
        let tabs = [
            ("TCP/IP", ConnectionMethod::TcpIp, 79.0),
            ("Socket", ConnectionMethod::Socket, 77.0),
            ("SSH Tunnel", ConnectionMethod::Ssh, 104.0),
        ];
        let mut x = (446.0 - 260.0) / 2.0;
        for (label, method, w) in tabs {
            let selected = self.connection_method == method;
            let response = ui.put(
                rect(o, x, 0.0, w, 25.0),
                egui::Button::new(egui::RichText::new(label).color(if selected {
                    Color32::WHITE
                } else {
                    TEXT
                }))
                .fill(if selected {
                    BLUE
                } else {
                    Color32::from_gray(252)
                })
                .corner_radius(5),
            );
            if response.clicked() && !self.busy {
                self.connection_method = method;
            }
            x += w;
        }
        ui.add_enabled_ui(!self.busy, |ui| {
            field(ui, o, 49.0, "Name:", &mut self.connection_name, false, "");
            for (i, color) in [
                Color32::GRAY,
                Color32::from_rgb(230, 77, 70),
                Color32::from_rgb(239, 151, 49),
                Color32::from_rgb(226, 201, 52),
                Color32::from_rgb(93, 176, 79),
                Color32::from_rgb(80, 165, 219),
                Color32::from_rgb(159, 111, 188),
            ]
            .into_iter()
            .enumerate()
            {
                ui.painter().circle_filled(
                    o + Vec2::new(153.0 + i as f32 * 29.0, 89.0),
                    6.0,
                    color,
                );
            }
            let ssh = self.connection_method == ConnectionMethod::Ssh;
            if self.connection_method == ConnectionMethod::Socket {
                field(ui, o, 109.0, "Username:", &mut self.user, false, "");
                password_field(
                    ui,
                    o,
                    143.0,
                    "Password:",
                    &mut self.password,
                    &mut self.show_password,
                );
                field(
                    ui,
                    o,
                    177.0,
                    "Database:",
                    &mut self.initial_database,
                    false,
                    "optional",
                );
                field(
                    ui,
                    o,
                    211.0,
                    "Socket:",
                    &mut self.socket_path,
                    false,
                    "/tmp/mysql.sock",
                );
            } else {
                field(
                    ui,
                    o,
                    109.0,
                    if ssh { "MySQL Host:" } else { "Host:" },
                    &mut self.host,
                    false,
                    "",
                );
                field(ui, o, 143.0, "Username:", &mut self.user, false, "");
                password_field(
                    ui,
                    o,
                    177.0,
                    "Password:",
                    &mut self.password,
                    &mut self.show_password,
                );
                field(
                    ui,
                    o,
                    211.0,
                    "Database:",
                    &mut self.initial_database,
                    false,
                    "optional",
                );
                field(ui, o, 245.0, "Port:", &mut self.port, false, "3306");
            }
            if ssh {
                field(ui, o, 293.0, "SSH Host:", &mut self.ssh_host, false, "");
                field(ui, o, 327.0, "SSH User:", &mut self.ssh_user, false, "");
                password_field(
                    ui,
                    o,
                    361.0,
                    "SSH Password:",
                    &mut self.ssh_password,
                    &mut self.show_ssh_password,
                );
                field(ui, o, 395.0, "SSH Port:", &mut self.ssh_port, false, "22");
                field(
                    ui,
                    o,
                    429.0,
                    "SSH Key:",
                    &mut self.ssh_key_path,
                    false,
                    "none set",
                );
            }
            if ssh {
                // SSH agent status (fase 8): deteksi + tampilkan di form.
                let status = crate::database::detect_ssh_agent();
                let color = if status.available {
                    Color32::from_rgb(93, 176, 79) // hijau
                } else {
                    Color32::from_rgb(230, 77, 70) // merah
                };
                text(
                    ui,
                    o + Vec2::new(440.0, 30.0),
                    Align2::RIGHT_CENTER,
                    &status.message,
                    color,
                );
            }
            let y = if ssh { 471.0 } else { 279.0 };
            text(
                ui,
                o + Vec2::new(137.0, y + 11.0),
                Align2::RIGHT_CENTER,
                "Time Zone:",
                TEXT,
            );
            ui.scope_builder(
                egui::UiBuilder::new()
                    .max_rect(rect(o, 145.0, y, 247.0, 24.0))
                    .layout(egui::Layout::top_down(egui::Align::Min)),
                |ui| {
                    let selected = if self.time_zone.is_empty() {
                        "Use Server Time Zone"
                    } else {
                        &self.time_zone
                    };
                    egui::ComboBox::from_id_salt("connection_time_zone")
                        .width(247.0)
                        .height(260.0)
                        .selected_text(selected)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.time_zone,
                                String::new(),
                                "Use Server Time Zone",
                            );
                            ui.separator();
                            for zone in
                                include_str!("../../assets/connection/time-zones.txt").lines()
                            {
                                ui.selectable_value(&mut self.time_zone, zone.to_owned(), zone);
                            }
                        });
                },
            );
            // Pin checkbox boxes to the field column; Ui::put centers widgets of
            // different intrinsic widths and used to stagger this column.
            for (offset, value, title, x, width) in [
                (
                    34.0,
                    &mut self.allow_local_infile,
                    "Allow LOCAL_DATA_INFILE (insecure)",
                    145.0,
                    247.0,
                ),
                (
                    62.0,
                    &mut self.enable_cleartext_plugin,
                    "Enable Cleartext plugin (insecure)",
                    145.0,
                    247.0,
                ),
                (90.0, &mut self.require_ssl, "Require SSL", 145.0, 112.0),
                (
                    90.0,
                    &mut self.request_server_public_key,
                    "Get Public Key",
                    263.0,
                    129.0,
                ),
            ] {
                ui.scope_builder(
                    egui::UiBuilder::new()
                        .max_rect(rect(o, x, y + offset, width, 20.0))
                        .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    |ui| {
                        ui.add_enabled(!self.busy, egui::Checkbox::new(value, title));
                    },
                );
            }
        });
        let can_connect = !self.busy && self.connection_method == ConnectionMethod::TcpIp;
        ui.add_enabled_ui(can_connect, |ui| {
            if ui
                .put(
                    rect(o, 295.0, height - 45.0, 147.0, 32.0),
                    egui::Button::new(egui::RichText::new("Connect").color(Color32::WHITE))
                        .fill(BLUE)
                        .corner_radius(16),
                )
                .clicked()
            {
                self.connect();
            }
            // Save as Favorite (doc 03).
            if ui
                .put(
                    rect(o, 155.0, height - 45.0, 130.0, 32.0),
                    egui::Button::new("Save as Favorite"),
                )
                .clicked()
            {
                self.save_favorite();
            }
        });
        if self.busy {
            text(
                ui,
                o + Vec2::new(32.0, height - 29.0),
                Align2::LEFT_CENTER,
                "Connecting...",
                TEXT,
            );
        }
    }
}

fn field(
    ui: &mut egui::Ui,
    o: Pos2,
    y: f32,
    label: &str,
    value: &mut String,
    password: bool,
    hint: &str,
) {
    // XIB field origin: 3 + 10 + 22 + 110 = 145; width 247,
    // label ends 5 points before it; adjacent rows are 34 points apart.
    text(
        ui,
        o + Vec2::new(137.0, y + 11.0),
        Align2::RIGHT_CENTER,
        label,
        TEXT,
    );
    ui.put(
        rect(o, 145.0, y, 247.0, 22.0),
        egui::TextEdit::singleline(value)
            .password(password)
            .hint_text(hint)
            .margin(Vec2::new(4.0, 3.0)),
    );
}

fn password_field(
    ui: &mut egui::Ui,
    o: Pos2,
    y: f32,
    label: &str,
    value: &mut String,
    visible: &mut bool,
) {
    text(
        ui,
        o + Vec2::new(137.0, y + 11.0),
        Align2::RIGHT_CENTER,
        label,
        TEXT,
    );
    ui.put(
        rect(o, 145.0, y, 220.0, 22.0),
        egui::TextEdit::singleline(value)
            .password(!*visible)
            .id_salt(label)
            .margin(Vec2::new(4.0, 3.0)),
    );
    let bounds = rect(o, 367.0, y, 25.0, 22.0);
    let title = if *visible {
        "Hide password"
    } else {
        "Show password"
    };
    let response = ui.put(bounds, egui::Button::new("").frame(false));
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), title)
    });
    if response.on_hover_text(title).clicked() {
        *visible = !*visible;
    }
    icon(
        ui,
        bounds.shrink(2.0),
        if *visible { "eye-slash" } else { "eye" },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exported_symbols_contain_visible_pixels() {
        for bytes in [
            &include_bytes!("../../assets/connection/structure.png")[..],
            &include_bytes!("../../assets/connection/console.png")[..],
            &include_bytes!("../../assets/connection/quick-connect.png")[..],
            &include_bytes!("../../assets/connection/eye.png")[..],
        ] {
            let image = image::load_from_memory(bytes).unwrap().to_rgba8();
            assert!(image.pixels().any(|pixel| pixel[3] > 128));
            assert!(image.pixels().any(|pixel| pixel[3] == 0));
        }
    }

    #[test]
    fn connection_screen_renders_all_methods_on_cold_and_warm_frames() {
        // First-frame rendering previously deadlocked when a missing icon was
        // uploaded from inside Context::data_mut. Exercise the actual UI path.
        let ctx = egui::Context::default();
        let mut app = SequelAceApp::new();
        for method in [
            ConnectionMethod::TcpIp,
            ConnectionMethod::Socket,
            ConnectionMethod::Ssh,
        ] {
            app.connection_method = method;
            for _ in 0..2 {
                let output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(882.0, 759.0))),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| app.connection_window(ui));
                    },
                );
                assert!(!output.shapes.is_empty());
            }
        }
    }
}
