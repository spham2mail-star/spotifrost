//! The notch player on macOS: while the window is minimized or closed and a
//! song plays, a black shape grows out of the MacBook's notch, the way the
//! iPhone's Dynamic Island does.
//!
//! Collapsed, it is as tall as the notch and a little wider: the cover on
//! the left of the notch, a play or pause mark on the right. Its top corners
//! curve outward into the menu bar, so it reads as part of the notch. Under
//! the pointer it stays as tall as the notch and slides out to the right:
//! Previous, Play/Pause and Next appear beside the notch, and the cover stays
//! on the left, where a click opens the window. Paused, it stays, with a play
//! mark. On a screen without a notch the same shape hangs from the top centre
//! of the main screen.
//!
//! Everything here runs on the main thread. The controller sits in a static
//! for the AppKit callbacks, which the app only touches from the main thread.

use std::path::PathBuf;
use std::sync::Mutex;

use objc2::rc::Retained;
use objc2::{MainThreadOnly, define_class, sel};
use objc2_app_kit::{
    NSAnimatablePropertyContainer, NSAnimationContext, NSBackingStoreType, NSBezierPath, NSButton,
    NSColor, NSEvent, NSImage, NSImageView, NSScreen, NSTrackingArea, NSTrackingAreaOptions,
    NSView, NSWindow, NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_foundation::{MainThreadMarker, NSData, NSObject, NSPoint, NSRect, NSSize, NSString};

use super::notch::{NotchCommand, NotchTrackInfo, push_command};

static CONTROLLER: Mutex<Option<Island>> = Mutex::new(None);
/// Bumped each time a cover starts loading; a load only lands if it is
/// still the latest.
static ART_GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static PENDING_ART: Mutex<Option<(PathBuf, Vec<u8>)>> = Mutex::new(None);
/// Bumped to cancel a scheduled collapse.
static TIMER_TOKEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// How far the top corners curve out into the menu bar.
const EAR: f64 = 8.0;
/// The space between the notch and the cover or mark beside it.
const WING_PAD: f64 = 8.0;
/// The space between the three buttons.
const BUTTON_GAP: f64 = 6.0;
/// How long the shape takes to grow, and to shrink.
const GROW: f64 = 0.24;
const SHRINK: f64 = 0.20;
/// How long the pointer may be away before the shape shrinks.
const LINGER_MS: i64 = 260;
/// The height of the shape on a screen without a notch.
const NO_NOTCH_HEIGHT: f64 = 32.0;
/// A notch's width when macOS does not say.
const DEFAULT_NOTCH_WIDTH: f64 = 185.0;

unsafe extern "C" {
    static _dispatch_main_q: [u8; 0];
    fn dispatch_time(when: u64, delta: i64) -> u64;
    fn dispatch_after_f(
        when: u64,
        queue: *const u8,
        context: *mut std::ffi::c_void,
        work: extern "C" fn(*mut std::ffi::c_void),
    );
    fn dispatch_async_f(
        queue: *const u8,
        context: *mut std::ffi::c_void,
        work: extern "C" fn(*mut std::ffi::c_void),
    );
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "SpotifrostIslandActions"]
    pub struct SpotifrostIslandActions;

    impl SpotifrostIslandActions {
        #[unsafe(method(onPlayPause:))]
        fn on_play_pause(&self, _sender: &NSObject) {
            if let Ok(mut lock) = CONTROLLER.try_lock()
                && let Some(island) = lock.as_mut()
                && let Some(track) = island.track.as_mut()
            {
                // Answer the click at once; the app's next state confirms it.
                track.playing = !track.playing;
                let playing = track.playing;
                island.show_playing(playing);
            }
            push_command(NotchCommand::PlayPause);
        }

        #[unsafe(method(onNext:))]
        fn on_next(&self, _sender: &NSObject) {
            push_command(NotchCommand::Next);
        }

        #[unsafe(method(onPrevious:))]
        fn on_previous(&self, _sender: &NSObject) {
            push_command(NotchCommand::Previous);
        }

        #[unsafe(method(onOpen:))]
        fn on_open(&self, _sender: &NSObject) {
            push_command(NotchCommand::ShowWindow);
        }
    }
);

