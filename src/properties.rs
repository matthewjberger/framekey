use nightshade::prelude::*;

use crate::app::AnimateApp;
use crate::project::Shape;
use crate::tween;

pub fn draw_properties(app: &mut AnimateApp, ui_context: &egui::Context) {
    egui::SidePanel::right("properties")
        .resizable(true)
        .default_width(250.0)
        .min_width(150.0)
        .show(ui_context, |ui| {
            ui.heading("Properties");
            ui.separator();

            if app.selection.selected_objects.is_empty() {
                draw_canvas_properties(app, ui);
            } else {
                draw_object_properties(app, ui);
            }
        });
}

fn draw_canvas_properties(app: &mut AnimateApp, ui: &mut egui::Ui) {
    ui.label("Canvas");
    ui.separator();

    egui::Grid::new("canvas_props")
        .num_columns(2)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            ui.label("Width:");
            let mut width = app.project.canvas_width as f32;
            if ui
                .add(
                    egui::DragValue::new(&mut width)
                        .range(1.0..=7680.0)
                        .speed(1.0),
                )
                .changed()
            {
                app.project.canvas_width = width as u32;
            }
            ui.end_row();

            ui.label("Height:");
            let mut height = app.project.canvas_height as f32;
            if ui
                .add(
                    egui::DragValue::new(&mut height)
                        .range(1.0..=4320.0)
                        .speed(1.0),
                )
                .changed()
            {
                app.project.canvas_height = height as u32;
            }
            ui.end_row();

            ui.label("FPS:");
            let mut fps = app.project.frame_rate as f32;
            if ui
                .add(egui::DragValue::new(&mut fps).range(1.0..=120.0).speed(1.0))
                .changed()
            {
                app.project.frame_rate = fps as u32;
            }
            ui.end_row();

            ui.label("Frames:");
            let mut frames = app.project.total_frames as f32;
            if ui
                .add(
                    egui::DragValue::new(&mut frames)
                        .range(1.0..=10000.0)
                        .speed(1.0),
                )
                .changed()
            {
                app.project.total_frames = frames as u32;
            }
            ui.end_row();

            ui.label("Background:");
            let mut bg = app.project.background_color;
            if ui.color_edit_button_rgba_unmultiplied(&mut bg).changed() {
                app.project.background_color = bg;
            }
            ui.end_row();
        });
}

fn draw_object_properties(app: &mut AnimateApp, ui: &mut egui::Ui) {
    let selected_ids = app.selection.selected_objects.clone();

    ui.label(format!("{} object(s) selected", selected_ids.len()));
    ui.separator();

    let first_id = selected_ids[0];
    let mut found_object = None;

    for layer in &app.project.layers {
        if let Some(objects) = tween::resolve_frame(layer, app.current_frame) {
            for object in &objects {
                if object.id == first_id {
                    found_object = Some(object.clone());
                    break;
                }
            }
        }
        if found_object.is_some() {
            break;
        }
    }

    let source_object = match found_object {
        Some(object) => object,
        None => {
            ui.label("(Object not visible on current frame)");
            return;
        }
    };

    let mut position = source_object.position;
    let mut rotation_deg = source_object.rotation.to_degrees();
    let mut scale = source_object.scale;
    let mut fill = source_object.fill;
    let mut stroke = source_object.stroke;
    let mut stroke_width = source_object.stroke_width;

    let mut changed = false;

    egui::Grid::new("object_props")
        .num_columns(2)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            ui.label("X:");
            changed |= ui
                .add(egui::DragValue::new(&mut position[0]).speed(1.0))
                .changed();
            ui.end_row();

            ui.label("Y:");
            changed |= ui
                .add(egui::DragValue::new(&mut position[1]).speed(1.0))
                .changed();
            ui.end_row();

            ui.label("Rotation:");
            changed |= ui
                .add(
                    egui::DragValue::new(&mut rotation_deg)
                        .speed(1.0)
                        .suffix("°"),
                )
                .changed();
            ui.end_row();

            ui.label("Scale X:");
            changed |= ui
                .add(
                    egui::DragValue::new(&mut scale[0])
                        .speed(0.01)
                        .range(0.01..=100.0),
                )
                .changed();
            ui.end_row();

            ui.label("Scale Y:");
            changed |= ui
                .add(
                    egui::DragValue::new(&mut scale[1])
                        .speed(0.01)
                        .range(0.01..=100.0),
                )
                .changed();
            ui.end_row();

            ui.label("Fill:");
            changed |= ui.color_edit_button_rgba_unmultiplied(&mut fill).changed();
            ui.end_row();

            ui.label("Stroke:");
            changed |= ui
                .color_edit_button_rgba_unmultiplied(&mut stroke)
                .changed();
            ui.end_row();

            ui.label("Stroke W:");
            changed |= ui
                .add(
                    egui::DragValue::new(&mut stroke_width)
                        .speed(0.1)
                        .range(0.0..=100.0),
                )
                .changed();
            ui.end_row();
        });

    if changed {
        let rotation_rad = rotation_deg.to_radians();
        for layer in &mut app.project.layers {
            let has_selected = tween::resolve_frame(layer, app.current_frame)
                .map(|objects| {
                    objects
                        .iter()
                        .any(|object| selected_ids.contains(&object.id))
                })
                .unwrap_or(false);

            if has_selected {
                tween::ensure_keyframe_at(layer, app.current_frame);
            }

            if let Some(keyframe) = layer.keyframes.get_mut(&app.current_frame) {
                for object in &mut keyframe.objects {
                    if selected_ids.contains(&object.id) {
                        object.position = position;
                        object.rotation = rotation_rad;
                        object.scale = scale;
                        object.fill = fill;
                        object.stroke = stroke;
                        object.stroke_width = stroke_width;
                    }
                }
            }
        }
    }

    ui.separator();

    match &source_object.shape {
        Shape::Rectangle {
            width,
            height,
            corner_radius,
        } => {
            ui.label(format!("Rectangle: {}x{}", *width as u32, *height as u32));
            ui.label(format!("Corner Radius: {:.1}", corner_radius));
        }
        Shape::Ellipse { radius_x, radius_y } => {
            ui.label(format!(
                "Ellipse: {}x{}",
                (*radius_x * 2.0) as u32,
                (*radius_y * 2.0) as u32
            ));
        }
        Shape::Line { end_x, end_y } => {
            let length = (end_x * end_x + end_y * end_y).sqrt();
            ui.label(format!("Line: length {:.1}", length));
        }
        Shape::Path { points, closed } => {
            ui.label(format!(
                "Path: {} points, {}",
                points.len(),
                if *closed { "closed" } else { "open" }
            ));
        }
    }

    ui.separator();

    if ui.button("Delete Selected").clicked() {
        app.history.push(app.project.clone());
        for layer in &mut app.project.layers {
            let has_selected = tween::resolve_frame(layer, app.current_frame)
                .map(|objects| {
                    objects
                        .iter()
                        .any(|object| selected_ids.contains(&object.id))
                })
                .unwrap_or(false);

            if has_selected {
                tween::ensure_keyframe_at(layer, app.current_frame);
            }

            if let Some(keyframe) = layer.keyframes.get_mut(&app.current_frame) {
                keyframe
                    .objects
                    .retain(|object| !selected_ids.contains(&object.id));
            }
        }
        app.selection.selected_objects.clear();
    }
}
