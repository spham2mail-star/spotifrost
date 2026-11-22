//! The glass behind the main window on macOS: a see-through window with the
//! system's Liquid Glass under the interface, and the system accent colour
//! for the interface to draw with.
//!
//! On macOS 26 and later the material is an `NSGlassEffectView`, the same
//! Liquid Glass the Dock and widgets are made of. Earlier systems have no
//! such class, so they get the frosted `NSVisualEffectView` instead. Either
//! is placed in the window's frame view just below the view egui draws into,
//! so it fills the window and follows its size, and AppKit keeps it in step
//! with the light or dark appearance, the wallpaper tint and Reduce
//! Transparency by itself.
//!
//! Over the glass lie two frosted layers, drawn faintly and always in the
//! light material, so they lift the glass rather than grey it: one over the
//! whole window at 15% of its strength, and the haze over everything but the
//! player bar at 5%, so the page and the sidebars blur the desktop
//! a little more than the bar does. AppKit offers no blur radius for either
//! material, so the extra blur is mixed in this way.
//!
//! Every layer keeps its active look whether or not the window is in front,
//! and the window draws no shadow, so it looks the same focused and
//! unfocused and has no dark rim around its edge.

use std::ffi::c_void;

use objc2::rc::{Allocated, Retained};
use objc2::runtime::{AnyClass, AnyObject, Bool};
use objc2::{MainThreadMarker, msg_send};
use objc2_app_kit::{
    NSColor, NSView, NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectState,
    NSVisualEffectView, NSWindow,
};

/// `NSWindowBelow`.
const BELOW: isize = -1;
/// `NSViewWidthSizable | NSViewHeightSizable`. For the haze this keeps the
/// edge along the player bar where it is as the window resizes, whichever
/// way up the frame view counts.
const FILL: usize = 2 | 16;
/// `NSGlassEffectView.Style.regular`: the Dock's and widgets' glass.
const GLASS_STYLE_REGULAR: isize = 0;
/// How much of the frosted material is drawn where Liquid Glass is missing;
/// the rest is the clear desktop.
const FROSTED_OPACITY: f64 = 0.88;
/// How strongly the blur over the whole window is drawn: 15% more blur than
/// the glass alone.
const BLUR_OPACITY: f64 = 0.15;
/// How strongly the haze over the page and sidebars is drawn: 5% more blur
/// there than behind the player bar.
const HAZE_OPACITY: f64 = 0.05;
/// Marks the glass, so a window never gets two.
const IDENTIFIER: &str = "spotifrost-glass";
/// Marks the blur over the whole window.
const BLUR_IDENTIFIER: &str = "spotifrost-glass-blur";
/// Marks the haze over everything but the player bar.
const HAZE_IDENTIFIER: &str = "spotifrost-glass-haze";

/// Puts the glass behind the window whose egui view is `ns_view`, and the
/// haze over all of it until [`set_player_bar`] says where the bar is. Safe
/// to call again: a window that has them keeps the ones it has.
pub fn install(ns_view: *mut c_void) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let Some((view, window, frame_view)) = views(ns_view) else {
        return;
    };
    if subview(&frame_view, IDENTIFIER).is_some() {
        return;
    }

    window.setOpaque(false);
    window.setBackgroundColor(Some(&NSColor::clearColor()));
    // The shadow draws a dark rim around the window's edge, and deepens
    // while the window is in front, so the window would change shade as it
    // is clicked. Without it the edge is seamless, like the Dock's.
    window.setHasShadow(false);

    #[allow(unused_unsafe)]
    let bounds = unsafe { frame_view.bounds() };
    let identifier = objc2_foundation::NSString::from_str(IDENTIFIER);
    let liquid = AnyClass::get(c"NSGlassEffectView");
    let glass: Retained<NSView> = match liquid {
        // SAFETY: NSGlassEffectView is an NSView, made the NSView way.
        Some(class) => unsafe {
            let alloc: Allocated<NSView> = msg_send![class, alloc];
            let glass: Retained<NSView> = msg_send![alloc, initWithFrame: bounds];
            let () = msg_send![&*glass, setStyle: GLASS_STYLE_REGULAR];
            glass
        },
        None => {
            let frosted = frosted(mtm, bounds);
            // Let a little of the unblurred desktop through as well, so the
            // window reads as clear glass rather than frosted.
            frosted.setAlphaValue(FROSTED_OPACITY);
            Retained::into_super(frosted)
        }
    };

    let blur = frosted(mtm, bounds);
    lighten(&blur);
    blur.setAlphaValue(BLUR_OPACITY);
    let blur_identifier = objc2_foundation::NSString::from_str(BLUR_IDENTIFIER);
    let haze = frosted(mtm, bounds);
    lighten(&haze);
    haze.setAlphaValue(HAZE_OPACITY);
    let haze_identifier = objc2_foundation::NSString::from_str(HAZE_IDENTIFIER);
    unsafe {
        let () = msg_send![&*glass, setAutoresizingMask: FILL];
        let () = msg_send![&*glass, setIdentifier: &*identifier];
        let () = msg_send![&*blur, setAutoresizingMask: FILL];
        let () = msg_send![&*blur, setIdentifier: &*blur_identifier];
        let () = msg_send![&*haze, setAutoresizingMask: FILL];
        let () = msg_send![&*haze, setIdentifier: &*haze_identifier];
        // Bottom to top: the glass, the blur, the haze, then egui's view.
        let () = msg_send![
            &*frame_view,
            addSubview: &*haze,
            positioned: BELOW,
            relativeTo: view
        ];
        let () = msg_send![
            &*frame_view,
            addSubview: &*blur,
            positioned: BELOW,
            relativeTo: &*haze
        ];
        let () = msg_send![
            &*frame_view,
            addSubview: &*glass,
            positioned: BELOW,
            relativeTo: &*blur
        ];
    }
    log::info!(
        "{} installed behind the main window",
        if liquid.is_some() {
            "Liquid Glass"
        } else {
            "frosted glass"
        }
    );
}

