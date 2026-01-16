use eframe::egui;
use eframe::epaint::{Color32, Pos2};

pub struct Star {
    pub pos_normalized: (f32, f32),
    pub size: f32,
    pub base_brightness: u8,
    pub twinkle_speed: f32,
    pub twinkle_offset: f32,
    pub depth_layer: f32, // 0.0 = far back, 1.0 = close
}

pub struct Galaxy {
    pub pos_normalized: (f32, f32),
    pub size: f32,
    pub rotation: f32,
    pub color_hue: u8, // 0=blue, 1=purple, 2=orange
    pub depth_layer: f32,
}

pub struct Nebula {
    pub pos_normalized: (f32, f32),
    pub size: f32,
    pub color_hue: u8, // 0=red, 1=blue, 2=purple, 3=green
    pub depth_layer: f32,
    pub opacity: u8,
}

pub fn generate_stars(count: usize) -> Vec<Star> {
    let mut stars = Vec::new();

    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hash, Hasher};

    let random_state = RandomState::new();

    for i in 0..count {
        let mut hasher = random_state.build_hasher();
        i.hash(&mut hasher);
        let hash1 = hasher.finish();

        let mut hasher = random_state.build_hasher();
        (i + 1000).hash(&mut hasher);
        let hash2 = hasher.finish();

        let mut hasher = random_state.build_hasher();
        (i + 2000).hash(&mut hasher);
        let hash3 = hasher.finish();

        let mut hasher = random_state.build_hasher();
        (i + 3000).hash(&mut hasher);
        let hash4 = hasher.finish();

        let mut hasher = random_state.build_hasher();
        (i + 4000).hash(&mut hasher);
        let hash5 = hasher.finish();

        let x = (hash1 % 10000) as f32 / 10000.0;
        let y = (hash2 % 10000) as f32 / 10000.0;

        // Create four distinct depth layers
        // Distribution: 30% furthest, 30% far, 25% near, 15% closest
        let layer_roll = hash5 % 100;
        let depth_layer = if layer_roll < 30 {
            0.2 // Furthest layer - minimal parallax
        } else if layer_roll < 60 {
            0.5 // Far layer - slight parallax
        } else if layer_roll < 85 {
            0.8 // Near layer - noticeable parallax
        } else {
            1.0 // Closest layer - maximum parallax
        };

        // Size increases with depth (closer = larger)
        let size = if depth_layer >= 0.95 {
            ((hash3 % 3) + 3) as f32 // Closest: 3-5px
        } else if depth_layer >= 0.7 {
            ((hash3 % 2) + 2) as f32 // Near: 2-3px
        } else if depth_layer >= 0.4 {
            ((hash3 % 2) + 1) as f32 // Far: 1-2px
        } else {
            1.0 // Furthest: 1px
        };

        // Brightness increases with depth (closer = brighter)
        let base_brightness = if depth_layer >= 0.95 {
            ((hash1 % 60) + 195) as u8 // Closest: 195-255 (very bright)
        } else if depth_layer >= 0.7 {
            ((hash1 % 70) + 160) as u8 // Near: 160-230 (bright)
        } else if depth_layer >= 0.4 {
            ((hash1 % 70) + 110) as u8 // Far: 110-180 (dim)
        } else {
            ((hash1 % 60) + 80) as u8 // Furthest: 80-140 (very dim)
        };

        let twinkle_speed = ((hash4 % 200) + 100) as f32 / 100.0;
        let twinkle_offset = (hash2 % 628) as f32 / 100.0;

        stars.push(Star {
            pos_normalized: (x, y),
            size,
            base_brightness,
            twinkle_speed,
            twinkle_offset,
            depth_layer,
        });
    }

    stars
}

