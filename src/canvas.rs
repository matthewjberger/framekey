use nightshade::prelude::*;

use crate::app::AnimateApp;
use crate::onion;
use crate::project::{AnimObject, Shape};
use crate::selection;
use crate::tools;
use crate::tween;

#[derive(Clone)]
pub struct CanvasView {
    pub pan: egui::Vec2,
    pub zoom: f32,
    pub panel_rect: egui::Rect,
}

impl Default for CanvasView {
    fn default() -> Self {
        Self {
            pan: egui::Vec2::ZERO,
            zoom: 0.5,
            panel_rect: egui::Rect::NOTHING,
        }
    }
}

impl CanvasView {
    pub fn canvas_to_screen(&self, canvas_pos: egui::Pos2) -> egui::Pos2 {
        let center = self.panel_rect.center();
        egui::pos2(
            center.x + (canvas_pos.x + self.pan.x) * self.zoom,
            center.y + (canvas_pos.y + self.pan.y) * self.zoom,
        )
    }

    pub fn screen_to_canvas(&self, screen_pos: egui::Pos2) -> egui::Pos2 {
        let center = self.panel_rect.center();
        egui::pos2(
            (screen_pos.x - center.x) / self.zoom - self.pan.x,
            (screen_pos.y - center.y) / self.zoom - self.pan.y,
        )
    }

    pub fn canvas_size_to_screen(&self, size: f32) -> f32 {
        size * self.zoom
    }
}

pub fn draw_canvas(app: &mut AnimateApp, ui_context: &egui::Context) {
    egui::CentralPanel::default().show(ui_context, |ui| {
        let panel_rect = ui.available_rect_before_wrap();
        app.canvas_view.panel_rect = panel_rect;

        let (response, painter) = ui.allocate_painter(
            ui.available_size_before_wrap(),
            egui::Sense::click_and_drag(),
        );

        let clipped_painter = painter.with_clip_rect(panel_rect);

        draw_canvas_background(app, &clipped_painter);

        onion::draw_onion_skins(app, &app.canvas_view.clone(), &clipped_painter);

        draw_frame_objects(app, &clipped_painter);

        selection::draw_selection_indicators(app, &app.canvas_view.clone(), &clipped_painter);

        tools::draw_tool_preview(app, &app.canvas_view.clone(), &clipped_painter);

        handle_pan_zoom(app, &response, ui_context);

        match app.tool {
            crate::tools::Tool::Select => {
                selection::handle_select_tool(app, &response, ui_context);
            }
            _ => {
                tools::handle_drawing_tool(app, &response, ui_context);
            }
        }

        ui.ctx().request_repaint();
    });
}

fn handle_pan_zoom(app: &mut AnimateApp, response: &egui::Response, ui_context: &egui::Context) {
    if response.dragged_by(egui::PointerButton::Middle) {
        let delta = response.drag_delta();
        app.canvas_view.pan.x += delta.x / app.canvas_view.zoom;
        app.canvas_view.pan.y += delta.y / app.canvas_view.zoom;
    }

    let scroll_delta = ui_context.input(|input| input.smooth_scroll_delta.y);
    if scroll_delta != 0.0 && response.hovered() {
        let zoom_factor = 1.0 + scroll_delta * 0.001;
        let old_zoom = app.canvas_view.zoom;
        app.canvas_view.zoom = (app.canvas_view.zoom * zoom_factor).clamp(0.05, 10.0);

        if let Some(pointer_pos) = ui_context.input(|input| input.pointer.hover_pos()) {
            let center = app.canvas_view.panel_rect.center();
            let pointer_offset = pointer_pos - center;
            let adjust = pointer_offset * (1.0 / old_zoom - 1.0 / app.canvas_view.zoom);
            app.canvas_view.pan += adjust;
        }
    }
}

