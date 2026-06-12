use gpui::{Bounds, Pixels, point, px, size};

use crate::terminal::surface::TerminalDimensions;

/// Default terminal row pitch relative to terminal font size.
///
/// Keep terminal layout independent from ambient UI line-height while matching the
/// existing 14px terminal text size with a conventional monospace row pitch.
pub(crate) const DEFAULT_TERMINAL_LINE_HEIGHT_MULTIPLIER: f32 = 1.2;

#[derive(Clone, Copy, Debug)]
pub(crate) struct TerminalLayoutMetrics {
    pub origin: gpui::Point<Pixels>,
    pub render_bounds: Bounds<Pixels>,
    pub cell_width: Pixels,
    pub cell_height: Pixels,
    pub columns: u16,
    pub rows: u16,
    pub pixel_width: u32,
    pub pixel_height: u32,
}

impl TerminalLayoutMetrics {
    pub fn dimensions(self) -> TerminalDimensions {
        TerminalDimensions::new(
            self.columns,
            self.rows,
            pixels_to_terminal_px(self.cell_width),
            pixels_to_terminal_px(self.cell_height),
        )
    }
}

pub(crate) fn terminal_layout_metrics(
    bounds: Bounds<Pixels>,
    cell_width: Pixels,
    font_size: Pixels,
    scale_factor: f32,
) -> TerminalLayoutMetrics {
    terminal_layout_metrics_with_line_height(
        bounds,
        cell_width,
        font_size,
        DEFAULT_TERMINAL_LINE_HEIGHT_MULTIPLIER,
        scale_factor,
    )
}

pub(crate) fn terminal_layout_metrics_with_line_height(
    bounds: Bounds<Pixels>,
    cell_width: Pixels,
    font_size: Pixels,
    line_height_multiplier: f32,
    scale_factor: f32,
) -> TerminalLayoutMetrics {
    let scale_factor = scale_factor.max(1.0);
    let left = snap_up_to_device_pixel(bounds.left(), scale_factor);
    let top = snap_up_to_device_pixel(bounds.top(), scale_factor);
    let right = snap_down_to_device_pixel(bounds.right(), scale_factor).max(left);
    let bottom = snap_down_to_device_pixel(bounds.bottom(), scale_factor).max(top);

    let cell_width = snap_length_to_device_pixel(cell_width, scale_factor);
    let cell_height = snap_length_to_device_pixel(
        px((f32::from(font_size) * line_height_multiplier.max(1.0)).max(1.0)),
        scale_factor,
    );

    let available_width = f32::from(right - left).max(0.0);
    let available_height = f32::from(bottom - top).max(0.0);
    let cell_width_f = f32::from(cell_width).max(1.0 / scale_factor);
    let cell_height_f = f32::from(cell_height).max(1.0 / scale_factor);
    let columns = cells_that_fit(available_width, cell_width_f);
    let rows = cells_that_fit(available_height, cell_height_f);
    let render_width = cell_width * f32::from(columns);
    let render_height = cell_height * f32::from(rows);
    let render_bounds = Bounds::new(point(left, top), size(render_width, render_height));

    TerminalLayoutMetrics {
        origin: point(left, top),
        render_bounds,
        cell_width,
        cell_height,
        columns,
        rows,
        pixel_width: pixels_to_terminal_px(render_width),
        pixel_height: pixels_to_terminal_px(render_height),
    }
}

fn cells_that_fit(available: f32, cell: f32) -> u16 {
    let fit = floor_ratio_with_tolerance(available.max(0.0), cell.max(f32::EPSILON));
    fit.max(1.0).min(f32::from(u16::MAX)) as u16
}

fn floor_ratio_with_tolerance(numerator: f32, denominator: f32) -> f32 {
    let ratio = numerator / denominator;
    let rounded = ratio.round();
    if (rounded - ratio).abs() <= 0.0001 {
        rounded
    } else {
        ratio.floor()
    }
}

fn snap_up_to_device_pixel(value: Pixels, scale_factor: f32) -> Pixels {
    px((f32::from(value) * scale_factor).ceil() / scale_factor)
}

fn snap_down_to_device_pixel(value: Pixels, scale_factor: f32) -> Pixels {
    px((f32::from(value) * scale_factor).floor() / scale_factor)
}

fn snap_length_to_device_pixel(value: Pixels, scale_factor: f32) -> Pixels {
    px(((f32::from(value).max(0.0) * scale_factor).round()).max(1.0) / scale_factor)
}

fn pixels_to_terminal_px(value: Pixels) -> u32 {
    f32::from(value).ceil().max(1.0).min(u32::MAX as f32) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{bounds, point, size};

    #[test]
    fn snaps_origin_and_truncates_to_integer_cell_grid() {
        let metrics = terminal_layout_metrics_with_line_height(
            bounds(point(px(0.25), px(0.25)), size(px(81.0), px(61.0))),
            px(8.0),
            px(10.0),
            2.0,
            2.0,
        );

        assert_eq!(f32::from(metrics.origin.x), 0.5);
        assert_eq!(f32::from(metrics.origin.y), 0.5);
        assert_eq!(metrics.columns, 10);
        assert_eq!(metrics.rows, 3);
        assert_eq!(f32::from(metrics.render_bounds.size.width), 80.0);
        assert_eq!(f32::from(metrics.render_bounds.size.height), 60.0);
        assert_eq!(metrics.pixel_width, 80);
        assert_eq!(metrics.pixel_height, 60);
    }

    #[test]
    fn tolerates_floating_point_ratios_just_under_an_integer() {
        assert_eq!(floor_ratio_with_tolerance(79.99999, 8.0), 10.0);
        assert_eq!(floor_ratio_with_tolerance(79.9, 8.0), 9.0);
    }

    #[test]
    fn cell_sizes_snap_to_device_pixels() {
        let metrics = terminal_layout_metrics_with_line_height(
            bounds(point(px(0.0), px(0.0)), size(px(100.0), px(100.0))),
            px(7.26),
            px(13.0),
            1.1,
            2.0,
        );

        assert_eq!(f32::from(metrics.cell_width), 7.5);
        assert_eq!(f32::from(metrics.cell_height), 14.5);
    }
}
