//! The nested screen's size and output scale,
//!  as plain numbers independent of any display.
//!
//! The caller sets a logical size (`--size`,
//!  `resize`) and an output scale (`--scale`,
//!  `scale`).
//! The physical framebuffer,
//!  which screenshots and recordings capture,
//!  is the logical size
//! multiplied by the scale and rounded half away from zero,
//!  the rounding
//! `wp_fractional_scale_v1` prescribes for toplevel surfaces.
//!  Changing only the scale keeps the
//! logical size,
//!  the way a window keeps its size when it moves to an output with another
//! scale,
//!  so a hosted client sees a scale change without a resize.
//!  Input coordinates stay
//! logical at every scale.

/// What:
///      `use smithay::output::Scale;`.
///  Smithay's description of an output scale:
///  an
///           `Integer`,
///  a `Fractional`,
///  or a `Custom` pair of both.
/// Why:
///       `advertised` returns the value Smithay turns into the `wl_output.scale` and
///           `xdg_output.logical_size` events,
///  so tests can check what clients are told.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Scale } from "smithay/output";
/// ```
use smithay::output::Scale;

/// Denominator of every fractional scale on the wire.
///
/// What:
///      `pub const SCALE_DENOMINATOR: u32 = 120;`.
///  `u32` is an unsigned 32-bit integer
///           (siblings:
///  signed `i32`,
///  wider `u64`).
/// Why:
///       `wp_fractional_scale_v1.preferred_scale` carries "the numerator of a fraction
///           with a denominator of 120",
///  so a scale is exact only in 120ths.
///  `u32` matches
///           that event's `uint` argument,
///  and a scale is never negative.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SCALE_DENOMINATOR = 120;
/// ```
pub const SCALE_DENOMINATOR: u32 = 120;

/// Smallest accepted scale,
///  0.5,
///  in 120ths.
///
/// What:
///      `pub const MIN_SCALE_120THS: u32 = 60;`.
/// Why:
///       KDE's display settings offer 50 % to 300 % in 5 % steps of 120ths
///           (`KDE/kscreen` `kcm/ui/OutputPanel.qml` lines 113 to 132 at `3df1d1238e`),
///  so
///           this range covers every scale a real KDE output can have.
pub const MIN_SCALE_120THS: u32 = 60;

/// Largest accepted scale,
///  3.0,
///  in 120ths.
///
/// What:
///      `pub const MAX_SCALE_120THS: u32 = 360;`.
/// Why:
///       The upper end of the same KDE range;
///  it also bounds the framebuffer to three
///           times the logical size.
pub const MAX_SCALE_120THS: u32 = 360;

/// How far a typed scale may sit from a whole number of 120ths and still name that step.
///
/// What:
///      `const STEP_TOLERANCE: f64 = 0.01;`.
///  `f64` is a 64-bit float,
///  like TS `number`
///           (sibling:
///  32-bit `f32`).
/// Why:
///       Steps such as 4/3 have no finite decimal spelling;
///  `1.3333` is within 0.004
///           of step 160 and is accepted as it,
///  while `1.333` (0.04 away) is rejected.
const STEP_TOLERANCE: f64 = 0.01;

/// An output scale stored as a whole number of 120ths.
///
/// What:
///      `pub struct OutputScale { per_120: u32 }`.
///  A record with one private field.
///           `#[derive(...)]` asks the compiler to generate copying,
///  debug printing,
///  and
///           equality comparison.
/// Why:
///       Storing the wire numerator keeps every comparison exact;
///  a float field would
///           make `1.25 == 1.25` depend on how each value was computed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OutputScale = { readonly per120: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OutputScale {
    /// Scale numerator over `SCALE_DENOMINATOR`;
    ///  `120` means 1.
    per_120: u32,
}

/// Parsing and conversions for output scales.
///
/// What:
///      `impl OutputScale { ... }`.
///  Methods attached to the record type.
/// Why:
///       Keep the one grammar and the wire conversions beside the stored value.
impl OutputScale {
    /// The unscaled output, 1.
    ///
    /// What:
    ///      `pub const ONE: Self = Self { per_120: SCALE_DENOMINATOR };`.
    ///  A named
    ///           constant value of this type.
    /// Why:
    ///       The default for `--scale`,
    ///  matching the behavior before scaling existed.
    pub const ONE: Self = Self { per_120: SCALE_DENOMINATOR };