fn draw_canvas_background(app: &AnimateApp, painter: &egui::Painter) {
    let panel_bg = egui::Color32::from_rgb(50, 50, 50);
    painter.rect_filled(app.canvas_view.panel_rect, 0.0, panel_bg);

    let top_left = app.canvas_view.canvas_to_screen(egui::pos2(0.0, 0.0));
    let bottom_right = app.canvas_view.canvas_to_screen(egui::pos2(
        app.project.canvas_width as f32,
        app.project.canvas_height as f32,
    ));
    let canvas_rect = egui::Rect::from_two_pos(top_left, bottom_right);

    let bg = app.project.background_color;
    let bg_color = egui::Color32::from_rgba_unmultiplied(
        (bg[0] * 255.0) as u8,
        (bg[1] * 255.0) as u8,
        (bg[2] * 255.0) as u8,
        (bg[3] * 255.0) as u8,
    );

    painter.rect(
        canvas_rect,
        0.0,
        bg_color,
        egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)),
        egui::StrokeKind::Outside,
    );
}

fn draw_frame_objects(app: &AnimateApp, painter: &egui::Painter) {
    for layer_index in (0..app.project.layers.len()).rev() {
        let layer = &app.project.layers[layer_index];
        if !layer.visible {
            continue;
        }

        if let Some(objects) = tween::resolve_frame(layer, app.current_frame) {
            for object in &objects {
                render_object(object, &app.canvas_view, painter, layer.opacity);
            }
        }
    }
}