define_class!(
    #[unsafe(super(NSView))]
    #[thread_kind = MainThreadOnly]
    #[name = "SpotifrostIslandView"]
    pub struct SpotifrostIslandView;

    impl SpotifrostIslandView {
        #[unsafe(method(isFlipped))]
        fn is_flipped(&self) -> bool {
            true
        }

        #[unsafe(method(mouseEntered:))]
        fn mouse_entered(&self, _event: &NSEvent) {
            pointer_entered();
        }

        #[unsafe(method(mouseExited:))]
        fn mouse_exited(&self, _event: &NSEvent) {
            pointer_left();
        }

        #[unsafe(method(drawRect:))]
        fn draw_rect(&self, _dirty: NSRect) {
            #[allow(unused_unsafe)]
            let bounds = unsafe { self.bounds() };
            draw_shape(bounds);
        }
    }
);

/// Where the shape sits. The window is only as large as the shape needs, so
/// the empty screen around it still takes clicks: collapsed it is the
/// collapsed shape; while the shape grows or shrinks, the expanded one.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Geometry {
    /// The window, collapsed and expanded, in screen points.
    window_collapsed: NSRect,
    window_expanded: NSRect,
    /// The shape inside the expanded window (whose view is not flipped),
    /// collapsed and expanded.
    collapsed: NSRect,
    expanded: NSRect,
    /// The notch's height and width (the gap the wings sit beside).
    notch_height: f64,
    notch_width: f64,
}

impl Geometry {
    /// The collapsed shape filling the collapsed window.
    fn collapsed_alone(&self) -> NSRect {
        NSRect::new(NSPoint::ZERO, self.window_collapsed.size)
    }
}

struct Island {
    window: Retained<NSWindow>,
    shape: Retained<SpotifrostIslandView>,
    art: Retained<NSImageView>,
    mark: Retained<NSImageView>,
    previous: Retained<NSButton>,
    play: Retained<NSButton>,
    next: Retained<NSButton>,
    open: Retained<NSButton>,
    play_image: Retained<NSImage>,
    pause_image: Retained<NSImage>,
    _actions: Retained<SpotifrostIslandActions>,
    geometry: Geometry,
    expanded: bool,
    shown: bool,
    track: Option<NotchTrackInfo>,
    art_path: Option<PathBuf>,
}

// SAFETY: the island is only made and used on the main thread; the static
// only hands it between AppKit's callbacks and the app's main-thread pass.
unsafe impl Send for Island {}

/// The black shape: flush with the top of the screen, its top corners
/// curving out into the menu bar, its bottom corners rounded.
fn draw_shape(bounds: NSRect) {
    let width = bounds.size.width;
    let height = bounds.size.height;
    if width <= 2.0 * EAR || height <= 0.0 {
        return;
    }
    // Rounder as it grows, like the island.
    let radius = (height * 0.42)
        .clamp(6.0, 22.0)
        .min((width - 2.0 * EAR) / 2.0);
    let left = EAR;
    let right = width - EAR;
    let point = |x: f64, y: f64| NSPoint::new(x, y);
    #[allow(unused_unsafe)]
    unsafe {
        let path = NSBezierPath::bezierPath();
        // The view is flipped: y grows downward from the top edge.
        path.moveToPoint(point(0.0, 0.0));
        path.curveToPoint_controlPoint1_controlPoint2(
            point(left, EAR),
            point(left * 0.55, 0.0),
            point(left, EAR * 0.45),
        );
        path.lineToPoint(point(left, height - radius));
        path.curveToPoint_controlPoint1_controlPoint2(
            point(left + radius, height),
            point(left, height - radius * 0.45),
            point(left + radius * 0.45, height),
        );
        path.lineToPoint(point(right - radius, height));
        path.curveToPoint_controlPoint1_controlPoint2(
            point(right, height - radius),
            point(right - radius * 0.45, height),
            point(right, height - radius * 0.45),
        );
        path.lineToPoint(point(right, EAR));
        path.curveToPoint_controlPoint1_controlPoint2(
            point(width, 0.0),
            point(right, EAR * 0.45),
            point(width - EAR * 0.45, 0.0),
        );
        path.closePath();
        NSColor::blackColor().set();
        path.fill();
    }
}

