mod event_renderer;
mod stars;
mod timeline;

use eframe::egui;
use eframe::epaint::Color32;
use event_renderer::Camera;
use std::collections::HashMap;
use timeline::{Event, Timeline};

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(target_arch = "wasm32")]
use instant::Instant;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Lifeline Timeline"),
        ..Default::default()
    };

    eframe::run_native(
        "Lifeline",
        options,
        Box::new(|_cc| Ok(Box::new(LifelineApp::new()))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast;

    // Make sure panics are logged using console.error
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends
    tracing_wasm::set_as_global_default();

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
                Box::new(|_cc| Ok(Box::new(LifelineApp::new()))),
            )
            .await;

        // Remove loading spinner
        if let Some(loading) = document.get_element_by_id("loading") {
            loading.remove();
        }

        start_result.expect("failed to start eframe");
    });
}

struct LifelineApp {
    stars: Vec<stars::Star>,
    galaxies: Vec<stars::Galaxy>,
    nebulas: Vec<stars::Nebula>,
    start_time: Instant,
    timeline: Timeline,
    camera: Camera,
    // Click-to-stop state
    clicked_event_index: Option<usize>,
    frozen_positions: HashMap<usize, (f32, f32)>,
    resume_start_times: HashMap<usize, f32>,
    // UI state for adding events
    new_event_title: String,
    new_event_description: String,
    new_event_day: String,
    new_event_month: String,
    new_event_year: String,
    new_event_image_path: String,
    show_add_panel: bool,
    // Image cache
    image_cache: HashMap<String, egui::TextureHandle>,
}

impl LifelineApp {
    fn new() -> Self {
        // Generate background cosmic objects
        let stars = stars::generate_stars(150);
        let galaxies = stars::generate_galaxies(4);
        let nebulas = stars::generate_nebulas(3);

        // Create timeline with mock events
        let mut timeline = Timeline::new();

        // Golden yellow color for all events
        let golden_yellow = Color32::from_rgb(255, 215, 0);

        // Add mock events starting from 1996
        let mut event1 = Event::new(
            "Thomas was born".to_string(),
            "On February 21, 1996 Thomas was born.".to_string(),
            21,
            2,
            1996,
            None,
        );
        event1.color = golden_yellow;
        timeline.add_event(event1);

        let mut event2 = Event::new(
            "Leia was born".to_string(),
            "On July 8, 2025 Leia was born.".to_string(),
            8,
            7,
            2025,
            None,
        );
        event2.color = golden_yellow;
        timeline.add_event(event2);

        Self {
            stars,
            galaxies,
            nebulas,
            start_time: Instant::now(),
            timeline,
            camera: Camera {
                offset_x: 0.0,
                offset_y: 0.0,
                zoom: 1.0,
            },
            clicked_event_index: None,
            frozen_positions: HashMap::new(),
            resume_start_times: HashMap::new(),
            new_event_title: String::new(),
            new_event_description: String::new(),
            new_event_day: String::new(),
            new_event_month: String::new(),
            new_event_year: String::new(),
            new_event_image_path: String::new(),
            show_add_panel: false,
            image_cache: HashMap::new(),
        }
    }

    fn load_image_texture(
        &mut self,
        ctx: &egui::Context,
        path: &str,
    ) -> Option<egui::TextureHandle> {
        // Check cache first
        if let Some(texture) = self.image_cache.get(path) {
            return Some(texture.clone());
        }

        // Try to load the image
        let img_result = if path.starts_with("data:image") {
            // Handle base64 data URLs (for WASM)
            if let Some(base64_data) = path.split(',').nth(1) {
                #[cfg(target_arch = "wasm32")]
                {
                    use base64::Engine;
                    if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(base64_data)
                    {
                        image::load_from_memory(&bytes).ok()
                    } else {
                        None
                    }
                }
                #[cfg(not(target_arch = "wasm32"))]
                None
            } else {
                None
            }
        } else {
            // Handle file paths (for native)
            image::open(path).ok()
        };

        if let Some(img) = img_result {
            let img_rgba = img.to_rgba8();
            let size = [img_rgba.width() as usize, img_rgba.height() as usize];
            let pixels = img_rgba.as_flat_samples();

            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

            let texture = ctx.load_texture(path, color_image, egui::TextureOptions::LINEAR);

            self.image_cache.insert(path.to_string(), texture.clone());
            Some(texture)
        } else {
            None
        }
    }
}

