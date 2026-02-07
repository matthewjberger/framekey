use nightshade::prelude::*;

use crate::app::AnimateApp;
use crate::canvas::{CanvasView, render_object};
use crate::tween;

pub struct OnionSkinning {
    pub enabled: bool,
    pub frames_before: u32,
    pub frames_after: u32,
}

impl Default for OnionSkinning {
    fn default() -> Self {
        Self {
            enabled: false,
            frames_before: 2,
            frames_after: 2,
        }
    }
}

pub fn draw_onion_skins(app: &AnimateApp, view: &CanvasView, painter: &egui::Painter) {
    if !app.onion.enabled {
        return;
    }

    for offset in 1..=app.onion.frames_before {
        if app.current_frame < offset {
            continue;
        }
        let frame = app.current_frame - offset;
        let alpha = 0.3 / offset as f32;
        draw_ghost_frame(app, view, painter, frame, [1.0, 0.3, 0.3, alpha]);
    }

    for offset in 1..=app.onion.frames_after {
        let frame = app.current_frame + offset;
        if frame >= app.project.total_frames {
            continue;
        }
        let alpha = 0.3 / offset as f32;
        draw_ghost_frame(app, view, painter, frame, [0.3, 1.0, 0.3, alpha]);
    }
}

fn draw_ghost_frame(
    app: &AnimateApp,
    view: &CanvasView,
    painter: &egui::Painter,
    frame: u32,
    tint: [f32; 4],
) {
    for layer_index in (0..app.project.layers.len()).rev() {
        let layer = &app.project.layers[layer_index];
        if !layer.visible {
            continue;
        }

        if let Some(objects) = tween::resolve_frame(layer, frame) {
            for object in &objects {
                let mut tinted = object.clone();
                tinted.fill = [
                    tint[0] * object.fill[0],
                    tint[1] * object.fill[1],
                    tint[2] * object.fill[2],
                    tint[3],
                ];
                tinted.stroke = [
                    tint[0] * object.stroke[0],
                    tint[1] * object.stroke[1],
                    tint[2] * object.stroke[2],
                    tint[3],
                ];
                render_object(&tinted, view, painter, layer.opacity);
            }
        }
    }
}
