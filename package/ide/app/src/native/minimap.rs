//! The window's side of the minimap,
//! the scroll control beside the source text.
//!
//! The picture of the whole text is built on the minimap worker thread (`minimap_worker.rs`);
//! this module posts a
//!  request when what the picture is valid for changes,
//! takes in the finished pixels with one image setter,
//!  and turns a press,
//! a drag,
//! and a page step into scroll offsets with `ide_app::minimap::Geometry`.
//! The viewport box is not touched here:
//! the markup places it from the scroll offset alone,
//! so a scroll step costs
//!  this module nothing.
//!
//! The parts are split by what they answer for:
//!  `state.rs` holds the state and the drag,
//!  `pace.rs` decides when a request is posted,
//!  `request.rs` makes the key and posts the request,
//!  `picture.rs` takes in the finished pictures,
//!  and `pointer.rs` turns a press,
//! a drag,
//! and a page step into scroll offsets.

/// What the window keeps for the minimap besides the picture,
///  and the drag of the viewport box.
mod state;
/// The state the window holds.
pub(super) use state::MinimapState;

/// When a picture is requested:
///  the pure decision from two keys and the clock.
mod pace;
/// The pure decision and its periods are held against every kind of change by the tests.
#[cfg(test)]
pub(super) use pace::{DEBOUNCE, Plan, SLIDE, THROTTLE, plan};

/// Posting a request to the worker when what the picture is valid for changes.
mod request;
/// Called after every render and when the minimap is resized.
pub(super) use request::note;

/// Taking in the finished pictures.
mod picture;
/// The tests hand finished pictures to the window directly.
#[cfg(test)]
pub(super) use picture::{PICTURE_FRAMES_ASKED, apply};

/// A press on the minimap,
///  mapped again through the layout that follows it while nothing else has moved the view.
mod aim;
/// What the view's placement and the vertical mapping ask of a remembered press.
pub(super) use aim::{aim_again, forget_unless};

/// The minimap as a scroll control:
///  press,
///  drag,
///  step,
///  and resize.
mod pointer;
/// Binds the minimap's callbacks at startup.
pub(super) use pointer::bind;
