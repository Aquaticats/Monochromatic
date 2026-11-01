//! UI-side progress debounce bridge.
//!
//! This module sits in the binary crate because it talks to generated Slint
//! types.
//!  The pure timing rule lives in `music_player::progress`;
//!  this file
//! wires that rule to the on-screen seek bar and KDE taskbar progress.

/// What:
///      `use std::sync::{LazyLock, Mutex};`.
///  `LazyLock` is a value built on first use (the UI clock start below).
///  A standard-library lock that lets one thread mutate
///           a value at a time.
///  Sibling:
///  `RwLock`,
///  which has separate read and write
///           locking.
/// Why:
///       The update callback crosses from the engine thread to the UI thread,
///  so the
///           debounce state is shared behind a lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Mutex<T> ~ a shared value you must lock() before touching
/// ```
use std::sync::{LazyLock, Mutex};

/// What: `use std::cell::RefCell;`. A single-threaded cell that allows mutation through a shared reference (sibling:
/// `Cell<T>`, which only copies values in and out).
/// Why: the taskbar state machine and the Windows COM cache both sit in a `thread_local!`, which hands out `&` only.
use std::cell::RefCell;

/// What:
///      `use std::time::{Duration, Instant};`.
///  A standard-library elapsed time span.
///  `Instant` is a timestamp (the UI clock start).
/// Why:
///       The caller passes elapsed time since startup into the debouncer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Duration = number; // milliseconds
/// ```
use std::time::{Duration, Instant};

/// What:
///      `use music_player::command::Update;`.
///  The engine-to-UI update enum.
/// Why:
///       This bridge classifies `Position`,
///  `NowPlaying`,
///  and `Playing` updates.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Update } from "music-player/command";
/// ```
use music_player::command::Update;

/// What:
///      `use music_player::launcher::Launcher;`.
///  The KDE taskbar-progress signal
///           helper.
/// Why:
///       Decided taskbar values are delivered through it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Launcher } from "music-player/launcher";
/// ```
use music_player::launcher::Launcher;

/// What: `use music_player::launcher::taskbar_state::{TaskbarInput, TaskbarProgress, TaskbarUpdate};`. The taskbar
/// state machine and its input and output types.
/// Why: this module feeds every engine message to the machine and delivers what it decides.
use music_player::launcher::taskbar_state::{TaskbarInput, TaskbarProgress, TaskbarUpdate};

/// What:
///      `use music_player::progress::{ProgressDebouncer, ProgressUpdateKind};`.
///  The
///           pure debounce state and update-kind enum.
/// Why:
///       Keep timing policy tested in the library while this module handles UI
///           plumbing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ProgressDebouncer, ProgressUpdateKind } from "music-player/progress";
/// ```
use music_player::progress::{ProgressDebouncer, ProgressUpdateKind};

/// What:
///      `use crate::{apply_update, AppWindow};`.
///  Import the parent binary module's
///           generated Slint window type and existing update applier.
/// Why:
///       The generated `AppWindow` type only exists in this binary crate,
///  not in the
///           library crate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { applyUpdate, AppWindow } from "./main";
/// ```
use crate::ui_screen::apply_update;
use crate::AppWindow;

// What: `thread_local! { static TASKBAR_STATE: RefCell<TaskbarProgress> = ... }`. The one state machine behind the
// taskbar bar, kept on the UI thread (`thread_local!` gives each thread its own copy; `RefCell` allows mutation through
// a shared reference).
// Why: every engine message reaches this module on the UI thread, in the order the engine sent it, so one machine
// sees the whole history and the taskbar value never depends on what the window currently shows.
thread_local! {
    static TASKBAR_STATE: RefCell<TaskbarProgress> = const { RefCell::new(TaskbarProgress::new()) };
}

/// What: `static UI_CLOCK_START: LazyLock<Instant>`. The moment this module first handled an update.
/// Why: the taskbar rate limit runs on the time an update is handled on the UI thread, not on the time the engine
/// sent it; after a UI stall the backlog is replayed at once, and engine time would let every queued tick through.
static UI_CLOCK_START: LazyLock<Instant> = LazyLock::new(Instant::now);

