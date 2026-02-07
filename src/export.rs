use nightshade::prelude::*;

use crate::project::{AnimObject, Project, Shape};
use crate::tween;

pub fn export_png_sequence(project: &Project, folder: &std::path::Path) {
    for frame in 0..project.total_frames {
        let image = rasterize_frame(project, frame);
        let filename = format!("frame_{:04}.png", frame + 1);
        let path = folder.join(filename);
        let _ = image.save(path);
    }
}

pub fn export_sprite_sheet(project: &Project, path: &std::path::Path) {
    let columns = (project.total_frames as f64).sqrt().ceil() as u32;
    let rows = project.total_frames.div_ceil(columns);

    let sheet_width = columns * project.canvas_width;
    let sheet_height = rows * project.canvas_height;

    let mut sheet: image::RgbaImage = image::ImageBuffer::new(sheet_width, sheet_height);

    for frame in 0..project.total_frames {
        let frame_image = rasterize_frame(project, frame);
        let column = frame % columns;
        let row = frame / columns;
        let offset_x = column * project.canvas_width;
        let offset_y = row * project.canvas_height;

        for y in 0..project.canvas_height {
            for x in 0..project.canvas_width {
                let pixel = frame_image.get_pixel(x, y);
                sheet.put_pixel(offset_x + x, offset_y + y, *pixel);
            }
        }
    }

    let _ = sheet.save(path);
}

fn rasterize_frame(project: &Project, frame: u32) -> image::RgbaImage {
    let width = project.canvas_width;
    let height = project.canvas_height;

    let bg = project.background_color;
    let bg_pixel = image::Rgba([
        (bg[0] * 255.0) as u8,
        (bg[1] * 255.0) as u8,
        (bg[2] * 255.0) as u8,
        (bg[3] * 255.0) as u8,
    ]);

    let mut image_buffer: image::RgbaImage =
        image::ImageBuffer::from_pixel(width, height, bg_pixel);

    for layer_index in (0..project.layers.len()).rev() {
        let layer = &project.layers[layer_index];
        if !layer.visible {
            continue;
        }

        if let Some(objects) = tween::resolve_frame(layer, frame) {
            for object in &objects {
                rasterize_object(&mut image_buffer, object, layer.opacity);
            }
        }
    }

    image_buffer
}