impl eframe::App for LifelineApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for uploaded image in WASM
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    if let Ok(Some(data_url)) = storage.get_item("lifeline_temp_image") {
                        self.new_event_image_path = data_url;
                        let _ = storage.remove_item("lifeline_temp_image");
                    }
                }
            }
        }

        // Handle camera input - but only if add panel is hidden and no text edit is focused
        let pan_speed = 5.0;
        let zoom_speed = 0.1;

        let wants_keyboard_input = ctx.wants_keyboard_input();

        if !self.show_add_panel && !wants_keyboard_input {
            ctx.input(|i| {
                // WASD for panning
                if i.key_down(egui::Key::W) {
                    self.camera.offset_y += pan_speed;
                }
                if i.key_down(egui::Key::S) {
                    self.camera.offset_y -= pan_speed;
                }
                if i.key_down(egui::Key::A) {
                    self.camera.offset_x += pan_speed;
                }
                if i.key_down(egui::Key::D) {
                    self.camera.offset_x -= pan_speed;
                }
            });
        }

        // Scroll wheel for zooming - always available
        ctx.input(|i| {
            let scroll_delta = i.smooth_scroll_delta.y;
            if scroll_delta != 0.0 {
                self.camera.zoom *= 1.0 + scroll_delta * zoom_speed * 0.01;
                self.camera.zoom = self.camera.zoom.clamp(0.1, 5.0);
            }
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(egui::Color32::BLACK))
            .show(ctx, |ui| {
                let time = self.start_time.elapsed().as_secs_f32();

                // Draw background cosmic objects with parallax (back to front)
                let painter = ui.painter();
                let rect = ui.available_rect_before_wrap();

                // Render nebulas first (furthest back)
                stars::render_nebulas(
                    &self.nebulas,
                    painter,
                    rect,
                    time,
                    self.camera.offset_x,
                    self.camera.offset_y,
                    self.camera.zoom,
                );

                // Render galaxies
                stars::render_galaxies(
                    &self.galaxies,
                    painter,
                    rect,
                    time,
                    self.camera.offset_x,
                    self.camera.offset_y,
                    self.camera.zoom,
                );

                // Render stars (closest background layer)
                stars::render_stars(
                    &self.stars,
                    painter,
                    rect,
                    time,
                    self.camera.offset_x,
                    self.camera.offset_y,
                    self.camera.zoom,
                );

                // Pre-load images for events with image paths
                let image_paths: Vec<String> = self
                    .timeline
                    .events()
                    .iter()
                    .filter_map(|e| e.image_path.clone())
                    .filter(|path| !self.image_cache.contains_key(path))
                    .collect();

                for path in image_paths {
                    self.load_image_texture(ctx, &path);
                }

                // Draw timeline events (returns clicked event index if any)
                let new_clicked = event_renderer::render_timeline_events(
                    &self.timeline,
                    time,
                    ui,
                    &self.camera,
                    &mut self.frozen_positions,
                    &mut self.resume_start_times,
                    self.clicked_event_index,
                    &self.image_cache,
                );

                // Update clicked state
                self.clicked_event_index = new_clicked;
            });

        // Bottom panel for adding events (centered)
        egui::TopBottomPanel::bottom("add_event_panel")
            .show_separator_line(false)
            .frame(egui::Frame::new().fill(Color32::from_rgba_unmultiplied(0, 0, 0, 200)))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);

                    let button_text = if self.show_add_panel {
                        "Hide ▼"
                    } else {
                        "Add Event ▲"
                    };
                    if ui
                        .add_sized([150.0, 40.0], egui::Button::new(button_text))
                        .clicked()
                    {
                        self.show_add_panel = !self.show_add_panel;
                    }

                    if self.show_add_panel {
                        ui.add_space(10.0);

                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("Title:");
                                ui.text_edit_singleline(&mut self.new_event_title);

                                ui.label("Description:");
                                ui.text_edit_singleline(&mut self.new_event_description);
                            });

                            ui.add_space(5.0);

                            ui.horizontal(|ui| {
                                ui.label("Day:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.new_event_day)
                                        .desired_width(40.0),
                                );

                                ui.label("Month:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.new_event_month)
                                        .desired_width(40.0),
                                );

                                ui.label("Year:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.new_event_year)
                                        .desired_width(80.0),
                                );
                            });

                            ui.add_space(5.0);

                            ui.horizontal(|ui| {
                                ui.label("Image Path:");
                                ui.text_edit_singleline(&mut self.new_event_image_path);

                                #[cfg(not(target_arch = "wasm32"))]
                                if ui.button("Browse...").clicked() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter(
                                            "Images",
                                            &["png", "jpg", "jpeg", "gif", "bmp", "webp"],
                                        )
                                        .pick_file()
                                    {
                                        self.new_event_image_path = path.display().to_string();
                                    }
                                }

                                #[cfg(target_arch = "wasm32")]
                                if ui.button("Browse...").clicked() {
                                    let ctx = ui.ctx().clone();
                                    wasm_bindgen_futures::spawn_local(async move {
                                        if let Some(file) = rfd::AsyncFileDialog::new()
                                            .add_filter(
                                                "Images",
                                                &["png", "jpg", "jpeg", "gif", "bmp", "webp"],
                                            )
                                            .pick_file()
                                            .await
                                        {
                                            let data = file.read().await;
                                            // Convert to base64 data URL
                                            let base64 = base64::Engine::encode(
                                                &base64::engine::general_purpose::STANDARD,
                                                &data,
                                            );
                                            let data_url =
                                                format!("data:image/png;base64,{}", base64);

                                            // Store in a way we can retrieve it
                                            if let Some(window) = web_sys::window() {
                                                if let Ok(Some(storage)) = window.local_storage() {
                                                    let _ = storage
                                                        .set_item("lifeline_temp_image", &data_url);
                                                    ctx.request_repaint();
                                                }
                                            }
                                        }
                                    });
                                }

                                if ui
                                    .add_sized([120.0, 25.0], egui::Button::new("Add to Timeline"))
                                    .clicked()
                                {
                                    let day = self.new_event_day.parse::<u8>().unwrap_or(1);
                                    let month = self.new_event_month.parse::<u8>().unwrap_or(1);
                                    let year = self.new_event_year.parse::<i32>().unwrap_or(2026);

                                    let image_path = if self.new_event_image_path.is_empty() {
                                        None
                                    } else {
                                        Some(self.new_event_image_path.clone())
                                    };

                                    let golden_yellow = Color32::from_rgb(255, 215, 0);
                                    let mut event = Event::new(
                                        self.new_event_title.clone(),
                                        self.new_event_description.clone(),
                                        day,
                                        month,
                                        year,
                                        image_path,
                                    );
                                    event.color = golden_yellow;
                                    self.timeline.add_event(event);

                                    self.new_event_title.clear();
                                    self.new_event_description.clear();
                                    self.new_event_day.clear();
                                    self.new_event_month.clear();
                                    self.new_event_year.clear();
                                    self.new_event_image_path.clear();
                                }

                                if ui
                                    .add_sized([80.0, 25.0], egui::Button::new("Today"))
                                    .clicked()
                                {
                                    let image_path = if self.new_event_image_path.is_empty() {
                                        None
                                    } else {
                                        Some(self.new_event_image_path.clone())
                                    };

                                    let golden_yellow = Color32::from_rgb(255, 215, 0);
                                    let mut event = Event::new(
                                        self.new_event_title.clone(),
                                        self.new_event_description.clone(),
                                        16,
                                        1,
                                        2026,
                                        image_path,
                                    );
                                    event.color = golden_yellow;
                                    self.timeline.add_event(event);

                                    self.new_event_title.clear();
                                    self.new_event_description.clear();
                                    self.new_event_day.clear();
                                    self.new_event_month.clear();
                                    self.new_event_year.clear();
                                    self.new_event_image_path.clear();
                                }
                            });
                        });
                    }

                    ui.add_space(10.0);
                });
            });

        // Request continuous repaint for animation
        ctx.request_repaint();
    }
}
