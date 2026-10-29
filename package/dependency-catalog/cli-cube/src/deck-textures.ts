/**
 Per-probe canvas-texture factory for the mesh scatter layers.
 
 Bakes the probe's fill colour AND its npm name into a 2D canvas that
 deck.gl's `SimpleMeshLayer` consumes as `texture`. Texture sampling
 happens in the fragment shader, so the painted name is part of the
 mesh surface; front objects naturally occlude back objects, and
 faces rotating away from the camera hide their text the same way a
 real ball would.
 
 Why textures (and not a separate `TextLayer`): a billboard text
 layer at the glyph center is either occluded by the front face of
 its own mesh (`depthCompare: 'less-equal'`) or floats above every
 scene object regardless of position (`depthCompare: 'always'`). Both
 read as wrong; the user explicitly asked for "part of the mesh"
 semantics. Texture mapping is the standard way to get that.
 
 Sphere UV layout (equirectangular): the name is drawn in two
 horizontal stripes; one upright above the equator at `v ≈ 0.35`,
 one rotated 180° below the equator at `v ≈ 0.65`. Each stripe has
 two horizontal repetitions at `u = 0.25 / 0.75` for longitude
 coverage. The mix of orientations guarantees that, whatever
 combination of canvas-Y direction and texture-Y direction luma.gl
 actually chooses (it claims `flipY: false` but the empirical result
 on a sphere is that text drawn upright on the canvas reads
 upside-down on the sphere), at least one stripe is right-side-up
 from any camera rotation. The user proposed this when a single
 canvas-side flip kept rendering inverted on the sphere; the
 two-versions-per-texture approach sidesteps the orientation puzzle.
 
 Sphere font size auto-shrinks via `ctx.measureText` so even long
 names like `happy-rusty` fit inside their `TEXTURE_SIZE_PX /
 SPHERE_REPETITIONS` slot.
 
 Octahedron UV layout: every face is mapped to the same UV triangle
 `(0, 0), (1, 0), (0.5, 1)`. The name is drawn inside that triangle
 (once upright, once rotated 180°) for the same reason.
 
 The shape parameter selects which texture layout to use; spheres and
 octahedra have very different unwrappings.
 
 Textures are cached by `(probe.catalogKey, rgba, shape, withName)`
 so subsequent renders reuse the same canvas (and the same GPU
 texture upload) when nothing colour-relevant or name-bake-relevant
 has changed.
 
 @example
 ```ts
 import { makeProbeTexture } from './deck-textures.ts';
 const tex = makeProbeTexture({ probe, fillColor: [200, 60, 60, 255], shape: 'sphere', withName: true });
 new SimpleMeshLayer({ texture: tex, ... });
 ```
 */

import {
  HALF,
  QUARTER,
} from '@monochromatic-dev/const-core/ts';

import type { PackageProbe, } from './probe.ts';

//region Types

/**
 Mesh shape determines which UV layout the texture targets.
 
 Sphere: equirectangular projection (luma.gl `SphereGeometry`); name
 repeated horizontally so any longitude shows it.
 
 Octahedron: per-face `(0,0), (1,0), (0.5,1)` triangle; name drawn
 inside that triangle once.
 */
export type MeshShape = 'sphere' | 'octahedron';

/**
 RGBA tuple in `[0, 255]`.
 */
export type Rgba = readonly [
  number,
  number,
  number,
  number,
];

//endregion Types

//region Constants

/**
 Texture side length in pixels. Power-of-two for cleanest mipmaps.
 */
const TEXTURE_SIZE_PX = 512;
/**
 Maximum font size in pixels for the baked name; auto-shrunk on the sphere if the text overruns its slot.
 */
const FONT_SIZE_PX = 56;
/**
 Minimum font size in pixels; below this the text stops shrinking and just overflows; readability matters more than fit.
 */
const MIN_FONT_SIZE_PX = 22;
/**
 Black outline width in pixels around the white text fill.
 */
