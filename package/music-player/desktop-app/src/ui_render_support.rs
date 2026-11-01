//! Shared helpers of the rendered window tests: a software-rendered window, its pixels, and rounded-box geometry.
//!
//! What: `Rendered` builds the real `AppWindow` on Slint's software renderer, wires it like the focus tests do, and draws it
//! into a pixel buffer; `Frame` reads and compares those pixels; `RoundedBox` describes a box with per-corner radii and says
//! how far a point is from its edge.
//! Why: whether a focus ring is where D117 puts it, and whether the element inside it changed, are properties of the
//! pixels, and the geometry of the element is a property of the element tree; a test needs both on the same window.
//! Note: one platform is installed per test thread, so a test builds one `Rendered`.

/// What: `use std::rc::Rc;` imports the single-thread reference-counted pointer from the standard library (`std`). Siblings:
/// `Arc<T>` is the thread-safe version, `Box<T>` has exactly one owner, and a plain `T` is owned by one variable.
/// Why: the platform and the `Rendered` value both need to hold the same window, and a test runs on one thread, so the
/// cheaper non-atomic counter is enough.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Every object is shared by reference and garbage collected; there is nothing to import.
/// ```
use std::rc::Rc;

/// What: `use i_slint_backend_testing::ElementHandle;` imports the type that points at one element of the window's tree,
/// from the Slint testing crate (a separate package, named by its crate name instead of `std` or `crate`).
/// Why: `box_of` reads an element's position and size.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ElementHandle } from "i-slint-backend-testing";
/// ```
use i_slint_backend_testing::ElementHandle;
/// What: `use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};` imports two items from a nested
/// module path: a window that draws into memory, and the setting that says how a frame buffer is reused. The braces import
/// several names from one path in one line.
/// Why: the tests need pixels, which the normal windowing backend does not give.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { MinimalSoftwareWindow, RepaintBufferType } from "slint/platform/software_renderer";
/// ```
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
/// What: `use slint::platform::{Platform, WindowAdapter, WindowEvent};` imports the platform trait (an interface that says how to
/// create windows), the window adapter trait, and the enum of window events.
/// Why: the test platform implements `Platform`, and the helpers send `WindowEvent`s.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type Platform, type WindowAdapter, WindowEvent } from "slint/platform";
/// ```
use slint::platform::{Platform, WindowAdapter, WindowEvent};
/// What: `use slint::{ComponentHandle, PlatformError, Rgb8Pixel};` imports a trait that gives windows `.window()` and `.global()`
/// (a trait method is callable only when its trait is imported), the platform error type, and the 8-bit-per-channel pixel type.
/// Why: the window needs the first, the platform's signature needs the second, and the frame buffer is made of the third.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ComponentHandle, PlatformError, Rgb8Pixel } from "slint";
/// ```
use slint::{ComponentHandle, PlatformError, Rgb8Pixel};

/// What: `use crate::AppWindow;` imports the window type that Slint generated from the `.slint` markup. `crate::` means "from the
/// root of this crate".
/// Why: the helpers build and drive the real window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { AppWindow } from "./main";
/// ```
use crate::AppWindow;
/// What: `use crate::ui_host::Host;` imports the trait that the window's callbacks call to reach the engine.
/// Why: the tests plug in a recording host in place of the engine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type Host } from "./ui_host";
/// ```
use crate::ui_host::Host;
/// What: `use crate::ui_screen::{project, update_state};` imports two functions: the one that writes the screen state to the window,
/// and the one that changes the screen state.
/// Why: the helpers reset the state and project it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { project, updateState } from "./ui_screen";
/// ```
use crate::ui_screen::{project, update_state};
/// What: `use crate::ui_screen_view::ScreenState;` imports the record that holds everything the window shows.
/// Why: a test starts from an empty `ScreenState`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type ScreenState } from "./ui_screen_view";
/// ```
use crate::ui_screen_view::ScreenState;
/// What: `use crate::ui_test_support::{RecordingHost, load_library, settle};` imports the recording host, the function that loads the
/// test library into a window, and the function that runs Slint's pending timers.
/// Why: the rendered window is set up like the focus tests' window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RecordingHost, loadLibrary, settle } from "./ui_test_support";
/// ```
use crate::ui_test_support::{RecordingHost, load_library, settle};

/// What: `struct SoftwarePlatform { window }` is a record with one field. There is no `pub`, so it is private to this file.
/// `Rc<MinimalSoftwareWindow>` is a counted pointer: cloning it adds an owner and copies no window.
/// Why: Slint needs a platform to create a window, and this one draws into memory. It holds the window so every
/// `AppWindow` the test creates gets the same one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class SoftwarePlatform { constructor(private window: MinimalSoftwareWindow) {} }
/// ```
struct SoftwarePlatform {
    /// What: `window: Rc<MinimalSoftwareWindow>,` is the one field: a shared pointer to the software window. `Rc` is used, not
    /// `Arc`, because the test runs on one thread, and not `Box`, because the test and the platform both hold it.
    /// Why: the window every `AppWindow` of the test uses.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// window: MinimalSoftwareWindow;
    /// ```
    window: Rc<MinimalSoftwareWindow>,
}

/// What: `impl Platform for SoftwarePlatform { ... }` says that `SoftwarePlatform` implements the `Platform` trait (an interface)
/// and holds the body of its required method.
/// Why: Slint calls this when the first `AppWindow` is created.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class SoftwarePlatform implements Platform { createWindowAdapter() { ... } }
/// ```
impl Platform for SoftwarePlatform {
    /// What: `fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError>` gives the shared software window.
    /// `&self` is a read-only borrow. `Result<T, E>` is Rust's stand-in for a function that may throw: `Ok(value)` is success and
    /// `Err(error)` is failure. `Rc<dyn WindowAdapter>` is a counted pointer to any value that implements the `WindowAdapter` trait.
    /// Why: the signature is fixed by the `Platform` trait.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// createWindowAdapter(): WindowAdapter { return this.window; }
    /// ```
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        // What:     `return Ok(self.window.clone());` wraps a second owner of the window in `Ok`, the success case of `Result`.
        //           `.clone()` on an `Rc` copies the pointer and adds to the count; it does not copy the window. The `Rc` of the
        //           concrete window is accepted where `Rc<dyn WindowAdapter>` is wanted, because the window implements that trait.
        // Why:      Every window the test creates gets the same software window, and the call never fails.
        // Gotcha:   `Ok(...)` is a constructor for a value, not a statement that ends the program normally; `return` hands it out.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.window;
        // ```
        return Ok(self.window.clone());
    }
}