pub fn generate_galaxies(count: usize) -> Vec<Galaxy> {
    let mut galaxies = Vec::new();

    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hash, Hasher};

    let random_state = RandomState::new();

    for i in 0..count {
        let mut hasher = random_state.build_hasher();
        (i + 5000).hash(&mut hasher);
        let hash1 = hasher.finish();

        let mut hasher = random_state.build_hasher();
        (i + 6000).hash(&mut hasher);
        let hash2 = hasher.finish();

        let mut hasher = random_state.build_hasher();
        (i + 7000).hash(&mut hasher);
        let hash3 = hasher.finish();

        let x = (hash1 % 10000) as f32 / 10000.0;
        let y = (hash2 % 10000) as f32 / 10000.0;

        // Spread galaxies across the 3 furthest depth layers
        let layer_roll = hash3 % 3;
        let depth_layer = match layer_roll {
            0 => 0.2,  // Furthest layer
            1 => 0.35, // Mid-far layer
            _ => 0.5,  // Far layer
        };

        // Size varies with depth - closer galaxies are larger
        let size = if depth_layer >= 0.45 {
            ((hash3 % 60) + 50) as f32 // Far: 50-110px
        } else if depth_layer >= 0.3 {
            ((hash3 % 50) + 40) as f32 // Mid-far: 40-90px
        } else {
            ((hash3 % 40) + 30) as f32 // Furthest: 30-70px
        };

        let rotation = ((hash1 % 628) as f32) / 100.0;
        let color_hue = (hash2 % 3) as u8;

        galaxies.push(Galaxy {
            pos_normalized: (x, y),
            size,
            rotation,
            color_hue,
            depth_layer,
        });
    }

    galaxies
}

pub fn generate_nebulas(count: usize) -> Vec<Nebula> {
    let mut nebulas = Vec::new();

    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hash, Hasher};

    let random_state = RandomState::new();

    for i in 0..count {
        let mut hasher = random_state.build_hasher();
        (i + 8000).hash(&mut hasher);
        let hash1 = hasher.finish();

        let mut hasher = random_state.build_hasher();
        (i + 9000).hash(&mut hasher);
        let hash2 = hasher.finish();

        let mut hasher = random_state.build_hasher();
        (i + 10000).hash(&mut hasher);
        let hash3 = hasher.finish();

        let x = (hash1 % 10000) as f32 / 10000.0;
        let y = (hash2 % 10000) as f32 / 10000.0;

        // Spread nebulas across the 3 furthest depth layers
        let layer_roll = hash3 % 3;
        let depth_layer = match layer_roll {
            0 => 0.2,  // Furthest layer
            1 => 0.35, // Mid-far layer
            _ => 0.5,  // Far layer
        };

        // Size varies with depth - closer nebulas are larger
        let size = if depth_layer >= 0.45 {
            ((hash3 % 100) + 100) as f32 // Far: 100-200px
        } else if depth_layer >= 0.3 {
            ((hash3 % 80) + 80) as f32 // Mid-far: 80-160px
        } else {
            ((hash3 % 70) + 60) as f32 // Furthest: 60-130px
        };

        let color_hue = (hash2 % 4) as u8;
        let opacity = ((hash1 % 40) + 30) as u8; // 30-70

        nebulas.push(Nebula {
            pos_normalized: (x, y),
            size,
            color_hue,
            depth_layer,
            opacity,
        });
    }

    nebulas
}