const OUTLINE_WIDTH_PX = 6;
/**
 Horizontal repetitions of the name around the sphere equator. Two copies (at u=0.25 and u=0.75) give visibility from any rotation while keeping each slot wide enough for readable text.
 */
const SPHERE_REPETITIONS = 2;
/**
 Vertical position of the upright stripe in texture-space; sits just above the equator.
 */
const SPHERE_UPRIGHT_V = 0.35;
/**
 Vertical position of the rotated stripe in texture-space; sits just below the equator. Together with the upright stripe, this guarantees a readable orientation from every camera angle regardless of which way the texture sampler ends up mapping canvas-Y to sphere-Y.
 */
const SPHERE_FLIPPED_V = 0.65;
/**
 Fraction of the slot width the text may occupy before auto-shrink kicks in. Leaves a small margin for the outline.
 */
const SPHERE_SLOT_FILL_FRACTION = 0.85;
/**
 Module-level cache: `(catalogKey, rgba, shape, withName)` → built canvas. Avoids re-rendering on every state recompute.
 */
const TEXTURE_CACHE = new Map<string, HTMLCanvasElement>();
/**
 Octahedron upright stripe centre, in normalised texture v space.
 */
const OCTAHEDRON_UPRIGHT_V = QUARTER;
/**
 Octahedron rotated stripe centre, in normalised texture v space. Below the upright copy so the face shows both within the `(0,0), (1,0), (0.5,1)` UV triangle.
 */
const OCTAHEDRON_FLIPPED_V = HALF;

//endregion Constants

//region Helpers

/**
 Builds the stable cache key for a (probe, colour, shape, withName)
 tuple.
 
 @param probe - Source probe.
 
 @param fillColor - RGBA fill colour.
 
 @param shape - Mesh shape.
 
 @param withName - Whether the name is baked into the texture.
 
 @returns Cache key string.
 */
function cacheKey(
  {
    probe,
    fillColor,
    shape,
    withName,
  }: {
    readonly probe: PackageProbe;
    readonly fillColor: Rgba;
    readonly shape: MeshShape;
    readonly withName: boolean;
  },
): string {
  return `${probe.catalogKey}|${fillColor.join(',',)}|${shape}|${withName ? '1' : '0'}`;
}

/**
 Fills the canvas with the base colour. Sets `fillStyle` to a CSS
 rgba string derived from the byte-RGBA tuple.
 
 @param ctx - Target context.
 
 @param fillColor - RGBA tuple, 0 to 255.
 */
function paintBackground(
  {
    ctx,
    fillColor,
  }: {
    readonly ctx: CanvasRenderingContext2D;
    readonly fillColor: Rgba;
  },
): void {
  /**
   Byte-RGBA channels destructured so each can be formatted into the CSS rgba string.
   */
  const [
    r,
    g,
    b,
    a,
  ] = fillColor;
  ctx.fillStyle = `rgba(${r.toString()}, ${g.toString()}, ${b.toString()}, ${
    (a / 255).toString()
  })`;
  ctx.fillRect(
    0,
    0,
    TEXTURE_SIZE_PX,
    TEXTURE_SIZE_PX,
  );
}

/**
 Draws the name with a black outline and white fill at a given
 canvas centre, in the natural canvas orientation.
 
 @param ctx - Target context.
 
 @param text - Name string.
 
 @param x - Horizontal centre in texture pixels.
 
 @param y - Vertical centre in texture pixels.
 */
function paintUpright(
  {
    ctx,
    text,
    x,
    y,
  }: {
    readonly ctx: CanvasRenderingContext2D;
    readonly text: string;
    readonly x: number;
    readonly y: number;
  },
): void {
  ctx.strokeText(
    text,
    x,
    y,
  );
  ctx.fillText(
    text,
    x,
    y,
  );
}