/// What: `pub(crate) fn install_software_platform() -> Rc<MinimalSoftwareWindow>` installs the software renderer as this thread's
/// platform and gives its window. `pub(crate)` makes the function visible inside this crate only (siblings: plain `pub` reaches
/// other crates; no keyword is private to the file).
/// Why: every window of a rendered test, the real one or a small fixture, draws into memory through it; a thread installs one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function installSoftwarePlatform(): MinimalSoftwareWindow { ... }
/// ```
pub(crate) fn install_software_platform() -> Rc<MinimalSoftwareWindow> {
    // What:     `let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);` builds the in-memory window. `::new(...)` is
    //           an associated function (a constructor, like a static method), and `RepaintBufferType::NewBuffer` is one case of
    //           an enum that says the renderer starts from a fresh buffer each draw.
    // Why:      A fresh buffer each draw means a frame never carries pixels of the one before.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const window = new MinimalSoftwareWindow(RepaintBufferType.NewBuffer);
    // ```
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    // What:     `slint::platform::set_platform(Box::new(SoftwarePlatform { window: window.clone() })).expect("...");` registers the
    //           platform. `SoftwarePlatform { window: window.clone() }` builds the record with a second owner of the window.
    //           `Box::new(...)` moves it to the heap behind an owning pointer (siblings: `Rc` and `Arc` share ownership; here
    //           Slint wants sole ownership). `set_platform` answers a `Result`, and `.expect("...")` takes the success value or
    //           stops the program with that message when it failed.
    // Why:      Slint reads the platform once per thread; the test installs it before it builds a window.
    // Gotcha:   `.expect(...)` panics (ends the thread) on `Err`, like an uncaught `throw`; it is acceptable in test helpers.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // setPlatform(new SoftwarePlatform(window)); // throws if a platform is already installed
    // ```
    slint::platform::set_platform(Box::new(SoftwarePlatform { window: window.clone() })).expect("no platform is installed yet on this thread");
    // What:     `return window;` hands the original `Rc` to the caller; ownership moves out of the function.
    // Why:      The caller draws frames from the same window.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return window;
    // ```
    return window;
}

/// What: `pub(crate) fn draw_frame(window: &MinimalSoftwareWindow, width: usize, height: usize) -> Frame` draws a software window into
/// a pixel buffer and gives the pixels. `&MinimalSoftwareWindow` is a read-only borrow. `usize` is the unsigned integer wide enough
/// to index memory (siblings: `u32`, `u64`, `i32`).
/// Why: the first draws build what the properties asked for (a page that has just become visible), and the last draws it. `usize` is
/// used, not `u32`, because the sizes index the pixel buffer and `Vec` indexes take `usize`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function drawFrame(window: MinimalSoftwareWindow, width: number, height: number): Frame { ... }
/// ```
pub(crate) fn draw_frame(window: &MinimalSoftwareWindow, width: usize, height: usize) -> Frame {
    // What:     `let mut pixels = vec![Rgb8Pixel::default(); width * height];` makes the pixel buffer. `vec![value; count]` is a macro (the
    //           `!` marks code that expands at compile time) that builds a `Vec` of `count` copies of `value`; `Rgb8Pixel::default()` is
    //           the all-zero pixel. `mut` makes the variable changeable, because the renderer writes into it.
    // Why:      One flat buffer holds the whole frame, row after row.
    // Gotcha:   `let` without `mut` is TypeScript's `const`, and the default is `const`, the reverse of habit.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pixels: Rgb8Pixel[] = new Array(width * height).fill({ r: 0, g: 0, b: 0 });
    // ```
    let mut pixels = vec![Rgb8Pixel::default(); width * height];
    // What:     `for _ in 0..3 { ... }` repeats three times. `0..3` is a half-open range that includes 0 and excludes 3, and `_` as the
    //           loop variable says that the count itself is not used.
    // Why:      The first draws build lazily created parts of the window and the last one draws the settled picture.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (let i = 0; i < 3; i++) {
    // ```
    for _ in 0..3 {
        // What:     `slint::platform::update_timers_and_animations();` runs the timers and animations that are due.
        // Why:      Slint advances them only when asked, and a draw must see their latest state.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // slint.updateTimersAndAnimations();
        // ```
        slint::platform::update_timers_and_animations();
        // What:     `window.request_redraw();` marks the window as needing a draw.
        // Why:      `draw_if_needed` draws only a window that is marked.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // window.requestRedraw();
        // ```
        window.request_redraw();
        // What:     `window.draw_if_needed(|renderer| { renderer.render(&mut pixels, width); });` draws when needed. `|renderer| { ... }`
        //           is a closure (an anonymous function; its parameters sit between the bars). Slint calls it with the renderer, and the
        //           body renders into the buffer. `&mut pixels` lends the buffer for writing, which Rust allows to one borrower at a time.
        // Why:      The renderer writes the picture into `pixels`, one row of `width` pixels at a time.
        // Gotcha:   The closure borrows `pixels` for writing only while it runs; the buffer is usable again after the call.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // window.drawIfNeeded((renderer) => { renderer.render(pixels, width); });
        // ```
        window.draw_if_needed(|renderer| {
            renderer.render(&mut pixels, width);
        });
    }
    // What:     `return Frame { width, height, pixels };` builds the `Frame` record. A field written as a bare name is shorthand for
    //           `width: width`; `pixels` is moved into the record, so the buffer now belongs to the frame.
    // Why:      The caller reads the pixels from the frame.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { width, height, pixels };
    // ```
    return Frame { width, height, pixels };
}

