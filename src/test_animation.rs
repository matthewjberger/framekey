use std::collections::BTreeMap;

use crate::project::{AnimObject, Keyframe, Layer, Project, Shape, TweenType};

pub fn generate_bouncing_ball() -> Project {
    let ball_id = uuid::Uuid::new_v4();
    let shadow_id = uuid::Uuid::new_v4();

    let canvas_width = 1920u32;
    let canvas_height = 1080u32;
    let total_frames = 120u32;
    let frame_rate = 24u32;

    let ball_radius = 40.0f32;
    let ground_y = canvas_height as f32 - 200.0;
    let start_y = 150.0f32;
    let center_x = canvas_width as f32 / 2.0;
    let initial_height = ground_y - start_y;
    let restitution = 0.6f32;

    let shadow_y = ground_y + ball_radius + 10.0;

    let ball_fill = [0.95, 0.3, 0.15, 1.0];
    let ball_stroke = [0.7, 0.15, 0.05, 1.0];
    let ball_stroke_width = 2.0f32;

    let mut bounce_heights = Vec::new();
    let mut height = initial_height;
    for _ in 0..5 {
        height *= restitution;
        if height < 10.0 {
            break;
        }
        bounce_heights.push(height);
    }

    let mut time_segments: Vec<f32> = Vec::new();
    time_segments.push(initial_height.sqrt());
    for bounce_height in &bounce_heights {
        time_segments.push(bounce_height.sqrt());
        time_segments.push(bounce_height.sqrt());
    }

    let total_time: f32 = time_segments.iter().sum();
    let frames_available = (total_frames - 2) as f32;

    struct KeyframeData {
        frame: u32,
        ball_y: f32,
        ball_scale: [f32; 2],
        tween: TweenType,
    }

    let mut keyframe_data: Vec<KeyframeData> = Vec::new();

    keyframe_data.push(KeyframeData {
        frame: 0,
        ball_y: start_y,
        ball_scale: [0.92, 1.08],
        tween: TweenType::EaseIn,
    });

    let mut accumulated_frames = 0.0f32;
    let mut bounce_index = 0;
    let mut going_up = false;

    for segment_time in &time_segments {
        accumulated_frames += segment_time / total_time * frames_available;
        let frame = (accumulated_frames.round() as u32 + 1).min(total_frames - 1);

        if !going_up {
            let squash_amount = 0.3 * (1.0 - bounce_index as f32 * 0.12).max(0.1);
            keyframe_data.push(KeyframeData {
                frame,
                ball_y: ground_y,
                ball_scale: [1.0 + squash_amount, 1.0 - squash_amount],
                tween: TweenType::EaseOut,
            });
            going_up = true;
        } else {
            let peak_y = ground_y - bounce_heights[bounce_index];
            let stretch_amount = 0.08 * (1.0 - bounce_index as f32 * 0.15).max(0.02);
            keyframe_data.push(KeyframeData {
                frame,
                ball_y: peak_y,
                ball_scale: [1.0 - stretch_amount, 1.0 + stretch_amount],
                tween: TweenType::EaseIn,
            });
            bounce_index += 1;
            going_up = false;
        }
    }

    if keyframe_data
        .last()
        .is_none_or(|last| last.frame < total_frames - 1)
    {
        keyframe_data.push(KeyframeData {
            frame: total_frames - 1,
            ball_y: ground_y,
            ball_scale: [1.0, 1.0],
            tween: TweenType::None,
        });
    }

    let mut ball_keyframes: BTreeMap<u32, Keyframe> = BTreeMap::new();
    for data in &keyframe_data {
        let ball = AnimObject {
            id: ball_id,
            shape: Shape::Ellipse {
                radius_x: ball_radius,
                radius_y: ball_radius,
            },
            position: [center_x, data.ball_y],
            rotation: 0.0,
            scale: data.ball_scale,
            fill: ball_fill,
            stroke: ball_stroke,
            stroke_width: ball_stroke_width,
        };

        ball_keyframes.insert(
            data.frame,
            Keyframe {
                objects: vec![ball],
                tween: data.tween,
            },
        );
    }

    let mut shadow_keyframes: BTreeMap<u32, Keyframe> = BTreeMap::new();
    for data in &keyframe_data {
        let height_ratio = (ground_y - data.ball_y) / initial_height;
        let shadow_scale_x = 1.3 - 0.7 * height_ratio;
        let shadow_opacity = 0.5 - 0.35 * height_ratio;

        let shadow = AnimObject {
            id: shadow_id,
            shape: Shape::Ellipse {
                radius_x: 35.0,
                radius_y: 6.0,
            },
            position: [center_x, shadow_y],
            rotation: 0.0,
            scale: [shadow_scale_x, 1.0],
            fill: [0.0, 0.0, 0.0, shadow_opacity],
            stroke: [0.0, 0.0, 0.0, 0.0],
            stroke_width: 0.0,
        };

        shadow_keyframes.insert(
            data.frame,
            Keyframe {
                objects: vec![shadow],
                tween: data.tween,
            },
        );
    }

    let shadow_layer = Layer {
        id: uuid::Uuid::new_v4(),
        name: "Shadow".to_string(),
        visible: true,
        locked: false,
        opacity: 1.0,
        keyframes: shadow_keyframes,
    };

    let ball_layer = Layer {
        id: uuid::Uuid::new_v4(),
        name: "Ball".to_string(),
        visible: true,
        locked: false,
        opacity: 1.0,
        keyframes: ball_keyframes,
    };

    Project {
        name: "Bouncing Ball".to_string(),
        canvas_width,
        canvas_height,
        background_color: [0.95, 0.97, 1.0, 1.0],
        frame_rate,
        total_frames,
        layers: vec![ball_layer, shadow_layer],
    }
}
