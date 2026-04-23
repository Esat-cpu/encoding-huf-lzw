mod algorithms;

use eframe::egui::{self, Align2, Color32, FontId, Painter, Pos2, Stroke};
use std::time::{Duration, Instant};
use std::fs;

use crate::algorithms::huffman::{Huffman, Node, NodeKind};
use crate::algorithms::lzw::Lzw;


const FILE: &str = "file.txt";

#[derive(PartialEq, Copy, Clone)]
enum Algo {
    Huffman,
    Lzw,
}

#[derive(PartialEq)]
enum State {
    Still,
    Paused,
    Playing,
}

struct AlgoApp {
    selected: Algo,
    prev_selected: Algo,
    input: String,
    state: State,
    speed: f32,
    last_tick: Instant,
    step: usize,
    total_steps: usize,
    huf_scroll_check: bool,
    file: &'static str,
    huffman: Huffman,
    lzw: Lzw,
}

impl Default for AlgoApp {
    fn default() -> Self {
        Self {
            selected: Algo::Huffman,
            prev_selected: Algo::Huffman,
            input: "".to_owned(),
            state: State::Still,
            speed: 5.0,
            last_tick: Instant::now(),
            step: 0,
            total_steps: 0,
            huf_scroll_check: true,
            file: FILE,
            huffman: Huffman::default(),
            lzw: Lzw::default(),
        }
    }
}

impl AlgoApp {
    fn next_step(&mut self) {
        if self.step < self.total_steps {
            self.step += 1;
        } else {
            self.state = State::Still;
        }
    }

    fn reset(&mut self) {
        self.step = 0;
        self.total_steps = 0;
        self.state = State::Still;

        self.huffman = Huffman::default();
        self.lzw = Lzw::default();
    }
}