/// Keeps the haze off the bottom `height` points of the window, where the
/// player bar is, so the bar sits on the plain glass. `0.0` hazes the whole
/// window, for the screens without the bar.
pub fn set_player_bar(ns_view: *mut c_void, height: f64) {
    if MainThreadMarker::new().is_none() {
        return;
    }
    let Some((_, _, frame_view)) = views(ns_view) else {
        return;
    };
    let Some(haze) = subview(&frame_view, HAZE_IDENTIFIER) else {
        return;
    };
    #[allow(unused_unsafe)]
    let mut frame = unsafe { frame_view.bounds() };
    let flipped: Bool = unsafe { msg_send![&*frame_view, isFlipped] };
    let bar = height.clamp(0.0, frame.size.height);
    frame.size.height -= bar;
    if !flipped.as_bool() {
        // AppKit counts up from the bottom edge, where the bar is.
        frame.origin.y += bar;
    }
    unsafe {
        let () = msg_send![&*haze, setFrame: frame];
    }
}

/// The frosted material, blurring the desktop behind the window. It keeps
/// its active look when the window is not in front, rather than AppKit's
/// flatter, greyer inactive one.
fn frosted(mtm: MainThreadMarker, frame: objc2_foundation::NSRect) -> Retained<NSVisualEffectView> {
    let view = NSVisualEffectView::initWithFrame(mtm.alloc(), frame);
    view.setMaterial(NSVisualEffectMaterial::Sidebar);
    view.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
    view.setState(NSVisualEffectState::Active);
    view
}

/// Draws `view` in the light material even in dark mode, so the layers over
/// the glass add blur in a pale tone rather than a grey one.
fn lighten(view: &NSVisualEffectView) {
    let Some(class) = AnyClass::get(c"NSAppearance") else {
        return;
    };
    for name in ["NSAppearanceNameVibrantLight", "NSAppearanceNameAqua"] {
        let name = objc2_foundation::NSString::from_str(name);
        let appearance: Option<Retained<AnyObject>> =
            unsafe { msg_send![class, appearanceNamed: &*name] };
        if let Some(appearance) = appearance {
            unsafe {
                let () = msg_send![view, setAppearance: &*appearance];
            }
            return;
        }
    }
}

/// egui's view, its window and the window's frame view, which holds both.
fn views(ns_view: *mut c_void) -> Option<(&'static NSView, Retained<NSWindow>, Retained<NSView>)> {
    if ns_view.is_null() {
        return None;
    }
    // SAFETY: winit hands out its live NSView, on the main thread, and it
    // outlives every call here.
    let view: &'static NSView = unsafe { &*ns_view.cast::<NSView>() };
    let window: Option<Retained<NSWindow>> = unsafe { msg_send![view, window] };
    let frame_view: Option<Retained<NSView>> = unsafe { msg_send![view, superview] };
    Some((view, window?, frame_view?))
}

/// The frame view's subview marked `identifier`.
fn subview(frame_view: &NSView, identifier: &str) -> Option<Retained<AnyObject>> {
    let subviews: Retained<AnyObject> = unsafe { msg_send![frame_view, subviews] };
    let count: usize = unsafe { msg_send![&*subviews, count] };
    (0..count).find_map(|index| {
        let subview: Retained<AnyObject> = unsafe { msg_send![&*subviews, objectAtIndex: index] };
        let id: Option<Retained<objc2_foundation::NSString>> =
            unsafe { msg_send![&*subview, identifier] };
        id.is_some_and(|id| id.to_string() == identifier)
            .then_some(subview)
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
