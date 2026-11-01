/**
 OKLCH to sRGB conversion utilities.
 
 OKLCH (OKLab in polar form) is a perceptually uniform colour space:
 equal numerical deltas correspond to roughly equal perceived colour
 differences, unlike sRGB or HSL. Interpolating between two colours in
 OKLCH produces a visually smooth ramp; the same interpolation done in
 sRGB passes through muddy mid-tones (e.g. red↔green through brown).
 
 The OKLab → linear sRGB step uses the matrix from Björn Ottosson's
 "A perceptual color space for image processing" (2020). See
 https://bottosson.github.io/posts/oklab/.
 
 @example
 ```ts
 import { oklchLerpToSrgb } from './oklch.ts';
 
 const red = { L: 0.65, C: 0.22, H: 29 };
 const green = { L: 0.74, C: 0.20, H: 145 };
 const mid = oklchLerpToSrgb({ start: red, end: green, t: 0.5 });
 ```
 */

//region Types

/**
 Three-component OKLCH colour. `L` ∈ [0, 1] is lightness, `C` ∈ [0, ~0.4] is chroma, `H` is hue in degrees (0, 360).
 */
export type Oklch = {
  readonly L: number;
  readonly C: number;
  readonly H: number;
};

/**
 Clamped 8-bit sRGB triple.
 */
export type Rgb8 = readonly [
  number,
  number,
  number,
];

//endregion Types

//region Constants

/**
 Degrees in a half-circle; the denominator of the radians conversion.
 */
const HALF_CIRCLE_DEGREES = 180;
/**
 Degrees-to-radians factor; π/180.
 */
const DEG_TO_RAD = Math.PI
  / HALF_CIRCLE_DEGREES;
/**
 sRGB encoding piecewise boundary; values below take the linear segment.
 */
const SRGB_LINEAR_THRESHOLD = 0.0031308;
/**
 sRGB encoding linear-segment slope (standard).
 */
const SRGB_LINEAR_SLOPE = 12.92;
/**
 sRGB encoding gamma exponent (standard, ≈1/0.4167).
 */
const SRGB_GAMMA_EXPONENT = 2.4;
/**
 Reciprocal of {@link SRGB_GAMMA_EXPONENT}; used directly in the
 exponentiation.
 */
const SRGB_GAMMA_EXPONENT_RECIP = 1 / SRGB_GAMMA_EXPONENT;
/**
 sRGB encoding offset constant (standard).
 */
const SRGB_GAMMA_OFFSET = 0.055;
/**
 sRGB encoding gain constant (standard).
 */
const SRGB_GAMMA_GAIN = 1.055;
/**
 RGB channel maximum used for clamping and scaling.
 */
const RGB_MAX = 255;

//endregion Constants

//region Helpers

/**
 Encodes one linear-sRGB channel through the sRGB transfer curve.
 
 @param x - Linear-sRGB channel value, typically in `[0, 1]`.
 
 @returns sRGB channel value, typically in `[0, 1]`.
 */
function linearToSrgb(x: number,): number {
  if (x <= SRGB_LINEAR_THRESHOLD)
    return SRGB_LINEAR_SLOPE * x;
  return (SRGB_GAMMA_GAIN * (x ** SRGB_GAMMA_EXPONENT_RECIP)) - SRGB_GAMMA_OFFSET;
}

/**
 Clamps a value to `[0, 1]`, scales to `[0, 255]`, rounds to an integer.
 
 @param x - Source value.
 
 @returns Integer in `[0, 255]`.
 */
function to8Bit(x: number,): number {
  /**
   Source value pinned to the valid `[0, 1]` range before scaling.
   */
  const clamped = Math.min(
    1,
    Math.max(
      0,
      x,
    ),
  );
  return Math.round(clamped * RGB_MAX,);
}

//endregion Helpers

//region Public API

/**
 Converts an OKLCH colour to a clamped 8-bit sRGB tuple.
 
 Out-of-gamut OKLCH inputs (e.g. very saturated greens that don't
 exist in sRGB) are clamped per-channel, not via a proper
 gamut-mapping algorithm; the rendered colour may shift hue. Pick
 endpoint chroma values that stay inside sRGB to avoid this.
 
 @param color - Source colour.
 
 @returns `[r, g, b]` with each component in `[0, 255]`.
 
 @example
 ```ts
 oklchToSrgb({ L: 0.65, C: 0.22, H: 29 }); // → red-ish
 ```
 */