fn screen_top_inset(screen: &NSScreen) -> f64 {
    // safeAreaInsets arrived in macOS 12, with the first notched MacBooks.
    unsafe {
        let responds: bool = objc2::msg_send![screen, respondsToSelector: sel!(safeAreaInsets)];
        if !responds {
            return 0.0;
        }
    }
    screen.safeAreaInsets().top
}

/// The notch's width: the screen less the menu bar on either side of it.
fn notch_width(screen: &NSScreen, screen_width: f64) -> f64 {
    unsafe {
        let responds: bool =
            objc2::msg_send![screen, respondsToSelector: sel!(auxiliaryTopLeftArea)];
        if !responds {
            return DEFAULT_NOTCH_WIDTH;
        }
        let left: NSRect = objc2::msg_send![screen, auxiliaryTopLeftArea];
        let right: NSRect = objc2::msg_send![screen, auxiliaryTopRightArea];
        let width = screen_width - left.size.width - right.size.width;
        if width.is_finite() && width > 40.0 && width < screen_width {
            width
        } else {
            DEFAULT_NOTCH_WIDTH
        }
    }
}

/// The shape's place: under the notch of the screen that has one, else at
/// the top centre of the main screen.
fn compute_geometry(mtm: MainThreadMarker) -> Option<Geometry> {
    let screens = NSScreen::screens(mtm);
    let notched = screens.iter().find_map(|screen| {
        let inset = screen_top_inset(&screen);
        (inset > 0.0).then_some((screen, inset))
    });
    let (screen, notch_height, gap) = match notched {
        Some((screen, inset)) => {
            let width = screen.frame().size.width;
            let gap = notch_width(&screen, width);
            (screen, inset, gap)
        }
        None => {
            let screen = NSScreen::mainScreen(mtm)?;
            (screen, NO_NOTCH_HEIGHT, 0.0)
        }
    };
    let frame = screen.frame();
    Some(geometry_for(
        frame.origin.x,
        frame.origin.y,
        frame.size.width,
        frame.size.height,
        notch_height,
        gap,
    ))
}

/// The geometry for a screen at `x, y` sized `width × height` whose notch
/// is `notch_height` tall and `notch_width` wide (0 for no notch).
fn geometry_for(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    notch_height: f64,
    notch_width: f64,
) -> Geometry {
    // A wing holds a square as tall as the notch, with room around it.
    let wing = notch_height + WING_PAD;
    let collapsed_width = notch_width + 2.0 * wing + 2.0 * EAR;
    // Opened, the right wing widens for the three buttons; the shape keeps
    // its height and its left edge, so it slides out to the right only.
    let controls = 3.0 * button_side(notch_height) + 2.0 * BUTTON_GAP + 2.0 * WING_PAD;
    let expanded_width = collapsed_width - wing + controls.max(wing);
    let top = y + height;
    let left = x + width / 2.0 - collapsed_width / 2.0;
    let size = |width: f64| NSSize::new(width, notch_height);
    Geometry {
        window_collapsed: NSRect::new(
            NSPoint::new(left, top - notch_height),
            size(collapsed_width),
        ),
        window_expanded: NSRect::new(NSPoint::new(left, top - notch_height), size(expanded_width)),
        collapsed: NSRect::new(NSPoint::ZERO, size(collapsed_width)),
        expanded: NSRect::new(NSPoint::ZERO, size(expanded_width)),
        notch_height,
        notch_width,
    }
}

/// A button's side for a notch this tall.
fn button_side(notch_height: f64) -> f64 {
    (notch_height - 8.0).clamp(16.0, 26.0)
}

fn load_icon(mtm: MainThreadMarker, svg: &[u8], size: f64) -> Option<Retained<NSImage>> {
    let data = NSData::with_bytes(svg);
    let image = NSImage::initWithData(mtm.alloc(), &data)?;
    unsafe {
        let () = objc2::msg_send![&*image, setTemplate: true];
        let () = objc2::msg_send![&*image, setSize: NSSize::new(size, size)];
    }
    Some(image)
}

