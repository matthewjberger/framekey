use crate::project::Project;

pub fn save_project(project: &Project, path: &std::path::Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(project).map_err(|error| error.to_string())?;
    std::fs::write(path, json).map_err(|error| error.to_string())
}

pub fn load_project(path: &std::path::Path) -> Result<Project, String> {
    let data = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&data).map_err(|error| error.to_string())
}
