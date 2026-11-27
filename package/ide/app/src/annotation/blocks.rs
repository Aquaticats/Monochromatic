//! The blocks of virtual rows the store's hints and diagnostics call for,
//!  one per annotated line.
//!
//! Three things decide a block:
//!  the hints accumulated for the displayed revision,
//!  the message rows of its
//! diagnostics,
//!  and space held over from the previous revision after an external change.
//!  Assembly packs hint
//! rows with the production shaper,
//!  so a block's height is exactly what a frame will paint.

/// The block of each annotated line.
mod assemble;
/// The hint labels of each line,
/// and the space a reload holds open.
mod hints;
/// The message rows a line's diagnostics need.
mod message_rows;