/// What: `pub(crate) struct Frame { width, height, pixels }` is one rendered picture of the window.
/// Why: the tests read colours and compare two pictures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Frame = { width: number; height: number; pixels: Rgb8Pixel[] };
/// ```
pub(crate) struct Frame {
    /// What: `pub width: usize,` is the width in pixels. `pub` makes the field readable from every module of the crate. `usize` is the
    /// unsigned integer that indexes memory (siblings: `u32`, `u64`).
    /// Why: pixel positions are computed as `y * width + x`, which needs the width as a `usize`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// width: number;
    /// ```
    pub width: usize,
    /// What: `pub height: usize,` is the height in pixels, the same type as the width.
    /// Why: loops over rows stop at the height.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// height: number;
    /// ```
    pub height: usize,
    /// What: `pub pixels: Vec<Rgb8Pixel>,` is the pixels row by row. `Vec<T>` is an owned, growable list (siblings: `&[T]` is a borrowed
    /// view of one, `[T; N]` a fixed-size array); `Rgb8Pixel` is a red, green and blue pixel with one byte per channel.
    /// Why: the frame owns its pixels, and the size is known only at run time, so a `Vec` fits and a fixed array does not.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// pixels: Rgb8Pixel[];
    /// ```
    pub pixels: Vec<Rgb8Pixel>,
}

/// What: `impl Frame { ... }` opens the block that holds the methods of `Frame`.
/// Why: reading and comparing pixels belong to the picture.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Frame { /* the methods of the impl block */ }
/// ```
impl Frame {
    /// What: `pub(crate) fn at(&self, x: usize, y: usize) -> [u8; 3]` gives the pixel at a column and row as `[red, green, blue]`.
    /// `[u8; 3]` is an array of exactly three `u8` values; `u8` is an unsigned 8-bit integer, 0 to 255 (siblings: `u16`, `i8`).
    /// Why: a colour channel is one byte, so `u8` is exact and `u32` would waste room; a fixed array is returned, not a `Vec`,
    /// because the length is always three and an array needs no heap.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// at(x: number, y: number): [number, number, number] { ... }
    /// ```
    pub(crate) fn at(&self, x: usize, y: usize) -> [u8; 3] {
        // What:     `let pixel = self.pixels[y * self.width + x];` reads one pixel by its flat index. Square brackets index a `Vec`;
        //           the pixel is copied out because `Rgb8Pixel` is a small `Copy` type. An index past the end stops the program.
        // Why:      Rows are stored one after another, so row `y` starts at `y * width`.
        // Gotcha:   Out-of-range indexing panics instead of giving `undefined` as in TypeScript.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const pixel = this.pixels[y * this.width + x];
        // ```
        let pixel = self.pixels[y * self.width + x];
        // What:     `return [pixel.r, pixel.g, pixel.b];` builds a three-item array from the channels. Square brackets build an array here.
        // Why:      Tests compare colours as plain arrays with `==`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return [pixel.r, pixel.g, pixel.b];
        // ```
        return [pixel.r, pixel.g, pixel.b];
    }

    /// What: `pub(crate) fn differing_box(&self, other: &Self) -> Option<(usize, usize, usize, usize)>` gives the smallest box
    /// `(left, top, right, bottom)` (right and bottom exclusive) around every pixel that differs between this frame and another of the
    /// same size, or `None` when they are equal. `&Self` is a read-only borrow of a value of this same type (`Self` is an alias for
    /// `Frame`). `Option<T>` is Rust's `T | null`: `Some(value)` or `None`. The `T` here is a tuple of four `usize` values, a fixed
    /// group like TypeScript's `[number, number, number, number]`.
    /// Why: the pixels a focus ring adds are exactly the pixels that differ between the frame with and without it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// differingBox(other: Frame): [number, number, number, number] | null { ... }
    /// ```
    pub(crate) fn differing_box(&self, other: &Self) -> Option<(usize, usize, usize, usize)> {
        // What:     `let mut found: Option<(usize, usize, usize, usize)> = None;` starts with no box. `None` is the absent case of `Option`;
        //           `mut` lets the variable change.
        // Why:      The first differing pixel creates the box and later ones grow it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let found: [number, number, number, number] | null = null;
        // ```
        let mut found: Option<(usize, usize, usize, usize)> = None;
        // What:     `for y in 0..self.height { ... }` visits each row index; `0..self.height` is a half-open range.
        // Why:      Every pixel is compared.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (let y = 0; y < this.height; y++) {
        // ```
        for y in 0..self.height {
            // What:     `for x in 0..self.width { ... }` visits each column index of the row.
            // Why:      Inner loop of the pixel scan.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // for (let x = 0; x < this.width; x++) {
            // ```
            for x in 0..self.width {
                // What:     `if self.at(x, y) != other.at(x, y) { ... }` compares two three-item arrays element by element. `!=` is
                //           TypeScript's `!==`, but on arrays it compares contents, not identity.
                // Why:      Equal pixels leave the box alone.
                // Gotcha:   `==` on arrays compares values; TypeScript's `===` on arrays compares identity.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (!equalArrays(this.at(x, y), other.at(x, y))) {
                // ```
                if self.at(x, y) != other.at(x, y) {
                    // What:     `found = Some(match found { None => (x, y, x + 1, y + 1), Some((left, top, right, bottom)) => (...) });` grows
                    //           the box. `match found` takes the current optional box apart: `None` starts a one-pixel box, and
                    //           `Some((left, top, right, bottom))` splits the stored four-tuple into four names and answers a new tuple
                    //           with `left.min(x)` (the smaller of two numbers), `top.min(y)`, `right.max(x + 1)` (the larger) and
                    //           `bottom.max(y + 1)`. `Some(...)` wraps the result back in the present case of `Option`.
                    // Why:      Right and bottom are exclusive, so a pixel at `x` extends the box to `x + 1`.
                    // Gotcha:   `match` must cover every case of the `Option`; both `None` and `Some` are present.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // found = found == null ? [x, y, x + 1, y + 1] : [Math.min(found[0], x), Math.min(found[1], y), Math.max(found[2], x + 1), Math.max(found[3], y + 1)];
                    // ```
                    found = Some(match found {
                        None => (x, y, x + 1, y + 1),
                        Some((left, top, right, bottom)) => (left.min(x), top.min(y), right.max(x + 1), bottom.max(y + 1)),
                    });
                }
            }
        }
        // What:     `return found;` hands the optional box to the caller.
        // Why:      `None` tells the caller that the two frames are identical.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return found;
        // ```
        return found;
    }
}

