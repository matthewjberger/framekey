use crate::project::{AnimObject, Keyframe, Layer, TweenType};

pub fn resolve_frame(layer: &Layer, frame: u32) -> Option<Vec<AnimObject>> {
    let prev_entry = layer.keyframes.range(..=frame).next_back();
    let (prev_frame, prev_keyframe) = prev_entry?;

    if *prev_frame == frame {
        return Some(prev_keyframe.objects.clone());
    }

    if prev_keyframe.tween == TweenType::None {
        return Some(prev_keyframe.objects.clone());
    }

    let next_entry = layer.keyframes.range((frame + 1)..).next();
    let (next_frame, next_keyframe) = match next_entry {
        Some(entry) => entry,
        None => return Some(prev_keyframe.objects.clone()),
    };

    let raw_t = (frame - prev_frame) as f32 / (next_frame - prev_frame) as f32;
    let t = apply_easing(raw_t, prev_keyframe.tween);

    Some(interpolate_objects(
        &prev_keyframe.objects,
        &next_keyframe.objects,
        t,
    ))
}

pub fn ensure_keyframe_at(layer: &mut Layer, frame: u32) {
    if layer.keyframes.contains_key(&frame) {
        return;
    }
    let resolved = resolve_frame(layer, frame);
    let keyframe = match resolved {
        Some(objects) => Keyframe {
            objects,
            tween: TweenType::None,
        },
        None => Keyframe::default(),
    };
    layer.keyframes.insert(frame, keyframe);
}

fn apply_easing(t: f32, tween: TweenType) -> f32 {
    match tween {
        TweenType::None => t,
        TweenType::Linear => t,
        TweenType::EaseIn => t * t,
        TweenType::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
        TweenType::EaseInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
            }
        }
    }
}

fn interpolate_objects(from: &[AnimObject], to: &[AnimObject], t: f32) -> Vec<AnimObject> {
    let mut result = Vec::new();
    for from_obj in from {
        if let Some(to_obj) = to.iter().find(|object| object.id == from_obj.id) {
            result.push(interpolate_object(from_obj, to_obj, t));
        } else {
            result.push(from_obj.clone());
        }
    }
    result
}

fn interpolate_object(from: &AnimObject, to: &AnimObject, t: f32) -> AnimObject {
    AnimObject {
        id: from.id,
        shape: from.shape.clone(),
        position: lerp_arr2(from.position, to.position, t),
        rotation: lerp_angle(from.rotation, to.rotation, t),
        scale: lerp_arr2(from.scale, to.scale, t),
        fill: lerp_arr4(from.fill, to.fill, t),
        stroke: lerp_arr4(from.stroke, to.stroke, t),
        stroke_width: lerp_f32(from.stroke_width, to.stroke_width, t),
    }
}

fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp_arr2(a: [f32; 2], b: [f32; 2], t: f32) -> [f32; 2] {
    [lerp_f32(a[0], b[0], t), lerp_f32(a[1], b[1], t)]
}

fn lerp_arr4(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        lerp_f32(a[0], b[0], t),
        lerp_f32(a[1], b[1], t),
        lerp_f32(a[2], b[2], t),
        lerp_f32(a[3], b[3], t),
    ]
}

fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let mut diff = b - a;
    while diff > std::f32::consts::PI {
        diff -= 2.0 * std::f32::consts::PI;
    }
    while diff < -std::f32::consts::PI {
        diff += 2.0 * std::f32::consts::PI;
    }
    a + diff * t
}