export function oklchToSrgb(color: Oklch,): Rgb8 {
  /**
   Polar OKLCH components broken out for the trig conversion to Cartesian OKLab.
   */
  const {
    L,
    C,
    H,
  } = color;
  /**
   OKLab `a` axis (green→red); the Cartesian projection of chroma at the hue angle.
   */
  const a = C * Math
    .cos(H * DEG_TO_RAD,);
  /**
   OKLab `b` axis (blue→yellow); the Cartesian projection of chroma at the hue angle.
   */
  const b = C * Math
    .sin(H * DEG_TO_RAD,);

  /* oxlint-disable eslint/no-magic-numbers, stylistic/no-mixed-operators -- Ottosson 2020 OKLab→linear-sRGB matrix coefficients; naming or regrouping each obscures the math. */
  /**
   Ottosson stage 1: long-cone response in pre-cube-root form.
   */
  const lPrime = L + 0.3963377774
    * a
    + 0.2158037573
      * b;
  /**
   Ottosson stage 1: medium-cone response in pre-cube-root form.
   */
  const mPrime = L - 0.1055613458
    * a
    - 0.0638541728
      * b;
  /**
   Ottosson stage 1: short-cone response in pre-cube-root form.
   */
  const sPrime = L - 0.0894841775
    * a
    - 1.291485548
      * b;

  /**
   Ottosson stage 2: long-cone response cubed; undoes the OKLab cube-root step.
   */
  const lCubed = lPrime * lPrime
    * lPrime;
  /**
   Ottosson stage 2: medium-cone response cubed.
   */
  const mCubed = mPrime * mPrime
    * mPrime;
  /**
   Ottosson stage 2: short-cone response cubed.
   */
  const sCubed = sPrime * sPrime
    * sPrime;

  /**
   Ottosson stage 3: linear-sRGB red channel projected from cone responses.
   */
  const rLin = 4.0767416621 * lCubed
    - 3.3077115913
      * mCubed
    + 0.2309699292
      * sCubed;
  /**
   Ottosson stage 3: linear-sRGB green channel.
   */
  const gLin = -1.2684380046 * lCubed
    + 2.6097574011
      * mCubed
    - 0.3413193965
      * sCubed;
  /**
   Ottosson stage 3: linear-sRGB blue channel.
   */
  const bLin = -0.0041960863 * lCubed
    - 0.7034186147
      * mCubed
    + 1.707614701
      * sCubed;
  /* oxlint-enable eslint/no-magic-numbers, stylistic/no-mixed-operators */

  return [
    to8Bit(linearToSrgb(rLin,),),
    to8Bit(linearToSrgb(gLin,),),
    to8Bit(linearToSrgb(bLin,),),
  ];
}

/**
 Linearly interpolates two OKLCH colours by `t`, then converts to
 8-bit sRGB.
 
 Hue is interpolated along the direct numeric path; this is the
 shortest arc only when `Math.abs(end.H - start.H) ≤ 180`. For wider
 hue gaps this passes through the wrong side of the colour wheel.
 
 @param start - Colour returned when `t === 0`.
 
 @param end - Colour returned when `t === 1`.
 
 @param t - Interpolation parameter; values outside `[0, 1]`
   extrapolate linearly.
 
 @returns `[r, g, b]` with each component in `[0, 255]`.
 
 @example
 ```ts
 const red = { L: 0.65, C: 0.22, H: 29 };
 const green = { L: 0.74, C: 0.20, H: 145 };
 oklchLerpToSrgb({ start: red, end: green, t: 0 });    // red
 oklchLerpToSrgb({ start: red, end: green, t: 0.5 });  // amber
 oklchLerpToSrgb({ start: red, end: green, t: 1 });    // green
 ```
 */
export function oklchLerpToSrgb(
  {
    start,
    end,
    t,
  }: {
    readonly start: Oklch;
    readonly end: Oklch;
    readonly t: number;
  },
): Rgb8 {
  return oklchToSrgb({
    L: start.L
      + (t * (end.L
        - start
          .L)),
    C: start.C
      + (t * (end.C
        - start
          .C)),
    H: start.H
      + (t * (end.H
        - start
          .H)),
  },);
}

//endregion Public API