fn round_corners(view: &NSView, radius: f64) {
    view.setWantsLayer(true);
    unsafe {
        let layer: Option<Retained<NSObject>> = objc2::msg_send![view, layer];
        if let Some(layer) = layer {
            let () = objc2::msg_send![&*layer, setCornerRadius: radius];
            let () = objc2::msg_send![&*layer, setMasksToBounds: true];
        }
    }
}

fn button(
    mtm: MainThreadMarker,
    image: Option<&NSImage>,
    target: &SpotifrostIslandActions,
    action: objc2::runtime::Sel,
    label: &str,
) -> Retained<NSButton> {
    let button = NSButton::initWithFrame(mtm.alloc(), NSRect::ZERO);
    button.setBordered(false);
    button.setImage(image);
    button.setTitle(&NSString::from_str(""));
    unsafe {
        let white = NSColor::whiteColor();
        let () = objc2::msg_send![&*button, setContentTintColor: &*white];
        let label = NSString::from_str(label);
        let () = objc2::msg_send![&*button, setAccessibilityLabel: &*label];
        button.setTarget(Some(target));
        button.setAction(Some(action));
    }
    button
}

fn make(mtm: MainThreadMarker) -> Option<Island> {
    let geometry = compute_geometry(mtm)?;
    let window = unsafe {
        let window = NSWindow::initWithContentRect_styleMask_backing_defer(
            mtm.alloc(),
            geometry.window_collapsed,
            NSWindowStyleMask::Borderless,
            NSBackingStoreType::Buffered,
            false,
        );
        window.setReleasedWhenClosed(false);
        window
    };
    // Above the menu bar, which the shape reaches into.
    window.setLevel(27);
    window.setOpaque(false);
    window.setBackgroundColor(Some(&NSColor::clearColor()));
    window.setHasShadow(false);
    window.setIgnoresMouseEvents(false);
    window.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::Stationary
            | NSWindowCollectionBehavior::FullScreenAuxiliary
            | NSWindowCollectionBehavior::IgnoresCycle,
    );
    // The window's own view is see-through; only the shape draws.
    let root = NSView::initWithFrame(mtm.alloc(), geometry.collapsed_alone());
    window.setContentView(Some(&root));

    let shape: Retained<SpotifrostIslandView> = unsafe {
        objc2::msg_send![
            mtm.alloc::<SpotifrostIslandView>(),
            initWithFrame: geometry.collapsed_alone()
        ]
    };
    shape.setWantsLayer(true);
    unsafe {
        // Redraw the outline as the frame animates, rather than stretch it.
        let () = objc2::msg_send![&*shape, setLayerContentsRedrawPolicy: 2isize];
    }
    let tracking = unsafe {
        NSTrackingArea::initWithRect_options_owner_userInfo(
            mtm.alloc(),
            NSRect::ZERO,
            NSTrackingAreaOptions::MouseEnteredAndExited
                | NSTrackingAreaOptions::ActiveAlways
                | NSTrackingAreaOptions::InVisibleRect,
            Some(&shape),
            None,
        )
    };
    shape.addTrackingArea(&tracking);
    root.addSubview(&shape);

    let actions: Retained<SpotifrostIslandActions> =
        unsafe { objc2::msg_send![mtm.alloc::<SpotifrostIslandActions>(), init] };
    let art = NSImageView::initWithFrame(mtm.alloc(), NSRect::ZERO);
    unsafe {
        // NSImageScaleProportionallyUpOrDown.
        let () = objc2::msg_send![&*art, setImageScaling: 3usize];
    }
    let mark = NSImageView::initWithFrame(mtm.alloc(), NSRect::ZERO);
    unsafe {
        let white = NSColor::whiteColor();
        let () = objc2::msg_send![&*mark, setContentTintColor: &*white];
    }
    let play_image = load_icon(mtm, include_bytes!("../assets/icons/play-filled.svg"), 18.0)?;
    let pause_image = load_icon(
        mtm,
        include_bytes!("../assets/icons/pause-filled.svg"),
        18.0,
    )?;
    let back = load_icon(
        mtm,
        include_bytes!("../assets/icons/skip-back-filled.svg"),
        18.0,
    );
    let forward = load_icon(
        mtm,
        include_bytes!("../assets/icons/skip-forward-filled.svg"),
        18.0,
    );
    let previous = button(
        mtm,
        back.as_deref(),
        &actions,
        sel!(onPrevious:),
        "Previous",
    );
    let play = button(
        mtm,
        Some(&*play_image),
        &actions,
        sel!(onPlayPause:),
        "Play",
    );
    let next = button(mtm, forward.as_deref(), &actions, sel!(onNext:), "Next");
    // The cover is a button too: clicking it opens the window.
    let open = button(mtm, None, &actions, sel!(onOpen:), "Open Spotifrost");
    unsafe {
        // Clear, over the cover; NSBezelStyleShadowlessSquare draws nothing.
        let () = objc2::msg_send![&*open, setBezelStyle: 6usize];
        let () = objc2::msg_send![&*open, setTransparent: true];
    }
    let views: [&NSView; 6] = [&art, &mark, &previous, &play, &next, &open];
    for view in views {
        shape.addSubview(view);
    }
    let island = Island {
        window,
        shape,
        art,
        mark,
        previous,
        play,
        next,
        open,
        play_image,
        pause_image,
        _actions: actions,
        geometry,
        expanded: false,
        shown: false,
        track: None,
        art_path: None,
    };
    island.lay_out();
    Some(island)
}

