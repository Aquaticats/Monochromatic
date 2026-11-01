/**
 * Repeating corridor furniture: mullion columns, ceiling light strips,
 * and floor seams. These are the parallax anchors that make the constant
 * forward walk readable; the floor and ceiling slabs alone give no motion
 * cue.
 *
 * The world stands still and the camera moves. Segments recycle from
 * behind the camera to the far end, so a fixed pool covers an endless
 * corridor.
 */
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  BoxGeometry,
  Group,
  Mesh,
  MeshStandardMaterial,
  type Scene,
} from 'three/webgpu';

import { WORLD_TUNING, } from './scene.ts';

/**
 * Logger root shared by the glass demo client modules.
 */
const parentLogger = tagged({ tag: 'web-demo-glass', },);

/**
 * Tagged logger for the corridor system.
 */
const l = tagged({
  tag: 'corridor',
  l: parentLogger,
},);

/**
 * Corridor segment layout constants.
 */
export const CORRIDOR_TUNING = {
  /**
   * Distance between segments in meters.
   */
  spacing: 3.4,
  /**
   * Segment count in the pool; spacing times count is the visible run.
   */
  segmentCount: 20,
  /**
   * How far ahead of the camera recycled segments reappear, meters.
   */
  aheadDistance: 62,
  /**
   * How far behind the camera a segment may trail before recycling.
   */
  behindDistance: 6,
  /**
   * Wall column cross-section, meters.
   */
  column: {
    width: 0.14,
    depth: 0.22,
  },
  /**
   * Floor-level skirting glow cross-section and coverage.
   */
  skirt: {
    size: 0.05,
    inset: 0.1,
    coverage: 0.86,
  },
  /**
   * Ceiling light strip dimensions and drop below the ceiling.
   */
  strip: {
    widthFactor: 1.3,
    height: 0.04,
    depth: 0.18,
    drop: 0.03,
  },
  /**
   * Floor seam line dimensions.
   */
  seam: {
    height: 0.012,
    depth: 0.05,
    lift: 0.006,
  },
} as const;

/**
 * Corridor system handle: recycle segments as the camera advances.
 */
export type CorridorSystem = {
  /**
   * Moves segments that fell behind the camera to the far end.
   *
   * @param cameraZ - camera world z this frame
   */
  readonly recycle: (cameraZ: number,) => void;
};

/**
 * Builds one corridor segment: two wall columns, a ceiling light strip,
 * a floor seam line, and low skirting glows.
 *
 * @param structural - shared dark-metal material for columns and seams
 *
 * @param strip - shared emissive material for the ceiling light
 *
 * @param accent - shared dimmer emissive material for skirting
 *
 * @returns segment group ready for placement
 *
 * @example
 * ```ts
 * const segment = buildSegment({ structural, strip, accent },);
 * ```
 */
