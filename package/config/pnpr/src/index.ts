// Publish-planning helpers and the pnpr server config generator, built so unit tests exercise the bundled artifact.
// The package declares no entry points, so it never publishes itself.
export { renderPnprConfig, } from './generated-config-render.ts';
export {
  assertPnprPackageName,
  pnprManifestHasEntryPoint,
  selectPnprPublishedNames,
} from './generated-config-selection.ts';
export {
  PNPR_CONFIG_PATH,
  PNPR_EXCLUDED_PACKAGES,
} from './generated-config-settings.ts';
export {
  checkPnprConfig,
  describeDrift,
  describeFirstDifference,
  generatePnprConfig,
  PnprConfigDriftError,
  renderPnprConfigForWorkspace,
  type RenderedPnprConfig,
} from './generated-config-sync.ts';
export {
  isTypeScriptSourcePath,
  orderForPublishing,
  PnprConfigShapeError,
  prepareManifestForPnpr,
  readPnprPublishTarget,
  type PnprPublishTarget,
  type PublishCandidate,
} from './publish-plan.ts';
export {
  FORBIDDEN_RETRY_DELAY_MS,
  FORBIDDEN_RETRY_WINDOW_MS,
  isForbiddenPublishError,
  NPM_FORBIDDEN_MARKER,
  publishWithForbiddenRetry,
} from './publish-retry.ts';
export {
  readWorkspaceEntries,
  type WorkspaceEntry,
} from './registry-read.ts';
export {
  chooseDistTag,
  compareVersions,
} from './version-order.ts';
export {
  fieldOf,
  firstItem,
  isJsonRecord,
  isStringArray,
} from './json-shape.ts';