/// What: `pub(crate) struct Rendered { app, window, width, height }` is the real window on the software renderer, with the library
/// loaded.
/// Why: a test needs the window, the thing it draws into, and the size, together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Rendered { app: AppWindow; private window: MinimalSoftwareWindow; private width: number; private height: number; }
/// ```
pub(crate) struct Rendered {
    /// What: `pub app: AppWindow,` is the window under test; `pub` lets tests read it directly.
    /// Why: tests press keys and read properties through it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// app: AppWindow;
    /// ```
    pub app: AppWindow,
    /// What: `window: Rc<MinimalSoftwareWindow>,` is the software window the platform hands out, shared with the platform through an
    /// `Rc` (a counted pointer; siblings: `Arc` for threads, `Box` for one owner). It is private (no `pub`).
    /// Why: `frame` draws this window.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private window: MinimalSoftwareWindow;
    /// ```
    window: Rc<MinimalSoftwareWindow>,
    /// What: `width: usize,` is the width in pixels (`usize`, the unsigned integer that indexes memory; siblings: `u32`, `u64`).
    /// Why: the frame buffer's size.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private width: number;
    /// ```
    width: usize,
    /// What: `height: usize,` is the height in pixels, the same type as the width.
    /// Why: the frame buffer's size.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private height: number;
    /// ```
    height: usize,
}