    /// Parse a decimal scale such as `1`,
    ///  `1.25`,
    ///  or `2`.
    ///
    /// What:
    ///      `pub fn parse(text: &str) -> Result<Self, String>`.
    ///  Borrows the text and
    ///           returns the scale or a message.
    ///  `Result<T, String>` is success-or-failure,
    ///           because Rust has no exceptions;
    ///  `String` lets the message travel straight
    ///           back as a control-socket `err` line or a CLI error.
    /// Why:
    ///       The `--scale` option and the `scale` control verb share one grammar.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function parse(text: string): OutputScale { ... } // throws a string
    /// ```
    ///
    /// @example
    /// ```ts
    /// parse("1.25"); // => { per120: 150 }
    /// ```
    pub fn parse(text: &str) -> Result<Self, String> {
        // What:     `text.parse::<f64>()` reads the text as a 64-bit float and returns
        //           `Result<f64, ParseFloatError>`. `match` branches on it: `Ok(value)` binds
        //           the number, `Err(_)` discards the parser's own error. `format!` builds an
        //           owned `String` with `{text}` interpolated.
        // Why:      Name the rejected text and the accepted form instead of a parser detail.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const value = Number(text);
        // if (text.trim() === "" || Number.isNaN(value)) throw `scale must be a number ...`;
        // ```
        let value = match text.parse::<f64>() {
            Ok(number) => number,
            Err(_) => return Err(format!("scale must be a decimal number such as 1, 1.25, or 2, got: {text}")),
        };

        // What:     `.is_finite()` is false for infinity and NaN, which the float parser
        //           accepts as the words `inf` and `NaN`.
        // Why:      Neither can be converted to 120ths.
        if !value.is_finite() {
            return Err(format!("scale must be a finite number such as 1, 1.25, or 2, got: {text}"));
        }

        // What:     `value * f64::from(SCALE_DENOMINATOR)` converts to 120ths; `f64::from`
        //           widens the `u32` losslessly. `.round()` rounds half away from zero.
        // Why:      The nearest whole step is the only candidate this text can name.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const steps = value * 120;
        // const nearest = Math.sign(steps) * Math.round(Math.abs(steps));
        // ```
        let steps = value * f64::from(SCALE_DENOMINATOR);
        let nearest = steps.round();

        // What:     `(steps - nearest).abs()` is the distance to that step.
        // Why:      Refuse text that names no step, and tell the caller the two neighbors.
        if (steps - nearest).abs() > STEP_TOLERANCE {
            let below = steps.floor() / f64::from(SCALE_DENOMINATOR);
            let above = steps.ceil() / f64::from(SCALE_DENOMINATOR);
            return Err(format!(
                "scale must be a whole number of 120ths, the unit wp_fractional_scale_v1 sends, \
                 got: {text}; the nearest scales are {below:.4} and {above:.4}"
            ));
        }

        // What:     `nearest < f64::from(MIN_SCALE_120THS)` compares two floats. The range
        //           check runs on the float, before any integer conversion, so a huge value
        //           cannot wrap.
        // Why:      Keep the scale inside the range a KDE output can have.
        if nearest < f64::from(MIN_SCALE_120THS) || nearest > f64::from(MAX_SCALE_120THS) {
            return Err(format!("scale must be between 0.5 and 3, got: {text}"));
        }

        // What:     `nearest as u32` converts the in-range whole float to `u32`. `Ok(Self {
        //           per_120: ... })` builds the record and wraps it in the success variant.
        // Why:      The value is whole and within 60 to 360, so the conversion is exact.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { per120: nearest };
        // ```
        return Ok(Self { per_120: nearest as u32 });
    }

