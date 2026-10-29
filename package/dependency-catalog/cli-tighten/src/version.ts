/**
 Barrel re-export for version utilities.
 
 Semver parsing lives in `version-parse.ts`;
 installed-version resolution lives in `version-resolve.ts`.
 */

export {
  isStrictlyGreater,
  NOT_A_RANGE,
  type ParsedRange,
  parseRange,
} from './version-parse.ts';
export { NO_INSTALLED_VERSION, } from './version-read.ts';
export {
  readInstalledVersion,
  resolveNpmNames,
} from './version-resolve.ts';