/// What: `impl Rendered { ... }` opens the block that holds the methods of `Rendered`.
/// Why: building, drawing and waiting belong to the rendered window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Rendered { /* the methods of the impl block */ }
/// ```
impl Rendered {
    /// What: `pub(crate) fn new(width: usize, height: usize) -> Self` builds the window at a size, in the light scheme, with the test
    /// library loaded and the window active. `-> Self` means "returns a value of this same type", here `Rendered`.
    /// Why: the same setup as the focus tests (`fresh_window_at`) but on a renderer that draws.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static create(width: number, height: number): Rendered { ... }
    /// ```
    pub(crate) fn new(width: usize, height: usize) -> Self {
        // What:     `let window = install_software_platform();` installs the platform and keeps its window.
        // Why:      The window must exist before `AppWindow::new()` asks the platform for one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const window = installSoftwarePlatform();
        // ```
        let window = install_software_platform();
        // What:     `update_state(|state| { *state = ScreenState::default(); });` resets the screen state. The closure `|state| ...` receives
        //           the state as a mutable borrow, and `*state = value` writes through the borrow, replacing the whole state with an
        //           empty one from `ScreenState::default()`. The `*` follows a reference to the value it points at; it is not
        //           multiplication.
        // Why:      A test starts from a clean state, not from what a previous test left in the thread's global.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // updateState((state) => { Object.assign(state, defaultScreenState()); });
        // ```
        update_state(|state| {
            *state = ScreenState::default();
        });
        // What:     `crate::ui_settings::replace_saver(|_track_rows, _playing| return Ok(()));` installs a fake saver. The closure has two
        //           parameters whose names start with `_`, which marks them as deliberately unused. `Ok(())` is the success case of
        //           `Result` carrying `()`, Rust's empty value (close to `void`).
        // Why:      The Settings page would otherwise write the real session file during a test.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // replaceSaver((_trackRows, _playing) => undefined);
        // ```
        crate::ui_settings::replace_saver(|_track_rows, _playing| return Ok(()));
        crate::ui_settings::replace_auto_play_saver(|_enabled| return Ok(()));
        // What:     `let app = AppWindow::new().expect("...");` builds the window. `AppWindow::new()` answers a `Result`, and
        //           `.expect("...")` takes the success value or stops the thread with the message.
        // Why:      Every test needs the window; a failure to build it is a test failure.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const app = new AppWindow(); // throws on failure
        // ```
        let app = AppWindow::new().expect("the window builds on the software renderer");
        // What:     `app.window().set_size(slint::PhysicalSize::new(u32::try_from(width).unwrap_or(1), u32::try_from(height).unwrap_or(1)));`
        //           sizes the window in physical pixels. `u32::try_from(width)` converts the `usize` to a `u32` and answers a
        //           `Result` (`Err` when the number is too big); `.unwrap_or(1)` takes the success value or the fallback `1`.
        // Why:      Slint's size type is `u32`, not `usize`; the fallback keeps an absurd size from stopping the helper.
        // Gotcha:   `.unwrap_or(1)` silently drops the error; here a too-large size becomes a 1 px window, which no test uses.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // app.window().setSize({ width: Math.min(width, 2 ** 32 - 1), height: Math.min(height, 2 ** 32 - 1) });
        // ```
        app.window().set_size(slint::PhysicalSize::new(u32::try_from(width).unwrap_or(1), u32::try_from(height).unwrap_or(1)));
        // What:     `app.global::<crate::Theme>().set_scheme_override(1);` reaches a global singleton declared in the markup. `::<T>`
        //           (written after the method name) names a type argument; `1` selects the light scheme.
        // Why:      Pixel colours depend on the scheme, and the tests assume the light one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // app.global("Theme").setSchemeOverride(1);
        // ```
        app.global::<crate::Theme>().set_scheme_override(1);
        // What:     `let host = Rc::new(RecordingHost::default());` wraps a fresh recording host in a counted pointer. `Rc::new(...)`
        //           allocates the value on the heap with an owner count of one; `RecordingHost::default()` is a ready-made empty value.
        // Why:      The window's callbacks and the test both need to hold the host, which a plain value cannot be.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const host = new RecordingHost();
        // ```
        let host = Rc::new(RecordingHost::default());
        // What:     `let as_host: Rc<dyn Host> = Rc::clone(&host) as Rc<dyn Host>;` makes a second owner and views it as the trait.
        //           `Rc::clone(&host)` copies the pointer and adds to the count (the host itself is not copied; `&host` lends the
        //           original). `as Rc<dyn Host>` converts the pointer to one that holds "any value that implements `Host`".
        // Why:      The window's wiring functions accept the trait, so a test host and the engine are interchangeable.
        // Gotcha:   `as` here is an explicit conversion between pointer types, not a TypeScript type assertion; the compiler checks it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const asHost: Host = host;
        // ```
        let as_host: Rc<dyn Host> = Rc::clone(&host) as Rc<dyn Host>;
        // What:     `crate::ui_wire::wire_host_callbacks(&app, &as_host);` connects the window's callbacks to the host. `&app` and `&as_host`
        //           lend the values read-only, so both stay usable afterwards.
        // Why:      Without the wiring, a key press in the test would do nothing.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // wireHostCallbacks(app, asHost);
        // ```
        crate::ui_wire::wire_host_callbacks(&app, &as_host);
        // What:     `crate::ui_settings::wire_callbacks(&app);` connects the Settings page's callbacks.
        // Why:      The focus tests open Settings with a key.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // wireSettingsCallbacks(app);
        // ```
        crate::ui_settings::wire_callbacks(&app);
        // What:     `crate::ui_search::wire_route(&app);` connects the Search route's callbacks.
        // Why:      The focus tests open Search with a key.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // wireSearchRoute(app);
        // ```
        crate::ui_search::wire_route(&app);
        // What:     `crate::ui_search::wire_open(&app, Rc::clone(&as_host));` connects the Search page's open action, which needs its own owner of
        //           the host; `Rc::clone(&as_host)` makes that second owner without copying the host.
        // Why:      The callback outlives this function, so it must own a handle to the host.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // wireSearchOpen(app, asHost);
        // ```
        crate::ui_search::wire_open(&app, Rc::clone(&as_host));
        // What:     `crate::ui_menu::wire_request(&app);` connects the track menu's request callback.
        // Why:      The focus tests open the track menu with a key.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // wireMenuRequest(app);
        // ```
        crate::ui_menu::wire_request(&app);
        // What:     `app.show().expect("the window shows");` shows the window; `.expect` stops the thread with the message on failure.
        // Why:      A window that is not shown draws nothing.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // app.show(); // throws on failure
        // ```
        app.show().expect("the window shows");
        // What:     `app.window().dispatch_event(WindowEvent::WindowActiveChanged(true));` tells the window that it is active.
        //           `WindowEvent::WindowActiveChanged(true)` builds one case of the event enum, carrying `true`.
        // Why:      Focus rings and key handling depend on an active window.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // app.window().dispatchEvent({ kind: "windowActiveChanged", active: true });
        // ```
        app.window().dispatch_event(WindowEvent::WindowActiveChanged(true));
        // What:     `project(&app);` writes the screen state to the window's properties; `&app` lends the window.
        // Why:      The window shows the empty state before the library is loaded.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // project(app);
        // ```
        project(&app);
        // What:     `load_library(&app);` loads the test library through the same engine updates the app uses.
        // Why:      Focus and ring tests need rows and folders to land on.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // loadLibrary(app);
        // ```
        load_library(&app);
        // What:     `settle();` runs Slint's pending `changed` handlers and timers.
        // Why:      The window must have finished reacting before the test reads it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // settle();
        // ```
        settle();
        // What:     `return Self { app, window, width, height };` builds the value. `Self` stands for `Rendered`, and bare field names are
        //           shorthand for `app: app` and so on; every value is moved into the record.
        // Why:      The caller gets the window and everything needed to draw it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { app, window, width, height };
        // ```
        return Self { app, window, width, height };
    }

    /// What: `pub(crate) fn frame(&self) -> Frame` draws the window and gives the pixels.
    /// Why: the first draws build what the properties asked for (a page that has just become visible), and the last draws it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// frame(): Frame { return drawFrame(this.window, this.width, this.height); }
    /// ```
    pub(crate) fn frame(&self) -> Frame {
        // What:     `return draw_frame(&self.window, self.width, self.height);` lends the shared window (`&self.window` is a `&Rc<...>`, which
        //           Rust accepts where `&MinimalSoftwareWindow` is wanted, by following the pointer) and passes the two sizes by value.
        // Why:      `draw_frame` does the drawing and reads only the window.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return drawFrame(this.window, this.width, this.height);
        // ```
        return draw_frame(&self.window, self.width, self.height);
    }

    /// What: `pub(crate) fn wait(&self, milliseconds: u64)` lets real time pass so a 100 ms or 120 ms fade has finished, then runs the
    /// timers. `u64` is an unsigned 64-bit integer (siblings: `u32` is smaller, `i64` can be negative).
    /// Why: the software platform uses the real clock, and a state layer or the menu fade is mid-way until it has run. `Duration::from_millis`
    /// takes a `u64`, which is why the parameter is one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// wait(milliseconds: number): void { sleep(milliseconds); updateTimersAndAnimations(); }
    /// ```
    pub(crate) fn wait(&self, milliseconds: u64) {
        // What:     `std::thread::sleep(std::time::Duration::from_millis(milliseconds));` blocks the thread for a span of time.
        //           `Duration::from_millis(n)` builds a span of `n` milliseconds (siblings: `Instant` is a point in time, not a span).
        // Why:      Animations run on the real clock, so the test must really wait.
        // Gotcha:   Unlike `await sleep(...)` in TypeScript, this blocks the whole thread; nothing else runs meanwhile.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // await sleep(milliseconds);
        // ```
        std::thread::sleep(std::time::Duration::from_millis(milliseconds));
        // What:     `slint::platform::update_timers_and_animations();` runs the timers and animations that are now due.
        // Why:      The elapsed time must be applied before the next draw.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // slint.updateTimersAndAnimations();
        // ```
        slint::platform::update_timers_and_animations();
    }