impl Island {
    /// Places the cover, the mark and the buttons for the current state.
    fn lay_out(&self) {
        let geometry = self.geometry;
        let notch_height = geometry.notch_height;
        let collapsed_width = geometry.collapsed.size.width;
        let wing = (collapsed_width - 2.0 * EAR - geometry.notch_width) / 2.0;
        // The cover keeps its place on the left in both states.
        let small = (notch_height - 2.0 * 6.0).max(14.0);
        let art = NSRect::new(
            NSPoint::new(EAR + (wing - small) / 2.0, (notch_height - small) / 2.0),
            NSSize::new(small, small),
        );
        self.art.setFrame(art);
        self.open.setFrame(art);
        round_corners(&self.art, 5.0);
        let buttons: [&NSView; 3] = [&self.previous, &self.play, &self.next];
        if self.expanded {
            // Right of the notch, in a row, centred in the band's height.
            let side = button_side(notch_height);
            let mut x = EAR + wing + geometry.notch_width + WING_PAD;
            for view in buttons {
                view.setFrame(NSRect::new(
                    NSPoint::new(x, (notch_height - side) / 2.0),
                    NSSize::new(side, side),
                ));
                view.setHidden(false);
                x += side + BUTTON_GAP;
            }
            self.mark.setHidden(true);
        } else {
            let mark_side = (small - 6.0).max(10.0);
            self.mark.setFrame(NSRect::new(
                NSPoint::new(
                    collapsed_width - EAR - wing + (wing - mark_side) / 2.0,
                    (notch_height - mark_side) / 2.0,
                ),
                NSSize::new(mark_side, mark_side),
            ));
            for view in buttons {
                view.setHidden(true);
            }
            self.mark.setHidden(false);
        }
    }

    fn show_playing(&self, playing: bool) {
        let image: &NSImage = if playing {
            &self.pause_image
        } else {
            &self.play_image
        };
        self.play.setImage(Some(image));
        self.mark.setImage(Some(image));
        let label = NSString::from_str(if playing { "Pause" } else { "Play" });
        unsafe {
            let () = objc2::msg_send![&*self.play, setAccessibilityLabel: &*label];
        }
    }

    fn expand(&mut self) {
        if self.expanded || !self.shown {
            return;
        }
        self.expanded = true;
        // Room to grow first, with the shape where it was on screen.
        self.window
            .setFrame_display(self.geometry.window_expanded, false);
        self.shape.setFrame(self.geometry.collapsed);
        self.lay_out();
        NSAnimationContext::beginGrouping();
        let context = NSAnimationContext::currentContext();
        context.setDuration(GROW);
        context.setAllowsImplicitAnimation(true);
        self.shape.animator().setFrame(self.geometry.expanded);
        NSAnimationContext::endGrouping();
    }

