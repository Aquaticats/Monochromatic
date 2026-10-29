/**
 Self-test entry point for `@monochromatic-dev/llm-llm-type`.
 
 Imports each per-type test file so they all execute in sequence when the
 package's `mise run test` task fires.
 
 @module
 */

// oxlint-disable no-unassigned-import -- test files execute on import via top-level await

import './role.unit.test.ts';
import './message.unit.test.ts';
import './content-part.unit.test.ts';
import './completion.unit.test.ts';
