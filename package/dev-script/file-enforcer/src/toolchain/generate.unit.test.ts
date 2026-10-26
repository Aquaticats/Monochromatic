/**
 Unit tests for the deterministic artifact generators.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  renderToolchainLockJson,
  toolchainLockFromTools,
} from './lock.ts';
import {
  renderMpmManagersToml,
  renderMpmPackagesToml,
  tomlKey,
  tomlString,
} from './mpm-manifest.ts';
import { renderArtifacts, } from './generate.ts';
import { ALL_TOOLS, TOOLS, } from './plan.ts';

await describe({
  name: 'toolchain artifact generators',
  children: [
    describe({
      name: 'renderArtifacts',
      children: [
        it({
          name: 'renders byte-identical output across runs',
          fn: async () => {
            expect(renderArtifacts({ tools: ALL_TOOLS, },),)
              .toEqual(renderArtifacts({ tools: ALL_TOOLS, },),);
            expect(
              renderArtifacts({ tools: ALL_TOOLS, },)['toolchain.lock.json'],
            )
              .toBe(
                renderArtifacts({ tools: ALL_TOOLS, },)['toolchain.lock.json'],
              );
          },
        },),
        it({
          name: 'renders the three root artifacts',
          fn: async () => {
            expect(
              Object.keys(renderArtifacts({ tools: ALL_TOOLS, },),)
                .sort(),
            )
              .toEqual(
                [
                  'mpm.managers.toml',
                  'mpm.packages.toml',
                  'toolchain.lock.json',
                ],
              );
          },
        },),
      ],
    },),
    describe({
      name: 'toolchain.lock.json',
      children: [
        it({
          name: 'fixes the schema version and generatedFrom marker',
          fn: async () => {
            /**
             Rendered lock document.
             */
            const lock = toolchainLockFromTools({ tools: ALL_TOOLS, },);
            expect(lock.schemaVersion,)
              .toBe(1,);
            expect(lock.generatedFrom,)
              .toBe(
                'package/dev-script/file-enforcer/src/toolchain/plan.ts',
              );
          },
        },),
        it({
          name: 'emits every tool with the fixed key order',
          fn: async () => {
            /**
             Rendered lock JSON parsed back for key order inspection.
             */
            const parsed = JSON.parse(
              renderToolchainLockJson(toolchainLockFromTools({
                tools: ALL_TOOLS,
              },),),
            ) as {
              tools: Record<string, Record<string, unknown>>;
            };
            for (const tool of Object.values(parsed.tools,)) {
              expect(Object.keys(tool,),)
                .toEqual(
                  [
                    'request',
                    'version',
                    'route',
                    'manager',
                    'packageId',
                    'bins',
                    'os',
                    'env',
                    'options',
                    'artifacts',
                  ],
                );
            }
            expect(Object.keys(parsed.tools,)
              .length,)
              .toBe(ALL_TOOLS.length,);
          },
        },),
        it({
          name: 'keeps the declared tool order in the tools map',
          fn: async () => {
            /**
             Rendered lock JSON parsed back for key order inspection.
             */
            const parsed = JSON.parse(
              renderToolchainLockJson(toolchainLockFromTools({
                tools: ALL_TOOLS,
              },),),
            ) as { tools: Record<string, unknown>; };
            expect(Object.keys(parsed.tools,),)
              .toEqual(
                ALL_TOOLS.map(
                  (entry,) => entry.id,
                ),
              );
          },
        },),
        it({
          name: 'ends the JSON with exactly one newline',
          fn: async () => {
            /**
             Rendered lock JSON text.
             */
            const text = renderToolchainLockJson(toolchainLockFromTools({
              tools: TOOLS,
            },),);
            expect(text.endsWith('}\n',),)
              .toBe(true,);
            expect(text.endsWith('}\n\n',),)
              .toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: 'mpm.packages.toml',
      children: [
        it({
          name: 'declares one section per MPM manager id in sorted order',
          fn: async () => {
            /**
             Section headers of the rendered manifest.
             */
            const sections = renderMpmPackagesToml({ tools: ALL_TOOLS, },)
              .split('\n',)
              .filter(
                (line,) => line.startsWith('[',),
              );
            expect(sections,)
              .toEqual(
                [
                  '[cargo]',
                  '[conda]',
                  '[pnpm]',
                  '[rustup]',
                  '[uvx]',
                ],
              );
          },
        },),
        it({
          name: 'records every managed tool as package_id = "<version>"',
          fn: async () => {
            /**
             Rendered manifest text.
             */
            const text = renderMpmPackagesToml({ tools: ALL_TOOLS, },);
            expect(text.includes('fd-find = "10.5.0"',),)
              .toBe(true,);
            expect(text.includes('"@actions/languageserver" = "0.3.61"',),)
              .toBe(true,);
            expect(text.includes('nightly-2026-10-21 = "nightly-2026-10-21"',),)
              .toBe(true,);
            expect(text.includes('slopo = "0.9.0"',),)
              .toBe(true,);
          },
        },),
        it({
          name: 'carries no @version pin syntax in declaration keys',
          fn: async () => {
            /**
             Rendered manifest text.
             */
            const text = renderMpmPackagesToml({ tools: ALL_TOOLS, },);
            for (const line of text.split('\n',)) {
              if (!line.includes(' = ',))
                continue;
              /**
               * Declaration key of the line.
               */
              const key = line.split(' = ', 1)[0] ?? '';
              expect(/@v?\d/u.test(key,),)
                .toBe(false,);
            }
          },
        },),
      ],
    },),
    describe({
      name: 'mpm.managers.toml',
      children: [
        it({
          name: 'defines one pin-expressing definition per cargo tool',
          fn: async () => {
            /**
             Table headers of the rendered file.
             */
            const headers = renderMpmManagersToml({ tools: ALL_TOOLS, },)
              .split('\n',)
              .filter(
                (line,) => line.startsWith('[mpm.overrides.'),
              );
            expect(
              headers.filter(
                (line,) => !line.includes('.operations.'),
              )
                .length,
            )
              .toBe(16,);
            expect(
              headers.filter(
                (line,) => line.includes('.operations.'),
              )
                .length,
            )
              .toBe(32,);
          },
        },),
        it({
          name: 'bakes the locked version in a --version argument',
          fn: async () => {
            /**
             Rendered definitions text.
             */
            const text = renderMpmManagersToml({ tools: ALL_TOOLS, },);
            expect(
              text.includes(
                'args = ["binstall", "--no-confirm", "--version", "10.5.0", "{package_id}"]',
              ),
            )
              .toBe(true,);
            expect(
              text.includes(
                'args = ["binstall", "--no-confirm", "--version", "0.12.0", "--features", "unix", "{package_id}"]',
              ),
            )
              .toBe(true,);
          },
        },),
        it({
          name: 'never emits a {version} placeholder (rejected by MPM)',
          fn: async () => {
            expect(
              renderMpmManagersToml({ tools: ALL_TOOLS, },)
                .includes('{version}',),
            )
              .toBe(false,);
          },
        },),
        it({
          name: 'omits definitions whose pin needs @version syntax',
          fn: async () => {
            /**
             Rendered definitions text.
             */
            const text = renderMpmManagersToml({ tools: ALL_TOOLS, },);
            expect(text.includes('yaml-language-server',),)
              .toBe(false,);
            expect(text.includes('slopo',),)
              .toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: 'TOML primitives',
      children: [
        it({
          name: 'quotes strings and escapes delimiters and controls',
          fn: async () => {
            expect(tomlString('plain',),)
              .toBe('"plain"',);
            expect(tomlString('say "hi"',),)
              .toBe('"say \\"hi\\""',);
            expect(tomlString('back\\slash',),)
              .toBe('"back\\\\slash"',);
            expect(tomlString('line\nbreak',),)
              .toBe('"line\\nbreak"',);
            expect(tomlString('tab\there',),)
              .toBe('"tab\\there"',);
            expect(tomlString('\u0001',),)
              .toBe('"\\u0001"',);
          },
        },),
        it({
          name: 'keeps hostile strings inside their literal',
          fn: async () => {
            for (
              const hostile of [
                '" = "pwned',
                '\n[evil]\n',
                '\\u0022, x = "',
                'a] = 1\nb',
                '${name}',
                '`rm -rf /`',
              ]
            ) {
              /**
               Escaped literal for one hostile string.
               */
              const literal = tomlString(hostile,);
              expect(literal.startsWith('"',),)
                .toBe(true,);
              expect(literal.endsWith('"',),)
                .toBe(true,);
              /**
               Literal body with every escape sequence removed.
               */
              const unescaped = literal.slice(
                1,
                -1,
              )
                .replaceAll(
                  /\\./gu,
                  '',
                );
              expect(unescaped.includes('"',),)
                .toBe(false,);
              expect(unescaped.includes('\n',),)
                .toBe(false,);
            }
          },
        },),
        it({
          name: 'quotes keys only when the TOML grammar needs it',
          fn: async () => {
            expect(tomlKey('fd-find',),)
              .toBe('fd-find',);
            expect(tomlKey('@actions/languageserver',),)
              .toBe('"@actions/languageserver"',);
          },
        },),
      ],
    },),
  ],
},);
