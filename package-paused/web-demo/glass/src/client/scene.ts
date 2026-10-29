/**
 * Renderer, camera, and static world bootstrap.
 *
 * Glass realism is mostly reflections, so the environment gets the
 * budget: a procedural equirect map full of bright strips and hot spots
 * that streak across transmissive panes, matched by visible emissive
 * geometry in the corridor so reflections and world agree.
 */
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  CanvasTexture,
  Color,
  EquirectangularReflectionMapping,
  Fog,
  HemisphereLight,
  Mesh,
  MeshStandardMaterial,
  NeutralToneMapping,
  PerspectiveCamera,
  PlaneGeometry,
  PointLight,
  Scene,
  SRGBColorSpace,
  WebGPURenderer,
} from 'three/webgpu';

import { paintEnvironmentCanvas, } from './environment-paint.ts';

/**
 * Logger root shared by the glass demo client modules.
 */
const parentLogger = tagged({ tag: 'web-demo-glass', },);

/**
 * Tagged logger for scene bootstrap.
 */
const l = tagged({
  tag: 'scene',
  l: parentLogger,
},);

/**
 * World dimensions and walk parameters shared across systems.
 */
export const WORLD_TUNING = {
  /**
   * Corridor half width in meters; walls sit at +-this x.
   */
  corridorHalfWidth: 3,
  /**
   * Ceiling height in meters above the floor at y = 0.
   */
  ceilingHeight: 3.2,
  /**
   * Player eye height in meters.
   */
  eyeHeight: 1.62,
  /**
   * Constant forward walk speed in m/s along -z.
   */
  walkSpeed: 3,
  /**
   * Fog color and far plane matched so the corridor fades, not pops.
   */
  fogFar: 58,
  /**
   * Fog near distance in meters.
   */
  fogNear: 14,
  /**
   * Upper bound on device pixel ratio to keep transmission passes cheap.
   */
  maxPixelRatio: 1.5,
} as const;

/**
 * Deep-night backdrop color shared by fog, background, and clear color.
 */
export const NIGHT_COLOR = '#04070e';

/**
 * Camera, light, and slab rig numbers.
 */
const SCENE_RIG = {
  /**
   * Vertical field of view in degrees; wide enough to feel first person.
   */
  fov: 58,
  /**
   * Near clip plane in meters.
   */
  near: 0.08,
  /**
   * Far clip plane in meters.
   */
  far: 120,
  /**
   * Tone mapping exposure.
   */
  exposure: 1.15,
  /**
   * Hemisphere fill intensity.
   */
  hemisphereIntensity: 0.65,
  /**
   * Key light intensity in candela-ish three.js units.
   */
  keyIntensity: 42,
  /**
   * Key light physical decay exponent.
   */
  keyDecay: 2,
  /**
   * Key light offset from the camera, meters.
   */
  keyOffset: {
    x: -1.2,
    y: 1.1,
    z: -2.5,
  },
  /**
   * Floor and ceiling slab length along the corridor, meters.
   */
  slabLength: 220,
} as const;

/**
 * Everything the demo loop needs from the bootstrap.
 */
export type SceneKit = {
  /**
   * WebGPU renderer, already initialized.
   */
  readonly renderer: WebGPURenderer;
  /**
   * Scene owning all world objects.
   */
  readonly scene: Scene;
  /**
   * Walking first-person camera.
   */
  readonly camera: PerspectiveCamera;
  /**
   * True when the WebGPU backend is active, false on the WebGL2 fallback.
   */
  readonly usingWebGpu: boolean;
};


/**
 * Builds the static floor and ceiling slabs. Both are long planes the
 * loop re-centers under the camera each frame; the moving detail
 * (light strips, mullions) belongs to the corridor segments.
 *
 * @param scene - scene receiving the slabs
 *
 * @mutates scene - `scene.add(floor)` and `scene.add(ceiling)` register the slabs in the caller's scene graph.
 *
 * @returns floor and ceiling meshes for per-frame re-centering
 *
 * @example
 * ```ts
 * const slabs = buildSlabs(scene);
 * ```
 */
