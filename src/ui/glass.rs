//! The finish the glass palette puts on the whole window, on macOS: a faint
//! highlight along the window's edge, brightest at the top-left and
//! bottom-right corners and fading out before it closes, as on the Dock's
//! Liquid Glass, and a soft shadow under every piece of text so it reads
//! over whatever the blurred desktop shows. Icons are images, not text, so
//! they keep no shadow.

use egui::{Color32, Context, Id, LayerId, Order, Pos2, Rect, Shape, Stroke, Vec2, pos2, vec2};

use crate::theme::Palette;

/// Where the window's corner radius is kept, in window points, by whoever
/// knows it (`crate::mac_glass::corner_radius`).
const CORNER_RADIUS_ID: &str = "glass-corner-radius";
/// The corner radius to assume when the window does not say.
const DEFAULT_CORNER_RADIUS: f32 = 16.0;
/// The highlight's width, in points.
const RIM_WIDTH: f32 = 1.0;
/// The highlight's opacity at its brightest: barely there.
const RIM_PEAK: f32 = 0.18;
/// How far the highlight runs from each bright corner before it is gone, as
/// a share of the window's shorter side.
const RIM_REACH: f32 = 0.35;
/// The longest straight piece the highlight is drawn in, in points, so its
/// fade is smooth.
const RIM_STEP: f32 = 6.0;
/// Points drawn along each rounded corner.
const ARC_STEPS: usize = 16;
/// How far the text shadow falls below the text, in points.
const SHADOW_OFFSET: Vec2 = vec2(0.0, 1.0);
/// The text shadow's opacity in dark mode, where it is black.
const DARK_SHADOW_ALPHA: u8 = 90;
/// The text shadow's opacity in light mode, where it is a white glow.
const LIGHT_SHADOW_ALPHA: u8 = 110;

/// Records the window's corner radius, in window points.
pub fn set_corner_radius(ctx: &Context, radius: f32) {
    ctx.data_mut(|data| data.insert_temp(Id::new(CORNER_RADIUS_ID), radius));
}

/// Puts the finish on what this frame drew. Call once the frame's interface
/// is drawn. Nothing happens unless the palette is the glass.
pub fn finish(ctx: &Context, palette: &Palette) {
    if !palette.frosted {
        return;
    }
    shadow_text(ctx, palette);
    rim(ctx);
}

/// Adds a soft shadow under every text shape drawn this frame.
fn shadow_text(ctx: &Context, palette: &Palette) {
    let shadow = if palette.dark {
        Color32::from_black_alpha(DARK_SHADOW_ALPHA)
    } else {
        Color32::from_white_alpha(LIGHT_SHADOW_ALPHA)
    };
    let mut layers: Vec<LayerId> = ctx.memory(|memory| memory.layer_ids().collect());
    if !layers.contains(&LayerId::background()) {
        layers.push(LayerId::background());
    }
    ctx.graphics_mut(|graphics| {
        for layer in layers {
            let Some(list) = graphics.get_mut(layer) else {
                continue;
            };
            let drawn = std::mem::take(list);
            for clipped in drawn.all_entries() {
                if let Some(under) = shadowed(&clipped.shape, shadow) {
                    list.add(clipped.clip_rect, under);
                }
                list.add(clipped.clip_rect, clipped.shape.clone());
            }
        }
    });
}

/// The shadow under `shape`'s text, if it has any.
fn shadowed(shape: &Shape, color: Color32) -> Option<Shape> {
    match shape {
        Shape::Text(text) if !text.galley.is_empty() => {
            let mut under = text.clone();
            under.pos += SHADOW_OFFSET;
            under.override_text_color = Some(color);
            under.underline = Stroke::NONE;
            Some(Shape::Text(under))
        }
        Shape::Vec(shapes) => {
            let under: Vec<Shape> = shapes
                .iter()
                .filter_map(|shape| shadowed(shape, color))
                .collect();
            (!under.is_empty()).then_some(Shape::Vec(under))
        }
        _ => None,
    }
}