/// What: `fn send_taskbar_update(app: &AppWindow, launcher: &Launcher, update: TaskbarUpdate)`. Push one decided value
/// to the KDE LauncherEntry signal and, on Windows, to the native taskbar.
/// Why: the decision is already made by the state machine; this only delivers it.
fn send_taskbar_update(app: &AppWindow, launcher: &Launcher, update: TaskbarUpdate) {
    let fraction = update.fraction();
    tracing::debug!(percent = update.percent, visible = update.visible, "taskbar progress delivered to the launcher");
    launcher.set_progress(fraction, update.visible);
    // The Windows taskbar takes the same fraction and visibility.
    #[cfg(windows)]
    set_windows_taskbar_progress(app, fraction, update.visible);
    // Off Windows `app` has no use here; this keeps the parameter list identical on every platform.
    #[cfg(not(windows))]
    let _ = app;
}

/// What: `fn drive_taskbar(app: &AppWindow, launcher: &Launcher, update: &Update) -> bool`. Feed one
/// engine message to the taskbar state machine, deliver what it decides, and say whether the message was a seek.
/// Why: the taskbar sees every message, before and independent of the seek bar's rate limit, so a track change and
/// its position reset can never be dropped from the taskbar; the seek flag lets the caller keep the seek bar in step.
fn drive_taskbar(app: &AppWindow, launcher: &Launcher, update: &Update) -> bool {
    let handled_at = UI_CLOCK_START.elapsed();
    let (input, decided) = TASKBAR_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        let input = state.input_for(update);
        let decided = state.apply_update(update, handled_at);
        return (input, decided);
    });
    if let Some(decided_update) = decided {
        send_taskbar_update(app, launcher, decided_update);
    }
    return matches!(input, Some(TaskbarInput::Seek { .. }));
}

/// What:
///      `fn should_surface_progress(progress_debouncer: &Mutex<ProgressDebouncer>, elapsed: Duration, kind: ProgressUpdateKind) -> bool`.
///           Lock the shared debouncer and ask whether this update should repaint the seek bar now.
/// Why:
///       The update callback crosses threads,
///  so the state lives behind a mutex.
fn should_surface_progress(
    progress_debouncer: &Mutex<ProgressDebouncer>,
    elapsed: Duration,
    kind: ProgressUpdateKind,
) -> bool {
    let mut debouncer = progress_debouncer
        .lock()
        .expect("progress debouncer lock should not be poisoned");
    return debouncer.should_surface(elapsed, kind)
}

/// What:
///      `pub(crate) fn apply_update_with_progress_debounce(app: &AppWindow, launcher: &Launcher, progress_debouncer: &Mutex<ProgressDebouncer>, elapsed: Duration, update: Update)`.
///           Apply one engine update to the window, rate-limit the seek bar through `ProgressDebouncer`, and drive the
///           taskbar through its own state machine.
/// Why:
///       The taskbar value is decided from the engine's messages alone (see `launcher::taskbar_state`), so the seek
///           bar's gate cannot drop a value the taskbar needs, and the seek bar still updates at a human-visible cadence.
pub(crate) fn apply_update_with_progress_debounce(
    app: &AppWindow,
    launcher: &Launcher,
    progress_debouncer: &Mutex<ProgressDebouncer>,
    elapsed: Duration,
    update: Update,
) {
    // The taskbar is fed first and from every message; the window below is only a mirror.
    let is_seek = drive_taskbar(app, launcher, &update);
    match &update {
        // A tick is rate-limited; a seek always passes so the seek bar lands where the user clicked.
        Update::Position(_) => {
            let kind = if is_seek {
                ProgressUpdateKind::Immediate
            } else {
                ProgressUpdateKind::Debounced
            };
            if should_surface_progress(progress_debouncer, elapsed, kind) {
                apply_update(app, &update);
            } else {
                tracing::trace!("seek bar tick held for the rate limit");
            }
        }
        // A new track resets the seek bar together with the new duration. The engine's own `Position(0)` follows, but
        // the gate may drop it, which would leave the old position drawn against the new duration.
        Update::NowPlaying { .. } => {
            let _ = should_surface_progress(progress_debouncer, elapsed, ProgressUpdateKind::Immediate);
            apply_update(app, &update);
            apply_update(app, &Update::Position(0.0));
        }
        Update::Playing(_) => {
            let _ = should_surface_progress(progress_debouncer, elapsed, ProgressUpdateKind::Immediate);
            apply_update(app, &update);
        }
        _ => apply_update(app, &update),
    }
}