    fn collapse(&mut self) {
        if !self.expanded {
            return;
        }
        self.expanded = false;
        self.lay_out();
        NSAnimationContext::beginGrouping();
        let context = NSAnimationContext::currentContext();
        context.setDuration(SHRINK);
        context.setAllowsImplicitAnimation(true);
        self.shape.animator().setFrame(self.geometry.collapsed);
        NSAnimationContext::endGrouping();
        // Then give the screen around it back.
        schedule(SHRINK_DONE, (SHRINK * 1000.0) as i64 + 20);
    }

    /// Once the shape has shrunk, the window shrinks to it.
    fn finish_collapse(&self) {
        if self.expanded {
            return;
        }
        self.window
            .setFrame_display(self.geometry.window_collapsed, false);
        self.shape.setFrame(self.geometry.collapsed_alone());
    }

    /// Follows a display change: a screen added, removed or rearranged.
    fn follow_screens(&mut self, mtm: MainThreadMarker) {
        let Some(geometry) = compute_geometry(mtm) else {
            return;
        };
        if geometry == self.geometry {
            return;
        }
        self.geometry = geometry;
        if self.expanded {
            self.window
                .setFrame_display(geometry.window_expanded, false);
            self.shape.setFrame(geometry.expanded);
        } else {
            self.window
                .setFrame_display(geometry.window_collapsed, false);
            self.shape.setFrame(geometry.collapsed_alone());
        }
        self.lay_out();
    }

    fn show_art(&mut self, path: Option<PathBuf>) {
        if self.art_path == path {
            return;
        }
        self.art_path = path.clone();
        self.art.setImage(None);
        if let Some(path) = path {
            load_art(path);
        }
    }
}

fn load_art(path: PathBuf) {
    let generation = ART_GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    std::thread::spawn(move || {
        // A cover never needs more than this; a larger file is not one.
        if let Ok(meta) = std::fs::metadata(&path)
            && meta.len() <= 10 * 1024 * 1024
            && let Ok(bytes) = std::fs::read(&path)
            && ART_GENERATION.load(std::sync::atomic::Ordering::Relaxed) == generation
        {
            if let Ok(mut pending) = PENDING_ART.lock() {
                *pending = Some((path, bytes));
            }
            unsafe {
                dispatch_async_f(
                    &_dispatch_main_q as *const _ as *const u8,
                    std::ptr::null_mut(),
                    art_loaded,
                );
            }
        }
    });
}

extern "C" fn art_loaded(_context: *mut std::ffi::c_void) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let Ok(mut lock) = CONTROLLER.try_lock() else {
            return;
        };
        if let Some(island) = lock.as_mut() {
            apply_pending_art(island, mtm);
        }
    }));
}

/// Shows a cover that finished loading, if it is still the one wanted.
fn apply_pending_art(island: &mut Island, mtm: MainThreadMarker) {
    if let Ok(mut pending) = PENDING_ART.lock()
        && pending
            .as_ref()
            .is_some_and(|(path, _)| island.art_path.as_ref() == Some(path))
        && let Some((_, bytes)) = pending.take()
    {
        let data = NSData::with_bytes(&bytes);
        let image = NSImage::initWithData(mtm.alloc(), &data);
        island.art.setImage(image.as_deref());
    }
}

fn pointer_entered() {
    // A collapse on its way is called off.
    TIMER_TOKEN.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    // `try_lock`: AppKit may call in while the app's own pass holds it.
    if let Ok(mut lock) = CONTROLLER.try_lock()
        && let Some(island) = lock.as_mut()
    {
        island.expand();
    }
}

fn pointer_left() {
    schedule(COLLAPSE, LINGER_MS);
}

/// The two delayed steps: shrinking after the pointer has gone, and the
/// window following the shape once it has shrunk.
const COLLAPSE: u64 = 0;
const SHRINK_DONE: u64 = 1;

