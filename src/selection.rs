use nightshade::prelude::*;

use crate::app::AnimateApp;
use crate::canvas::CanvasView;
use crate::project::{AnimObject, Shape};
use crate::tween;

#[derive(Clone, Default)]
pub struct Selection {
    pub selected_objects: Vec<uuid::Uuid>,
    pub drag_start: Option<egui::Pos2>,
    pub drag_offset: Option<egui::Vec2>,
}

pub fn handle_select_tool(
    app: &mut AnimateApp,
    response: &egui::Response,
    ui_context: &egui::Context,
) {
    if response.clicked_by(egui::PointerButton::Primary)
        && let Some(pos) = response.interact_pointer_pos()
    {
        let canvas_pos = app.canvas_view.screen_to_canvas(pos);
        if let Some(object_id) = hit_test(app, canvas_pos) {
            let ctrl = ui_context.input(|input| input.modifiers.ctrl);
            if ctrl {
                if app.selection.selected_objects.contains(&object_id) {
                    app.selection.selected_objects.retain(|id| *id != object_id);
                } else {
                    app.selection.selected_objects.push(object_id);
                }
            } else {
                app.selection.selected_objects = vec![object_id];
            }
        } else {
            app.selection.selected_objects.clear();
        }
    }

    if response.drag_started_by(egui::PointerButton::Primary)
        && let Some(pos) = response.interact_pointer_pos()
    {
        let canvas_pos = app.canvas_view.screen_to_canvas(pos);
        if let Some(object_id) = hit_test(app, canvas_pos) {
            if !app.selection.selected_objects.contains(&object_id) {
                app.selection.selected_objects = vec![object_id];
            }
            app.selection.drag_start = Some(canvas_pos);
            app.selection.drag_offset = Some(egui::Vec2::ZERO);

            app.history.push(app.project.clone());
        }
    }

    if response.dragged_by(egui::PointerButton::Primary)
        && app.selection.drag_start.is_some()
        && let Some(start) = app.selection.drag_start
        && let Some(pos) = ui_context.input(|input| input.pointer.latest_pos())
    {
        let canvas_pos = app.canvas_view.screen_to_canvas(pos);
        let delta = canvas_pos - start;
        let prev_offset = app.selection.drag_offset.unwrap_or(egui::Vec2::ZERO);
        let movement = delta - prev_offset;

        move_selected_objects(app, movement);
        app.selection.drag_offset = Some(delta);
    }

    if response.drag_stopped() {
        app.selection.drag_start = None;
        app.selection.drag_offset = None;
    }
}

fn hit_test(app: &AnimateApp, canvas_pos: egui::Pos2) -> Option<uuid::Uuid> {
    for layer_index in 0..app.project.layers.len() {
        let layer = &app.project.layers[layer_index];
        if !layer.visible || layer.locked {
            continue;
        }

        if let Some(objects) = tween::resolve_frame(layer, app.current_frame) {
            for object in objects.iter().rev() {
                if point_in_object(canvas_pos, object) {
                    return Some(object.id);
                }
            }
        }
    }
    None
}

fn point_in_object(point: egui::Pos2, object: &AnimObject) -> bool {
    let local_x = point.x - object.position[0];
    let local_y = point.y - object.position[1];

    let cos_r = (-object.rotation).cos();
    let sin_r = (-object.rotation).sin();
    let unrotated_x = local_x * cos_r - local_y * sin_r;
    let unrotated_y = local_x * sin_r + local_y * cos_r;

    match &object.shape {
        Shape::Rectangle { width, height, .. } => {
            let half_w = width * object.scale[0] / 2.0;
            let half_h = height * object.scale[1] / 2.0;
            unrotated_x.abs() <= half_w && unrotated_y.abs() <= half_h
        }
        Shape::Ellipse { radius_x, radius_y } => {
            let scaled_rx = radius_x * object.scale[0];
            let scaled_ry = radius_y * object.scale[1];
            if scaled_rx < 0.001 || scaled_ry < 0.001 {
                return false;
            }
            (unrotated_x / scaled_rx).powi(2) + (unrotated_y / scaled_ry).powi(2) <= 1.0
        }
        Shape::Line { end_x, end_y } => {
            let line_end_x = end_x * object.scale[0];
            let line_end_y = end_y * object.scale[1];
            let line_len_sq = line_end_x * line_end_x + line_end_y * line_end_y;
            if line_len_sq < 0.001 {
                return false;
            }
            let t = (unrotated_x * line_end_x + unrotated_y * line_end_y) / line_len_sq;
            let t_clamped = t.clamp(0.0, 1.0);
            let closest_x = t_clamped * line_end_x;
            let closest_y = t_clamped * line_end_y;
            let dist_sq = (unrotated_x - closest_x).powi(2) + (unrotated_y - closest_y).powi(2);
            dist_sq <= 25.0
        }
        Shape::Path { points, .. } => {
            if points.is_empty() {
                return false;
            }
            let threshold = 8.0;
            for point_index in 0..points.len() {
                let point = &points[point_index];
                let px = point.position[0] * object.scale[0];
                let py = point.position[1] * object.scale[1];
                if (unrotated_x - px).abs() < threshold && (unrotated_y - py).abs() < threshold {
                    return true;
                }
                if point_index > 0 {
                    let prev = &points[point_index - 1];
                    let prev_x = prev.position[0] * object.scale[0];
                    let prev_y = prev.position[1] * object.scale[1];
                    let segments = 8;
                    for step in 0..=segments {
                        let t = step as f32 / segments as f32;
                        let seg_x = prev_x + (px - prev_x) * t;
                        let seg_y = prev_y + (py - prev_y) * t;
                        let dist_sq = (unrotated_x - seg_x).powi(2) + (unrotated_y - seg_y).powi(2);
                        if dist_sq <= threshold * threshold {
                            return true;
                        }
                    }
                }
            }
            false
        }
    }
}

