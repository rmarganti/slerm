use gpui::Rgba;

use crate::{runtime::AttentionSeverity, theme};

pub fn attention_color(severity: AttentionSeverity) -> Rgba {
    let theme = theme::active();
    match severity {
        AttentionSeverity::None => theme.minus1,
        AttentionSeverity::Info => theme.info,
        AttentionSeverity::Activity => theme.plus2,
        AttentionSeverity::NeedsUser => theme.warning,
        AttentionSeverity::Error => theme.error,
    }
}

pub fn terminal_attention_icon(severity: AttentionSeverity) -> &'static str {
    match severity {
        AttentionSeverity::None => "◦",
        AttentionSeverity::Info => "•",
        AttentionSeverity::Activity => "●",
        AttentionSeverity::NeedsUser => "◆",
        AttentionSeverity::Error => "!",
    }
}

pub fn project_attention_icon(severity: AttentionSeverity) -> &'static str {
    match severity {
        AttentionSeverity::None => "",
        severity => terminal_attention_icon(severity),
    }
}
