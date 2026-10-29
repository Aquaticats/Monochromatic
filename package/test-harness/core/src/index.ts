import { redirectLogScratch, } from './log-scratch.ts';

// First statement of the entry: the logger resolves its file sink destination
// only on the first record, so redirecting here still lands every record in
// per-run scratch even though the imports above evaluated first.
redirectLogScratch();

export { caught, } from './caught.ts';
export {
  extractAssertionExpression,
  extractLocationSubstring,
  isIntegerString,
  readAssertionSites,
  formatErrorDeep,
  formatFailure,
} from '@monochromatic-dev/test-harness-core-diagnostic/ts';
export type { AssertionSite, } from '@monochromatic-dev/test-harness-core-diagnostic/ts';

export { describe, } from './describe.ts';
export type {
  DescribeChild,
  DescribeOptions,
  DescribeResult,
} from './describe.ts';

/**
 Cases a suite runs at once when neither it nor any suite around it sets
 `concurrency`. Exported so a file that gathers its suites under one
 sequential root can hand each suite back the width it ran at alone.
 */
export { DEFAULT_CONCURRENCY, } from './descriptor.ts';
export type { TestDescriptor, } from './descriptor.ts';

export { it, } from './it.ts';
export type {
  ItOptions,
  ItResult,
  TestContext,
} from './it.ts';

export {
  createScopedExpect,
  expect,
  expectTypeOf,
} from '@monochromatic-dev/test-harness-core-expect/ts';
export type {
  AssertionTracker,
  AsyncMatcherSet,
  ExpectResult,
  MatcherSet,
  ScopedExpect,
} from '@monochromatic-dev/test-harness-core-expect/ts';

export type { DisposableSandbox, } from '@monochromatic-dev/test-harness-core-sandbox/ts';