    /// What: `pub(crate) fn set_dark(&self, dark: bool)` switches between the light scheme (false) and the dark scheme (true).
    /// Why: pixel tests run in both schemes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// setDark(dark: boolean): void { this.app.global("Theme").setSchemeOverride(dark ? 2 : 1); settle(); }
    /// ```
    pub(crate) fn set_dark(&self, dark: bool) {
        // What:     `self.app.global::<crate::Theme>().set_scheme_override(if dark { 2 } else { 1 });` passes the value of an `if`
        //           expression. In Rust `if` produces a value, like TypeScript's `cond ? a : b`; both branches must have the same
        //           type, and neither ends in `;`.
        // Why:      The scheme override takes `2` for dark and `1` for light.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.app.global("Theme").setSchemeOverride(dark ? 2 : 1);
        // ```
        self.app.global::<crate::Theme>().set_scheme_override(if dark { 2 } else { 1 });
        // What:     `settle();` runs Slint's pending handlers and timers.
        // Why:      The scheme change must take effect before the next read.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // settle();
        // ```
        settle();
    }

    /// What: `pub(crate) fn ring_frames(&self) -> (Frame, Frame)` draws the window with the keyboard ring on and again with it off,
    /// and gives both. `(Frame, Frame)` is a tuple, a fixed pair like TypeScript's `[Frame, Frame]`.
    /// Why: the focus stays where it is, so the two frames differ only by what the ring draws; any other difference is a
    /// focus-driven change that D117 forbids.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ringFrames(): [Frame, Frame] { /* ring on, wait, draw; ring off, wait, draw; ring on */ }
    /// ```
    pub(crate) fn ring_frames(&self) -> (Frame, Frame) {
        // What:     `let focus = self.app.global::<crate::Focus>();` reaches the focus global singleton declared in the markup.
        //           `::<crate::Focus>` (a "turbofish") names the type argument of `global`.
        // Why:      The ring switch lives on that global.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const focus = this.app.global("Focus");
        // ```
        let focus = self.app.global::<crate::Focus>();
        // A page that has just opened, or a state layer, is still fading for up to 150 ms; both frames are drawn after it ended.
        // What:     `focus.set_ring_visible(true);` turns the keyboard ring on.
        // Why:      The first frame is the one with the ring.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // focus.setRingVisible(true);
        // ```
        focus.set_ring_visible(true);
        // What:     `self.wait(160);` lets 160 ms of real time pass (longer than the 150 ms fades) and runs the timers.
        // Why:      Both frames must be drawn after every fade has ended.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // await this.wait(160);
        // ```
        self.wait(160);
        // What:     `let on = self.frame();` draws the window with the ring on.
        // Why:      It is the first of the two frames to compare.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const on = this.frame();
        // ```
        let on = self.frame();
        // What:     `focus.set_ring_visible(false);` turns the keyboard ring off.
        // Why:      The second frame is the one without the ring.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // focus.setRingVisible(false);
        // ```
        focus.set_ring_visible(false);
        // What:     `self.wait(160);` waits again for the fades.
        // Why:      Turning the ring off may itself start a fade.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // await this.wait(160);
        // ```
        self.wait(160);
        // What:     `let off = self.frame();` draws the window with the ring off.
        // Why:      It is the second frame to compare.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const off = this.frame();
        // ```
        let off = self.frame();
        // What:     `focus.set_ring_visible(true);` puts the ring back on.
        // Why:      The window is left as the test found it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // focus.setRingVisible(true);
        // ```
        focus.set_ring_visible(true);
        // What:     `settle();` runs Slint's pending handlers.
        // Why:      The restored ring state must take effect.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // settle();
        // ```
        settle();
        // What:     `return (on, off);` builds a pair of the two frames; both are moved into it.
        // Why:      The caller compares them.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return [on, off];
        // ```
        return (on, off);
    }
}

/// What: `pub(crate) fn box_of(element: &ElementHandle) -> (f32, f32, f32, f32)` gives an element's window position and size as
/// `(x, y, width, height)`. `f32` is a 32-bit floating point number (siblings: `f64` is twice the width, `i32` is an integer).
/// Why: the element tree reports logical pixels as `f32`, so the pair of position and size is kept in the same type; an `f64` would
/// only add casts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function boxOf(element: ElementHandle): [number, number, number, number] { ... }
/// ```
pub(crate) fn box_of(element: &ElementHandle) -> (f32, f32, f32, f32) {
    // What:     `let position = element.absolute_position();` reads where the element sits in the window.
    // Why:      The position is relative to the window, not to the parent.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const position = element.absolutePosition();
    // ```
    let position = element.absolute_position();
    // What:     `let size = element.size();` reads the element's width and height.
    // Why:      The box needs both.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const size = element.size();
    // ```
    let size = element.size();
    // What:     `return (position.x, position.y, size.width, size.height);` builds a four-item tuple from the fields.
    // Why:      A plain tuple is easier to compare in a test than two records.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return [position.x, position.y, size.width, size.height];
    // ```
    return (position.x, position.y, size.width, size.height);
}