/**
 Draws the name with a black outline and white fill at a given
 canvas centre, rotated 180° around that centre; i.e. upside-down
 in canvas coords. Paired with {@link paintUpright} on the same
 texture so the sphere shows at least one readable copy from any
 camera angle.
 
 @param ctx - Target context.
 
 @param text - Name string.
 
 @param x - Horizontal centre in texture pixels.
 
 @param y - Vertical centre in texture pixels.
 */
function paintRotated180(
  {
    ctx,
    text,
    x,
    y,
  }: {
    readonly ctx: CanvasRenderingContext2D;
    readonly text: string;
    readonly x: number;
    readonly y: number;
  },
): void {
  ctx.save();
  ctx.translate(
    x,
    y,
  );
  ctx.rotate(Math.PI,);
  ctx.strokeText(
    text,
    0,
    0,
  );
  ctx.fillText(
    text,
    0,
    0,
  );
  ctx.restore();
}

/**
 Returns the largest font size between {@link MIN_FONT_SIZE_PX} and
 {@link FONT_SIZE_PX} that fits `text` inside the slot width times
 {@link SPHERE_SLOT_FILL_FRACTION}.
 
 Uses `ctx.measureText` at `FONT_SIZE_PX` and rescales proportionally
 ; text width is linear in font size for a given typeface, so one
 measurement is enough.
 
 @param ctx - Target context (must already have `font` set so subsequent measureText returns the correct width).
 
 @param text - The string that will be drawn.
 
 @param slotWidthPx - Width of the slot the text must fit inside.
 
 @returns Font size in pixels.
 */
function pickFontSize(
  {
    ctx,
    text,
    slotWidthPx,
  }: {
    readonly ctx: CanvasRenderingContext2D;
    readonly text: string;
    readonly slotWidthPx: number;
  },
): number {
  /**
   Measured width at the maximum font size; one measurement is enough since width scales linearly.
   */
  const measuredAtMax = ctx.measureText(text,)
    .width;
  /**
   Width budget after reserving the fill-fraction margin for the outline.
   */
  const targetWidth = slotWidthPx * SPHERE_SLOT_FILL_FRACTION;
  if (measuredAtMax <= targetWidth)
    return FONT_SIZE_PX;
  /**
   Proportionally-rescaled font size before clamping to the minimum.
   */
  const scaled = FONT_SIZE_PX * (targetWidth / measuredAtMax);
  return Math.max(
    MIN_FONT_SIZE_PX,
    scaled,
  );
}

//endregion Helpers

//region Public API

/**
 Builds (or retrieves from cache) the canvas texture for one probe.
 
 Sphere variant: repeats the name {@link SPHERE_REPETITIONS} times along
 the equator so the label is visible from any longitude.
 
 Octahedron variant: paints the name once inside the face triangle
 `(0,0), (1,0), (0.5,1)` (in normalised UV); every face of the
 octahedron maps to that triangle so the label shows on each face.
 
 @param probe - Source probe.
 
 @param fillColor - RGBA tuple matching the probe's data-derived colour at full alpha.
 
 @param shape - Mesh shape selecting the UV layout.
 
 @param withName - When `false`, the texture is colour-only (no name baked).
 
 @returns Canvas element ready to pass to `SimpleMeshLayer.texture`.
 
 @throws When `document.createElement('canvas').getContext('2d')` returns null (no Canvas2D support).
 
 @example
 ```ts
 const tex = makeProbeTexture({
   probe,
   fillColor: [200, 60, 60, 255],
   shape: 'sphere',
   withName: true,
 });
 ```
 */