pub fn render_stars(
    stars: &[Star],
    painter: &egui::Painter,
    rect: egui::Rect,
    time: f32,
    camera_offset_x: f32,
    camera_offset_y: f32,
    camera_zoom: f32,
) {
    let center_x = rect.center().x;
    let center_y = rect.center().y;
    let screen_width = rect.width();
    let screen_height = rect.height();

    for star in stars {
        // Calculate parallax effect based on depth layer
        let parallax_strength = star.depth_layer;

        // Apply parallax: closer stars move more with camera movement
        let parallax_x = camera_offset_x * parallax_strength * 0.3;
        let parallax_y = camera_offset_y * parallax_strength * 0.3;

        // Zoom parallax: closer stars appear to zoom more
        let zoom_factor = 1.0 + (camera_zoom - 1.0) * parallax_strength * 0.5;

        // Calculate base position with parallax
        let base_x = rect.left() + star.pos_normalized.0 * screen_width;
        let base_y = rect.top() + star.pos_normalized.1 * screen_height;

        // Apply parallax offset and zoom from center
        let x = center_x + (base_x - center_x) * zoom_factor + parallax_x;
        let y = center_y + (base_y - center_y) * zoom_factor + parallax_y;

        // Infinite wrapping: render star in a 3x3 tiled grid for seamless infinite field
        for x_tile in -1..=1 {
            for y_tile in -1..=1 {
                let tiled_x = x + (x_tile as f32 * screen_width);
                let tiled_y = y + (y_tile as f32 * screen_height);

                // Only render if within or near the visible area (with margin for star size)
                if tiled_x >= rect.left() - 50.0
                    && tiled_x <= rect.right() + 50.0
                    && tiled_y >= rect.top() - 50.0
                    && tiled_y <= rect.bottom() + 50.0
                {
                    let pos = Pos2::new(tiled_x, tiled_y);

                    // Twinkle effect
                    let twinkle =
                        ((time * star.twinkle_speed + star.twinkle_offset).sin() + 1.0) / 2.0;
                    let brightness_variation = (twinkle * 100.0) as u8;
                    let current_brightness =
                        star.base_brightness.saturating_sub(brightness_variation);

                    painter.circle_filled(
                        pos,
                        star.size * zoom_factor,
                        Color32::from_rgb(
                            current_brightness,
                            current_brightness,
                            current_brightness,
                        ),
                    );
                }
            }
        }
    }
}

pub fn render_galaxies(
    galaxies: &[Galaxy],
    painter: &egui::Painter,
    rect: egui::Rect,
    time: f32,
    camera_offset_x: f32,
    camera_offset_y: f32,
    camera_zoom: f32,
) {
    let center_x = rect.center().x;
    let center_y = rect.center().y;
    let screen_width = rect.width();
    let screen_height = rect.height();

    for galaxy in galaxies {
        let parallax_strength = galaxy.depth_layer;
        let parallax_x = camera_offset_x * parallax_strength * 0.3;
        let parallax_y = camera_offset_y * parallax_strength * 0.3;
        let zoom_factor = 1.0 + (camera_zoom - 1.0) * parallax_strength * 0.5;

        let base_x = rect.left() + galaxy.pos_normalized.0 * screen_width;
        let base_y = rect.top() + galaxy.pos_normalized.1 * screen_height;
        let x = center_x + (base_x - center_x) * zoom_factor + parallax_x;
        let y = center_y + (base_y - center_y) * zoom_factor + parallax_y;

        // Infinite wrapping
        for x_tile in -1..=1 {
            for y_tile in -1..=1 {
                let tiled_x = x + (x_tile as f32 * screen_width);
                let tiled_y = y + (y_tile as f32 * screen_height);

                if tiled_x >= rect.left() - 200.0
                    && tiled_x <= rect.right() + 200.0
                    && tiled_y >= rect.top() - 200.0
                    && tiled_y <= rect.bottom() + 200.0
                {
                    let pos = Pos2::new(tiled_x, tiled_y);
                    let size = galaxy.size * zoom_factor;

                    // Choose color based on hue
                    let color = match galaxy.color_hue {
                        0 => Color32::from_rgba_unmultiplied(100, 120, 200, 40), // Blue
                        1 => Color32::from_rgba_unmultiplied(150, 100, 200, 40), // Purple
                        _ => Color32::from_rgba_unmultiplied(200, 140, 100, 40), // Orange
                    };

                    // Slow rotation
                    let rotation = galaxy.rotation + time * 0.05;

                    // Draw spiral galaxy effect with multiple layers
                    for layer in 0..5 {
                        let layer_f = layer as f32;
                        let layer_size = size * (1.0 - layer_f * 0.15);
                        let layer_alpha = (40.0 * (1.0 - layer_f * 0.2)) as u8;

                        painter.circle_filled(
                            pos,
                            layer_size,
                            Color32::from_rgba_unmultiplied(
                                color.r(),
                                color.g(),
                                color.b(),
                                layer_alpha,
                            ),
                        );
                    }

                    // Bright core
                    painter.circle_filled(
                        pos,
                        size * 0.2,
                        Color32::from_rgba_unmultiplied(255, 255, 220, 80),
                    );
                }
            }
        }
    }
}