/// The highlight along the window's edge. A fullscreen window has square
/// corners and no edge to catch the light, so it gets none.
fn rim(ctx: &Context) {
    if ctx.input(|input| input.viewport().fullscreen.unwrap_or(false)) {
        return;
    }
    let window_radius = ctx
        .data(|data| data.get_temp::<f32>(Id::new(CORNER_RADIUS_ID)))
        .unwrap_or(DEFAULT_CORNER_RADIUS);
    let rect = ctx.viewport_rect().shrink(RIM_WIDTH / 2.0);
    let radius = (window_radius / ctx.zoom_factor() - RIM_WIDTH / 2.0)
        .clamp(0.0, rect.width().min(rect.height()) / 2.0);
    let points = outline(rect, radius);
    let bright = [
        corner_point(rect, radius, 225.0),
        corner_point(rect, radius, 45.0),
    ];
    let reach = (rect.width().min(rect.height()) * RIM_REACH).max(1.0);
    let glow = |point: Pos2| {
        let distance = bright
            .iter()
            .map(|corner| corner.distance(point))
            .fold(f32::INFINITY, f32::min);
        let t = (1.0 - distance / reach).max(0.0);
        t * t * (3.0 - 2.0 * t)
    };
    let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("glass-rim")));
    for pair in points.windows(2) {
        let strength = (glow(pair[0]) + glow(pair[1])) / 2.0;
        let alpha = (strength * RIM_PEAK * 255.0).round() as u8;
        if alpha > 0 {
            painter.line_segment(
                [pair[0], pair[1]],
                Stroke::new(RIM_WIDTH, Color32::from_white_alpha(alpha)),
            );
        }
    }
}

/// The point on the rounded corner of `rect` at `degrees`, measured as egui
/// draws: 0 to the right, 90 down. 225 is the top-left corner's middle, 45
/// the bottom-right's.
fn corner_point(rect: Rect, radius: f32, degrees: f32) -> Pos2 {
    let (sin, cos) = degrees.to_radians().sin_cos();
    let center = pos2(
        if cos < 0.0 {
            rect.left() + radius
        } else {
            rect.right() - radius
        },
        if sin < 0.0 {
            rect.top() + radius
        } else {
            rect.bottom() - radius
        },
    );
    center + vec2(cos, sin) * radius
}

/// The outline of `rect` with corners rounded by `radius`, once round and
/// back to its start, in pieces no longer than [`RIM_STEP`].
fn outline(rect: Rect, radius: f32) -> Vec<Pos2> {
    let mut corners = Vec::with_capacity(4 * (ARC_STEPS + 1) + 1);
    // Top-left, top-right, bottom-right, bottom-left, clockwise on screen.
    for start in [180.0_f32, 270.0, 0.0, 90.0] {
        for step in 0..=ARC_STEPS {
            let degrees = start + 90.0 * step as f32 / ARC_STEPS as f32;
            corners.push(corner_point(rect, radius, degrees));
        }
    }
    corners.push(corners[0]);
    let mut points = vec![corners[0]];
    for pair in corners.windows(2) {
        let pieces = (pair[0].distance(pair[1]) / RIM_STEP).ceil().max(1.0) as usize;
        for piece in 1..=pieces {
            points.push(pair[0].lerp(pair[1], piece as f32 / pieces as f32));
        }
    }
    points
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corner_points_land_on_their_corners() {
        let rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(400.0, 300.0));
        let top_left = corner_point(rect, 16.0, 225.0);
        let bottom_right = corner_point(rect, 16.0, 45.0);
        assert!(top_left.x < 16.0 && top_left.y < 16.0, "{top_left:?}");
        assert!(
            bottom_right.x > 384.0 && bottom_right.y > 284.0,
            "{bottom_right:?}"
        );
    }

    #[test]
    fn outline_closes_and_stays_on_the_edge() {
        let rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(400.0, 300.0));
        let points = outline(rect, 16.0);
        assert!(points.first().unwrap().distance(*points.last().unwrap()) < 0.01);
        assert!(
            points
                .windows(2)
                .all(|pair| pair[0].distance(pair[1]) <= RIM_STEP + 0.01)
        );
        assert!(
            points
                .iter()
                .all(|point| rect.expand(0.01).contains(*point))
        );
    }
}
