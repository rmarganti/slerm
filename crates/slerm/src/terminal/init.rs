use std::sync::OnceLock;

use crate::terminal::font::TerminalFontSelection;

static TERMINAL_ENVIRONMENT: OnceLock<TerminalEnvironment> = OnceLock::new();

/// Process-wide terminal rendering environment discovered at app startup.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalEnvironment {
    pub font: TerminalFontSelection,
}

/// Initialize process-wide terminal prerequisites. The current libghostty-vt
/// crate initializes its core lazily; Kitty PNG decoder setup is deferred until
/// the graphics phase. Keeping this entry point makes startup ownership clear.
pub fn init() -> &'static TerminalEnvironment {
    TERMINAL_ENVIRONMENT.get_or_init(|| TerminalEnvironment {
        font: TerminalFontSelection::discover(),
    })
}
