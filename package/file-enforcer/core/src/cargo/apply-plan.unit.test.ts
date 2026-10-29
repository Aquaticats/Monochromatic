import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseTomlEdit, tomlHas, } from '@monochromatic-dev/toml-edit-core/ts';

import { applyCargoPlan, } from './apply-plan.ts';
import type { CargoManifestPlan, } from './types.ts';

/**
 Canonical lint block text used by the insertion tests.
 */
const LINTS_BLOCK = `[lints.clippy]
disallowed_methods = "deny"
implicit_return = "deny"
needless_return = "allow"
`;

/**
 Canonical empty-workspace block text used by the insertion tests.
 */
const WORKSPACE_BLOCK = `[workspace]
`;

await describe({
  name: '',
  children: [
    describe({
      name: applyCargoPlan.name,
      children: [
        it({
          name: 'rewrites a drifted scalar when the guard resolves',
          fn: async () => {
            /** Manifest whose edition must converge to 2024 */
            const content = '[package]\nname = "x"\nedition = "2021"\n';
            /** Plan owning package.edition */
            const plan = {
              enforcements: [
                {
                  guardPath: ['package',],
                  path: ['package', 'edition',],
                  value: '2024',
                },
              ],
              blocks: [],
            } satisfies CargoManifestPlan;
            expect(applyCargoPlan({ content, plan, },),).toContain('edition = "2024"',);
          },
        },),
        it({
          name: 'adds keys inside an existing table without moving them to top level',
          fn: async () => {
            /** Manifest carrying only the first canonical lint key */
            const content = '[package]\nname = "x"\n\n[lints.clippy]\ndisallowed_methods = "deny"\n';
            /** Plan adding the two remaining canonical lint keys */
            const plan = {
              enforcements: [
                {
                  guardPath: ['lints', 'clippy',],
                  path: ['lints', 'clippy', 'implicit_return',],
                  value: 'deny',
                },
                {
                  guardPath: ['lints', 'clippy',],
                  path: ['lints', 'clippy', 'needless_return',],
                  value: 'allow',
                },
              ],
              blocks: [],
            } satisfies CargoManifestPlan;
            /** Result carrying all three lint keys under the table */
            const result = applyCargoPlan({ content, plan, },);
            expect(result,).toContain('implicit_return = "deny"',);
            expect(result,).toContain('needless_return = "allow"',);
            expect(result.startsWith('[package]',),).toBe(true,);
          },
        },),
        it({
          name: 'appends canonical blocks only when the target table is absent',
          fn: async () => {
            /** Manifest with neither a lints table nor a workspace table */
            const content = '[package]\nname = "x"\nedition = "2024"\n';
            /** Plan inserting both absent blocks */
            const plan = {
              enforcements: [],
              blocks: [
                { absentPath: ['lints', 'clippy',], text: LINTS_BLOCK, },
                { absentPath: ['workspace',], text: WORKSPACE_BLOCK, },
              ],
            } satisfies CargoManifestPlan;
            /** Result carrying both appended blocks */
            const result = applyCargoPlan({ content, plan, },);
            expect(tomlHas({
              edit: parseTomlEdit({ source: result, },),
              path: ['workspace',],
            },),).toBe(true,);
            expect(tomlHas({
              edit: parseTomlEdit({ source: result, },),
              path: ['lints', 'clippy', 'implicit_return',],
            },),).toBe(true,);
          },
        },),
        it({
          name: 'leaves a present block untouched (no duplicate insertion)',
          fn: async () => {
            /** Manifest already carrying an empty workspace table */
            const content = '[package]\nname = "x"\n\n[workspace]\n';
            /** Plan whose workspace insertion must be skipped */
            const plan = {
              enforcements: [],
              blocks: [
                { absentPath: ['workspace',], text: WORKSPACE_BLOCK, },
              ],
            } satisfies CargoManifestPlan;
            expect(applyCargoPlan({ content, plan, },),).toBe(content,);
          },
        },),
        it({
          name: 'heals a declared dependency but never conjures an absent one',
          fn: async () => {
            /** Manifest with a drifted tracing and no serde */
            const content = '[package]\nname = "x"\n\n[dependencies]\ntracing = "0.2"\n';
            /** Plan owning tracing and serde */
            const plan = {
              enforcements: [
                {
                  guardPath: ['dependencies', 'tracing',],
                  path: ['dependencies', 'tracing',],
                  value: '0.1',
                },
                {
                  guardPath: ['dependencies', 'serde',],
                  path: ['dependencies', 'serde',],
                  value: { version: '1', features: ['derive',], },
                },
              ],
              blocks: [],
            } satisfies CargoManifestPlan;
            /** Result healing tracing, leaving serde absent */
            const result = applyCargoPlan({ content, plan, },);
            expect(result,).toContain('tracing = "0.1"',);
            expect(result.includes('serde',),).toBe(false,);
          },
        },),
        it({
          name: 'returns byte-identical text when everything already matches',
          fn: async () => {
            /** Manifest already at canonical values */
            const content = '[package]\nname = "x"\nedition = "2024"\n\n[dependencies]\ntracing = "0.1"\n';
            /** Plan enforcing the already-present canonical values */
            const plan = {
              enforcements: [
                {
                  guardPath: ['package',],
                  path: ['package', 'edition',],
                  value: '2024',
                },
                {
                  guardPath: ['dependencies', 'tracing',],
                  path: ['dependencies', 'tracing',],
                  value: '0.1',
                },
              ],
              blocks: [],
            } satisfies CargoManifestPlan;
            expect(applyCargoPlan({ content, plan, },),).toBe(content,);
          },
        },),
      ],
    },),
  ],
},);
