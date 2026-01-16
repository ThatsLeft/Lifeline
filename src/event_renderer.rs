use crate::timeline::{Event, Timeline};
use eframe::egui;
use eframe::epaint::{Color32, Pos2};

#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

#[cfg(target_arch = "wasm32")]
use web_time::Duration;

pub struct Camera {
    pub offset_x: f32,
    pub offset_y: f32,
    pub zoom: f32,
}

pub struct EventInteraction {
    pub clicked_index: Option<usize>,
    pub delete_index: Option<usize>,
}

pub fn render_timeline_events(
    timeline: &Timeline,
    time: f32,
    ui: &mut egui::Ui,
    camera: &Camera,
    frozen_positions: &mut std::collections::HashMap<usize, (f32, f32)>,
    resume_start_times: &mut std::collections::HashMap<usize, f32>,
    previous_clicked: Option<usize>,
    image_cache: &std::collections::HashMap<String, egui::TextureHandle>,
) -> EventInteraction {
    if timeline.events().is_empty() {
        return EventInteraction {
            clicked_index: None,
            delete_index: None,
        };
    }

    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter();

    let timeline_y = rect.center().y;
    let timeline_start_x = rect.left() + 50.0;
    let timeline_end_x = rect.right() - 50.0;
    let timeline_width = timeline_end_x - timeline_start_x;

    let pointer_pos = ui.input(|i| i.pointer.hover_pos());
    let is_clicking = ui.input(|i| i.pointer.primary_down());
    let is_ctrl_held = ui.input(|i| i.modifiers.ctrl || i.modifiers.command);
    let mut clicked_event_index = None;
    let mut hovered_event_data: Option<(Pos2, Event, usize)> = None;
    let mut delete_event_index = None;

    let events = timeline.events();
    if let (Some(first), Some(last)) = (events.first(), events.last()) {
        let time_range = last
            .timestamp
            .duration_since(first.timestamp)
            .unwrap_or(Duration::from_secs(1));

        for (i, event) in events.iter().enumerate() {
            let time_offset = event
                .timestamp
                .duration_since(first.timestamp)
                .unwrap_or(Duration::from_secs(0));

            let position_ratio = if time_range.as_secs() > 0 {
                time_offset.as_secs_f32() / time_range.as_secs_f32()
            } else {
                0.5
            };

            // Base position on timeline
            let base_x = timeline_start_x + position_ratio * timeline_width;

            // Calculate animated position
            let wave_speed = 1.5 + (i as f32 * 0.1) % 1.0;
            let wave_phase = i as f32 * 2.0;
            let wave_amplitude = 50.0 + (i as f32 * 10.0) % 30.0;
            let wave_offset = (time * wave_speed + wave_phase).sin() * wave_amplitude;

            let pulse_speed = 0.8 + (i as f32 * 0.15) % 0.5;
            let pulse_phase = i as f32 * 1.5;
            let pulse_amplitude = 30.0 + (i as f32 * 8.0) % 25.0;
            let pulse_offset = (time * pulse_speed + pulse_phase).sin() * pulse_amplitude;

            let animated_x = (base_x * camera.zoom) + camera.offset_x;
            let animated_y =
                ((timeline_y + wave_offset + pulse_offset) * camera.zoom) + camera.offset_y;

            // Determine actual position to use (frozen or animating)
            let (x, y) = if let Some(&frozen) = frozen_positions.get(&i) {
                // Event is frozen - check if we're resuming
                if let Some(&resume_start) = resume_start_times.get(&i) {
                    // Calculate smooth resume animation with easing
                    let resume_duration = 1.0; // 1 second smooth resume
                    let resume_progress = ((time - resume_start) / resume_duration).min(1.0);

                    // Ease-out cubic for smooth deceleration
                    let eased_progress = 1.0 - (1.0 - resume_progress).powi(3);

                    // Interpolate from frozen to animated position
                    let lerp_x = frozen.0 + (animated_x - frozen.0) * eased_progress;
                    let lerp_y = frozen.1 + (animated_y - frozen.1) * eased_progress;

                    // If resume is complete, remove from frozen and resume tracking
                    if resume_progress >= 1.0 {
                        frozen_positions.remove(&i);
                        resume_start_times.remove(&i);
                        (animated_x, animated_y)
                    } else {
                        (lerp_x, lerp_y)
                    }
                } else {
                    // Frozen without resuming
                    frozen
                }
            } else {
                // Not frozen, use animated position
                (animated_x, animated_y)
            };

            let event_pos = Pos2::new(x, y);

            // Check if mouse is hovering over this position
            let is_hovered = if let Some(pointer) = pointer_pos {
                let distance =
                    ((pointer.x - event_pos.x).powi(2) + (pointer.y - event_pos.y).powi(2)).sqrt();
                distance < 50.0
            } else {
                false
            };

            // Handle click-to-stop and release-to-resume
            if is_hovered && is_clicking {
                // Ctrl+Click to delete
                if is_ctrl_held {
                    delete_event_index = Some(i);
                } else {
                    // Regular click - freeze it
                    if !frozen_positions.contains_key(&i) {
                        frozen_positions.insert(i, (x, y));
                        resume_start_times.remove(&i); // Cancel any ongoing resume
                    }
                    clicked_event_index = Some(i);
                    hovered_event_data = Some((pointer_pos.unwrap(), event.clone(), i));
                }
            } else if Some(i) == previous_clicked && !is_clicking {
                // Was clicked last frame but released now - start smooth resume
                if frozen_positions.contains_key(&i) && !resume_start_times.contains_key(&i) {
                    resume_start_times.insert(i, time);
                }
            } else if Some(i) != previous_clicked && !is_clicking {
                // Not clicked and wasn't clicked last frame - ensure clean state
                if !resume_start_times.contains_key(&i) {
                    frozen_positions.remove(&i);
                }
            }

            // Show tooltip on hover
            if is_hovered {
                hovered_event_data = Some((pointer_pos.unwrap(), event.clone(), i));
            }

            render_burning_star(painter, event_pos, i, time, event, is_hovered);
            render_event_label(painter, x, y, event, time, i, is_hovered);
        }
    }

    // Render tooltip after releasing painter borrow
    if let Some((pos, event, _index)) = hovered_event_data {
        render_event_tooltip(ui, pos, &event, image_cache);
    }

    EventInteraction {
        clicked_index: clicked_event_index,
        delete_index: delete_event_index,
    }
}