//region Windows taskbar progress (ITaskbarList3)
// Purpose: the Windows-native equivalent of the Linux LauncherEntry progress bar.
// Windows exposes per-window taskbar progress through the ITaskbarList3 COM
// interface, so this region creates that interface once per UI thread and pushes
// the same fraction/visibility `send_taskbar_update` receives from the state machine. It is
// compiled only on Windows; every item is `#[cfg(windows)]`.

/// What:
///      `use windows::Win32::Foundation::HWND;`.
///  The Win32 window-handle type (a
///           newtype around a raw pointer to the window).
/// Why:
///       ITaskbarList3's methods take the target window's `HWND`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HWND = number; // an opaque window handle
/// ```
#[cfg(windows)]
use windows::Win32::Foundation::HWND;

/// What:
///      `use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx};`.
///           COM bootstrap items:
///  `CoInitializeEx` initializes COM on the thread;
///           `CoCreateInstance` builds a COM object by class id;
///  `CLSCTX_INPROC_SERVER`
///           asks for an in-process implementation;
///  `COINIT_APARTMENTTHREADED` is the
///           single-threaded-apartment mode winit's window thread already uses.
/// Why:
///       Needed to construct the taskbar COM object on the UI thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED } from "windows-com";
/// ```
#[cfg(windows)]
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
};

/// What:
///      `use windows::Win32::UI::Shell::{ITaskbarList3, TBPF_NOPROGRESS, TBPF_NORMAL, TaskbarList};`.
///           `ITaskbarList3` is the COM INTERFACE (a fat pointer to vtable methods);
///           `TaskbarList` is the CLASS ID (`GUID`) of the concrete shell object that
///           implements it;
///  `TBPF_NORMAL`/`TBPF_NOPROGRESS` are progress-state flags (show
///           a normal bar / hide the bar).
/// Why:
///       These drive the taskbar progress bar.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ITaskbarList3, TaskbarList, TBPF_NORMAL, TBPF_NOPROGRESS } from "windows-shell";
/// ```
#[cfg(windows)]
use windows::Win32::UI::Shell::{ITaskbarList3, TBPF_NOPROGRESS, TBPF_NORMAL, TaskbarList};

// What:     `thread_local! { static TASKBAR: RefCell<Option<ITaskbarList3>> = const { RefCell::new(None) }; }`.
//           A per-thread static holding the cached COM interface, or `None` until first
//           use. `thread_local!` gives each thread its OWN copy; `const { ... }` is a
//           compile-time initializer (no lazy runtime init needed).
// Why:      `ITaskbarList3` is a single-threaded-apartment COM object (`!Send`), so it
//           must never cross threads; a `thread_local` on the UI thread is the natural
//           home, and caching it avoids recreating it on every progress tick.
//
// In TS you'd write (pseudocode):
// ```ts
// // per-UI-thread cache: let taskbar: ITaskbarList3 | null = null;
// ```
#[cfg(windows)]
thread_local! {
    static TASKBAR: RefCell<Option<ITaskbarList3>> = const { RefCell::new(None) };
}

