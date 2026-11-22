//! The glass behind the main window on macOS: a see-through window that
//! blurs the desktop behind it, with the system's Liquid Glass over the blur,
//! and the system accent colour for the interface to draw with.
//!
//! The blur is the window server's own background blur, the one terminal
//! apps use for a clear, blurred window: it blurs what is behind the window
//! without tinting it, so the window reads as clear glass rather than milky.
//! On macOS 26 and later an `NSGlassEffectView` in its clear style lies over
//! it, the Liquid Glass the Dock is made of, placed in the window's frame view
//! just below the view egui draws into so it fills the window and follows its
//! size.
//!
//! The window draws no shadow, so it has no dark rim around its edge, and
//! nothing here follows whether the window is in front: it looks the same
//! focused and unfocused.

use std::ffi::c_void;

use objc2::rc::{Allocated, Retained};
use objc2::runtime::{AnyClass, AnyObject};
use objc2::{MainThreadMarker, msg_send};
use objc2_app_kit::{NSColor, NSView, NSWindow};

/// `NSWindowBelow`.
const BELOW: isize = -1;
/// `NSViewWidthSizable | NSViewHeightSizable`.
const FILL: usize = 2 | 16;
/// `NSGlassEffectView.Style.clear`: see-through Liquid Glass, without the
/// regular style's frosting.
const GLASS_STYLE_CLEAR: isize = 1;
/// How far the desktop behind the window is blurred, in points.
const BLUR_RADIUS: i64 = 40;
/// Marks the glass, so a window never gets two.
const IDENTIFIER: &str = "spotifrost-glass";

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

/// Puts the glass behind the window whose egui view is `ns_view`. Safe to
/// call again: a window that has it keeps the one it has.
pub fn install(ns_view: *mut c_void) {
    let Some(_mtm) = MainThreadMarker::new() else {
        return;
    };
    if ns_view.is_null() {
        return;
    }
    // SAFETY: winit hands out its live NSView, on the main thread.
    let view: &NSView = unsafe { &*ns_view.cast::<NSView>() };
    let window: Option<Retained<NSWindow>> = unsafe { msg_send![view, window] };
    let Some(window) = window else {
        return;
    };
    let superview: Option<Retained<NSView>> = unsafe { msg_send![view, superview] };
    let Some(frame_view) = superview else {
        return;
    };
    if has_glass(&frame_view) {
        return;
    }

    window.setOpaque(false);
    window.setBackgroundColor(Some(&NSColor::clearColor()));
    // The shadow is what draws the dark rim around the window, and it
    // deepens while the window is in front.
    window.setHasShadow(false);
    let number: isize = unsafe { msg_send![&*window, windowNumber] };
    let status =
        unsafe { CGSSetWindowBackgroundBlurRadius(CGSMainConnectionID(), number, BLUR_RADIUS) };
    if status != 0 {
        log::warn!("the window server refused the background blur ({status})");
    }

    let Some(class) = AnyClass::get(c"NSGlassEffectView") else {
        log::info!("clear blur behind the main window, without Liquid Glass");
        return;
    };
    #[allow(unused_unsafe)]
    let bounds = unsafe { frame_view.bounds() };
    let identifier = objc2_foundation::NSString::from_str(IDENTIFIER);
    // SAFETY: NSGlassEffectView is an NSView, made the NSView way.
    unsafe {
        let alloc: Allocated<NSView> = msg_send![class, alloc];
        let glass: Retained<NSView> = msg_send![alloc, initWithFrame: bounds];
        let () = msg_send![&*glass, setStyle: GLASS_STYLE_CLEAR];
        let () = msg_send![&*glass, setAutoresizingMask: FILL];
        let () = msg_send![&*glass, setIdentifier: &*identifier];
        let () = msg_send![
            &*frame_view,
            addSubview: &*glass,
            positioned: BELOW,
            relativeTo: view
        ];
    }
    log::info!("clear Liquid Glass installed behind the main window");
}

/// Whether the frame view already holds the glass.
fn has_glass(frame_view: &NSView) -> bool {
    let subviews: Retained<AnyObject> = unsafe { msg_send![frame_view, subviews] };
    let count: usize = unsafe { msg_send![&*subviews, count] };
    (0..count).any(|index| {
        let subview: Retained<AnyObject> = unsafe { msg_send![&*subviews, objectAtIndex: index] };
        let id: Option<Retained<objc2_foundation::NSString>> =
            unsafe { msg_send![&*subview, identifier] };
        id.is_some_and(|id| id.to_string() == IDENTIFIER)
    })
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
