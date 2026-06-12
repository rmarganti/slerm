use std::{fs, path::PathBuf};

/// Preferred terminal font families, ordered by how likely GPUI/CoreText are to
/// resolve them to the installed JetBrains Mono Nerd Font files.
pub const JETBRAINS_MONO_FAMILY_CANDIDATES: &[&str] = &[
    "JetBrainsMono Nerd Font Mono",
    "JetBrains Mono Nerd Font Mono",
    "JetBrains Mono",
    "JetBrainsMono Nerd Font",
];

/// Font selection discovered during startup. Phase 2 will use this when
/// measuring GPUI text metrics for terminal cells.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalFontSelection {
    pub family: String,
    pub file: Option<PathBuf>,
}

impl TerminalFontSelection {
    pub fn discover() -> Self {
        let file = jetbrains_mono_font_files()
            .into_iter()
            .find(|path| path.file_name().is_some_and(is_preferred_mono_font_file));

        Self {
            family: JETBRAINS_MONO_FAMILY_CANDIDATES[0].to_string(),
            file,
        }
    }
}

pub fn jetbrains_mono_font_files() -> Vec<PathBuf> {
    let Some(home) = std::env::var_os("HOME") else {
        return Vec::new();
    };
    let font_dir = PathBuf::from(home).join("Library/Fonts");
    let Ok(entries) = fs::read_dir(font_dir) else {
        return Vec::new();
    };

    let mut files = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("JetBrainsMono") && name.ends_with(".ttf"))
        })
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn is_preferred_mono_font_file(name: &std::ffi::OsStr) -> bool {
    matches!(
        name.to_str(),
        Some("JetBrainsMonoNerdFontMono-Regular.ttf")
            | Some("JetBrainsMonoNerdFont-Regular.ttf")
            | Some("JetBrainsMonoNerdFontMono-Medium.ttf")
            | Some("JetBrainsMonoNerdFont-Medium.ttf")
    )
}
