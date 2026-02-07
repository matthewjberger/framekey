use crate::canvas::CanvasView;
use crate::history::History;
use crate::onion::OnionSkinning;
use crate::playback::PlaybackState;
use crate::project::Project;
use crate::selection::Selection;
use crate::tools::{Tool, ToolState};

pub struct AnimateApp {
    pub project: Project,
    pub current_frame: u32,
    pub active_layer: usize,
    pub tool: Tool,
    pub tool_state: ToolState,
    pub canvas_view: CanvasView,
    pub selection: Selection,
    pub history: History,
    pub playback: PlaybackState,
    pub onion: OnionSkinning,
    pub fill_color: [f32; 4],
    pub stroke_color: [f32; 4],
    pub stroke_width: f32,
    pub save_path: Option<std::path::PathBuf>,
    #[cfg(target_arch = "wasm32")]
    pub pending_project_load: std::rc::Rc<std::cell::RefCell<Option<Vec<u8>>>>,
}

impl Default for AnimateApp {
    fn default() -> Self {
        Self {
            project: Project::default(),
            current_frame: 0,
            active_layer: 0,
            tool: Tool::Select,
            tool_state: ToolState::Idle,
            canvas_view: CanvasView::default(),
            selection: Selection::default(),
            history: History::new(),
            playback: PlaybackState::default(),
            onion: OnionSkinning::default(),
            fill_color: [0.2, 0.5, 0.8, 1.0],
            stroke_color: [0.0, 0.0, 0.0, 1.0],
            stroke_width: 2.0,
            save_path: None,
            #[cfg(target_arch = "wasm32")]
            pending_project_load: std::rc::Rc::new(std::cell::RefCell::new(None)),
        }
    }
}
