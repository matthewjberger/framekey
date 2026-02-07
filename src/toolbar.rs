use nightshade::prelude::*;

use crate::app::AnimateApp;
use crate::tools::Tool;

pub fn draw_toolbar(app: &mut AnimateApp, ui_context: &egui::Context) {
    egui::SidePanel::left("toolbar")
        .resizable(false)
        .exact_width(48.0)
        .show(ui_context, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(4.0);

                let tools = [
                    (Tool::Select, "Sel", "Select (V)"),
                    (Tool::Rectangle, "Rect", "Rectangle (R)"),
                    (Tool::Ellipse, "Ell", "Ellipse (E)"),
                    (Tool::Line, "Line", "Line (L)"),
                    (Tool::Pen, "Pen", "Pen (P)"),
                    (Tool::Pencil, "Pcl", "Pencil (B)"),
                ];

                for (tool, label, tooltip) in tools {
                    let is_active = app.tool == tool;
                    let button = egui::Button::new(egui::RichText::new(label).size(11.0))
                        .min_size(egui::vec2(40.0, 32.0))
                        .selected(is_active);

                    if ui.add(button).on_hover_text(tooltip).clicked() {
                        app.tool = tool;
                    }
                }

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(4.0);

                ui.label(egui::RichText::new("Fill").size(9.0));
                ui.color_edit_button_rgba_unmultiplied(&mut app.fill_color);

                ui.add_space(4.0);

                ui.label(egui::RichText::new("Strk").size(9.0));
                ui.color_edit_button_rgba_unmultiplied(&mut app.stroke_color);

                ui.add_space(4.0);

                ui.label(egui::RichText::new("SW").size(9.0));
                ui.add(
                    egui::DragValue::new(&mut app.stroke_width)
                        .range(0.0..=50.0)
                        .speed(0.1),
                );
            });
        });
}

pub fn handle_tool_shortcuts(app: &mut AnimateApp, ui_context: &egui::Context) {
    if ui_context.wants_keyboard_input() {
        return;
    }
    ui_context.input(|input| {
        if input.key_pressed(egui::Key::V) {
            app.tool = Tool::Select;
        }
        if input.key_pressed(egui::Key::R) {
            app.tool = Tool::Rectangle;
        }
        if input.key_pressed(egui::Key::E) {
            app.tool = Tool::Ellipse;
        }
        if input.key_pressed(egui::Key::L) {
            app.tool = Tool::Line;
        }
        if input.key_pressed(egui::Key::P) {
            app.tool = Tool::Pen;
        }
        if input.key_pressed(egui::Key::B) {
            app.tool = Tool::Pencil;
        }
    });
}
