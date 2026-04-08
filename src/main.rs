#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod gears;

use eframe::egui::*;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let mut circle_drawer = CircleDrawer::new();
    circle_drawer.default();

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    return eframe::run_native(
        "Circle Drawer",
        options,
        Box::new(|_cc| Ok(Box::new(circle_drawer))),
    );
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    let mut circle_drawer = CircleDrawer::new();
    circle_drawer.default();

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(circle_drawer))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}

struct CircleDrawer {
    label: String,
    segments: u16,
    radius: f32,
    // TODO : Add radius limits
    stroke_color: Color32,
    stroke_width: f32,
    circles_position: Vec<Vec2>,
    circles_points: Vec<Vec<glam::Vec2>>,
    circles_strokes: Vec<Stroke>,
}

impl CircleDrawer {
    fn new() -> Self {
        Self {
            label: String::from("Circle Drawer"),
            segments: 20,
            radius: 100.0,
            stroke_color: Color32::RED,
            stroke_width: 2.0,
            // circles: vec!{},
            circles_position: vec![],
            circles_points: vec![],
            circles_strokes: vec![],
        }
    }

    // Draw Default Circles
    fn default(&mut self) {
        self.create_circle(vec2(400.0, 300.0), 100.0, 20, Color32::RED, 2.0);
        self.create_circle(vec2(450.0, 300.0), 150.0, 20, Color32::GREEN, 2.0);
        self.create_circle(vec2(500.0, 300.0), 200.0, 20, Color32::BLUE, 2.0);
    }

    fn create_circle(
        &mut self,
        circle_position: Vec2,
        radius: f32,
        segments: u16,
        stroke_color: Color32,
        stroke_width: f32,
    ) {
        let circle_points = gears::circle_points(radius, segments);
        let stroke = Stroke::new(stroke_width, stroke_color);

        self.add_circle(circle_position, circle_points, stroke);
    }

    fn add_circle(
        &mut self,
        circle_position: Vec2,
        circle_points: Vec<glam::Vec2>,
        stroke: Stroke,
    ) {
        // self.circles.push(self.circles.len() as u16);

        self.circles_position.push(circle_position);
        self.circles_points.push(circle_points);
        self.circles_strokes.push(stroke);
    }

    fn add_circles(
        &mut self,
        circles_positions: Vec<Vec2>,
        circles_points: Vec<Vec<glam::Vec2>>,
        strokes: Vec<Stroke>,
    ) {
        for i in 0..circles_points.len() {
            self.add_circle(circles_positions[i], circles_points[i].clone(), strokes[i]);
        }
    }

    fn clear_circles(&mut self) {
        // self.circles.clear();
        self.circles_position.clear();
        self.circles_points.clear();
        self.circles_strokes.clear();
    }

    fn draw_circle(
        &self,
        painter: &Painter,
        stroke: Stroke,
        base_pos: Pos2,
        circle_points: &Vec<glam::Vec2>,
    ) {
        let draw_circle_edge = |start_pos: glam::Vec2, end_pos: glam::Vec2| {
            let pos1: Pos2 = base_pos + vec2(start_pos.x, start_pos.y);
            let pos2: Pos2 = base_pos + vec2(end_pos.x, end_pos.y);

            painter.line_segment([pos1, pos2], stroke);
        };

        // Draw Edges
        for index in 0..circle_points.len() - 1 {
            draw_circle_edge(circle_points[index], circle_points[index + 1]);
        }

        // Draw Last Edge
        draw_circle_edge(circle_points[circle_points.len() - 1], circle_points[0]);
    }
}

impl eframe::App for CircleDrawer {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading(&self.label);

            // Interactions

            if ui.button("Clear Circles").clicked() {
                self.clear_circles();
            }

            // Circle Settings
            ui.horizontal(|ui| {
                ui.label("Segments :");
                ui.add(egui::Slider::new(&mut self.segments, 5..=100));
                ui.label("Radius :");
                ui.add(egui::Slider::new(&mut self.radius, 50.0..=300.0));
            });

            // Stroke Settings
            ui.horizontal(|ui| {
                ui.label("Stroke Width :");
                ui.add(egui::Slider::new(&mut self.stroke_width, 1.0..=10.0));

                let color_label = ui.label("Stroke Color :");
                ui.color_edit_button_srgba(&mut self.stroke_color)
                    .labelled_by(color_label.id);
            });

            // Circles Draw Area

            let (response, painter) = ui.allocate_painter(
                ui.available_size(), // All remaining space
                Sense::click(),
            );

            // Actions

            let mut left_mouse_button_released = false;
            let mut global_mouse_position = Pos2::new(0.0, 0.0);

            // Inputs Handling
            ui.input(|i: &InputState| {
                // Change Radius with mousewheel
                let delta = i.smooth_scroll_delta;
                self.radius = (self.radius + delta.y).clamp(50.0, 300.0);

                left_mouse_button_released = i.pointer.primary_released();

                if let Some(mouse_position) = i.pointer.hover_pos() {
                    global_mouse_position = mouse_position;
                }
            });

            // Create Circle on Click
            if response.hovered() && left_mouse_button_released {
                self.create_circle(
                    vec2(global_mouse_position.x, global_mouse_position.y),
                    self.radius,
                    self.segments,
                    self.stroke_color,
                    self.stroke_width,
                );
            }

            // Draw Circles

            for i in 0..self.circles_position.len() {
                self.draw_circle(
                    &painter,
                    self.circles_strokes[i],
                    Pos2::new(self.circles_position[i].x, self.circles_position[i].y),
                    &self.circles_points[i],
                );
            }
        });
    }
}