    /// The scale numerator over 120,
    ///  the value `wp_fractional_scale_v1.preferred_scale` sends.
    ///
    /// What:
    ///      `pub fn per_120(self) -> u32`.
    ///  Takes the value by copy (the type is `Copy`).
    /// Why:
    ///       Tests and the e2e check compare it with the client's protocol log.
    #[must_use]
    pub fn per_120(self) -> u32 {
        return self.per_120;
    }

    /// The scale as a float factor,
    ///  for example `1.25`.
    ///
    /// What:
    ///      `pub fn factor(self) -> f64`.
    ///  Divides the numerator by 120.
    /// Why:
    ///       Smithay's fractional-scale state and size arithmetic take a float.
    #[must_use]
    pub fn factor(self) -> f64 {
        return f64::from(self.per_120) / f64::from(SCALE_DENOMINATOR);
    }

    /// The scale Smithay advertises to clients for this value.
    ///
    /// What:
    ///      `pub fn advertised(self) -> Scale`.
    ///  Returns `Scale::Fractional(factor)`,
    ///  one
    ///           variant of Smithay's scale enum.
    /// Why:
    ///       Smithay sends `wl_output.scale` as this value rounded up to an integer and
    ///           `xdg_output.logical_size` as the mode divided by it;
    ///  the fractional value
    ///           itself reaches clients through `wp_fractional_scale_v1`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// advertised(): Scale { return { kind: "fractional", value: this.factor() }; }
    /// ```
    #[must_use]
    pub fn advertised(self) -> Scale {
        return Scale::Fractional(self.factor());
    }
}

/// The nested screen:
///  a logical size plus the output scale.
///
/// What:
///      `pub struct ScreenGeometry { ... }` with two `i32` logical dimensions and an
///           `OutputScale`.
///  `i32` is signed 32-bit (sibling:
///  unsigned `u32`).
/// Why:
///       `i32` matches Smithay's geometry types and the existing `--size` parser,
///  so no
///           conversion is needed where the size meets the compositor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScreenGeometry = { logicalWidth: number; logicalHeight: number; scale: OutputScale };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenGeometry {
    /// Width the hosted client is configured with,
    ///  in logical pixels.
    pub logical_width: i32,
    /// Height the hosted client is configured with,
    ///  in logical pixels.
    pub logical_height: i32,
    /// Output scale every surface is told to render at.
    pub scale: OutputScale,
}

/// Size arithmetic between the logical screen,
///  its physical framebuffer,
///  and the parent window.
///
/// What:
///      `impl ScreenGeometry { ... }`.
/// Why:
///       Every conversion lives here so it is tested without a display.
impl ScreenGeometry {
    /// The physical framebuffer size:
    ///  the logical size times the scale,
    ///  rounded half away from zero.
    ///
    /// What:
    ///      `pub fn physical_size(self) -> (i32, i32)`.
    ///  Returns a two-element tuple.
    /// Why:
    ///       This is the buffer a client draws at this scale and the size screenshots take.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// physicalSize(): [number, number] { return [round(w * s), round(h * s)]; }
    /// ```
    ///
    /// @example
    /// ```ts
    /// ({ logicalWidth: 1100, logicalHeight: 660, scale: 1.25 }).physicalSize(); // => [1375, 825]
    /// ```
    #[must_use]
    pub fn physical_size(self) -> (i32, i32) {
        // What:     `f64::from(self.logical_width)` widens the `i32` to a float without loss;
        //           `.round()` rounds half away from zero; `as i32` converts back, saturating
        //           at the `i32` limits instead of wrapping.
        // Why:      `wp_fractional_scale_v1` defines toplevel buffer sizes with this rounding.
        let width = (f64::from(self.logical_width) * self.scale.factor()).round() as i32;
        let height = (f64::from(self.logical_height) * self.scale.factor()).round() as i32;
        return (width, height);
    }

