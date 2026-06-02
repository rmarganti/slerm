use gpui::{Rgba, rgb};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub fg: Rgba,
    pub bg: Rgba,

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

pub fn active() -> Theme {
    neverforest()
}

pub fn neverforest() -> Theme {
    Theme {
        fg: rgb(0xd3c6aa),
        bg: rgb(0x1e2326),

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