impl eframe::App for AlgoApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.selected != self.prev_selected {
            self.reset();
            self.prev_selected = self.selected;
        }
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
        if self.state == State::Playing && self.last_tick.elapsed() >= delay {
            self.next_step();
            self.last_tick = Instant::now();
        }
        if self.state == State::Playing {
            ctx.request_repaint();
        }

        // Left Panel
        egui::Panel::left("controls")
            .resizable(false)
            .default_size(220.0)
            .show_inside(ui, |ui| {
                ui.add_space(20.0);
                ui.heading("🔬 Menu");
                ui.separator();
                ui.add_space(10.0);

                ui.label("Choose Algorithm:");
                ui.add_space(5.0);
                ui.selectable_value(&mut self.selected, Algo::Huffman, "🌳 Huffman Encoding");
                ui.add_space(5.0);
                ui.selectable_value(&mut self.selected, Algo::Lzw, "📑 LZW Encoding");

                if self.selected == Algo::Huffman {
                    ui.add_space(20.0);
                    ui.checkbox(&mut self.huf_scroll_check, "Scroll Tree");
                }

                ui.add_space(20.0);
                ui.label("Text:");
                ui.text_edit_singleline(&mut self.input);

                if ui.button(format!("Read from the '{}' file", self.file)).clicked() {
                    self.input = readline_from_file(self.file);
                }

                ui.add_space(20.0);
                ui.label("Work Flow");
                ui.add(egui::Slider::new(&mut self.speed, 1.0..=10.0).text("Speed"));

                ui.add_space(20.0);

                let btn_text = if self.state == State::Playing {
                    "⏸  PAUSE"
                } else if self.state == State::Paused {
                    "▶  CONTINUE"
                } else {
                    "▶  START"
                };

                let btn = egui::Button::new(btn_text).fill(if self.state == State::Playing {
                    Color32::from_rgb(180, 50, 50)
                } else {
                    Color32::from_rgb(50, 150, 80)
                });

                if ui.add_sized([ui.available_width(), 45.0], btn).clicked() {
                    if self.state == State::Still {
                        self.step = 0;
                        match self.selected {
                            Algo::Huffman => {
                                self.huffman = Huffman::encode(&self.input);
                                // 1: freq table steps (for each unique character)
                                let freq_steps = self.huffman.freq_table.len();
                                // 2: building tree steps (for each merge)
                                let merge_steps = freq_steps.saturating_sub(1);
                                // 3: code table steps (for each unique character)
                                let code_steps = self.huffman.code_table.len();
                                self.total_steps = freq_steps + merge_steps + code_steps;
                            }
                            Algo::Lzw => {
                                self.lzw = Lzw::encode(&self.input);
                                self.total_steps = self.lzw.steps.len();
                            }
                        }
                    }
                    self.state = if self.state == State::Playing {
                        State::Paused
                    } else {
                        State::Playing
                    };
                }

                // Step Info Label
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(5.0);
                ui.label(format!("Step: {} / {}", self.step, self.total_steps));
            });

        // Right Panel
        egui::Panel::right("encoder")
            .resizable(true)
            .default_size(240.0)
            .show_inside(ui, |ui| {
                ui.add_space(20.0);
                ui.heading("Table");
                ui.separator();
                ui.add_space(10.0);

                match self.selected {
                    Algo::Huffman => {
                        // Convert frequency table to Vec
                        let mut freq_vec: Vec<(char, u32)> = self
                            .huffman
                            .freq_table
                            .iter()
                            .map(|(&ch, &fr)| (ch, fr))
                            .collect();
                        freq_vec.sort_by_key(|&(_, f)| f);

                        let freq_count = freq_vec.len();
                        let merge_count = freq_count.saturating_sub(1);

                        // Code phase
                        let code_phase_start = freq_count + merge_count;

                        // Convert code table to Vec
                        let code_vec: Vec<(char, String)> = freq_vec
                            .iter()
                            .filter_map(|(ch, _)| {
                                self.huffman.code_table.get(ch).map(|c| (*ch, c.clone()))
                            })
                            .collect();

                        ui.add(egui::Label::new(
                            egui::RichText::new("Char  Freq  Code").monospace().weak(),
                        ));
                        ui.add_space(4.0);

                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.set_min_width(ui.available_width());

                            for (i, (ch, freq)) in freq_vec.iter().enumerate() {
                                // Is this line visible
                                if i >= self.step && self.step <= freq_count {
                                    break;
                                }

                                // Is the code column full
                                let code_str = if self.step > code_phase_start + i {
                                    code_vec.get(i).map(|(_, c)| c.as_str()).unwrap_or("*")
                                } else {
                                    "*"
                                };

                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.colored_label(
                                            Color32::from_rgb(100, 180, 255),
                                            format!("  {}   ", ch),
                                        );
                                        ui.colored_label(
                                            Color32::from_rgb(220, 200, 100),
                                            format!(" {:4}  ", freq),
                                        );
                                        ui.colored_label(
                                            if code_str == "*" {
                                                Color32::GRAY
                                            } else {
                                                Color32::from_rgb(100, 220, 140)
                                            },
                                            format!("{:>8}", code_str),
                                        );
                                    });
                                });
                            }
                        });
                    }
                    Algo::Lzw => {
                        ui.label(egui::RichText::new("Code  String").monospace().weak());
                        ui.add_space(4.0);

                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.set_min_width(ui.available_width());

                            let mut dict_vec: Vec<(String, String)> = self
                                .lzw
                                .code_table
                                .iter()
                                .map(|(k, v)| (k.clone(), v.clone()))
                                .collect();
                            dict_vec.sort_by_key(|(_, v)| v.parse::<usize>().unwrap_or(0));

                            // Display only the entries added so far
                            let visible_dict: Vec<_> = dict_vec
                                .iter()
                                .filter(|(_, v)| v.parse::<usize>().unwrap_or(0) < self.step)
                                .collect();

                            for (key, code) in &visible_dict {
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.colored_label(
                                            Color32::from_rgb(220, 200, 100),
                                            format!("  {:<3}    ", code),
                                        );
                                        ui.colored_label(
                                            Color32::from_rgb(100, 180, 255),
                                            format!("  {:>4} ", key),
                                        );
                                    });
                                });
                            }
                        });
                    }
                }
            });

        // Middle Panel
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.add_space(10.0);
            ui.vertical_centered(|ui| {
                let phase = if self.huffman.freq_table.is_empty() {
                    "—".to_owned()
                } else {
                    let freq_count = self.huffman.freq_table.len();
                    let merge_count = freq_count.saturating_sub(1);
                    let code_start = freq_count + merge_count;
                    if self.step <= freq_count {
                        "Building frequency table".to_owned()
                    } else if self.step <= code_start {
                        "Building tree".to_owned()
                    } else {
                        "Building code table".to_owned()
                    }
                };
                ui.heading(format!("Step {}  —  {}", self.step, phase));
            });
            ui.separator();

            egui::Frame::canvas(ui.style())
                .fill(ui.style().visuals.extreme_bg_color)
                .corner_radius(15.0)
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.set_min_size(ui.available_size());

                    match self.selected {
                        Algo::Huffman => {
                            egui::ScrollArea::horizontal().show(ui, |ui| {
                                let freq_count = self.huffman.freq_table.len();
                                let merge_count = freq_count.saturating_sub(1);

                                let tree_phase_start = freq_count;
                                let tree_phase_end = freq_count + merge_count;

                                // Show the tree only in tree phase and after
                                if self.step > tree_phase_start || self.step >= self.total_steps {
                                    let visible_merges = if self.step > tree_phase_end {
                                        merge_count
                                    } else {
                                        self.step - tree_phase_start
                                    };


                                    let width =
                                        if let Some(node) = &self.huffman.tree_root && self.huf_scroll_check {
                                            subtree_width(node, 80.0)
                                        } else {
                                            ui.available_width()
                                        };

                                    let (response, painter) =
                                        ui.allocate_painter(
                                            egui::Vec2::new(width, ui.available_height()),
                                            egui::Sense::hover(),
                                        );
                                    let rect = response.rect;

                                    if let Some(root) = &self.huffman.tree_root {
                                        draw_node(
                                            &painter,
                                            root,
                                            rect.center().x,
                                            rect.top() + 80.0,
                                            rect.width() / 4.0,
                                            visible_merges,
                                        );
                                    }
                                } else {
                                    // The canvas is empty in frequency steps
                                    let (response, painter) =
                                        ui.allocate_painter(
                                            ui.available_size(),
                                            egui::Sense::hover(),
                                        );
                                    painter.text(
                                        response.rect.center(),
                                        Align2::CENTER_CENTER,
                                        "Tree will appear here",
                                        FontId::proportional(16.0),
                                        Color32::DARK_GRAY,
                                    );
                                }
                            });
                        }
                        Algo::Lzw => {
                            // Column headers
                            ui.add_space(8.0);
                            egui::Grid::new("lzw_header")
                                .num_columns(7)
                                .min_col_width(60.0)
                                .show(ui, |ui| {
                                    for header in [
                                        "Step", "Input", "Buffer", "In Dict", "Temp", "ATD",
                                        "Output",
                                    ] {
                                        ui.label(
                                            egui::RichText::new(header)
                                                .monospace()
                                                .strong()
                                                .color(Color32::GRAY),
                                        );
                                    }
                                    ui.end_row();
                                });

                            ui.separator();

                            egui::ScrollArea::vertical().show(ui, |ui| {
                                egui::Grid::new("lzw_steps")
                                    .num_columns(7)
                                    .min_col_width(60.0)
                                    .striped(true)
                                    .show(ui, |ui| {
                                        for (i, step) in self.lzw.steps.iter().enumerate() {
                                            if i >= self.step {
                                                break;
                                            }

                                            let add_cell =
                                                |ui: &mut egui::Ui, text: &str, color: Color32| {
                                                    ui.colored_label(color, text);
                                                };

                                            add_cell(
                                                ui,
                                                &step.number.to_string(),
                                                Color32::from_rgb(180, 180, 180),
                                            );
                                            add_cell(
                                                ui,
                                                &step.input.to_string(),
                                                Color32::from_rgb(100, 200, 255),
                                            );
                                            add_cell(
                                                ui,
                                                &step.buffer,
                                                Color32::from_rgb(220, 220, 100),
                                            );

                                            let in_dict_str =
                                                if step.in_dict { "✔" } else { "×" };
                                            let in_dict_color = if step.in_dict {
                                                Color32::from_rgb(100, 220, 100)
                                            } else {
                                                Color32::from_rgb(220, 100, 100)
                                            };
                                            add_cell(ui, in_dict_str, in_dict_color);

                                            add_cell(
                                                ui,
                                                &step.temp,
                                                Color32::from_rgb(200, 160, 255),
                                            );

                                            let atd_color = if step.atd == "--" {
                                                Color32::GRAY
                                            } else {
                                                Color32::from_rgb(255, 180, 80)
                                            };
                                            add_cell(ui, &step.atd, atd_color);

                                            let output_color = if step.output == "--" {
                                                Color32::GRAY
                                            } else {
                                                Color32::from_rgb(100, 240, 160)
                                            };
                                            add_cell(ui, &step.output, output_color);

                                            ui.end_row();
                                        }
                                    });
                            });
                        }
                    }
                });
        });
    }
}