/// Runs `step` on the main thread after `delay_ms`, unless the pointer
/// comes back first.
fn schedule(step: u64, delay_ms: i64) {
    let token = TIMER_TOKEN.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
    let payload = (token << 1) | (step & 1);
    unsafe {
        let when = dispatch_time(0, delay_ms * 1_000_000);
        dispatch_after_f(
            when,
            &_dispatch_main_q as *const _ as *const u8,
            payload as usize as *mut std::ffi::c_void,
            step_due,
        );
    }
}

extern "C" fn step_due(context: *mut std::ffi::c_void) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let payload = context as usize as u64;
        if TIMER_TOKEN.load(std::sync::atomic::Ordering::SeqCst) != payload >> 1 {
            return;
        }
        if let Ok(mut lock) = CONTROLLER.try_lock()
            && let Some(island) = lock.as_mut()
        {
            if payload & 1 == COLLAPSE {
                island.collapse();
            } else {
                island.finish_collapse();
            }
        }
    }));
}

/// Nothing to set up ahead: the island is made the first time it shows.
pub fn init() {}

/// Whether the island is open or on its way: the app then keeps its frames
/// coming while it animates.
pub fn is_active() -> bool {
    CONTROLLER
        .lock()
        .ok()
        .and_then(|lock| lock.as_ref().map(|island| island.shown && island.expanded))
        .unwrap_or(false)
}

/// Shows the island while it is `enabled`, the window is `away` (minimized
/// or closed) and a song is loaded, playing or paused; hides it otherwise. Called on each of
/// the app's passes on the main thread.
pub fn sync_state(enabled: bool, away: bool, track: Option<&NotchTrackInfo>) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    // Paused, it stays, with a play mark; it goes when nothing is loaded.
    let wanted = enabled && away && track.is_some();
    let Ok(mut lock) = CONTROLLER.lock() else {
        return;
    };
    if wanted && lock.is_none() {
        *lock = make(mtm);
    }
    let Some(island) = lock.as_mut() else {
        return;
    };
    if !wanted {
        if island.shown {
            island.window.orderOut(None);
            island.shown = false;
            island.expanded = false;
            island.finish_collapse();
            island.lay_out();
        }
        return;
    }
    island.follow_screens(mtm);
    if let Some(track) = track {
        let playing_changed = island
            .track
            .as_ref()
            .is_none_or(|known| known.playing != track.playing);
        island.show_art(track.art_path.clone());
        apply_pending_art(island, mtm);
        island.track = Some(track.clone());
        if playing_changed {
            island.show_playing(track.playing);
        }
    }
    if !island.shown {
        island.shown = true;
        island.window.orderFrontRegardless();
        island.shape.setNeedsDisplay(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_island_hangs_from_the_notch_and_slides_out_to_the_right() {
        let geometry = geometry_for(0.0, 0.0, 1470.0, 956.0, 32.0, 185.0);
        let collapsed = geometry.window_collapsed;
        let expanded = geometry.window_expanded;
        // Collapsed: centred, its top at the screen's, as tall as the notch,
        // wider than it by a wing each side.
        assert!((collapsed.origin.x + collapsed.size.width / 2.0 - 735.0).abs() < 0.01);
        assert!((collapsed.origin.y + collapsed.size.height - 956.0).abs() < 0.01);
        assert_eq!(collapsed.size.height, 32.0);
        assert!(collapsed.size.width > 185.0 + 2.0 * EAR);
        // Opened: same height, same left edge, wider to the right only.
        assert_eq!(expanded.size.height, collapsed.size.height);
        assert_eq!(expanded.origin.x, collapsed.origin.x);
        assert_eq!(expanded.origin.y, collapsed.origin.y);
        assert!(expanded.size.width > collapsed.size.width);
    }

    #[test]
    fn without_a_notch_the_island_is_a_pill_at_the_top() {
        let geometry = geometry_for(0.0, 0.0, 1920.0, 1080.0, NO_NOTCH_HEIGHT, 0.0);
        assert_eq!(geometry.collapsed.size.height, NO_NOTCH_HEIGHT);
        assert!(geometry.collapsed.size.width < 200.0);
    }
}
