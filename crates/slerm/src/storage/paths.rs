use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    if let Some(xdg_data_home) = std::env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(xdg_data_home).join("slerm");
    }

    home_dir()
        .map(|home| home.join(".local/share/slerm"))
        .unwrap_or_else(|| PathBuf::from(".slerm"))
}

pub fn workspace_file() -> PathBuf {
    data_dir().join("workspace.json")
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}
