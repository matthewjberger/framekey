use crate::project::Project;

const MAX_HISTORY: usize = 100;

pub struct History {
    pub undo_stack: Vec<Project>,
    pub redo_stack: Vec<Project>,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn push(&mut self, project: Project) {
        self.undo_stack.push(project);
        self.redo_stack.clear();
        if self.undo_stack.len() > MAX_HISTORY {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self, current: &mut Project) -> bool {
        if let Some(previous) = self.undo_stack.pop() {
            self.redo_stack.push(current.clone());
            *current = previous;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self, current: &mut Project) -> bool {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(current.clone());
            *current = next;
            true
        } else {
            false
        }
    }
}
