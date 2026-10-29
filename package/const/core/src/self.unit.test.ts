/**
 Self-test entry point for `@monochromatic-dev/const-core`.
 
 Imports each per-category test file so they all execute in sequence when
 the package's `mise run test` task fires.
 
 @module
 */

// oxlint-disable no-unassigned-import -- test files execute on import via top-level await

import './ascii.unit.test.ts';
import './time.unit.test.ts';
import './byte.unit.test.ts';
import './http-status.unit.test.ts';
import './fraction.unit.test.ts';
