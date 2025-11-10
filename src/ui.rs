use crate::simulator::{SimulationConfig, Simulator};
use crate::instruction::Instruction;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};

pub struct TierraApp {
    pub simulator: Simulator,
    pub steps_per_frame: usize,
    pub auto_run: bool,
    pub config: SimulationConfig,
    pub memory_view_offset: usize,
    pub memory_view_size: usize,
}

impl Default for TierraApp {
    fn default() -> Self {
        let config = SimulationConfig::default();
        let mut simulator = Simulator::new(config.clone());
        simulator.initialize_with_ancestor();

        Self {
            simulator,
            steps_per_frame: 100,
            auto_run: false,
            config,
            memory_view_offset: 0,
            memory_view_size: 256,
        }
    }
}

impl eframe::App for TierraApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Auto-run simulation
        if self.auto_run {
            self.simulator.run_steps(self.steps_per_frame);
            ctx.request_repaint();
        }

        // Top panel - controls
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Tierra - Artificial Life Simulation");

                ui.separator();

                if ui.button(if self.auto_run { "⏸ Pause" } else { "▶ Run" }).clicked() {
                    self.auto_run = !self.auto_run;
                }

                if ui.button("⏭ Step").clicked() {
                    self.simulator.step();
                }

                if ui.button("🔄 Reset").clicked() {
                    self.simulator.reset();
                    self.simulator.initialize_with_ancestor();
                    self.auto_run = false;
                }

                ui.separator();

                ui.label("Steps/frame:");
                ui.add(egui::Slider::new(&mut self.steps_per_frame, 1..=1000).logarithmic(true));
            });
        });

        // Left panel - statistics
        egui::SidePanel::left("stats_panel").min_width(250.0).show(ctx, |ui| {
            ui.heading("Statistics");
            ui.separator();

            let stats = &self.simulator.stats;

            ui.label(format!("Population: {}", stats.current_population));
            ui.label(format!("Total Instructions: {}", stats.total_instructions));
            ui.label(format!("Total Born: {}", stats.total_organisms_created));
            ui.label(format!("Total Died: {}", stats.total_organisms_died));
            ui.label(format!("Mutations: {}", stats.total_mutations));

            ui.separator();

            ui.label(format!("Memory: {:.1}%", stats.memory_usage_percent()));
            ui.label(format!("Replications: {} / {}",
                stats.successful_replications,
                stats.successful_replications + stats.failed_replications
            ));
            ui.label(format!("Success Rate: {:.1}%", stats.replication_success_rate() * 100.0));

            ui.separator();

            if let Some(size) = stats.most_common_size() {
                ui.label(format!("Most Common Size: {}", size));
            }
            ui.label(format!("Highest Generation: {}", stats.highest_generation()));

            ui.separator();
            ui.heading("Configuration");

            ui.label(format!("Mutation Rate: {:.4}", self.simulator.config.mutation_rate));
            if ui.add(egui::Slider::new(&mut self.simulator.config.mutation_rate, 0.0..=0.1).text("Mutation")).changed() {
                // Mutation rate changed
            }

            ui.label(format!("Max Population: {}", self.simulator.config.max_population));
            ui.add(egui::Slider::new(&mut self.simulator.config.max_population, 10..=500).text("Max Pop"));

            ui.label(format!("Time Slice: {}", self.simulator.config.time_slice));
            ui.add(egui::Slider::new(&mut self.simulator.config.time_slice, 1..=100).text("Time Slice"));

            ui.separator();
            ui.heading("Population Graph");

            // Simple population history graph
            let history = &stats.population_history;
            if !history.is_empty() {
                let max_pop = history.iter().max().copied().unwrap_or(1).max(1);
                let graph_height = 100.0;
                let graph_width = ui.available_width();

                let (response, painter) = ui.allocate_painter(
                    Vec2::new(graph_width, graph_height),
                    egui::Sense::hover()
                );

                let rect = response.rect;
                painter.rect_filled(rect, 0.0, Color32::from_gray(20));

                if history.len() > 1 {
                    let points: Vec<Pos2> = history.iter().enumerate().map(|(i, &pop)| {
                        let x = rect.min.x + (i as f32 / (history.len() - 1) as f32) * rect.width();
                        let y = rect.max.y - (pop as f32 / max_pop as f32) * rect.height();
                        Pos2::new(x, y)
                    }).collect();

                    painter.add(egui::Shape::line(points, Stroke::new(2.0, Color32::GREEN)));
                }

                painter.text(
                    rect.left_top() + Vec2::new(5.0, 5.0),
                    egui::Align2::LEFT_TOP,
                    format!("Max: {}", max_pop),
                    egui::FontId::proportional(10.0),
                    Color32::WHITE,
                );
            }
        });

        // Right panel - organisms list
        egui::SidePanel::right("organisms_panel").min_width(200.0).show(ctx, |ui| {
            ui.heading("Organisms");
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut organisms: Vec<_> = self.simulator.organisms.iter()
                    .filter(|o| o.alive)
                    .collect();

                organisms.sort_by_key(|o| o.id);

                for organism in organisms.iter().take(50) {
                    ui.group(|ui| {
                        ui.label(format!("ID: {}", organism.id));
                        ui.label(format!("Size: {}", organism.size));
                        ui.label(format!("Gen: {}", organism.generation));
                        ui.label(format!("Addr: {:#x}", organism.address));
                        ui.label(format!("Cycles: {}", organism.cycles));
                        ui.label(format!("Errors: {}", organism.errors));
                    });
                }

                if organisms.len() > 50 {
                    ui.label(format!("... and {} more", organisms.len() - 50));
                }
            });
        });

        // Central panel - memory visualization
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Memory Visualization");

            ui.horizontal(|ui| {
                ui.label("View offset:");
                ui.add(egui::Slider::new(&mut self.memory_view_offset, 0..=self.simulator.memory.size().saturating_sub(self.memory_view_size)));

                ui.label("View size:");
                ui.add(egui::Slider::new(&mut self.memory_view_size, 64..=2048).logarithmic(true));
            });

            ui.separator();

            // Draw memory as a grid
            let available_size = ui.available_size();
            let cell_size = 4.0;
            let cells_per_row = (available_size.x / cell_size).floor() as usize;

            if cells_per_row > 0 {
                let (response, painter) = ui.allocate_painter(
                    Vec2::new(available_size.x, available_size.y - 20.0),
                    egui::Sense::hover()
                );

                let rect = response.rect;

                for i in 0..self.memory_view_size.min(cells_per_row * ((rect.height() / cell_size) as usize)) {
                    let addr = self.memory_view_offset + i;
                    if addr >= self.simulator.memory.size() {
                        break;
                    }

                    let inst = self.simulator.memory.read(addr);
                    let color = instruction_to_color(inst);

                    let row = i / cells_per_row;
                    let col = i % cells_per_row;

                    let x = rect.min.x + col as f32 * cell_size;
                    let y = rect.min.y + row as f32 * cell_size;

                    let cell_rect = Rect::from_min_size(
                        Pos2::new(x, y),
                        Vec2::new(cell_size - 1.0, cell_size - 1.0)
                    );

                    painter.rect_filled(cell_rect, 0.0, color);
                }

                // Draw organism boundaries
                for organism in self.simulator.organisms.iter().filter(|o| o.alive) {
                    if organism.address >= self.memory_view_offset &&
                       organism.address < self.memory_view_offset + self.memory_view_size {

                        let rel_addr = organism.address - self.memory_view_offset;
                        // Draw border around organism
                        let size_cells = organism.size.min(self.memory_view_size - rel_addr);
                        for i in 0..size_cells {
                            let cell_row = (rel_addr + i) / cells_per_row;
                            let cell_col = (rel_addr + i) % cells_per_row;
                            let cell_x = rect.min.x + cell_col as f32 * cell_size;
                            let cell_y = rect.min.y + cell_row as f32 * cell_size;

                            let cell_rect = Rect::from_min_size(
                                Pos2::new(cell_x, cell_y),
                                Vec2::new(cell_size - 1.0, cell_size - 1.0)
                            );

                            if i == 0 || i == size_cells - 1 {
                                painter.rect_stroke(cell_rect, 0.0, Stroke::new(1.0, Color32::YELLOW));
                            }
                        }
                    }
                }
            }

            ui.separator();
            ui.label("Colors: Instructions are color-coded by type");
            ui.horizontal(|ui| {
                ui.colored_label(Color32::GRAY, "■ Nop");
                ui.colored_label(Color32::LIGHT_BLUE, "■ Jump");
                ui.colored_label(Color32::LIGHT_GREEN, "■ Data");
                ui.colored_label(Color32::LIGHT_RED, "■ Arithmetic");
                ui.colored_label(Color32::LIGHT_YELLOW, "■ Memory");
                ui.colored_label(Color32::from_rgb(255, 200, 100), "■ Stack");
            });
        });
    }
}

/// Convert an instruction to a color for visualization
fn instruction_to_color(inst: Instruction) -> Color32 {
    match inst {
        Instruction::Nop0 | Instruction::Nop1 => Color32::from_gray(60),
        Instruction::IfCZ | Instruction::JmpB | Instruction::JmpF | Instruction::Call | Instruction::Ret =>
            Color32::from_rgb(100, 150, 255),
        Instruction::MovDC | Instruction::MovCD | Instruction::Adr | Instruction::AdrB | Instruction::AdrF =>
            Color32::from_rgb(100, 255, 100),
        Instruction::IncA | Instruction::IncB | Instruction::IncC | Instruction::DecC =>
            Color32::from_rgb(255, 100, 100),
        Instruction::MallocA | Instruction::Divide =>
            Color32::from_rgb(255, 255, 100),
        Instruction::PushA | Instruction::PushB | Instruction::PushC | Instruction::PushD |
        Instruction::PopA | Instruction::PopB | Instruction::PopC | Instruction::PopD =>
            Color32::from_rgb(255, 200, 100),
        Instruction::Halt => Color32::from_rgb(255, 0, 0),
    }
}