fn rasterize_object(image_buffer: &mut image::RgbaImage, object: &AnimObject, layer_opacity: f32) {
    let (width, height) = image_buffer.dimensions();

    match &object.shape {
        Shape::Rectangle {
            width: rect_width,
            height: rect_height,
            ..
        } => {
            let half_w = rect_width * object.scale[0] / 2.0;
            let half_h = rect_height * object.scale[1] / 2.0;
            let diagonal = (half_w * half_w + half_h * half_h).sqrt();
            let extent = if object.rotation.abs() > 0.001 {
                diagonal
            } else {
                half_w.max(half_h)
            };
            let bound_w = if object.rotation.abs() > 0.001 {
                extent
            } else {
                half_w
            };
            let bound_h = if object.rotation.abs() > 0.001 {
                extent
            } else {
                half_h
            };
            let min_x = ((object.position[0] - bound_w).floor() as i32).max(0) as u32;
            let min_y = ((object.position[1] - bound_h).floor() as i32).max(0) as u32;
            let max_x = ((object.position[0] + bound_w).ceil() as u32).min(width - 1);
            let max_y = ((object.position[1] + bound_h).ceil() as u32).min(height - 1);

            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let local_x = x as f32 - object.position[0];
                    let local_y = y as f32 - object.position[1];
                    let cos_r = (-object.rotation).cos();
                    let sin_r = (-object.rotation).sin();
                    let unrotated_x = local_x * cos_r - local_y * sin_r;
                    let unrotated_y = local_x * sin_r + local_y * cos_r;

                    if unrotated_x.abs() <= half_w && unrotated_y.abs() <= half_h {
                        let is_border = unrotated_x.abs() > half_w - object.stroke_width
                            || unrotated_y.abs() > half_h - object.stroke_width;

                        let color = if is_border && object.stroke_width > 0.0 {
                            object.stroke
                        } else {
                            object.fill
                        };

                        blend_pixel(image_buffer, x, y, color, layer_opacity);
                    }
                }
            }
        }
        Shape::Ellipse { radius_x, radius_y } => {
            let scaled_rx = radius_x * object.scale[0];
            let scaled_ry = radius_y * object.scale[1];
            let max_radius = scaled_rx.max(scaled_ry);
            let bound = if object.rotation.abs() > 0.001 {
                max_radius
            } else {
                0.0
            };
            let bound_x = if object.rotation.abs() > 0.001 {
                bound
            } else {
                scaled_rx
            };
            let bound_y = if object.rotation.abs() > 0.001 {
                bound
            } else {
                scaled_ry
            };
            let min_x =
                ((object.position[0] - bound_x - object.stroke_width).floor() as i32).max(0) as u32;
            let min_y =
                ((object.position[1] - bound_y - object.stroke_width).floor() as i32).max(0) as u32;
            let max_x =
                ((object.position[0] + bound_x + object.stroke_width).ceil() as u32).min(width - 1);
            let max_y = ((object.position[1] + bound_y + object.stroke_width).ceil() as u32)
                .min(height - 1);

            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let local_x = x as f32 - object.position[0];
                    let local_y = y as f32 - object.position[1];
                    let cos_r = (-object.rotation).cos();
                    let sin_r = (-object.rotation).sin();
                    let unrotated_x = local_x * cos_r - local_y * sin_r;
                    let unrotated_y = local_x * sin_r + local_y * cos_r;

                    if scaled_rx > 0.001 && scaled_ry > 0.001 {
                        let dist =
                            (unrotated_x / scaled_rx).powi(2) + (unrotated_y / scaled_ry).powi(2);
                        if dist <= 1.0 {
                            let inner_rx = (scaled_rx - object.stroke_width).max(0.0);
                            let inner_ry = (scaled_ry - object.stroke_width).max(0.0);
                            let inner_dist = if inner_rx > 0.0 && inner_ry > 0.0 {
                                (unrotated_x / inner_rx).powi(2) + (unrotated_y / inner_ry).powi(2)
                            } else {
                                2.0
                            };

                            let color = if inner_dist > 1.0 && object.stroke_width > 0.0 {
                                object.stroke
                            } else {
                                object.fill
                            };

                            blend_pixel(image_buffer, x, y, color, layer_opacity);
                        }
                    }
                }
            }
        }
        Shape::Line { end_x, end_y } => {
            let start_x = object.position[0];
            let start_y = object.position[1];
            let line_end_x = start_x + end_x * object.scale[0];
            let line_end_y = start_y + end_y * object.scale[1];

            let thickness = object.stroke_width.max(1.0);
            let min_px = ((start_x.min(line_end_x) - thickness).floor() as i32).max(0) as u32;
            let min_py = ((start_y.min(line_end_y) - thickness).floor() as i32).max(0) as u32;
            let max_px = ((start_x.max(line_end_x) + thickness).ceil() as u32).min(width - 1);
            let max_py = ((start_y.max(line_end_y) + thickness).ceil() as u32).min(height - 1);

            let dx = line_end_x - start_x;
            let dy = line_end_y - start_y;
            let line_len_sq = dx * dx + dy * dy;

            for y in min_py..=max_py {
                for x in min_px..=max_px {
                    let px = x as f32 - start_x;
                    let py = y as f32 - start_y;

                    if line_len_sq < 0.001 {
                        continue;
                    }

                    let t = ((px * dx + py * dy) / line_len_sq).clamp(0.0, 1.0);
                    let closest_x = t * dx;
                    let closest_y = t * dy;
                    let dist = ((px - closest_x).powi(2) + (py - closest_y).powi(2)).sqrt();

                    if dist <= thickness / 2.0 {
                        blend_pixel(image_buffer, x, y, object.stroke, layer_opacity);
                    }
                }
            }
        }
        Shape::Path { .. } => {}
    }
}

fn blend_pixel(
    image_buffer: &mut image::RgbaImage,
    x: u32,
    y: u32,
    color: [f32; 4],
    layer_opacity: f32,
) {
    let alpha = color[3] * layer_opacity;
    if alpha < 0.001 {
        return;
    }

    let existing = image_buffer.get_pixel(x, y);
    let src_r = (color[0] * 255.0) as u8;
    let src_g = (color[1] * 255.0) as u8;
    let src_b = (color[2] * 255.0) as u8;
    let src_a = (alpha * 255.0) as u8;

    let alpha_f = alpha;
    let inv_alpha = 1.0 - alpha_f;

    let result = image::Rgba([
        (src_r as f32 * alpha_f + existing[0] as f32 * inv_alpha) as u8,
        (src_g as f32 * alpha_f + existing[1] as f32 * inv_alpha) as u8,
        (src_b as f32 * alpha_f + existing[2] as f32 * inv_alpha) as u8,
        (src_a as f32 + existing[3] as f32 * inv_alpha).min(255.0) as u8,
    ]);

    image_buffer.put_pixel(x, y, result);
}
