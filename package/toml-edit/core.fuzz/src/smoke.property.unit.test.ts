/**
 Smoke property proving the fuzz harness wiring end to end.
 
 This file imports `@monochromatic-dev/toml-edit-core` through its built
 package entry point (not a sibling source import), so it verifies that the
 shipped artifact, the `@monochromatic-dev/test-harness-core/ts` harness, and
 fast-check all run together under node in both bounded mode (normal unit
 suite) and campaign mode (the `fuzz` task). The property itself is the
 weakest useful one: a bare key assigned a non-negative integer round-trips
 byte-identically through `parseTomlEdit` then `tomlStringify` in the default
 splice mode.
 
 Run plan and seed policy: see `../fuzz-budget.ts`.
 
 @module
 */

import {
  assert,
  asyncProperty,
  constantFrom,
  nat,
  record,
  string,
} from 'fast-check';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  parseTomlEdit,
  tomlStringify,
} from '@monochromatic-dev/toml-edit-core';

import { fuzzRunPlan, } from './fuzz-budget.ts';

//region Constants and arbitraries

/**
 Run plan resolved once for every property in this file.
 */
const RUN = fuzzRunPlan();

/**
 Arbitrary single TOML bare key (letters, digits, underscore, hyphen).
 Constraining the unit to bare characters avoids a quoting branch in this
 smoke layer; richer key shapes belong to the Phase 2 generators.
 */
const bareKeyArbitrary = string({
  minLength: 1,
  maxLength: 12,
  unit: constantFrom(
    'a',
    'b',
    'Z',
    '0',
    '9',
    '_',
    '-',
  ),
},);

//endregion Constants and arbitraries

await describe({
  name: 'fuzz smoke',
  children: [
    it({
      name: 'bare key plus non-negative integer round-trips byte-identically through the built artifact',
      timeout: RUN.timeout,
      fn: async () => {
        await assert(
          asyncProperty(
            record({
              key: bareKeyArbitrary,
              value: nat(),
            },),
            async function roundTripsBuiltArtifact({
              key,
              value,
            },) {
              /**
               Source document built from the generated key and integer.
               */
              const source = `${key} = ${String(value,)}\n`;
              /**
               Splice-mode re-emission of the parsed document.
               */
              const text = tomlStringify({ edit: parseTomlEdit({ source, },), },);
              expect(text,).toBe(source,);
            },
          ),
          RUN.params,
        );
      },
    },),
  ],
},);