/// What:
///      `fn window_hwnd(app: &AppWindow) -> Option<HWND>`.
///  Resolve the running
///           window's Win32 handle,
///  or `None` if it is not yet realized or not a
///           winit/Win32 window.
///  Module-private,
///  Windows-only.
/// Why:
///       The handle exists only once the event loop has created the window,
///  which is
///           why this is resolved lazily on each progress tick rather than at startup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function windowHwnd(app: AppWindow): HWND | null { ... }
/// ```
#[cfg(windows)]
fn window_hwnd(app: &AppWindow) -> Option<HWND> {
    /// What:
    ///      `use slint::ComponentHandle;`.
    ///  Brings the `.window()` accessor into scope
    ///           (a trait method is callable only when its trait is imported).
    /// Why:
    ///       `app.window()` returns the `slint::Window` the next call needs.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// import { ComponentHandle } from "slint";
    /// ```
    use slint::ComponentHandle;
    /// What:
    ///      `use i_slint_backend_winit::WinitWindowAccessor;`.
    ///  The extension trait
    ///           adding `.with_winit_window(...)` to `slint::Window` on the winit backend.
    /// Why:
    ///       It exposes the underlying winit window,
    ///  from which the raw handle comes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// import { WinitWindowAccessor } from "slint-winit-backend";
    /// ```
    use i_slint_backend_winit::WinitWindowAccessor;
    /// What:
    ///      `use i_slint_backend_winit::winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};`.
    ///           `HasWindowHandle` provides `.window_handle()`;
    ///  `RawWindowHandle` is the
    ///           per-platform handle enum.
    ///  Imported through winit's OWN re-export so the
    ///           `raw-window-handle` version matches winit's exactly (no separate dep).
    /// Why:
    ///       Needed to read and match the platform window handle.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// import { HasWindowHandle, RawWindowHandle } from "winit/raw-window-handle";
    /// ```
    use i_slint_backend_winit::winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    // What:     `app.window().with_winit_window(|w| { ... }).flatten()`. `with_winit_window`
    //           runs the closure with the live winit `Window`, returning
    //           `Some(closure_result)` or `None` if there is no winit window. The closure
    //           itself returns `Option<HWND>`, so the outer result is `Option<Option<HWND>>`;
    //           `.flatten()` collapses it to `Option<HWND>`. Tail expression -> return value.
    // Why:      Reach into the winit window to read its raw Win32 handle.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return (app.window().withWinitWindow((w) => ...) ?? null);
    // ```
    app.window()
        .with_winit_window(|w| {
            // What:     `match w.window_handle().ok()?.as_raw() { ... }`. `.window_handle()`
            //           returns `Result<WindowHandle, _>`; `.ok()?` yields the handle or
            //           returns `None` from this closure on error. `.as_raw()` converts it
            //           to the `RawWindowHandle` enum.
            // Why:      Inspect which platform handle this is.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const raw = w.windowHandle(); if (!raw) return null;
            // ```
            match w.window_handle().ok()?.as_raw() {
                // What:     `RawWindowHandle::Win32(handle) => Some(HWND(handle.hwnd.get() as *mut core::ffi::c_void))`.
                //           On Windows the handle is the `Win32` variant; `handle.hwnd` is a
                //           `NonZeroIsize`, `.get()` reads the raw `isize`, and `as *mut
                //           core::ffi::c_void` turns it into the pointer the `windows`
                //           crate's `HWND(*mut c_void)` newtype wraps.
                // Why:      Build the `HWND` ITaskbarList3 needs.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (raw.kind === "win32") return new HWND(raw.hwnd);
                // ```
                RawWindowHandle::Win32(handle) => {
                    Some(HWND(handle.hwnd.get() as *mut core::ffi::c_void))
                }
                // What:     `_ => None`. Any other platform variant (cannot happen in a
                //           Windows build, but the match must be exhaustive).
                // Why:      Only the Win32 handle is usable here.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // return null;
                // ```
                _ => None,
            }
        })
        .flatten()
}