pub fn render_nebulas(
    nebulas: &[Nebula],
    painter: &egui::Painter,
    rect: egui::Rect,
    time: f32,
    camera_offset_x: f32,
    camera_offset_y: f32,
    camera_zoom: f32,
) {
    let center_x = rect.center().x;
    let center_y = rect.center().y;
    let screen_width = rect.width();
    let screen_height = rect.height();

    for nebula in nebulas {
        let parallax_strength = nebula.depth_layer;
        let parallax_x = camera_offset_x * parallax_strength * 0.3;
        let parallax_y = camera_offset_y * parallax_strength * 0.3;
        let zoom_factor = 1.0 + (camera_zoom - 1.0) * parallax_strength * 0.5;

        let base_x = rect.left() + nebula.pos_normalized.0 * screen_width;
        let base_y = rect.top() + nebula.pos_normalized.1 * screen_height;
        let x = center_x + (base_x - center_x) * zoom_factor + parallax_x;
        let y = center_y + (base_y - center_y) * zoom_factor + parallax_y;

        // Infinite wrapping
        for x_tile in -1..=1 {
            for y_tile in -1..=1 {
                let tiled_x = x + (x_tile as f32 * screen_width);
                let tiled_y = y + (y_tile as f32 * screen_height);

                if tiled_x >= rect.left() - 300.0
                    && tiled_x <= rect.right() + 300.0
                    && tiled_y >= rect.top() - 300.0
                    && tiled_y <= rect.bottom() + 300.0
                {
                    let pos = Pos2::new(tiled_x, tiled_y);
                    let size = nebula.size * zoom_factor;

                    // Subtle pulsing effect
                    let pulse = ((time * 0.3).sin() + 1.0) / 2.0;
                    let opacity = (nebula.opacity as f32 * (0.7 + pulse * 0.3)) as u8;

                    // Choose color based on hue
                    let color = match nebula.color_hue {
                        0 => Color32::from_rgba_unmultiplied(200, 80, 100, opacity), // Red
                        1 => Color32::from_rgba_unmultiplied(80, 120, 220, opacity), // Blue
                        2 => Color32::from_rgba_unmultiplied(180, 100, 220, opacity), // Purple
                        _ => Color32::from_rgba_unmultiplied(100, 200, 150, opacity), // Green
                    };

                    // Draw cloudy nebula effect with multiple soft layers
                    for layer in 0..7 {
                        let layer_f = layer as f32;
                        let offset_x = (time * 0.1 + layer_f).cos() * size * 0.1;
                        let offset_y = (time * 0.15 + layer_f).sin() * size * 0.1;
                        let layer_pos = Pos2::new(tiled_x + offset_x, tiled_y + offset_y);
                        let layer_size = size * (0.8 + layer_f * 0.1);
                        let layer_opacity = (opacity as f32 * (1.0 - layer_f * 0.12)) as u8;

                        painter.circle_filled(
                            layer_pos,
                            layer_size,
                            Color32::from_rgba_unmultiplied(
                                color.r(),
                                color.g(),
                                color.b(),
                                layer_opacity,
                            ),
                        );
                    }
                }
            }
        }
    }
}