export function makeProbeTexture(
  {
    probe,
    fillColor,
    shape,
    withName,
  }: {
    readonly probe: PackageProbe;
    readonly fillColor: Rgba;
    readonly shape: MeshShape;
    readonly withName: boolean;
  },
): HTMLCanvasElement {
  /**
   Stable cache identity for the texture inputs; reused on subsequent calls.
   */
  const key = cacheKey({
    probe,
    fillColor,
    shape,
    withName,
  },);
  /**
   Previously-built canvas for this key, or undefined on first build.
   */
  const cached = TEXTURE_CACHE.get(key,);
  if (cached !== undefined)
    return cached;
  /**
   Fresh canvas sized to the texture dimensions; populated by the painting steps below.
   */
  const canvas = document.createElement('canvas',);
  canvas.width = TEXTURE_SIZE_PX;
  canvas.height = TEXTURE_SIZE_PX;
  /**
   Drawing context for the new canvas; nullable when Canvas2D is unavailable.
   */
  const ctx = canvas.getContext('2d',);
  if (ctx === null)
    throw new Error('Canvas2D context unavailable',);
  paintBackground({
    ctx,
    fillColor,
  },);
  if (!withName) {
    TEXTURE_CACHE.set(
      key,
      canvas,
    );
    return canvas;
  }
  ctx.font = `700 ${FONT_SIZE_PX.toString()}px sans-serif`;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.lineJoin = 'round';
  ctx.strokeStyle = 'rgba(0, 0, 0, 1)';
  ctx.fillStyle = 'rgba(255, 255, 255, 1)';
  if (shape === 'sphere') {
    /**
     Per-repetition horizontal slot width; sets the auto-shrink budget.
     */
    const stepPx = TEXTURE_SIZE_PX / SPHERE_REPETITIONS;
    /**
     Font size that fits the longest text inside one slot, clamped to the minimum.
     */
    const fontSize = pickFontSize({
      ctx,
      text: probe.npmName,
      slotWidthPx: stepPx,
    },);
    ctx.font = `700 ${fontSize.toString()}px sans-serif`;
    ctx.lineWidth = OUTLINE_WIDTH_PX * (fontSize / FONT_SIZE_PX);
    /**
     Canvas-Y of the upright stripe; just above the equator.
     */
    const uprightYPx = SPHERE_UPRIGHT_V * TEXTURE_SIZE_PX;
    /**
     Canvas-Y of the rotated stripe; just below the equator.
     */
    const flippedYPx = SPHERE_FLIPPED_V * TEXTURE_SIZE_PX;
    /**
     Per-repetition horizontal centres; one stamp pair per offset along the equator.
     */
    const offsets = Array.from(
      {
        length: SPHERE_REPETITIONS,
      },
      function asOffset(
        _,
        i,
      ) {
        return (i + HALF) * stepPx;
      },
    );
    for (const xPx of offsets) {
      paintUpright({
        ctx,
        text: probe.npmName,
        x: xPx,
        y: uprightYPx,
      },);
      paintRotated180({
        ctx,
        text: probe.npmName,
        x: xPx,
        y: flippedYPx,
      },);
    }
  }
  else {
    ctx.lineWidth = OUTLINE_WIDTH_PX;
    /**
     Canvas-X centre of the single octahedron stamp; horizontal midpoint of the texture.
     */
    const centreX = TEXTURE_SIZE_PX * HALF;
    /**
     Canvas-Y of the upright stripe.
     
     Octahedron faces map to UV triangle `(0,0), (1,0), (0.5,1)`.
     Two stripes inside the triangle; upright at `v = 1/4` and
     rotated 180° at `v = 1/2`: so a reader sees at least one
     readable orientation per face regardless of how the texture
     winds up oriented on the sphere.
     */
    const uprightYPx = OCTAHEDRON_UPRIGHT_V * TEXTURE_SIZE_PX;
    /**
     Canvas-Y of the rotated stripe; pairs with `uprightYPx` for orientation coverage.
     */
    const flippedYPx = OCTAHEDRON_FLIPPED_V * TEXTURE_SIZE_PX;
    paintUpright({
      ctx,
      text: probe.npmName,
      x: centreX,
      y: uprightYPx,
    },);
    paintRotated180({
      ctx,
      text: probe.npmName,
      x: centreX,
      y: flippedYPx,
    },);
  }
  TEXTURE_CACHE.set(
    key,
    canvas,
  );
  return canvas;
}

//endregion Public API