function buildSlabs(scene: Scene,): {
  readonly floor: Mesh;
  readonly ceiling: Mesh;
} {
  /**
   * Polished dark floor: glossy enough to mirror the strip lights.
   */
  const floor = new Mesh(
    new PlaneGeometry(
      WORLD_TUNING.corridorHalfWidth * 2,
      SCENE_RIG.slabLength,
    ),
    new MeshStandardMaterial({
      color: '#0a0f16',
      metalness: 0.72,
      roughness: 0.24,
      envMapIntensity: 1.4,
    },),
  );
  floor.rotation
    .x = (-Math.PI) / 2;
  scene.add(floor,);
  /**
   * Matte ceiling slab carrying the emissive strips visually.
   */
  const ceiling = new Mesh(
    new PlaneGeometry(
      WORLD_TUNING.corridorHalfWidth * 2,
      SCENE_RIG.slabLength,
    ),
    new MeshStandardMaterial({
      color: '#0b111c',
      metalness: 0.3,
      roughness: 0.7,
    },),
  );
  ceiling.rotation
    .x = Math.PI / 2;
  ceiling.position
    .y = WORLD_TUNING.ceilingHeight;
  scene.add(ceiling,);
  return {
    floor,
    ceiling,
  };
}

/**
 * Creates renderer, scene, camera, environment, lights, and slabs, and
 * awaits backend initialization. The caller owns the animation loop.
 *
 * @param canvas - page canvas the renderer draws into
 *
 * @returns initialized scene kit plus the slabs for re-centering
 *
 * @example
 * ```ts
 * const { kit, floor, ceiling } = await bootstrapScene({ canvas },);
 * ```
 */
export async function bootstrapScene(
  { canvas, }: Readonly<{ canvas: HTMLCanvasElement; }>,
): Promise<SceneKit & {
  readonly floor: Mesh;
  readonly ceiling: Mesh;
}> {
  /**
   * Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: bootstrapScene.name,
    l,
  },);
  innerL.info('initializing renderer',);
  /**
   * WebGPU renderer; falls back to WebGL2 automatically when the page
   * has no GPU adapter.
   */
  const renderer = new WebGPURenderer({
    canvas,
    antialias: true,
  },);
  renderer.toneMapping = NeutralToneMapping;
  renderer.toneMappingExposure = SCENE_RIG.exposure;
  await renderer.init();
  /**
   * Whether the WebGPU backend won; the HUD reports it. Probed with an
   * `in` guard because the shared Backend type does not declare the flag.
   */
  const usingWebGpu = ('isWebGPUBackend' in renderer.backend)
    && (renderer.backend
      .isWebGPUBackend
      === true);
  innerL.info(`renderer ready, backend: ${usingWebGpu ? 'WebGPU' : 'WebGL2'}`,);
  /**
   * Scene owning all world objects.
   */
  const scene = new Scene();
  scene.background = new Color(NIGHT_COLOR,);
  scene.fog = new Fog(
    NIGHT_COLOR,
    WORLD_TUNING.fogNear,
    WORLD_TUNING.fogFar,
  );
  /**
   * Equirect environment texture painted by {@link paintEnvironmentCanvas}.
   */
  const environment = new CanvasTexture(paintEnvironmentCanvas(),);
  environment.mapping = EquirectangularReflectionMapping;
  environment.colorSpace = SRGBColorSpace;
  scene.environment = environment;
  /**
   * Walking first-person camera; the loop drives z and adds bob.
   */
  const camera = new PerspectiveCamera(
    SCENE_RIG.fov,
    1,
    SCENE_RIG.near,
    SCENE_RIG.far,
  );
  camera.position
    .set(
      0,
      WORLD_TUNING.eyeHeight,
      0,
    );
  /**
   * Soft ambient fill: cool sky, near-black ground.
   */
  scene.add(new HemisphereLight(
    '#9cc8e8',
    '#050810',
    SCENE_RIG.hemisphereIntensity,
  ),);
  /**
   * Warm-cool key light traveling with the camera; re-parented to the
   * camera so panes ahead always catch a highlight.
   */
  const key = new PointLight(
    '#cfe8ff',
    SCENE_RIG.keyIntensity,
    0,
    SCENE_RIG.keyDecay,
  );
  key.position
    .set(
      SCENE_RIG.keyOffset
        .x,
      SCENE_RIG.keyOffset
        .y,
      SCENE_RIG.keyOffset
        .z,
    );
  camera.add(key,);
  scene.add(camera,);
  /**
   * Floor and ceiling slabs re-centered by the loop.
   */
  const slabs = buildSlabs(scene,);
  return {
    renderer,
    scene,
    camera,
    usingWebGpu,
    ...slabs,
  };
}
