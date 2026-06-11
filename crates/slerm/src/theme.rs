use gpui::{Rgba, rgb};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub fg: Rgba,
    pub bg: Rgba,

    pub terminal: TerminalTheme,

    pub minus3: Rgba,
    pub minus2: Rgba,
    pub minus1: Rgba,
    pub base: Rgba,
    pub plus1: Rgba,
    pub plus2: Rgba,
    pub plus3: Rgba,
    pub plus4: Rgba,

    pub success: Rgba,
    pub error: Rgba,
    pub warning: Rgba,
    pub info: Rgba,

    pub border: Rgba,

    pub float_bg: Rgba,

    pub select_fg: Rgba,
    pub select_bg: Rgba,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct TerminalTheme {
    pub foreground: u32,
    pub background: u32,
    pub cursor: u32,
    pub cursor_text: u32,
    pub selection_foreground: u32,
    pub selection_background: u32,
    pub palette: [u32; 16],
}

pub fn active() -> Theme {
    neverforest()
}

pub fn neverforest() -> Theme {
    Theme {
        fg: rgb(0xd3c6aa),
        bg: rgb(0x1e2326),

        terminal: TerminalTheme {
            foreground: 0xd3c6aa,
            background: 0x1d2226,
            cursor: 0xd3c6aa,
            cursor_text: 0x1d2226,
            selection_foreground: 0xd3c6aa,
            selection_background: 0x3e474c,
            palette: [
                0x4b565c, 0xe67e80, 0xa2ccae, 0xdbbc7f, 0x87b5c1, 0xd699b6, 0xaeccc6, 0xd3c6aa,
                0x77817d, 0xeea9aa, 0xc3decb, 0xe6d1a7, 0xa9cad2, 0xe5bdd0, 0xcde0dc, 0xe4ddcc,
            ],
        },

        minus3: rgb(0x343f44),
        minus2: rgb(0x3d484d),
        minus1: rgb(0x7a8478),
        base: rgb(0x9da9a0),
        plus1: rgb(0xd3c6aa),
        plus2: rgb(0x83c092),
        plus3: rgb(0xa7c080),
        plus4: rgb(0xd3c6aa),

        success: rgb(0xa7c080),
        error: rgb(0xe67e80),
        warning: rgb(0xdbbc7f),
        info: rgb(0x7fbbb3),

        border: rgb(0x3d484d),

        float_bg: rgb(0x272e33),

        select_fg: rgb(0xd3c6aa),
        select_bg: rgb(0x3d484d),
    }
}
