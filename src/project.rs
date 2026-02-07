use std::collections::BTreeMap;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Project {
    pub name: String,
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub background_color: [f32; 4],
    pub frame_rate: u32,
    pub total_frames: u32,
    pub layers: Vec<Layer>,
}

impl Default for Project {
    fn default() -> Self {
        let mut layer = Layer::new("Layer 1".to_string());
        layer.keyframes.insert(0, Keyframe::default());
        Self {
            name: "Untitled".to_string(),
            canvas_width: 1920,
            canvas_height: 1080,
            background_color: [1.0, 1.0, 1.0, 1.0],
            frame_rate: 24,
            total_frames: 120,
            layers: vec![layer],
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Layer {
    pub id: uuid::Uuid,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub keyframes: BTreeMap<u32, Keyframe>,
}

impl Layer {
    pub fn new(name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name,
            visible: true,
            locked: false,
            opacity: 1.0,
            keyframes: BTreeMap::new(),
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Keyframe {
    pub objects: Vec<AnimObject>,
    pub tween: TweenType,
}

impl Default for Keyframe {
    fn default() -> Self {
        Self {
            objects: Vec::new(),
            tween: TweenType::None,
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct AnimObject {
    pub id: uuid::Uuid,
    pub shape: Shape,
    pub position: [f32; 2],
    pub rotation: f32,
    pub scale: [f32; 2],
    pub fill: [f32; 4],
    pub stroke: [f32; 4],
    pub stroke_width: f32,
}

impl AnimObject {
    pub fn new(
        shape: Shape,
        position: [f32; 2],
        fill: [f32; 4],
        stroke: [f32; 4],
        stroke_width: f32,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            shape,
            position,
            rotation: 0.0,
            scale: [1.0, 1.0],
            fill,
            stroke,
            stroke_width,
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum Shape {
    Rectangle {
        width: f32,
        height: f32,
        corner_radius: f32,
    },
    Ellipse {
        radius_x: f32,
        radius_y: f32,
    },
    Line {
        end_x: f32,
        end_y: f32,
    },
    Path {
        points: Vec<PathPoint>,
        closed: bool,
    },
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct PathPoint {
    pub position: [f32; 2],
    pub control_in: Option<[f32; 2]>,
    pub control_out: Option<[f32; 2]>,
}

#[derive(Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TweenType {
    None,
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}
