/**
 Reusable no-Vite Electron app build, distribution, and Wayland-test helpers.
 
 @example
 ```ts
 import { stageElectronApp } from '@monochromatic-dev/electron-infra-core/ts';
 ```
 
 @packageDocumentation
 */

export {
  writeJsonFileAtomically,
  type JsonObject,
  type JsonScalar,
} from './atomic-json.js';
export {
  distributeElectronApp,
  selectDistributionTargets,
  type ElectronAppDistributionOptions,
} from './distribute.js';
export {
  parseElectronDistributionArgs,
  type DistributionCliOptions,
} from './distribution-args.js';
export {
  DISTRIBUTION_TARGETS,
  targetKey,
  type DistributionArch,
  type DistributionPlatform,
  type DistributionTarget,
} from './distribution-targets.js';
export { readElectronVersion, } from './electron-version.js';
export {
  stageElectronApp,
  type ElectronAppStageOptions,
  type StaticAssetMapping,
} from './stage.js';
export { type ExpectedObservedState, } from './wayland-state.js';
export {
  runWaylandElectronBoundaryTest,
  type WaylandBoundaryStep,
  type WaylandElectronBoundaryTestOptions,
} from './wayland-test.js';