pub fn render_object(
    object: &AnimObject,
    view: &CanvasView,
    painter: &egui::Painter,
    layer_opacity: f32,
) {
    let fill = color_with_opacity(object.fill, layer_opacity);
    let stroke_color = color_with_opacity(object.stroke, layer_opacity);
    let stroke = egui::Stroke::new(object.stroke_width * view.zoom, stroke_color);

    let pos = egui::pos2(object.position[0], object.position[1]);
    let screen_pos = view.canvas_to_screen(pos);

    match &object.shape {
        Shape::Rectangle {
            width,
            height,
            corner_radius,
        } => {
            let half_w = width * object.scale[0] / 2.0;
            let half_h = height * object.scale[1] / 2.0;

            if object.rotation.abs() < 0.001 {
                let screen_min = view.canvas_to_screen(egui::pos2(
                    object.position[0] - half_w,
                    object.position[1] - half_h,
                ));
                let screen_max = view.canvas_to_screen(egui::pos2(
                    object.position[0] + half_w,
                    object.position[1] + half_h,
                ));
                let rect = egui::Rect::from_two_pos(screen_min, screen_max);
                let screen_radius = corner_radius * view.zoom;
                painter.rect(rect, screen_radius, fill, stroke, egui::StrokeKind::Outside);
            } else {
                let corners = [
                    [-half_w, -half_h],
                    [half_w, -half_h],
                    [half_w, half_h],
                    [-half_w, half_h],
                ];
                let rotated: Vec<egui::Pos2> = corners
                    .iter()
                    .map(|[corner_x, corner_y]| {
                        let rotated_x =
                            corner_x * object.rotation.cos() - corner_y * object.rotation.sin();
                        let rotated_y =
                            corner_x * object.rotation.sin() + corner_y * object.rotation.cos();
                        view.canvas_to_screen(egui::pos2(
                            object.position[0] + rotated_x,
                            object.position[1] + rotated_y,
                        ))
                    })
                    .collect();

                let shape = egui::epaint::PathShape::convex_polygon(rotated, fill, stroke);
                painter.add(shape);
            }
        }
        Shape::Ellipse { radius_x, radius_y } => {
            let scaled_rx = radius_x * object.scale[0];
            let scaled_ry = radius_y * object.scale[1];
            let screen_rx = view.canvas_size_to_screen(scaled_rx);
            let screen_ry = view.canvas_size_to_screen(scaled_ry);

            if (screen_rx - screen_ry).abs() < 0.5 && object.rotation.abs() < 0.001 {
                painter.circle(screen_pos, screen_rx, fill, stroke);
            } else {
                let segments = 64;
                let points: Vec<egui::Pos2> = (0..segments)
                    .map(|segment_index| {
                        let angle =
                            2.0 * std::f32::consts::PI * segment_index as f32 / segments as f32;
                        let ellipse_x = angle.cos() * scaled_rx;
                        let ellipse_y = angle.sin() * scaled_ry;
                        let rotated_x =
                            ellipse_x * object.rotation.cos() - ellipse_y * object.rotation.sin();
                        let rotated_y =
                            ellipse_x * object.rotation.sin() + ellipse_y * object.rotation.cos();
                        view.canvas_to_screen(egui::pos2(
                            object.position[0] + rotated_x,
                            object.position[1] + rotated_y,
                        ))
                    })
                    .collect();

                let shape = egui::epaint::PathShape::convex_polygon(points, fill, stroke);
                painter.add(shape);
            }
        }
        Shape::Line { end_x, end_y } => {
            let end_canvas = egui::pos2(
                object.position[0] + end_x * object.scale[0],
                object.position[1] + end_y * object.scale[1],
            );
            let screen_end = view.canvas_to_screen(end_canvas);
            painter.line_segment([screen_pos, screen_end], stroke);
        }
        Shape::Path { points, closed } => {
            if points.len() < 2 {
                return;
            }

            let mut screen_points = Vec::new();
            for path_point_index in 0..points.len() {
                let point = &points[path_point_index];
                let canvas_pt = egui::pos2(
                    object.position[0] + point.position[0] * object.scale[0],
                    object.position[1] + point.position[1] * object.scale[1],
                );

                if path_point_index > 0 {
                    let prev = &points[path_point_index - 1];
                    if prev.control_out.is_some() || point.control_in.is_some() {
                        let control_out = prev.control_out.unwrap_or(prev.position);
                        let control_in = point.control_in.unwrap_or(point.position);
                        for step in 1..=16 {
                            let t = step as f32 / 16.0;
                            let bezier = cubic_bezier(
                                prev.position,
                                control_out,
                                control_in,
                                point.position,
                                t,
                            );
                            let canvas_bezier = egui::pos2(
                                object.position[0] + bezier[0] * object.scale[0],
                                object.position[1] + bezier[1] * object.scale[1],
                            );
                            screen_points.push(view.canvas_to_screen(canvas_bezier));
                        }
                        continue;
                    }
                }
                screen_points.push(view.canvas_to_screen(canvas_pt));
            }

            if *closed && points.len() > 2 {
                let last = points.last().unwrap();
                let first = &points[0];
                if last.control_out.is_some() || first.control_in.is_some() {
                    let control_out = last.control_out.unwrap_or(last.position);
                    let control_in = first.control_in.unwrap_or(first.position);
                    for step in 1..=16 {
                        let t = step as f32 / 16.0;
                        let bezier =
                            cubic_bezier(last.position, control_out, control_in, first.position, t);
                        let canvas_bezier = egui::pos2(
                            object.position[0] + bezier[0] * object.scale[0],
                            object.position[1] + bezier[1] * object.scale[1],
                        );
                        screen_points.push(view.canvas_to_screen(canvas_bezier));
                    }
                }

                let shape = egui::epaint::PathShape::convex_polygon(screen_points, fill, stroke);
                painter.add(shape);
            } else {
                let path_shape = egui::epaint::PathShape::line(screen_points, stroke);
                painter.add(path_shape);
            }
        }
    }
}

fn color_with_opacity(color: [f32; 4], opacity: f32) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        (color[0] * 255.0) as u8,
        (color[1] * 255.0) as u8,
        (color[2] * 255.0) as u8,
        (color[3] * opacity * 255.0) as u8,
    )
}

fn cubic_bezier(p0: [f32; 2], p1: [f32; 2], p2: [f32; 2], p3: [f32; 2], t: f32) -> [f32; 2] {
    let one_minus_t = 1.0 - t;
    let one_minus_t_sq = one_minus_t * one_minus_t;
    let one_minus_t_cu = one_minus_t_sq * one_minus_t;
    let t_sq = t * t;
    let t_cu = t_sq * t;

    [
        one_minus_t_cu * p0[0]
            + 3.0 * one_minus_t_sq * t * p1[0]
            + 3.0 * one_minus_t * t_sq * p2[0]
            + t_cu * p3[0],
        one_minus_t_cu * p0[1]
            + 3.0 * one_minus_t_sq * t * p1[1]
            + 3.0 * one_minus_t * t_sq * p2[1]
            + t_cu * p3[1],
    ]
}
