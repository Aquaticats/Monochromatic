//region Damage and draw exports
// The damage sample and the draw sample.
// Re-exported by `index.ts`, which is the package's one public entry.

export {
  collectTwoLaneShippedRegions,
  DAMAGE_LANES,
  type DamageLane,
  DamageRegionError,
  regionIdOf,
  regionsOfLane,
  type ShippedRegion,
  type ShippedRegionCensus,
} from './corpus-run/damage-region-v2.ts';
export {
  buildCase,
  drawRegions,
} from './corpus-run/damage-sample-draw.ts';
export {
  emptyPoolSays,
  poolLines,
  wroteLine,
} from './corpus-run/damage-sample-lines.ts';
export { collectShippedRegions, } from './corpus-run/damage-sample-pool.ts';
export { sampleDamage, } from './corpus-run/damage-sample-run.ts';
export {
  type BandedEntry,
  type EntryContribution,
  loadEntry,
} from './corpus-run/draw-entry-load.ts';
export {
  type DrawOutputs,
  trackDrawOutputs,
} from './corpus-run/draw-outputs.ts';
export {
  DrawReconcileError,
  type DrawReconcileFault,
  reconcileSentence,
  type TypeofName,
} from './corpus-run/draw-reconcile.ts';
export { poolBandLines, } from './corpus-run/draw-sample-bands.ts';
export {
  type DrawPool,
  readDrawPool,
} from './corpus-run/draw-sample-pool.ts';
export { drawGradingSample, } from './corpus-run/draw-sample-run.ts';
export {
  type DrawPaths,
  writeDrawSheets,
} from './corpus-run/draw-sample-sheets.ts';

//endregion Damage and draw exports