fn subtree_width(node: &Node, spacing: f32) -> f32 {
    match (&node.left, &node.right) {
        (None, None) => spacing,

        (Some(left), None) => {
            subtree_width(left, spacing) + spacing
        }

        (None, Some(right)) => {
            spacing + subtree_width(right, spacing)
        }

        (Some(left), Some(right)) => {
            subtree_width(left, spacing)
                + spacing
                + subtree_width(right, spacing)
        }
    }
}


// Draw node recursively — show as many merges as visible_merges
fn draw_node(
    painter: &Painter,
    node: &Node,
    x: f32,
    y: f32,
    half_width: f32,
    visible_merges: usize,
) {
    let radius = 22.0;
    let level_height = 75.0;
    let center = Pos2::new(x, y);

    // Is this node visible
    let self_visible = match node.val {
        NodeKind::Leaf(_) => true,
        NodeKind::Internal => node.order <= visible_merges,
    };

    // Left child
    if let Some(left) = &node.left {
        let child_x = x - half_width;
        let child_y = y + level_height;

        let child_visible = match left.val {
            NodeKind::Leaf(_) => true,
            NodeKind::Internal => left.order <= visible_merges,
        };

        // Draw the edge if both nodes are visible
        if self_visible && child_visible {
            painter.line_segment(
                [center, Pos2::new(child_x, child_y)],
                Stroke::new(1.5, Color32::from_rgb(80, 120, 160)),
            );
            painter.text(
                Pos2::new((x + child_x) / 2.0 - 8.0, (y + child_y) / 2.0),
                Align2::CENTER_CENTER,
                "0",
                FontId::proportional(11.0),
                Color32::from_rgb(100, 200, 100),
            );
        }

        // Draw child recursively - even if the parent is not visible
        draw_node(
            painter,
            left,
            child_x,
            child_y,
            half_width / 2.0,
            visible_merges,
        );
    }

    // Right child
    if let Some(right) = &node.right {
        let child_x = x + half_width;
        let child_y = y + level_height;

        let child_visible = match right.val {
            NodeKind::Leaf(_) => true,
            NodeKind::Internal => right.order <= visible_merges,
        };

        if self_visible && child_visible {
            painter.line_segment(
                [center, Pos2::new(child_x, child_y)],
                Stroke::new(1.5, Color32::from_rgb(80, 120, 160)),
            );
            painter.text(
                Pos2::new((x + child_x) / 2.0 + 8.0, (y + child_y) / 2.0),
                Align2::CENTER_CENTER,
                "1",
                FontId::proportional(11.0),
                Color32::from_rgb(100, 200, 100),
            );
        }

        draw_node(
            painter,
            right,
            child_x,
            child_y,
            half_width / 2.0,
            visible_merges,
        );
    }

    // Do not draw if it is not visible
    if !self_visible {
        return;
    }

    // Circle
    let (fill, stroke_color) = match node.val {
        NodeKind::Leaf(_) => (
            Color32::from_rgb(25, 70, 45),
            Color32::from_rgb(60, 180, 100),
        ),
        NodeKind::Internal => (
            Color32::from_rgb(25, 45, 85),
            Color32::from_rgb(80, 140, 220),
        ),
    };

    painter.circle_filled(center, radius, fill);
    painter.circle_stroke(center, radius, Stroke::new(1.5, stroke_color));

    // Text
    match &node.val {
        NodeKind::Leaf(ch) => {
            painter.text(
                Pos2::new(x, y - 6.0),
                Align2::CENTER_CENTER,
                ch.to_string(),
                FontId::proportional(14.0),
                Color32::from_rgb(100, 230, 140),
            );
            painter.text(
                Pos2::new(x, y + 8.0),
                Align2::CENTER_CENTER,
                node.freq.to_string(),
                FontId::proportional(10.0),
                Color32::from_rgb(150, 200, 160),
            );
        }
        NodeKind::Internal => {
            painter.text(
                center,
                Align2::CENTER_CENTER,
                node.freq.to_string(),
                FontId::proportional(12.0),
                Color32::from_rgb(140, 190, 255),
            );
        }
    }
}



fn readline_from_file(file: &str) -> String {
    let content = fs::read_to_string(file).unwrap_or_default();
    content.lines().next().unwrap_or("").to_string()
}



fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([1000.0, 700.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Encode",
        options,
        Box::new(|_cc| Ok(Box::new(AlgoApp::default()))),
    )
}

