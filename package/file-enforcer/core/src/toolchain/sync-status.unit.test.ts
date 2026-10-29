/**
 Unit tests for sync ordering, status comparison, and provider helpers.
 */

import {
  existsSync,
  mkdtempSync,
  readFileSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  ALL_TOOLS,
  appliesToHost,
  normalizeVersion,
  parseBinList,
  parseCargoBinstallList,
  releaseTagUrl,
  ReleaseTagUrlError,
  seedBinConfig,
  syncOrder,
  toolById,
  versionMatches,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'sync and status helpers',
  children: [
    describe({
      name: syncOrder.name,
      children: [
        it({
          name: 'stages bin, uv, mpm, rust, cargo-binstall, node and pnpm first',
          fn: async () => {
            /**
             Ordered tool ids.
             */
            const ordered = syncOrder({ entries: ALL_TOOLS, },)
              .map(
              (entry,) => entry.id,
              );
            expect(ordered.slice(
              0,
              7,
            ),)
              .toEqual(
              [
                'bin',
                'uv',
                'mpm',
                'rust',
                'cargo-binstall',
                'node',
                'pnpm',
              ],
              );
          },
        },),
        it({
          name: 'keeps the remaining tools in plan order',
          fn: async () => {
            /**
             Ordered tool ids after the staging prefix.
             */
            const rest = syncOrder({ entries: ALL_TOOLS, },)
              .map(
              (entry,) => entry.id,
              )
              .slice(7,);
            expect(rest.includes('bun',),)
              .toBe(true,);
            expect(rest[0],)
              .toBe('bun',);
          },
        },),
        it({
          name: 'never drops or duplicates an entry',
          fn: async () => {
            expect(syncOrder({ entries: ALL_TOOLS, },)
              .length,)
              .toBe(ALL_TOOLS.length,);
          },
        },),
      ],
    },),
    describe({
      name: appliesToHost.name,
      children: [
        it({
          name: 'admits unfiltered tools on any platform',
          fn: async () => {
            expect(
              appliesToHost(toolById('node',),),
            )
              .toBe(true,);
          },
        },),
        it({
          name: 'admits linux-only tools on linux and rejects elsewhere',
          fn: async () => {
            /**
             Linux-only tool from the plan.
             */
            const linuxOnly = toolById('conda:bubblewrap',);
            expect(appliesToHost(linuxOnly,),)
              .toBe(process.platform === 'linux',);
          },
        },),
      ],
    },),
    describe({
      name: 'version comparison',
      children: [
        it({
          name: 'drops one leading v',
          fn: async () => {
            expect(normalizeVersion('v1.3.2',),)
              .toBe('1.3.2',);
            expect(normalizeVersion('1.3.2',),)
              .toBe('1.3.2',);
            expect(normalizeVersion('nightly-2026-10-21',),)
              .toBe('nightly-2026-10-21',);
            // Release tags carry name prefixes (the bin registry records them).
            expect(normalizeVersion('bun-v1.3.14',),)
              .toBe('1.3.14',);
          },
        },),
        it({
          name: 'matches equal and platform-suffixed toolchain ids',
          fn: async () => {
            expect(versionMatches({
              observed: 'v1.3.2',
              locked: '1.3.2',
            },),)
              .toBe(true,);
            expect(versionMatches({
              observed: 'nightly-2026-10-21-x86_64-unknown-linux-gnu',
              locked: 'nightly-2026-10-21',
            },),)
              .toBe(true,);
            expect(versionMatches({
              observed: '1.3.1',
              locked: '1.3.2',
            },),)
              .toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: releaseTagUrl.name,
      children: [
        it({
          name: 'derives the release-tag URL from an artifact URL',
          fn: async () => {
            expect(
              releaseTagUrl(
                'https://github.com/FiloSottile/age/releases/download/v1.3.2/age-v1.3.2-linux-amd64.tar.gz',
              ),
            )
              .toBe(
              'https://github.com/FiloSottile/age/releases/tag/v1.3.2',
              );
          },
        },),
        it({
          name: 'decodes encoded tags and rejects non-release URLs',
          fn: async () => {
            expect(
              releaseTagUrl(
                'https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.12%2B8/OpenJDK.tar.gz',
              ),
            )
              .toBe(
              'https://github.com/adoptium/temurin21-binaries/releases/tag/jdk-21.0.12+8',
              );
            expect(function nonRelease(): void {
              releaseTagUrl('https://nodejs.org/dist/v26.12.0/node.tar.gz',);
            },)
              .toThrow(ReleaseTagUrlError,);
            expect(function missingTag(): void {
              releaseTagUrl('https://github.com/o/r/releases/download//asset.tar.gz',);
            },)
              .toThrow(ReleaseTagUrlError,);
          },
        },),
      ],
    },),
    describe({
      name: 'listing parsers',
      children: [
        it({
          name: 'parses the bin list table into name to version pairs',
          fn: async () => {
            /**
             Parsed versions from a real `bin list` shaped table.
             */
            const versions = parseBinList(
              [
                '',
                'Path                                       Version  URL                                             Status',
                '/fixture/.local/bin/age                    v1.3.2   https://github.com/FiloSottile/age/releases/tag/v1.3.2  OK',
                '/fixture/.local/bin/rg                     15.2.0   https://github.com/BurntSushi/ripgrep/releases/tag/15.2.0  OK',
                '/fixture/.local/bin/pinned                 *1.5.7   releases.hashicorp.com/terraform                   OK',
                '',
              ].join('\n',),
            );
            expect(versions.get('age',),)
              .toBe('1.3.2',);
            expect(versions.get('rg',),)
              .toBe('15.2.0',);
            expect(versions.get('pinned',),)
              .toBe('1.5.7',);
          },
        },),
        it({
          name: 'parses cargo binstall --list rows',
          fn: async () => {
            /**
             Parsed versions from a real `cargo binstall --list` shaped table.
             */
            const versions = parseCargoBinstallList(
              [
                'Package              Cargo Ver    Binstall Ver   Method            ',
                '-------------------------------------------------------------------',
                'cargo-fuzz           0.13.1       -              cargo install     ',
                'cargo-ndk            4.1.2        4.1.2          cargo binstall    ',
                '-------------------------------------------------------------------',
                'Total: 2 | Binstall: 1 | Source: 1 | Local: 0 | Git: 0 | Drifted: 0',
              ].join('\n',),
            );
            expect(versions.get('cargo-fuzz',),)
              .toBe('0.13.1',);
            expect(versions.get('cargo-ndk',),)
              .toBe('4.1.2',);
          },
        },),
      ],
    },),
    describe({
      name: seedBinConfig.name,
      children: [
        it({
          name: 'converges default_path in the scope config',
          fn: async () => {
            /**
             Disposable directory for the scope config.
             */
            const home = mkdtempSync(join(
              tmpdir(),
              'toolchain-bin-config-',
            ),);
            /**
             Scope-local configuration path.
             */
            const configPath = join(
              home,
              'bin.config.json',
            );
            /**
             Config path seeded by the first call.
             */
            const seeded = await seedBinConfig({
              downloadDir: '/fixture/toolchain/bin',
              configPath,
            },);
            expect(seeded,)
              .toBe(configPath,);
            expect(existsSync(seeded,),)
              .toBe(true,);
            expect(
              JSON.parse(readFileSync(
                seeded,
                'utf8',
              ),) as { default_path: string; },
            )
              .toEqual({
                default_path: '/fixture/toolchain/bin',
                bins: {},
              },);
            // A second run with the same toolchain bin dir is idempotent.
            expect(await seedBinConfig({
              downloadDir: '/fixture/toolchain/bin',
              configPath,
            },),)
              .toBe(seeded,);
            // A stale default_path converges back to the toolchain bin dir.
            writeFileSync(
              seeded,
              JSON.stringify({
                default_path: '/stale',
                bins: {},
              },),
              'utf8',
            );
            await seedBinConfig({
              downloadDir: '/fixture/toolchain/bin',
              configPath,
            },);
            expect(
              (JSON.parse(readFileSync(
                seeded,
                'utf8',
              ),) as { default_path: string; })
                .default_path,
            )
              .toBe('/fixture/toolchain/bin',);
          },
        },),
        it({
          name: 'keeps recorded installs while converging default_path',
          fn: async () => {
            /**
             Disposable directory and explicit config path.
             */
            const home = mkdtempSync(join(
              tmpdir(),
              'toolchain-bin-config-',
            ),);
            /**
             Explicit scope config with one recorded install.
             */
            const explicit = join(
              home,
              'explicit.json',
            );
            writeFileSync(
              explicit,
              `${
                JSON.stringify({
                  default_path: '/stale',
                  bins: {
                    '/fixture/toolchain/bin/tool': { version: 'v1', },
                  },
                },)
              }\n`,
              'utf8',
            );
            expect(await seedBinConfig({
              downloadDir: '/fixture/toolchain/bin',
              configPath: explicit,
            },),)
              .toBe(explicit,);
            expect(
              (JSON.parse(readFileSync(
                explicit,
                'utf8',
              ),) as { bins: Record<string, unknown>; default_path: string; }),
            )
              .toEqual({
                default_path: '/fixture/toolchain/bin',
                bins: {
                  '/fixture/toolchain/bin/tool': { version: 'v1', },
                },
              },);
          },
        },),
      ],
    },),
  ],
},);