/// What: `pub(crate) struct RoundedBox { x, y, width, height, radii }` is a box with a radius at each corner.
/// `#[derive(Clone, Copy, Debug)]` asks the compiler to write three standard abilities: `Clone` and `Copy` (a value is duplicated by
/// copying bits, so passing it never uses it up) and `Debug` (printing with `{:?}`).
/// Why: the focus-ring tests compute distances to a rounded edge; a plain value that can be copied freely keeps the maths simple.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RoundedBox = { x: number; y: number; width: number; height: number; radii: [number, number, number, number] };
/// ```
#[derive(Clone, Copy, Debug)]
pub(crate) struct RoundedBox {
    /// What: `pub x: f32,` is the left edge. `f32` is a 32-bit floating point number (siblings: `f64`, `i32`).
    /// Why: layout positions are `f32` in Slint.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// x: number;
    /// ```
    pub x: f32,
    /// What: `pub y: f32,` is the top edge, the same type as `x`.
    /// Why: layout positions are `f32` in Slint.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// y: number;
    /// ```
    pub y: f32,
    /// What: `pub width: f32,` is the width.
    /// Why: layout sizes are `f32` in Slint.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// width: number;
    /// ```
    pub width: f32,
    /// What: `pub height: f32,` is the height.
    /// Why: layout sizes are `f32` in Slint.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// height: number;
    /// ```
    pub height: f32,
    /// What: `pub radii: [f32; 4],` holds the radii of the top left, top right, bottom right and bottom left corners. `[f32; 4]` is an
    /// array of exactly four `f32` values (siblings: `Vec<f32>` is growable, `&[f32]` is a borrowed view).
    /// Why: there are always four corners, so a fixed array fits and needs no heap; it also keeps `RoundedBox` `Copy`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// radii: [number, number, number, number];
    /// ```
    pub radii: [f32; 4],
}

/// What: `impl RoundedBox { ... }` opens the block that holds the methods of `RoundedBox`.
/// Why: building and measuring a box belong to the type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RoundedBox { /* the methods of the impl block */ }
/// ```
impl RoundedBox {
    /// What: `pub(crate) fn new(x: f32, y: f32, width: f32, height: f32, radius: f32) -> Self` makes a box with the same radius at every
    /// corner.
    /// Why: most boxes in the tests are uniformly rounded.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static create(x: number, y: number, width: number, height: number, radius: number): RoundedBox { ... }
    /// ```
    pub(crate) fn new(x: f32, y: f32, width: f32, height: f32, radius: f32) -> Self {
        // What:     `return Self { x, y, width, height, radii: [radius; 4] };` builds the record. Bare names (`x`, `y`, `width`, `height`)
        //           are shorthand for `x: x` and so on. `[radius; 4]` is the repeat form of an array literal: four copies of `radius`.
        // Why:      Every corner gets the same radius.
        // Gotcha:   `[radius; 4]` is "value; count", not a one-item array that holds the number 4.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { x, y, width, height, radii: [radius, radius, radius, radius] };
        // ```
        return Self { x, y, width, height, radii: [radius; 4] };
    }

    /// What: `pub(crate) fn grown(&self, amount: f32) -> Self` gives the box grown by an amount on every side, the way D117 grows the
    /// element for each ring: every corner's radius grows by the amount, so a square corner becomes a corner of that radius when the box
    /// grows and stays square when it shrinks.
    /// Why: the near ring's outer edge is the element grown by 2 px and the outer ring's outer edge is the element grown by 3 px,
    /// each with its corner radii grown the same, as Firefox's `PaintAutoStyleOutline` does.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// grown(amount: number): RoundedBox { /* offset each edge by amount; each radius by amount, never below 0 */ }
    /// ```
    pub(crate) fn grown(&self, amount: f32) -> Self {
        // What:     `let grow = |radius: f32| return if radius > 0.0 || amount > 0.0 { (radius + amount).max(0.0) } else { 0.0 };` defines a
        //           closure (an anonymous function) named `grow`. `|radius: f32|` is its parameter list with a type. The body is an `if`
        //           expression used as the returned value: a corner that is rounded, or any growth, gives `radius + amount` raised to zero
        //           when it would be negative (`.max(0.0)` picks the larger of two floats); a square corner that shrinks stays `0.0`. The
        //           closure also uses `amount`, which it captures from the method.
        // Why:      A square corner becomes rounded when the box grows, and stays square when it shrinks.
        // Gotcha:   `0.0` is a float literal; writing `0` would not compile, because floats and integers never mix without a cast.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const grow = (radius: number) => (radius > 0 || amount > 0 ? Math.max(radius + amount, 0) : 0);
        // ```
        let grow = |radius: f32| return if radius > 0.0 || amount > 0.0 { (radius + amount).max(0.0) } else { 0.0 };
        // What:     `return Self { x: self.x - amount, ..., radii: self.radii.map(grow) };` builds the grown box. Each edge moves out by
        //           `amount`, and the width and height gain `2.0 * amount`. `self.radii.map(grow)` applies the closure to each of the four
        //           radii of the array and answers a new array of the same length; `.map` on an array is eager, unlike on an iterator.
        // Why:      Growing by `amount` on every side moves the left and top edges out and widens the box by two amounts.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { x: this.x - amount, y: this.y - amount, width: this.width + 2 * amount, height: this.height + 2 * amount, radii: this.radii.map(grow) };
        // ```
        return Self {
            x: self.x - amount,
            y: self.y - amount,
            width: self.width + 2.0 * amount,
            height: self.height + 2.0 * amount,
            radii: self.radii.map(grow),
        };
    }