    /// The parent-window size to request,
    ///  in the parent's logical pixels,
    ///  rounded up.
    ///
    /// What:
    ///      `pub fn parent_request(self, parent_scale: f64) -> (u32, u32)`.
    ///  The parent
    ///           compositor sizes the nested window in its own logical pixels,
    ///  then multiplies
    ///           by its own scale (the host output's,
    ///  unrelated to this screen's scale).
    /// Why:
    ///       Rounding up guarantees the framebuffer is never smaller than
    ///           `physical_size`,
    ///  so no client pixel is cut off;
    ///  at most a strip narrower than
    ///           the parent's scale shows the background at the right and bottom edges.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// parentRequest(parentScale: number): [number, number] {
    ///   const [w, h] = this.physicalSize();
    ///   return [Math.ceil(w / parentScale), Math.ceil(h / parentScale)];
    /// }
    /// ```
    #[must_use]
    pub fn parent_request(self, parent_scale: f64) -> (u32, u32) {
        let divisor = usable_parent_scale(parent_scale);
        let (width, height) = self.physical_size();
        // `as u32` saturates: a negative or oversized float becomes 0 or `u32::MAX`.
        let request_width = (f64::from(width) / divisor).ceil() as u32;
        let request_height = (f64::from(height) / divisor).ceil() as u32;
        return (request_width, request_height);
    }

    /// Whether a framebuffer of this size is the parent's rounding of `physical_size`.
    ///
    /// What:
    ///      `pub fn fits_framebuffer(self, framebuffer: (i32, i32), parent_scale: f64) ->
    ///           bool`.
    ///  Allows each dimension to differ by up to the parent's scale rounded up.
    /// Why:
    ///       A parent at scale 2 or 3 can only produce multiples of its scale,
    ///  so an exact
    ///           match is impossible for some sizes;
    ///  anything farther off means the parent
    ///           kept its own size (for example a maximized window).
    #[must_use]
    pub fn fits_framebuffer(self, framebuffer: (i32, i32), parent_scale: f64) -> bool {
        let tolerance = usable_parent_scale(parent_scale).ceil() as i32;
        let (width, height) = self.physical_size();
        // `abs_diff` returns the unsigned distance between two signed integers without overflow.
        let width_distance = framebuffer.0.abs_diff(width);
        let height_distance = framebuffer.1.abs_diff(height);
        return width_distance <= tolerance.unsigned_abs() && height_distance <= tolerance.unsigned_abs();
    }

    /// The geometry after the parent window changed size by itself.
    ///
    /// What:
    ///      `pub fn after_parent_resize(self, framebuffer: (i32, i32), parent_scale: f64)
    ///           -> Self`.
    ///  Keeps this geometry when the new framebuffer is only the parent's
    ///           rounding;
    ///  otherwise derives the logical size from the framebuffer.
    /// Why:
    ///       A parent-initiated resize (a user dragging the window edge,
    ///  a tiling
    ///           layout) changes the screen,
    ///  but rounding must never nudge the logical size by
    ///           a pixel and so resize the hosted client for nothing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// afterParentResize(framebuffer, parentScale) {
    ///   if (this.fitsFramebuffer(framebuffer, parentScale)) return this;
    ///   return { ...this, logicalWidth: round(framebuffer[0] / s), logicalHeight: ... };
    /// }
    /// ```
    #[must_use]
    pub fn after_parent_resize(self, framebuffer: (i32, i32), parent_scale: f64) -> Self {
        if self.fits_framebuffer(framebuffer, parent_scale) {
            return self;
        }
        let factor = self.scale.factor();
        // `.max(1)` keeps a degenerate framebuffer from producing a zero-sized screen.
        let logical_width = ((f64::from(framebuffer.0) / factor).round() as i32).max(1);
        let logical_height = ((f64::from(framebuffer.1) / factor).round() as i32).max(1);
        return Self { logical_width, logical_height, scale: self.scale };
    }
}

/// The parent's scale,
///  or 1 when the value cannot divide a size.
///
/// What:
///      `fn usable_parent_scale(parent_scale: f64) -> f64`.
///  Private helper.
/// Why:
///       winit reports a positive finite scale,
///  but a zero or NaN here would turn a size
///           request into infinity;
///  1 is the scale a parent has before it reports one.
fn usable_parent_scale(parent_scale: f64) -> f64 {
    if parent_scale.is_finite() && parent_scale > 0.0 {
        return parent_scale;
    }
    return 1.0;
}