fn render_burning_star(
    painter: &egui::Painter,
    event_pos: Pos2,
    index: usize,
    time: f32,
    _event: &Event,
    is_hovered: bool,
) {
    let i = index as f32;

    // Multiple frequency oscillations for realistic flame-like behavior
    let slow_pulse = ((time * 0.6 + i).sin() + 1.0) / 2.0;
    let medium_pulse = ((time * 1.8 + i * 2.0).sin() + 1.0) / 2.0;
    let fast_flicker = ((time * 4.5 + i * 3.0).sin() + 1.0) / 2.0;
    let rapid_twinkle = ((time * 8.0 + i * 5.0).cos() + 1.0) / 2.0;

    let intensity =
        (slow_pulse * 0.4 + medium_pulse * 0.3 + fast_flicker * 0.2 + rapid_twinkle * 0.1)
            .clamp(0.4, 1.0);
    let size_variation = (slow_pulse * 0.5 + fast_flicker * 0.5).clamp(0.6, 1.3);

    // Expand star size when hovered
    let base_size = if is_hovered { 20.0 } else { 14.0 };
    let star_size = base_size * size_variation;

    // Draw star rays/flares
    render_star_rays(
        painter,
        event_pos,
        star_size,
        fast_flicker,
        intensity,
        time,
        i,
    );

    // Draw glow layers
    render_glow_layers(painter, event_pos, star_size, intensity);

    // Draw core
    render_star_core(painter, event_pos, star_size, intensity, rapid_twinkle);
}

fn render_star_rays(
    painter: &egui::Painter,
    event_pos: Pos2,
    star_size: f32,
    fast_flicker: f32,
    intensity: f32,
    time: f32,
    i: f32,
) {
    let ray_rotation = time * 0.3 + i;
    for ray_i in 0..4 {
        let angle = ray_rotation + (ray_i as f32 * std::f32::consts::PI / 2.0);
        let ray_length = star_size * (2.5 + fast_flicker * 1.5);

        for t in 0..8 {
            let t_norm = t as f32 / 8.0;
            let ray_pos = Pos2::new(
                event_pos.x + angle.cos() * ray_length * t_norm,
                event_pos.y + angle.sin() * ray_length * t_norm,
            );
            let ray_alpha = ((1.0 - t_norm) * intensity * 40.0) as u8;
            let ray_size = (1.0 - t_norm * 0.8) * 2.0;
            painter.circle_filled(
                ray_pos,
                ray_size,
                Color32::from_rgba_unmultiplied(255, 215, 0, ray_alpha),
            );
        }
    }
}

fn render_glow_layers(painter: &egui::Painter, event_pos: Pos2, star_size: f32, intensity: f32) {
    // Outer burning halo
    let halo_size = star_size * 4.0;
    let halo_alpha = (25.0 * intensity) as u8;
    painter.circle_filled(
        event_pos,
        halo_size,
        Color32::from_rgba_unmultiplied(255, 180, 0, halo_alpha),
    );

    // Far glow
    let far_glow_size = star_size * 3.0;
    let far_alpha = (40.0 * intensity) as u8;
    painter.circle_filled(
        event_pos,
        far_glow_size,
        Color32::from_rgba_unmultiplied(255, 200, 0, far_alpha),
    );

    // Outer glow
    let outer_glow_size = star_size * 2.2;
    let outer_alpha = (60.0 * intensity) as u8;
    painter.circle_filled(
        event_pos,
        outer_glow_size,
        Color32::from_rgba_unmultiplied(255, 215, 0, outer_alpha),
    );

    // Middle glow
    let mid_glow_size = star_size * 1.4;
    let mid_alpha = (120.0 * intensity) as u8;
    painter.circle_filled(
        event_pos,
        mid_glow_size,
        Color32::from_rgba_unmultiplied(255, 220, 50, mid_alpha),
    );

    // Inner bright core
    let inner_size = star_size * 0.8;
    let inner_brightness = (200.0 + 55.0 * intensity) as u8;
    painter.circle_filled(
        event_pos,
        inner_size,
        Color32::from_rgb(255, inner_brightness, 100),
    );
}

