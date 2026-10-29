/**
 Unit tests for the deterministic artifact generators.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  ALL_TOOLS,
  renderArtifacts,
  renderMpmManagersToml,
  renderMpmPackagesToml,
  renderToolchainLockJson,
  tomlKey,
  tomlString,
  toolById,
  toolchainLockFromTools,
  TOOLS,
} from '../../dist/final/node/index.mjs';

/**
 Whether a declaration key carries `@version` pin syntax: an `@` right before an optional `v` and a digit.

 @param key - Declaration key text

 @returns `true` when some `@` starts a version
 */
function hasVersionPin(key: string,): boolean {
  return key.split('@',)
    .slice(1,)
    .some(
    function startsWithVersion(afterAt,): boolean {
      /**
       Text after an optional `v`.
       */
      const digits = afterAt.startsWith('v',)
        ? afterAt.slice(1,)
        : afterAt;
      return (digits.length > 0) && '0123456789'.includes(digits.charAt(0,),);
    },
    );
}

/**
 Removes every backslash escape pair from a TOML string literal body.

 @param body - Literal text between the quotes

 @returns Text with each backslash and the character it escapes dropped
 */
function withoutEscapes(body: string,): string {
  /**
   Code points of the body.
   */
  const chars = body.split('',);
  /**
   Characters outside any escape pair.
   */
  const kept: string[] = [];
  for (let index = 0; index < chars.length; index += 1) {
    if (chars[index] === '\\') {
      // The escaped character is part of the escape, so it is dropped too.
      index += 1;
      continue;
    }
    kept.push(chars[index] ?? '',);
  }
  return kept.join('',);
}

await describe({
  name: 'toolchain artifact generators',
  children: [
    describe({
      name: renderArtifacts.name,
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
                .toSorted(),
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
              'package/file-enforcer/core/src/toolchain/plan.ts',
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
            /**
             Every lock key in its fixed order; `manager`, `packageId` and `os` are omitted when absent.
             */
            const keyOrder = [
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
            ];
            /**
             Keys every tool record carries.
             */
            const required = new Set([
              'request',
              'version',
              'route',
              'bins',
              'env',
              'options',
              'artifacts',
            ],);
            for (const tool of Object.values(parsed.tools,)) {
              /**
               Keys this record carries, in file order.
               */
              const keys = new Set(Object.keys(tool,),);
              expect([...keys,],)
                .toEqual(keyOrder.filter(function present(key,): boolean {
                  return keys.has(key,);
                },),);
              expect([...required,].every(function carried(key,): boolean {
                return keys.has(key,);
              },),)
                .toBe(true,);
              expect(Object.values(tool,)
                .includes(null,),)
                .toBe(false,);
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
            /**
             Pinned nightly, read from the plan because it moves with every toolchain update.
             */
            const rustVersion = toolById('rust',).version;
            expect(text.includes(`${rustVersion} = "${rustVersion}"`,),)
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
               Declaration key of the line.
               */
              const key = line.split(' = ', 1)[0] ?? '';
              expect(hasVersionPin(key,),)
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
              .toBe(18,);
            expect(
              headers.filter(
                (line,) => line.includes('.operations.'),
              )
                .length,
            )
              .toBe(36,);
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
              .toBe(String.raw`"say \"hi\""`,);
            expect(tomlString(String.raw`back\slash`,),)
              .toBe(String.raw`"back\\slash"`,);
            expect(tomlString('line\nbreak',),)
              .toBe(String.raw`"line\nbreak"`,);
            expect(tomlString('tab\there',),)
              .toBe(String.raw`"tab\there"`,);
            expect(tomlString('\u0001',),)
              .toBe(String.raw`"\u0001"`,);
          },
        },),
        it({
          name: 'keeps hostile strings inside their literal',
          fn: async () => {
            for (
              const hostile of [
                '" = "pwned',
                '\n[evil]\n',
                String.raw`\u0022, x = "`,
                'a] = 1\nb',
                // A template literal with an escaped dollar holds the placeholder text without interpolating it.
                `\${name}`,
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
              const unescaped = withoutEscapes(literal.slice(
                1,
                -1,
              ),);
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