    /// What: `pub(crate) fn distance(&self, px: f32, py: f32) -> f32` gives the signed distance from a point to the box's edge: negative
    /// inside, positive outside.
    /// Why: a ring pixel is one between the outer and the inner edge, and the tests need a margin around both edges where the
    /// anti-aliased pixels are not judged.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// distance(px: number, py: number): number { /* signed distance to a rounded rectangle */ }
    /// ```
    pub(crate) fn distance(&self, px: f32, py: f32) -> f32 {
        // What:     `let center_x = self.x + self.width / 2.0;` computes the horizontal middle of the box. `/ 2.0` is float division.
        // Why:      The signed-distance formula works from the centre.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const centerX = this.x + this.width / 2;
        // ```
        let center_x = self.x + self.width / 2.0;
        // What:     `let center_y = self.y + self.height / 2.0;` computes the vertical middle of the box.
        // Why:      Same as `center_x`, for the other axis.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const centerY = this.y + this.height / 2;
        // ```
        let center_y = self.y + self.height / 2.0;
        // What:     `let right = px >= center_x;` stores a comparison result in a `bool`. `>=` is the same operator as in TypeScript.
        // Why:      The side of the centre tells which corner's radius applies.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const right: boolean = px >= centerX;
        // ```
        let right = px >= center_x;
        // What:     `let bottom = py >= center_y;` stores the vertical comparison.
        // Why:      Same as `right`, for the other axis.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const bottom: boolean = py >= centerY;
        // ```
        let bottom = py >= center_y;
        // What:     `let radius = match (right, bottom) { (false, false) => self.radii[0], ... };` picks the radius of the corner nearest the
        //           point. `(right, bottom)` is a tuple of two booleans, and each arm lists one combination as a tuple pattern. The
        //           arms are indexed by `self.radii[n]`, the array of four radii in the order top left, top right, bottom right,
        //           bottom left.
        // Why:      Each corner may have its own radius.
        // Gotcha:   `match` must cover every combination; the four arms are all four combinations of two booleans.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const radius = this.radii[(bottom ? 2 : 0) + (right !== bottom ? 1 : 0)]; // same four-way choice
        // ```
        let radius = match (right, bottom) {
            (false, false) => self.radii[0],
            (true, false) => self.radii[1],
            (true, true) => self.radii[2],
            (false, true) => self.radii[3],
        };
        // What:     `let qx = (px - center_x).abs() - (self.width / 2.0 - radius);` is the horizontal part of the rounded-box distance.
        //           `.abs()` is the absolute value of a float.
        // Why:      Folding the point into one quadrant lets one formula serve all four corners.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const qx = Math.abs(px - centerX) - (this.width / 2 - radius);
        // ```
        let qx = (px - center_x).abs() - (self.width / 2.0 - radius);
        // What:     `let qy = (py - center_y).abs() - (self.height / 2.0 - radius);` is the vertical part.
        // Why:      Same as `qx`, for the other axis.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const qy = Math.abs(py - centerY) - (this.height / 2 - radius);
        // ```
        let qy = (py - center_y).abs() - (self.height / 2.0 - radius);
        // What:     `return qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - radius;` combines the two parts. `.max(0.0)` raises a negative
        //           part to zero, `.hypot(b)` is the length of the hypotenuse of the two sides (the square root of the sum of squares), and
        //           `.max(qy).min(0.0)` is the larger of `qx` and `qy` capped at zero. Subtracting `radius` turns the distance to the inner
        //           rectangle into the distance to the rounded edge.
        // Why:      This is the standard signed-distance function of a rounded rectangle; it is negative inside, zero on the edge and
        //           positive outside.
        // Gotcha:   `qx.max(0.0).hypot(...)` chains methods on a float; there is no `Math.hypot(a, b)` free function in Rust.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return Math.hypot(Math.max(qx, 0), Math.max(qy, 0)) + Math.min(Math.max(qx, qy), 0) - radius;
        // ```
        return qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - radius;
    }

    /// What: `pub(crate) fn near_a_curve(&self, px: f32, py: f32, margin: f32) -> bool` tells whether a point is within a margin of one of
    /// the box's rounded corners.
    /// Why: the software renderer approximates the coverage of an arc, so a pixel on a curve is only roughly its band's color,
    /// while a pixel on a straight edge is exact; a test judges the two differently.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// nearACurve(px: number, py: number, margin: number): boolean { /* inside a corner square and the radius is not zero */ }
    /// ```
    pub(crate) fn near_a_curve(&self, px: f32, py: f32, margin: f32) -> bool {
        // What:     `let center_x = self.x + self.width / 2.0;` computes the horizontal middle of the box.
        // Why:      The corner is chosen by the side of the centre the point lies on.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const centerX = this.x + this.width / 2;
        // ```
        let center_x = self.x + self.width / 2.0;
        // What:     `let center_y = self.y + self.height / 2.0;` computes the vertical middle of the box.
        // Why:      Same as `center_x`, for the other axis.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const centerY = this.y + this.height / 2;
        // ```
        let center_y = self.y + self.height / 2.0;
        // What:     `let radius = match (px >= center_x, py >= center_y) { ... };` is the same four-way choice as in `distance`, with the two
        //           comparisons written inline inside the tuple instead of in named variables.
        // Why:      The radius of the nearest corner decides how big the curved zone is.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const radius = this.radii[(py >= centerY ? 2 : 0) + ((px >= centerX) !== (py >= centerY) ? 1 : 0)];
        // ```
        let radius = match (px >= center_x, py >= center_y) {
            (false, false) => self.radii[0],
            (true, false) => self.radii[1],
            (true, true) => self.radii[2],
            (false, true) => self.radii[3],
        };
        // What:     `return radius > 0.0 && (px - center_x).abs() > ... && (py - center_y).abs() > ...;` is one boolean over three lines. `&&` is
        //           "and". The point is near a curve when the corner has a radius and the point lies in the corner square plus the margin,
        //           on both axes.
        // Why:      A corner with radius zero has no curve, and a point away from the corner square lies on a straight edge.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return radius > 0 && Math.abs(px - centerX) > this.width / 2 - radius - margin && Math.abs(py - centerY) > this.height / 2 - radius - margin;
        // ```
        return radius > 0.0
            && (px - center_x).abs() > self.width / 2.0 - radius - margin
            && (py - center_y).abs() > self.height / 2.0 - radius - margin;
    }
}