fn render_star_core(
    painter: &egui::Painter,
    event_pos: Pos2,
    star_size: f32,
    intensity: f32,
    rapid_twinkle: f32,
) {
    // White hot center
    let core_size = star_size * 0.5 * intensity;
    let core_alpha = (220.0 + 35.0 * intensity) as u8;
    painter.circle_filled(
        event_pos,
        core_size,
        Color32::from_rgba_unmultiplied(255, 255, 255, core_alpha),
    );

    // Extra bright center spark
    let spark_size = star_size * 0.2 * rapid_twinkle;
    painter.circle_filled(event_pos, spark_size, Color32::WHITE);
}

fn render_event_label(
    painter: &egui::Painter,
    x: f32,
    y: f32,
    event: &Event,
    time: f32,
    index: usize,
    is_hovered: bool,
) {
    let i = index as f32;
    let slow_pulse = ((time * 0.6 + i).sin() + 1.0) / 2.0;
    let medium_pulse = ((time * 1.8 + i * 2.0).sin() + 1.0) / 2.0;
    let fast_flicker = ((time * 4.5 + i * 3.0).sin() + 1.0) / 2.0;
    let rapid_twinkle = ((time * 8.0 + i * 5.0).cos() + 1.0) / 2.0;

    let intensity =
        (slow_pulse * 0.4 + medium_pulse * 0.3 + fast_flicker * 0.2 + rapid_twinkle * 0.1)
            .clamp(0.4, 1.0);

    // Position text above the event (which already has camera transform applied)
    let text_pos = Pos2::new(x, y - 30.0);

    // Text glow
    painter.text(
        text_pos,
        egui::Align2::CENTER_BOTTOM,
        &event.title,
        egui::FontId::proportional(14.0),
        Color32::from_rgba_unmultiplied(
            event.color.r(),
            event.color.g(),
            event.color.b(),
            (100.0 * intensity) as u8,
        ),
    );

    // Text main
    let font_size = if is_hovered { 16.0 } else { 14.0 };
    painter.text(
        text_pos,
        egui::Align2::CENTER_BOTTOM,
        &event.title,
        egui::FontId::proportional(font_size),
        Color32::WHITE,
    );
}

fn render_event_tooltip(
    ui: &mut egui::Ui,
    pointer_pos: Pos2,
    event: &Event,
    image_cache: &std::collections::HashMap<String, egui::TextureHandle>,
) {
    egui::Area::new(egui::Id::new("event_tooltip"))
        .fixed_pos(egui::Pos2::new(pointer_pos.x + 15.0, pointer_pos.y + 15.0))
        .interactable(false)
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style())
                .fill(Color32::from_rgba_unmultiplied(20, 20, 20, 240))
                .stroke(egui::Stroke::new(2.0, Color32::from_rgb(255, 215, 0)))
                .corner_radius(8.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.set_max_width(300.0);

                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(&event.title)
                                .size(18.0)
                                .color(Color32::from_rgb(255, 215, 0))
                                .strong(),
                        );

                        ui.add_space(4.0);

                        // Display the full date
                        let month_name = get_month_name(event.month);
                        let date_str = format!("{} {}, {}", month_name, event.day, event.year);
                        ui.label(
                            egui::RichText::new(date_str)
                                .size(13.0)
                                .color(Color32::from_rgb(180, 180, 180))
                                .italics(),
                        );

                        ui.add_space(8.0);

                        ui.label(
                            egui::RichText::new(&event.description)
                                .size(14.0)
                                .color(Color32::from_rgb(220, 220, 220)),
                        );

                        // Display image if available
                        if let Some(image_path) = &event.image_path {
                            ui.add_space(8.0);

                            if let Some(texture) = image_cache.get(image_path) {
                                // Display the actual image
                                let max_size = egui::Vec2::new(280.0, 200.0);
                                let img_size = texture.size_vec2();

                                // Calculate scaled size maintaining aspect ratio
                                let scale = (max_size.x / img_size.x)
                                    .min(max_size.y / img_size.y)
                                    .min(1.0);
                                let display_size = img_size * scale;

                                ui.image((texture.id(), display_size));
                            } else {
                                // Show path if image couldn't be loaded
                                ui.label(
                                    egui::RichText::new(format!("Image: {}", image_path))
                                        .size(12.0)
                                        .color(Color32::from_rgb(150, 150, 150)),
                                );
                            }
                        }

                        // Hint text
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("💡 Ctrl+Click to delete")
                                .color(Color32::from_gray(180))
                                .italics(),
                        );
                    });
                });
        });
}

fn get_month_name(month: u8) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}