function buildSegment(
  {
    structural,
    strip,
    accent,
  }: Readonly<{
    structural: MeshStandardMaterial;
    strip: MeshStandardMaterial;
    accent: MeshStandardMaterial;
  }>,
): Group {
  /**
   * Segment root positioned along z by the recycler.
   */
  const segment = new Group();
  for (const side of [
    -1,
    1,
  ]) {
    /**
     * Wall column at this side of the corridor.
     */
    const column = new Mesh(
      new BoxGeometry(
        CORRIDOR_TUNING.column
          .width,
        WORLD_TUNING.ceilingHeight,
        CORRIDOR_TUNING.column
          .depth,
      ),
      structural,
    );
    column.position
      .set(
      side * WORLD_TUNING.corridorHalfWidth,
      WORLD_TUNING.ceilingHeight / 2,
      0,
    );
    segment.add(column,);
    /**
     * Low skirting glow strip hugging the floor beside the column.
     */
    const skirt = new Mesh(
      new BoxGeometry(
        CORRIDOR_TUNING.skirt
          .size,
        CORRIDOR_TUNING.skirt
          .size,
        CORRIDOR_TUNING.spacing
          * CORRIDOR_TUNING.skirt
          .coverage,
      ),
      accent,
    );
    skirt.position
      .set(
        side * (WORLD_TUNING.corridorHalfWidth
          - CORRIDOR_TUNING.skirt
          .inset),
        CORRIDOR_TUNING.skirt
          .size,
        (-CORRIDOR_TUNING.spacing) / 2,
      );
    segment.add(skirt,);
  }
  /**
   * Ceiling light strip across the corridor.
   */
  const light = new Mesh(
    new BoxGeometry(
      WORLD_TUNING.corridorHalfWidth
        * CORRIDOR_TUNING.strip
        .widthFactor,
      CORRIDOR_TUNING.strip
        .height,
      CORRIDOR_TUNING.strip
        .depth,
    ),
    strip,
  );
  light.position
    .set(
      0,
      WORLD_TUNING.ceilingHeight
        - CORRIDOR_TUNING.strip
        .drop,
      0,
    );
  segment.add(light,);
  /**
   * Floor seam line marking segment boundaries underfoot.
   */
  const seam = new Mesh(
    new BoxGeometry(
      WORLD_TUNING.corridorHalfWidth * 2,
      CORRIDOR_TUNING.seam
        .height,
      CORRIDOR_TUNING.seam
        .depth,
    ),
    structural,
  );
  seam.position
    .set(
      0,
      CORRIDOR_TUNING.seam
        .lift,
      0,
    );
  segment.add(seam,);
  return segment;
}

/**
 * Creates the corridor segment pool and its recycler.
 *
 * @param scene - scene receiving the segments
 *
 * @mutates scene - `scene.add(segment)` registers pooled segment groups in the caller's scene graph.
 *
 * @returns corridor system handle
 *
 * @example
 * ```ts
 * const corridor = createCorridor({ scene },);
 * corridor.recycle(camera.position.z,);
 * ```
 */
export function createCorridor(
  { scene, }: Readonly<{ scene: Scene; }>,
): CorridorSystem {
  /**
   * Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: createCorridor.name,
    l,
  },);
  /**
   * Dark anodized metal shared by columns and seams.
   */
  const structural = new MeshStandardMaterial({
    color: '#131a24',
    metalness: 0.85,
    roughness: 0.38,
  },);
  /**
   * Hot ceiling strip material; emissive intensity carries the corridor's
   * main visible light and matches the environment map's ceiling band.
   */
  const strip = new MeshStandardMaterial({
    color: '#e8f6ff',
    emissive: '#dff2ff',
    emissiveIntensity: 5,
    metalness: 0,
    roughness: 0.4,
  },);
  /**
   * Dim floor-level accent glow.
   */
  const accent = new MeshStandardMaterial({
    color: '#0d2233',
    emissive: '#2e7fb8',
    emissiveIntensity: 1.6,
    metalness: 0,
    roughness: 0.5,
  },);
  /**
   * Segment pool laid out at even spacing ahead of the start position.
   */
  const segments = Array.from(
    { length: CORRIDOR_TUNING.segmentCount, },
    function placeSegment(
      _ignored: unknown,
      index: number,
    ): Group {
      /**
       * One pooled segment.
       */
      const segment = buildSegment({
        structural,
        strip,
        accent,
      },);
      segment.position
        .z = (-index) * CORRIDOR_TUNING.spacing;
      scene.add(segment,);
      return segment;
    },
  );
  innerL.info(`corridor ready: ${String(segments.length,)} segments`,);
  return {
    recycle: function recycle(cameraZ: number,): void {
      for (const segment of segments)
        if (segment.position
          .z
          > (cameraZ
          + CORRIDOR_TUNING.behindDistance))
          segment.position
            .z -= CORRIDOR_TUNING.segmentCount * CORRIDOR_TUNING.spacing;
    },
  };
}