/// What:
///      `fn create_taskbar_list() -> Option<ITaskbarList3>`.
///  Initialize COM on this
///           thread and build the taskbar COM object,
///  or `None` on any failure.
///           Module-private,
///  Windows-only.
/// Why:
///       Done once and cached;
///  pulled into its own function so `set_..._progress` reads
///           cleanly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function createTaskbarList(): ITaskbarList3 | null { ... }
/// ```
#[cfg(windows)]
fn create_taskbar_list() -> Option<ITaskbarList3> {
    // What:     `unsafe { ... }`. COM calls are raw FFI, so the whole body is `unsafe`
    //           (Rust cannot verify the COM contracts). `unsafe` means "trust me", not
    //           "dangerous".
    // Why:      Required to call `CoInitializeEx` / `CoCreateInstance`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // no equivalent: raw platform calls
    // ```
    unsafe {
        // What:     `let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);`. Ensure COM
        //           is initialized on this thread in single-threaded-apartment mode. winit
        //           already initializes OLE (STA) on its window thread, so this typically
        //           returns `S_FALSE` (already initialized) and just bumps the init count;
        //           `let _ =` discards the returned `HRESULT`.
        // Why:      `CoCreateInstance` requires an initialized apartment; calling this
        //           defensively makes the code correct even if winit's init changes.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // coInitializeEx(null, "apartmentThreaded"); // ignore result
        // ```
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        // What:     `let taskbar: ITaskbarList3 = CoCreateInstance(&TaskbarList, None, CLSCTX_INPROC_SERVER).ok()?;`.
        //           Create the shell taskbar object and ask for its `ITaskbarList3`
        //           interface. `&TaskbarList` is the class id; `None` means no aggregating
        //           outer object; `CLSCTX_INPROC_SERVER` loads it in-process. Returns
        //           `Result<ITaskbarList3>`; `.ok()?` yields the interface or returns `None`
        //           on failure.
        // Why:      This object is what actually moves the taskbar bar.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const taskbar = coCreateInstance(TaskbarList, null, "inproc"); if (!taskbar) return null;
        // ```
        let taskbar: ITaskbarList3 =
            CoCreateInstance(&TaskbarList, None, CLSCTX_INPROC_SERVER).ok()?;
        // What:     `taskbar.HrInit().ok()?;`. Required one-time initialization of the
        //           taskbar interface. Returns `Result<()>`; `.ok()?` returns `None` on
        //           failure.
        // Why:      ITaskbarList must be `HrInit`-ed before other methods are called.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!taskbar.hrInit()) return null;
        // ```
        taskbar.HrInit().ok()?;
        // What:     `Some(taskbar)`. Wrap the ready interface as present. Tail expression ->
        //           return value.
        // Why:      Hand the cached interface back to the caller.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return taskbar;
        // ```
        Some(taskbar)
    }
}

