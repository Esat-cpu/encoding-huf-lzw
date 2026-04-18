mod algorithms;

use eframe::egui;
use std::time::{Duration, Instant};


#[derive(PartialEq)]
enum Algo { Huffman, Lzw }

struct Mapping {
    key: String,
    value: String,
}

struct AlgoApp {
    selected: Algo,
    input: String,
    is_playing: bool,
    speed: f32, // 1.0 - 10.0
    last_tick: Instant,
    step: usize,
    dictionary: Vec<Mapping>,
}

impl Default for AlgoApp {
    fn default() -> Self {
        Self {
            selected: Algo::Huffman,
            input: "ALGORITHM".to_owned(),
            is_playing: false,
            speed: 5.0,
            last_tick: Instant::now(),
            step: 0,
            dictionary: vec![
                Mapping { key: "A".into(), value: "0".into() },
                Mapping { key: "L".into(), value: "11".into() },
            ],
        }
    }
}

impl eframe::App for AlgoApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Style
        let mut style = (*ctx.global_style()).clone();

        style.visuals.widgets.noninteractive.corner_radius = 12.0.into();
        style.visuals.widgets.inactive.corner_radius = 8.0.into();
        style.visuals.widgets.active.corner_radius = 8.0.into();
        style.visuals.widgets.hovered.corner_radius = 8.0.into();
        ctx.set_global_style(style);

        // Timer
        let delay = Duration::from_millis((1100.0 - self.speed * 100.0).max(50.0) as u64);
        if self.is_playing && self.last_tick.elapsed() >= delay {
            self.step += 1;
            self.last_tick = Instant::now();
        }
        if self.is_playing { ctx.request_repaint(); }

        // Left Panel
        egui::Panel::left("controls").resizable(false).default_size(220.0).show_inside(ui, |ui| {
            ui.add_space(20.0);
            ui.heading("🔬 Menu");
            ui.separator();
            ui.add_space(10.0);

            ui.label("Choose Algorithm:");
            ui.add_space(5.0);

            ui.selectable_value(&mut self.selected, Algo::Huffman, "🌳 Huffman Encoding");
            ui.add_space(5.0);
            ui.selectable_value(&mut self.selected, Algo::Lzw, "📑 LZW Encoding");

            ui.add_space(20.0);
            ui.label("Text:");
            ui.text_edit_singleline(&mut self.input);

            ui.add_space(20.0);
            ui.label("Work Flow");
            ui.add(egui::Slider::new(&mut self.speed, 1.0..=10.0).text("Speed"));

            ui.add_space(20.0);
            let btn_text = if self.is_playing { "⏸  STOP" } else { "▶  START" };
            let btn = egui::Button::new(btn_text)
                .fill(if self.is_playing { egui::Color32::from_rgb(180, 50, 50) } else { egui::Color32::from_rgb(50, 150, 80) });

            if ui.add_sized([ui.available_width(), 45.0], btn).clicked() {
                self.is_playing = !self.is_playing;
            }
        });

        // Right Panel
        egui::Panel::right("encoder").resizable(true).default_size(220.0).show_inside(ui, |ui| {
            ui.add_space(20.0);
            ui.heading("Coders");
            ui.separator();
            ui.add_space(10.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                for item in &self.dictionary {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.colored_label(egui::Color32::LIGHT_BLUE, &item.key);
                            ui.label("=");
                            ui.colored_label(egui::Color32::LIGHT_YELLOW, &item.value);
                        });
                    });
                }
            });
        });

        // Middle Panel
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.add_space(10.0);
            ui.vertical_centered(|ui| {
                ui.heading(format!("Step {}", self.step));
            });
            ui.separator();

            egui::Frame::canvas(ui.style())
                .fill(ui.style().visuals.extreme_bg_color)
                .corner_radius(15.0)
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.set_min_size(ui.available_size());

                    if self.selected == Algo::Huffman {
                        let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::hover());
                        let center = response.rect.center_top() + egui::vec2(0.0, 30.0);

                        painter.circle_filled(center, 25.0, egui::Color32::from_rgb(60, 120, 180));
                        painter.circle_stroke(center, 25.0, egui::Stroke::new(2.0, egui::Color32::WHITE));
                        painter.text(
                            center,
                            egui::Align2::CENTER_CENTER,
                            "Root",
                            egui::FontId::proportional(16.0),
                            egui::Color32::WHITE
                        );
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label(egui::RichText::new("📊 LZW Table Field").size(24.0).color(egui::Color32::DARK_GRAY));
                        });
                    }
                });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([1000.0, 700.0]),
        ..Default::default()
    };

    eframe::run_native(
        "AlgoLab Pro",
        options,
        Box::new(|_cc| Ok(Box::new(AlgoApp::default()))),
    )
}
