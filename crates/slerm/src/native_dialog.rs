use std::path::PathBuf;

/// Open a native directory chooser for adding projects.
///
/// GPUI 0.2.2 does not expose a stable folder/open-directory dialog API, so
/// keep the platform integration behind this tiny wrapper. `rfd` uses the
/// native `NSOpenPanel` implementation on macOS, which is the near-term target.
pub fn pick_project_folder() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Add Project")
        .pick_folder()
}
