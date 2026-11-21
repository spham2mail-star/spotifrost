//! The glass behind the main window on macOS: a see-through window with the
//! system's frosted material under the interface, and the system accent
//! colour for the interface to draw with.
//!
//! The material is an `NSVisualEffectView` placed in the window's frame view
//! just below the view egui draws into, so it fills the window and follows
//! its size. AppKit blurs what is behind the window, tints it with the
//! wallpaper when "Allow wallpaper tinting in windows" is on, and follows the
//! light or dark appearance by itself. With Reduce Transparency on, AppKit
//! draws the material solid.

use std::ffi::c_void;

use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject};
use objc2::{MainThreadMarker, msg_send};
use objc2_app_kit::{
    NSColor, NSView, NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectState,
    NSVisualEffectView, NSWindow,
};

/// `NSWindowBelow`.
const BELOW: isize = -1;
/// `NSViewWidthSizable | NSViewHeightSizable`.
const FILL: usize = 2 | 16;
/// How much of the frosted material is drawn; the rest is the clear desktop.
const GLASS_OPACITY: f64 = 0.88;
/// Marks the material view, so a window never gets two.
const IDENTIFIER: &str = "spotifrost-glass";

/// Puts the glass behind the window whose egui view is `ns_view`. Safe to
/// call again: a window that has it keeps the one it has.
pub fn install(ns_view: *mut c_void) {
    let Some(mtm) = MainThreadMarker::new() else {
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
    let identifier = objc2_foundation::NSString::from_str(IDENTIFIER);
    let subviews: Retained<AnyObject> = unsafe { msg_send![&*frame_view, subviews] };
    let count: usize = unsafe { msg_send![&*subviews, count] };
    for index in 0..count {
        let subview: Retained<AnyObject> = unsafe { msg_send![&*subviews, objectAtIndex: index] };
        let id: Option<Retained<objc2_foundation::NSString>> =
            unsafe { msg_send![&*subview, identifier] };
        if id.is_some_and(|id| id.to_string() == IDENTIFIER) {
            return;
        }
    }

    window.setOpaque(false);
    window.setBackgroundColor(Some(&NSColor::clearColor()));

    #[allow(unused_unsafe)]
    let bounds = unsafe { frame_view.bounds() };
    let glass = NSVisualEffectView::initWithFrame(mtm.alloc(), bounds);
    glass.setMaterial(NSVisualEffectMaterial::Sidebar);
    glass.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
    glass.setState(NSVisualEffectState::FollowsWindowActiveState);
    // Let a little of the unblurred desktop through as well, so the window
    // reads as clear glass rather than frosted.
    glass.setAlphaValue(GLASS_OPACITY);
    unsafe {
        let () = msg_send![&*glass, setAutoresizingMask: FILL];
        let () = msg_send![&*glass, setIdentifier: &*identifier];
        let () = msg_send![
            &*frame_view,
            addSubview: &*glass,
            positioned: BELOW,
            relativeTo: view
        ];
    }
    log::info!("glass installed behind the main window");
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
