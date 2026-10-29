export {
  extractAssertionExpression,
  extractLocationSubstring,
  isIntegerString,
  readAssertionSites,
} from './assertion-source.ts';
export type { AssertionSite, } from './assertion-source.ts';

export {
  formatErrorDeep,
  formatFailure,
} from './format-error.ts';

export {
  DEFAULT_MAX_DIFFERENCES,
  errorDifferenceLines,
  valueDifferenceLines,
} from './value-difference.ts';

export {
  describeSide,
  enumerableKeys,
  renderValue,
  valueKind,
  valueRole,
} from './value-representation.ts';

export {
  entryDifference,
} from './list-difference.ts';

export {
  createEntryKeyRegistry,
  entryKey,
} from './value-entry-key.ts';
export type {
  EntryDifference,
} from './list-difference.ts';
export type {
  EntryKeyRegistry,
} from './value-entry-key.ts';