/// What:
///      `fn set_windows_taskbar_progress(app: &AppWindow, fraction: f64, visible: bool)`.
///           Push the current progress to the Windows taskbar bar.
///  Module-private,
///           Windows-only.
///  Runs on the UI thread.
/// Why:
///       The Windows counterpart to the Linux `Launcher::set_progress`;
///  called from
///           `emit_launcher_progress` under `#[cfg(windows)]`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function setWindowsTaskbarProgress(app, fraction, visible): void { ... }
/// ```
#[cfg(windows)]
fn set_windows_taskbar_progress(app: &AppWindow, fraction: f64, visible: bool) {
    // What:     `let hwnd = match window_hwnd(app) { Some(h) => h, None => return };`.
    //           Resolve the window handle, or bail out silently if it is not ready.
    // Why:      Without a handle there is nothing to drive; best-effort like the Linux path
    //           (a missing handle never disrupts playback).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const hwnd = windowHwnd(app); if (!hwnd) return;
    // ```
    let hwnd = match window_hwnd(app) {
        Some(h) => h,
        None => return,
    };
    // What:     `TASKBAR.with(|cell| { ... })`. Access this thread's cached COM interface
    //           cell. `with` runs the closure with a `&RefCell<...>`.
    // Why:      Reuse the one interface instead of recreating it each tick.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // withThreadLocal(TASKBAR, (cell) => { ... });
    // ```
    TASKBAR.with(|cell| {
        // What:     `let needs_init = cell.borrow().is_none();`. `.borrow()` takes a shared
        //           runtime borrow; `.is_none()` reads whether the cache is still empty,
        //           releasing the borrow at the end of THIS statement (so the `borrow_mut`
        //           below does not overlap it).
        // Why:      Decide whether to build the interface without holding a borrow across
        //           the mutation.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const needsInit = cell.value === null;
        // ```
        let needs_init = cell.borrow().is_none();
        // What:     `if needs_init { *cell.borrow_mut() = create_taskbar_list(); }`. Build
        //           and store the interface on first use (or after a prior failed attempt).
        //           `*cell.borrow_mut() = ...` writes through the runtime-checked MUTABLE
        //           borrow (the `*` dereferences the guard to assign the inner value).
        // Why:      Lazily create the COM object the first time progress is shown.
        // Gotcha:   `borrow_mut` panics if another borrow is live; that is why `needs_init`
        //           was read and released first.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (needsInit) cell.value = createTaskbarList();
        // ```
        if needs_init {
            *cell.borrow_mut() = create_taskbar_list();
        }
        // What:     `let guard = cell.borrow();`. Take a shared runtime borrow to read the
        //           cached interface.
        // Why:      Need a reference to call methods on the interface.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const taskbar = cell.value;
        // ```
        let guard = cell.borrow();
        // What:     `if let Some(taskbar) = guard.as_ref() { ... }`. `.as_ref()` borrows the
        //           inner `Option<ITaskbarList3>` as `Option<&ITaskbarList3>`; proceed only
        //           when the interface exists (creation may have failed, leaving `None`).
        // Why:      A failed creation is retried next tick; never panic on it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (taskbar) { ... }
        // ```
        if let Some(taskbar) = guard.as_ref() {
            // What:     `unsafe { ... }`. The COM method calls are raw FFI.
            // Why:      Required to call ITaskbarList3 methods.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // raw platform calls
            // ```
            unsafe {
                // What:     `if visible { ... } else { ... }`. Show a moving bar when
                //           playing, otherwise clear it.
                // Why:      The bar should appear only while real-duration audio plays.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (visible) { ... } else { ... }
                // ```
                if visible {
                    // What:     `let total: u64 = 1000;`. A fixed denominator for the
                    //           progress ratio. `u64` is the unsigned 64-bit integer
                    //           SetProgressValue takes (siblings: `u32`, `usize`).
                    // Why:      ITaskbarList3 wants completed/total as integers, so we scale
                    //           the 0..1 fraction onto a 0..1000 range.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // const total = 1000;
                    // ```
                    let total: u64 = 1000;
                    // What:     `let completed = (fraction.clamp(0.0, 1.0) * total as f64) as u64;`.
                    //           `.clamp(0.0, 1.0)` pins the fraction into 0..=1, `* total as
                    //           f64` scales to the denominator (widening `total` for the
                    //           multiply), and `as u64` truncates the result back to an
                    //           integer.
                    // Why:      Convert the float fraction into the integer numerator.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // const completed = Math.floor(Math.min(1, Math.max(0, fraction)) * total);
                    // ```
                    let completed = (fraction.clamp(0.0, 1.0) * total as f64) as u64;
                    // What:     `let _ = taskbar.SetProgressState(hwnd, TBPF_NORMAL);`. Put
                    //           the bar in the normal (green, determinate) state. Returns
                    //           `Result<()>`; `let _ =` ignores failures.
                    // Why:      A no-progress window must switch to a normal bar before a
                    //           value is shown.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // taskbar.setProgressState(hwnd, "normal");
                    // ```
                    let _ = taskbar.SetProgressState(hwnd, TBPF_NORMAL);
                    // What:     `let _ = taskbar.SetProgressValue(hwnd, completed, total);`.
                    //           Move the bar to `completed/total`. Result ignored.
                    // Why:      This is the actual progress position.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // taskbar.setProgressValue(hwnd, completed, total);
                    // ```
                    let _ = taskbar.SetProgressValue(hwnd, completed, total);
                } else {
                    // What:     `let _ = taskbar.SetProgressState(hwnd, TBPF_NOPROGRESS);`.
                    //           Clear the bar (no progress shown). Result ignored.
                    // Why:      Hide the bar when paused or between tracks.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // taskbar.setProgressState(hwnd, "noProgress");
                    // ```
                    let _ = taskbar.SetProgressState(hwnd, TBPF_NOPROGRESS);
                }
            }
        }
    });
}
//endregion