fn move_selected_objects(app: &mut AnimateApp, delta: egui::Vec2) {
    if app.active_layer >= app.project.layers.len() {
        return;
    }

    let selected = app.selection.selected_objects.clone();

    for layer in &mut app.project.layers {
        let has_selected = tween::resolve_frame(layer, app.current_frame)
            .map(|objects| objects.iter().any(|object| selected.contains(&object.id)))
            .unwrap_or(false);

        if has_selected {
            tween::ensure_keyframe_at(layer, app.current_frame);
        }

        if let Some(keyframe) = layer.keyframes.get_mut(&app.current_frame) {
            for object in &mut keyframe.objects {
                if selected.contains(&object.id) {
                    object.position[0] += delta.x;
                    object.position[1] += delta.y;
                }
            }
        }
    }
}

pub fn draw_selection_indicators(app: &AnimateApp, view: &CanvasView, painter: &egui::Painter) {
    if app.selection.selected_objects.is_empty() {
        return;
    }

    for layer in &app.project.layers {
        if !layer.visible {
            continue;
        }
        if let Some(objects) = tween::resolve_frame(layer, app.current_frame) {
            for object in &objects {
                if app.selection.selected_objects.contains(&object.id) {
                    draw_bounding_box(object, view, painter);
                }
            }
        }
    }
}

fn draw_bounding_box(object: &AnimObject, view: &CanvasView, painter: &egui::Painter) {
    let (half_w, half_h, center_offset) = get_object_bounds(object);

    let corners = [
        [center_offset[0] - half_w, center_offset[1] - half_h],
        [center_offset[0] + half_w, center_offset[1] - half_h],
        [center_offset[0] + half_w, center_offset[1] + half_h],
        [center_offset[0] - half_w, center_offset[1] + half_h],
    ];

    let screen_corners: Vec<egui::Pos2> = corners
        .iter()
        .map(|[corner_x, corner_y]| {
            let rotated_x = corner_x * object.rotation.cos() - corner_y * object.rotation.sin();
            let rotated_y = corner_x * object.rotation.sin() + corner_y * object.rotation.cos();
            view.canvas_to_screen(egui::pos2(
                object.position[0] + rotated_x,
                object.position[1] + rotated_y,
            ))
        })
        .collect();

    let selection_stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 150, 255));
    for corner_index in 0..4 {
        painter.line_segment(
            [
                screen_corners[corner_index],
                screen_corners[(corner_index + 1) % 4],
            ],
            selection_stroke,
        );
    }

    let handle_size = 4.0;
    for corner in &screen_corners {
        let handle_rect =
            egui::Rect::from_center_size(*corner, egui::vec2(handle_size * 2.0, handle_size * 2.0));
        painter.rect_filled(handle_rect, 0.0, egui::Color32::WHITE);
        painter.rect(
            handle_rect,
            0.0,
            egui::Color32::TRANSPARENT,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 150, 255)),
            egui::StrokeKind::Outside,
        );
    }
}

fn get_object_bounds(object: &AnimObject) -> (f32, f32, [f32; 2]) {
    match &object.shape {
        Shape::Rectangle { width, height, .. } => (
            width * object.scale[0] / 2.0,
            height * object.scale[1] / 2.0,
            [0.0, 0.0],
        ),
        Shape::Ellipse { radius_x, radius_y } => (
            radius_x * object.scale[0],
            radius_y * object.scale[1],
            [0.0, 0.0],
        ),
        Shape::Line { end_x, end_y } => {
            let scaled_end_x = end_x * object.scale[0];
            let scaled_end_y = end_y * object.scale[1];
            let center_x = scaled_end_x / 2.0;
            let center_y = scaled_end_y / 2.0;
            let half_w = scaled_end_x.abs() / 2.0;
            let half_h = scaled_end_y.abs() / 2.0;
            (half_w.max(5.0), half_h.max(5.0), [center_x, center_y])
        }
        Shape::Path { points, .. } => {
            if points.is_empty() {
                return (10.0, 10.0, [0.0, 0.0]);
            }
            let min_x = points
                .iter()
                .map(|point| point.position[0] * object.scale[0])
                .fold(f32::INFINITY, f32::min);
            let min_y = points
                .iter()
                .map(|point| point.position[1] * object.scale[1])
                .fold(f32::INFINITY, f32::min);
            let max_x = points
                .iter()
                .map(|point| point.position[0] * object.scale[0])
                .fold(f32::NEG_INFINITY, f32::max);
            let max_y = points
                .iter()
                .map(|point| point.position[1] * object.scale[1])
                .fold(f32::NEG_INFINITY, f32::max);
            let center_x = (min_x + max_x) / 2.0;
            let center_y = (min_y + max_y) / 2.0;
            let half_w = ((max_x - min_x) / 2.0).max(5.0);
            let half_h = ((max_y - min_y) / 2.0).max(5.0);
            (half_w, half_h, [center_x, center_y])
        }
    }
}
