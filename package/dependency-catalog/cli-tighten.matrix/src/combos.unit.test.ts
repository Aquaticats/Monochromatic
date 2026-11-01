/**
 Unit tests for the matrix fixture builder and scenario set.
 
 Pure host tests (no containers): pin that `buildWorkspaceYaml` encodes the
 catalog floor, override, and per-scenario linker, hoist, and extra settings,
 that the `hoist:` line is omitted under the pnp linker, and that the scenario
 set covers the expected behaviours.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  buildRootPackageJson,
  buildWorkspaceYaml,
  EXPECTED_TIGHTENED,
  FIXTURE_ACTIVE,
  FIXTURE_FLOOR,
  FIXTURE_PACKAGE,
  SCENARIOS,
} from './combos.ts';

await describe({
  name: 'combos',
  children: [
    it({
      name: 'encodes the catalog floor and the pinning override',
      fn: async () => {
        /** Workspace file for an isolated scenario. */
        const yaml = buildWorkspaceYaml({
          label: 'isolated',
          nodeLinker: 'isolated',
          hoist: true,
          mutation: 'none',
          expect: 'tighten',
        },);
        expect(yaml.includes(`'${FIXTURE_PACKAGE}': '>=${FIXTURE_FLOOR}'`,),).toBe(true,);
        expect(yaml.includes('nodeLinker: isolated',),).toBe(true,);
        expect(yaml.includes('hoist: true',),).toBe(true,);
        expect(yaml.includes(`${FIXTURE_PACKAGE}: ${FIXTURE_ACTIVE}`,),).toBe(true,);
      },
    },),

    it({
      name: 'omits the hoist line under the pnp linker',
      fn: async () => {
        /** Workspace file for the pnp scenario, where hoist does not apply. */
        const yaml = buildWorkspaceYaml({
          label: 'pnp',
          nodeLinker: 'pnp',
          hoist: false,
          mutation: 'none',
          expect: 'tighten',
        },);
        expect(yaml.includes('nodeLinker: pnp',),).toBe(true,);
        expect(yaml.includes('hoist:',),).toBe(false,);
      },
    },),

    it({
      name: 'appends extraSettings lines (store-relocating settings)',
      fn: async () => {
        /** Workspace file for a scenario that renames the modules directory. */
        const yaml = buildWorkspaceYaml({
          label: 'modulesDir renamed',
          nodeLinker: 'isolated',
          hoist: false,
          extraSettings: ['modulesDir: node_modules_alt',],
          mutation: 'none',
          expect: 'tighten',
        },);
        expect(yaml.includes('modulesDir: node_modules_alt',),).toBe(true,);
        expect(yaml.includes('overrides:',),).toBe(true,);
      },
    },),

    it({
      name: 'covers each missing-X scenario as an error or miss',
      fn: async () => {
        /** Scenarios that delete a required file or directory. */
        const removalScenarios = SCENARIOS.filter(function isRemoval(scenario,): boolean {
          return scenario.mutation
            .startsWith('remove-',);
        },);
        /** Labels of removal scenarios that expect a clean tighten despite the removal. */
        const tightenAnyway = removalScenarios.filter(function isTighten(scenario,): boolean {
          return scenario.expect
            === 'tighten';
        },);
        // remove-lockfile, remove-store, remove-some-modules, remove-pnpm, and remove-pnp-cjs still tighten.
        expect(tightenAnyway.length,).toBe(5,);
      },
    },),

    it({
      name: 'expected tightened line names floor and active version',
      fn: async () => {
        expect(EXPECTED_TIGHTENED,).toBe(`${FIXTURE_PACKAGE}: >=${FIXTURE_FLOOR} -> >=${FIXTURE_ACTIVE}`,);
      },
    },),

    it({
      name: 'pins packageManager to the passed monorepo pnpm spec',
      fn: async () => {
        /** Root manifest built with a resolved pnpm spec. */
        const rootJson = buildRootPackageJson('pnpm@11.11.0',);
        expect(rootJson.includes('"packageManager": "pnpm@11.11.0"',),).toBe(true,);
      },
    },),

    it({
      name: 'covers the store-only case that expects an UNDCL',
      fn: async () => {
        /** Scenarios whose expectation is the store-present-but-undeclared classification. */
        const undeclared = SCENARIOS.filter(function isUndeclared(scenario,): boolean {
          return scenario.expect
            === 'undeclared';
        },);
        expect(undeclared.length,).toBe(1,);
        expect(undeclared[0]?.mutation,).toBe('orphan-store-copy',);
      },
    },),
  ],
},);
