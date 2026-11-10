/**
 The package's own exports map, probed from the specifier the fuzz sidecars import.

 A sidecar imports `@monochromatic-dev/fuzz-core/ts`; a test that imported `./run-budget.ts` would keep passing after the
 subpath stopped resolving, and every sidecar would fail at its first property file instead.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  DEFAULT_FUZZ_RUNS,
  fuzzRunsFromEnvironment,
} from '@monochromatic-dev/fuzz-core/ts';

await describe({
  name: '@monochromatic-dev/fuzz-core/ts',
  children: [
    it({
      name: 'resolves the run budget through the exports map',
      fn: async () => {
        expect(typeof fuzzRunsFromEnvironment,).toBe('function',);
      },
    },),
    it({
      name: 'publishes a default count a sidecar can fall back to',
      fn: async () => {
        expect(typeof DEFAULT_FUZZ_RUNS,).toBe('number',);
        expect(DEFAULT_FUZZ_RUNS > 0,).toBe(true,);
      },
    },),
  ],
},);
