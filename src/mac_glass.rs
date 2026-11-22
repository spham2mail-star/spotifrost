//! The glass behind the main window on macOS, made to match the Dock and the
//! widgets: the desktop behind the window softly blurred, under a cool,
//! see-through tint, and the system accent colour for the interface to draw
//! with.
//!
//! The blur is the window server's own background blur, which blurs what is
//! behind the window without tinting it; the tint is the window's background
//! colour, drawn once over the whole window so the sidebar, the page and the
//! player bar all share it. Neither follows whether the window is in front,
//! unlike AppKit's materials, so the window looks the same focused and
//! unfocused. The window draws no shadow, so its edge has no dark rim.

use std::ffi::c_void;

use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject};
use objc2::{MainThreadMarker, msg_send};
use objc2_app_kit::{NSColor, NSView, NSWindow};

/// How far the desktop behind the window is blurred, in points: about as
/// soft as a widget's, so what lies behind stays recognizable.
const BLUR_RADIUS: i64 = 20;
/// The tint over the blur in dark mode, as sRGB and opacity: a cool dark
/// between a widget's and the Dock's.
const DARK_TINT: [f64; 4] = [0.07, 0.08, 0.12, 0.26];
/// The tint over the blur in light mode.
const LIGHT_TINT: [f64; 4] = [0.97, 0.97, 0.99, 0.30];

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    /// The window server connection, as winit uses it for the same blur.
    fn CGSMainConnectionID() -> *mut AnyObject;
    /// Blurs what lies behind a see-through window, untinted.
    fn CGSSetWindowBackgroundBlurRadius(
        connection: *mut AnyObject,
        window: isize,
        radius: i64,
    ) -> i32;
}

/// Makes the window whose egui view is `ns_view` see-through, blurred and
/// tinted for `dark` or light mode. Safe to call again, as when the system
/// appearance changes.
pub fn install(ns_view: *mut c_void, dark: bool) {
    if MainThreadMarker::new().is_none() || ns_view.is_null() {
        return;
    }
    // SAFETY: winit hands out its live NSView, on the main thread.
    let view: &NSView = unsafe { &*ns_view.cast::<NSView>() };
    let window: Option<Retained<NSWindow>> = unsafe { msg_send![view, window] };
    let Some(window) = window else {
        return;
    };

    let [red, green, blue, alpha] = if dark { DARK_TINT } else { LIGHT_TINT };
    let tint = NSColor::colorWithSRGBRed_green_blue_alpha(red, green, blue, alpha);
    window.setOpaque(false);
    window.setBackgroundColor(Some(&tint));
    // The shadow draws a dark rim around the window's edge, and deepens
    // while the window is in front.
    window.setHasShadow(false);
    let number: isize = unsafe { msg_send![&*window, windowNumber] };
    let status =
        unsafe { CGSSetWindowBackgroundBlurRadius(CGSMainConnectionID(), number, BLUR_RADIUS) };
    if status == 0 {
        log::info!(
            "glass behind the main window ({} tint)",
            if dark { "dark" } else { "light" }
        );
    } else {
        log::warn!("the window server refused the background blur ({status})");
    }
}

/// The radius of the window's rounded corners, in points, for the highlight
/// along its edge. AppKit only says through a private property, read here
/// when the window answers it; `None` otherwise.
pub fn corner_radius(ns_view: *mut c_void) -> Option<f64> {
    MainThreadMarker::new()?;
    if ns_view.is_null() {
        return None;
    }
    // SAFETY: winit hands out its live NSView, on the main thread.
    let view: &NSView = unsafe { &*ns_view.cast::<NSView>() };
    let window: Option<Retained<NSWindow>> = unsafe { msg_send![view, window] };
    let window = window?;
    let selector = objc2::sel!(_cornerRadius);
    let answers: bool = unsafe { msg_send![&*window, respondsToSelector: selector] };
    if !answers {
        return None;
    }
    let radius: f64 = unsafe { msg_send![&*window, _cornerRadius] };
    (radius.is_finite() && radius > 0.0 && radius < 100.0).then_some(radius)
}

/// The accent colour chosen in System Settings → Appearance, as sRGB.
/// `None` when AppKit cannot say, and the interface keeps its own.
pub fn accent() -> Option<[u8; 3]> {
    MainThreadMarker::new()?;
    let accent = NSColor::controlAccentColor();
    let space_class = AnyClass::get(c"NSColorSpace")?;
    unsafe {
        let space: Option<Retained<AnyObject>> = msg_send![space_class, sRGBColorSpace];
        let space = space?;
        let srgb: Option<Retained<AnyObject>> = msg_send![&*accent, colorUsingColorSpace: &*space];
        let srgb = srgb?;
        let red: f64 = msg_send![&*srgb, redComponent];
        let green: f64 = msg_send![&*srgb, greenComponent];
        let blue: f64 = msg_send![&*srgb, blueComponent];
        let byte = |value: f64| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
        Some([byte(red), byte(green), byte(blue)])
    }
}
