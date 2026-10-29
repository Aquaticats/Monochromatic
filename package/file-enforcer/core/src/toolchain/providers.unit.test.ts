/**
 Unit tests for toolchain provider convergence.

 Each case runs against a disposable fixture home: fake CLIs placed where the
 toolchain actually installs them, a loopback artifact server, and a PATH
 decoy standing in for the machine's retired shims. These tests lock the
 2026-10-26 machine-convergence defects: the `bin` CLI invoked at the wrong
 path, providers reporting failed runs as installed, gzip payloads copied
 still-compressed, and a seeded `bin` config left shadowed by an empty one.
 */

import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  writeFileSync,
} from 'node:fs';
import { createHash, } from 'node:crypto';
import { gzipSync, } from 'node:zlib';
import { createServer as createHttpServer, type Server, } from 'node:http';
import { tmpdir, } from 'node:os';
import { dirname, join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  installArchive,
  installBinSourced,
  installCargoBinstall,
  installCargoGit,
  installGoInstall,
  installMpmPnpm,
  seedBinConfig,
} from './providers.ts';
import { hostPlatformId, } from './layout.ts';
import {
  BIN_BOOTSTRAP,
  ALL_TOOLS,
} from './plan.ts';
import { resolveToolchainPaths, toolInstallDir, type ToolchainPaths, } from './paths.ts';
import { observeTool, } from './status.ts';
import type { ToolPlanEntry, } from './types.ts';

//region Fixtures

/**
 Disposable fixture: home, toolchain paths, and a fake-CLI log.
 */
type Fixture = {
  readonly root: string;
  readonly paths: ToolchainPaths;
  readonly env: Record<string, string>;
  readonly logFile: string;
};

/**
 Builds a disposable fixture whose home and toolchain root are throwaway.
 
 @returns The fixture
 */
function makeFixture(): Fixture {
  /**
   Disposable root doubling as the fixture home.
   */
  const root = mkdtempSync(join(
    tmpdir(),
    'toolchain-converge-',
  ),);
  /**
   Environment isolated to the fixture home.
   */
  const env = {
    HOME: root,
    XDG_DATA_HOME: join(
      root,
      'data',
    ),
  };
  mkdirSync(join(
    root,
    'data',
    'monochromatic-dev',
    'toolchain',
    'bin',
  ), { recursive: true },);
  return {
    root,
    paths: resolveToolchainPaths({
      worktreeRoot: root,
      env,
    },),
    env,
    logFile: join(
      root,
      'cli.log',
    ),
  };
}

/**
 Writes an executable fake CLI that logs its argv and runs a behavior body.
 
 @param options - Destination path, fixture log, and inline JavaScript body
 */
function writeFakeCli(
  {
    cliPath,
    logFile,
    body,
  }: {
    readonly cliPath: string;
    readonly logFile: string;
    readonly body: string;
  },
): void {
  mkdirSync(
    dirname(cliPath,),
    { recursive: true },
  );
  writeFileSync(
    cliPath,
    [
      // The interpreter running this test, so the fixture works under any runtime.
      `#!${process.execPath}`,
      `const fs = require('node:fs');`,
      `fs.appendFileSync(${JSON.stringify(logFile,)}, JSON.stringify(process.argv.slice(2,)) + '\\n');`,
      body,
    ].join('\n',),
    'utf8',
  );
  chmodSync(
    cliPath,
    0o755,
  );
}

/**
 Reads the fake-CLI log as one argv array per line.
 
 @param fixture - Fixture holding the log file
 
 @returns Argv arrays in invocation order
 */
function readCliLog(fixture: Fixture,): readonly (readonly string[])[] {
  if (!existsSync(fixture.logFile,))
    return [];
  return readFileSync(
    fixture.logFile,
    'utf8',
  )
    .split('\n',)
    .filter(
    (line,) => line.trim() !== '',
    )
    .map(
    (line,) => JSON.parse(line,) as string[],
    );
}

/**
 Plan entry fields a fixture case overrides.
 */
type FixtureEntryOverrides = {
  readonly id?: ToolPlanEntry['id'];
  readonly request?: ToolPlanEntry['request'];
  readonly version?: ToolPlanEntry['version'];
  readonly route?: ToolPlanEntry['route'];
  readonly manager?: string;
  readonly packageId?: string;
  readonly bins?: ToolPlanEntry['bins'];
  readonly options?: ToolPlanEntry['options'];
  readonly artifacts?: ToolPlanEntry['artifacts'];
};

/**
 Builds a plan entry for one fixture case.
 
 @param overrides - Fields to set over the base entry
 
 @returns A plan entry shaped like the real plan
 */
function fixtureEntry(overrides: FixtureEntryOverrides,): ToolPlanEntry {
  return {
    id: 'fixture-tool',
    request: { track: 'latest' },
    version: '1.2.3',
    route: 'bin-sourced',
    bins: ['fixture-tool'],
    options: {},
    env: {},
    artifacts: {},
    ...overrides,
  };
}

/**
 Replaces the process `PATH` until the returned handle is disposed.
 
 @param path - `PATH` value to install
 
 @returns Handle that restores the earlier `PATH` when disposed
 */
function overridePath(path: string,): Disposable {
  /**
   `PATH` before the override.
   */
  const previousPath = process.env.PATH;
  process.env.PATH = path;
  return {
    [Symbol.dispose](): void {
      process.env.PATH = previousPath;
    },
  };
}

//endregion Fixtures

/**
 File name suffixes of archive payloads the archive route extracts.
 */
const ARCHIVE_SUFFIXES: readonly string[] = [
  '.tar.gz',
  '.tgz',
  '.tar.xz',
  '.tar.bz2',
  '.tar',
  '.zip',
];

await describe({
  name: 'toolchain provider convergence',
  concurrency: 1,
  children: [
    //region bin CLI location

    describe({
      name: installBinSourced.name,
      children: [
        it({
          name: 'installs through the bin CLI at its bootstrap install directory',
          fn: async () => {
            /**
             Fixture whose `bin` CLI sits where the archive route installs it.
             */
            const fixture = makeFixture();
            /**
             Bootstrap install directory of the `bin` CLI.
             */
            const bootDir = toolInstallDir({
              toolchainRoot: fixture.paths.toolchainRoot,
              id: BIN_BOOTSTRAP.id,
              version: BIN_BOOTSTRAP.version,
            },);
            writeFakeCli({
              cliPath: join(
                bootDir,
                'bin',
              ),
              logFile: fixture.logFile,
              body: [
                "const path = require('node:path');",
                "const args = process.argv.slice(2);",
                "const dest = args[5] ?? 'fixture-tool';",
                String.raw`fs.writeFileSync(path.join(path.dirname(process.argv[1]), '..', '..', 'bin', dest), '#!/bin/sh\n');`,
                'process.exit(0);',
              ].join('\n',),
            },);
            /**
             Entry whose artifact URL pins the release tag.
             */
            const entry = fixtureEntry({
              artifacts: {
                [hostPlatformId()]: {
                  url: 'https://github.com/example/demo/releases/download/v1.2.3/demo.tar.gz',
                  sha256: '0'.repeat(64,),
                },
              },
            },);
            await installBinSourced({
              entry,
              paths: fixture.paths,
              env: fixture.env,
              extraDirs: [],
            },);
            /**
             Argv log of the bootstrap CLI: the install ran through it.
             */
            const log = readCliLog(fixture,);
            expect(log.length,)
              .toBeGreaterThan(0,);
            expect(log[0]?.[0],)
              .toBe('install',);
            expect(
              existsSync(join(
                fixture.paths.toolchainBinDir,
                'fixture-tool',
              ),),
            )
              .toBe(true,);
          },
        },),

        it({
          name: 'throws when no bin CLI resolves instead of reporting success',
          fn: async () => {
            /**
             Fixture with no `bin` CLI installed anywhere.
             */
            const fixture = makeFixture();
            let caught: unknown;
            try {
              await installBinSourced({
                entry: fixtureEntry({
                  artifacts: {
                    [hostPlatformId()]: {
                      url: 'https://github.com/example/demo/releases/download/v1.2.3/demo.tar.gz',
                      sha256: '0'.repeat(64,),
                    },
                  },
                },),
                paths: fixture.paths,
                env: fixture.env,
                extraDirs: [],
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,)
              .toBeInstanceOf(Error,);
            expect((caught as Error)
              .message,)
              .toContain('bin',);
          },
        },),

        it({
          name: 'throws when the bin CLI fails every attempt instead of reporting success',
          fn: async () => {
            /**
             Fixture whose `bin` CLI always fails.
             */
            const fixture = makeFixture();
            writeFakeCli({
              cliPath: join(
                toolInstallDir({
                  toolchainRoot: fixture.paths.toolchainRoot,
                  id: BIN_BOOTSTRAP.id,
                  version: BIN_BOOTSTRAP.version,
                },),
                'bin',
              ),
              logFile: fixture.logFile,
              body: 'process.exit(1);',
            },);
            let caught: unknown;
            try {
              await installBinSourced({
                entry: fixtureEntry({
                  artifacts: {
                    [hostPlatformId()]: {
                      url: 'https://github.com/example/demo/releases/download/v1.2.3/demo.tar.gz',
                      sha256: '0'.repeat(64,),
                    },
                  },
                },),
                paths: fixture.paths,
                env: fixture.env,
                extraDirs: [],
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,)
              .toBeInstanceOf(Error,);
          },
        },),
      ],
    },),

    //endregion bin CLI location

    //region Status probe

    describe({
      name: observeTool.name,
      children: [
        it({
          name: 'observes bin-sourced versions through the bootstrap CLI install directory',
          fn: async () => {
            /**
             Fixture whose `bin list` reports one locked version.
             */
            const fixture = makeFixture();
            writeFakeCli({
              cliPath: join(
                toolInstallDir({
                  toolchainRoot: fixture.paths.toolchainRoot,
                  id: BIN_BOOTSTRAP.id,
                  version: BIN_BOOTSTRAP.version,
                },),
                'bin',
              ),
              logFile: fixture.logFile,
              body: String.raw`process.stdout.write('/x/fixture-tool  v9.9.9  https://example  ok\n');`,
            },);
            /**
             Entry declaring the reported binary.
             */
            const entry = fixtureEntry({ bins: ['fixture-tool'], },);
            expect(await observeTool({
              entry,
              paths: fixture.paths,
              env: fixture.env,
            },),)
              .toBe('9.9.9',);
          },
        },),
        it({
          name: 'observes uv tool versions across registry name spelling',
          fn: async () => {
            /**
             Fixture whose `uv tool list` reports the registry-normalized name.
             */
            const fixture = makeFixture();
            /**
             Staged `uv` entry the probe resolves as its sibling.
             */
            const uvEntry = fixtureEntry({
              id: 'uv',
              version: '0.13.4',
              route: 'archive',
              bins: ['uv'],
            },);
            writeFakeCli({
              cliPath: join(
                toolInstallDir({
                  toolchainRoot: fixture.paths.toolchainRoot,
                  id: uvEntry.id,
                  version: uvEntry.version,
                },),
                'uv',
              ),
              logFile: fixture.logFile,
              body: String.raw`process.stdout.write('huggingface-hub v2.2.0\n');`,
            },);
            /**
             Entry spelled with the PyPI underscore form.
             */
            const entry = fixtureEntry({
              id: 'pipx:huggingface_hub',
              route: 'mpm-uvx',
              version: '2.2.0',
              manager: 'uvx',
              packageId: 'huggingface_hub',
              bins: ['hf'],
            },);
            expect(await observeTool({
              entry,
              paths: fixture.paths,
              env: fixture.env,
              tools: [uvEntry, entry,],
            },),)
              .toBe('2.2.0',);
          },
        },),
      ],
    },),

    //endregion Status probe

    //region Manager commands and failures

    describe({
      name: 'manager installs',
      children: [
        it({
          name: 'installMpmPnpm runs the toolchain pnpm, not a PATH shim',
          fn: async () => {
            /**
             Fixture with a toolchain pnpm and a decoy shim earlier on PATH.
             */
            const fixture = makeFixture();
            /**
             Decoy directory standing in for retired machine shims.
             */
            const decoyDir = join(
              fixture.root,
              'decoy',
            );
            writeFakeCli({
              cliPath: join(
                decoyDir,
                'pnpm',
              ),
              logFile: join(
                fixture.root,
                'decoy.log',
              ),
              body: 'process.exit(0);',
            },);
            writeFakeCli({
              cliPath: join(
                fixture.paths.toolchainBinDir,
                'pnpm',
              ),
              logFile: fixture.logFile,
              body: 'process.exit(0);',
            },);
            /**
             PATH whose first entries are the decoy shims.
             */
            using pathOverride = overridePath(`${decoyDir}:${process.env.PATH ?? ''}`,);
            await installMpmPnpm({
              entry: fixtureEntry({
                id: 'npm:fixture',
                route: 'mpm-pnpm',
                packageId: 'fixture-pkg',
                bins: ['fixture-pkg'],
              },),
              paths: fixture.paths,
              env: fixture.env,
              extraDirs: [fixture.paths.toolchainBinDir],
            },);
            expect(readCliLog(fixture,)
              .length,)
              .toBeGreaterThan(0,);
            expect(
              existsSync(join(
                fixture.root,
                'decoy.log',
              ),),
            )
              .toBe(false,);
          },
        },),

        it({
          name: 'installCargoBinstall throws when the install command fails',
          fn: async () => {
            /**
             Fixture whose toolchain cargo always fails.
             */
            const fixture = makeFixture();
            writeFakeCli({
              cliPath: join(
                fixture.paths.toolchainBinDir,
                'cargo',
              ),
              logFile: fixture.logFile,
              body: 'process.exit(3);',
            },);
            /**
             PATH restricted to the fixture so no real cargo resolves.
             */
            using pathOverride = overridePath(fixture.paths.toolchainBinDir,);
            let caught: unknown;
            try {
              await installCargoBinstall({
                entry: fixtureEntry({
                  id: 'cargo:fixture',
                  route: 'cargo-binstall',
                  packageId: 'fixture-crate',
                  bins: ['fixture-crate'],
                },),
                paths: fixture.paths,
                env: fixture.env,
                extraDirs: [fixture.paths.toolchainBinDir],
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,)
              .toBeInstanceOf(Error,);
          },
        },),
      ],
    },),

    //endregion Manager commands and failures

    //region Scoped installs

    describe({
      name: installCargoGit.name,
      children: [
        it({
          name: 'installs the pinned git revision into the tool install directory',
          fn: async () => {
            /**
             Fixture whose fake `cargo` records its argv.
             */
            const fixture = makeFixture();
            writeFakeCli({
              cliPath: join(
                fixture.paths.toolchainBinDir,
                'cargo',
              ),
              logFile: fixture.logFile,
              body: 'process.exit(0);',
            },);
            /**
             Install directory the `--root` argument must name.
             */
            const installDir = toolInstallDir({
              toolchainRoot: fixture.paths.toolchainRoot,
              id: 'cargo:example/demo',
              version: 'ref:85e3eb7',
            },);
            using pathOverride = overridePath(fixture.paths.toolchainBinDir,);
            await installCargoGit({
              entry: fixtureEntry({
                id: 'cargo:example/demo',
                request: { pinned: 'ref:85e3eb7', },
                version: 'ref:85e3eb7',
                route: 'cargo-git',
                manager: 'cargo',
                packageId: 'demo-lsp',
                bins: ['demo-lsp'],
                options: {
                  cargoGit: {
                    url: 'https://github.com/example/demo',
                  },
                },
              },),
              paths: fixture.paths,
              env: fixture.env,
              extraDirs: [fixture.paths.toolchainBinDir],
            },);
            /**
             Recorded invocations.
             */
            const calls = readCliLog(fixture,);
            expect(calls.length,)
              .toBe(1,);
            expect(calls[0],)
              .toEqual([
                'install',
                '--git',
                'https://github.com/example/demo',
                '--rev',
                '85e3eb7',
                '--root',
                installDir,
                'demo-lsp',
              ],);
          },
        },),
        it({
          name: 'throws naming the tool when the git build fails',
          fn: async () => {
            /**
             Fixture whose fake `cargo` fails every build.
             */
            const fixture = makeFixture();
            writeFakeCli({
              cliPath: join(
                fixture.paths.toolchainBinDir,
                'cargo',
              ),
              logFile: fixture.logFile,
              body: "process.stderr.write('build failed\\n');\nprocess.exit(1);",
            },);
            using pathOverride = overridePath(fixture.paths.toolchainBinDir,);
            /**
             Provider rejection.
             */
            let caught: unknown;
            try {
              await installCargoGit({
                entry: fixtureEntry({
                  id: 'cargo:example/demo',
                  request: { pinned: 'ref:85e3eb7', },
                  version: 'ref:85e3eb7',
                  route: 'cargo-git',
                  packageId: 'demo-lsp',
                  bins: ['demo-lsp'],
                  options: {
                    cargoGit: {
                      url: 'https://github.com/example/demo',
                    },
                  },
                },),
                paths: fixture.paths,
                env: fixture.env,
                extraDirs: [fixture.paths.toolchainBinDir],
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,)
              .toBeInstanceOf(Error,);
            expect((caught as Error).message.includes('cargo:example/demo',),)
              .toBe(true,);
          },
        },),
        it({
          name: 'throws when the entry carries no git source',
          fn: async () => {
            /**
             Fixture needing no CLIs: the provider rejects first.
             */
            const fixture = makeFixture();
            /**
             Provider rejection.
             */
            let caught: unknown;
            try {
              await installCargoGit({
                entry: fixtureEntry({
                  id: 'cargo:example/demo',
                  request: { pinned: 'ref:85e3eb7', },
                  version: 'ref:85e3eb7',
                  route: 'cargo-git',
                  packageId: 'demo-lsp',
                  bins: ['demo-lsp'],
                },),
                paths: fixture.paths,
                env: fixture.env,
                extraDirs: [fixture.paths.toolchainBinDir],
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,)
              .toBeInstanceOf(Error,);
            expect((caught as Error).message.includes('cargoGit',),)
              .toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: installGoInstall.name,
      children: [
        it({
          name: 'builds the module at its locked version into a scoped GOBIN',
          fn: async () => {
            /**
             Fixture whose fake `go` records its argv.
             */
            const fixture = makeFixture();
            /**
             Staged `go` entry the provider resolves as its sibling.
             */
            const goEntry = fixtureEntry({
              id: 'go',
              version: '1.27.3',
              route: 'archive',
              bins: ['go'],
            },);
            writeFakeCli({
              cliPath: join(
                toolInstallDir({
                  toolchainRoot: fixture.paths.toolchainRoot,
                  id: goEntry.id,
                  version: goEntry.version,
                },),
                'go',
              ),
              logFile: fixture.logFile,
              body: 'process.exit(0);',
            },);
            /**
             Entry being installed.
             */
            const entry = fixtureEntry({
              id: 'go:github.com/example/demo',
              request: { track: 'latest', },
              version: '0.98.1',
              route: 'go-install',
              packageId: 'github.com/example/demo',
              bins: ['demo'],
            },);
            using pathOverride = overridePath(fixture.paths.toolchainBinDir,);
            await installGoInstall({
              entry,
              paths: fixture.paths,
              env: fixture.env,
              extraDirs: [fixture.paths.toolchainBinDir],
              tools: [goEntry, entry,],
            },);
            /**
             Recorded invocations.
             */
            const calls = readCliLog(fixture,);
            expect(calls.length,)
              .toBe(1,);
            expect(calls[0],)
              .toEqual([
                'install',
                'github.com/example/demo@v0.98.1',
              ],);
            expect(
              existsSync(join(
                toolInstallDir({
                  toolchainRoot: fixture.paths.toolchainRoot,
                  id: entry.id,
                  version: entry.version,
                },),
                'bin',
              ),),
            )
              .toBe(true,);
          },
        },),
        it({
          name: 'throws when go is not staged in the scope plan',
          fn: async () => {
            /**
             Fixture whose scope plan declares no `go`.
             */
            const fixture = makeFixture();
            /**
             Provider rejection.
             */
            let caught: unknown;
            try {
              await installGoInstall({
                entry: fixtureEntry({
                  id: 'go:github.com/example/demo',
                  version: '0.98.1',
                  route: 'go-install',
                  packageId: 'github.com/example/demo',
                  bins: ['demo'],
                },),
                paths: fixture.paths,
                env: fixture.env,
                extraDirs: [fixture.paths.toolchainBinDir],
                tools: [],
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,)
              .toBeInstanceOf(Error,);
            expect((caught as Error).message.includes('no tool',),)
              .toBe(true,);
          },
        },),
      ],
    },),

    //endregion Scoped installs

    //region bin config seeding

    describe({
      name: seedBinConfig.name,
      children: [
        it({
          name: 'converges an existing empty config with default_path',
          fn: async () => {
            /**
             Fixture root holding the scope's own config file.
             */
            const fixture = makeFixture();
            /**
             Scope-local configuration path.
             */
            const configPath = join(
              fixture.root,
              'toolchain',
              'bin.config.json',
            );
            mkdirSync(
              dirname(configPath,),
              { recursive: true },
            );
            writeFileSync(
              configPath,
              '',
              'utf8',
            );
            seedBinConfig({
              downloadDir: fixture.paths.toolchainBinDir,
              configPath,
            },);
            /**
             Seeded configuration contents.
             */
            const seeded = JSON.parse(
              readFileSync(
                configPath,
                'utf8',
              ),) as { default_path?: string; };
            expect(seeded.default_path,)
              .toBe(fixture.paths.toolchainBinDir,);
          },
        },),
      ],
    },),

    //endregion bin config seeding

    //region Archive payloads

    describe({
      name: installArchive.name,
      children: [
        it({
          name: 'decompresses a gzip single-file payload before installing it',
          timeout: 30_000,
          fn: async () => {
            /**
             Fixture serving one gzip-compressed binary.
             */
            const fixture = makeFixture();
            /**
             Payload bytes the archive decompresses to.
             */
            const payload = Buffer.from('ELF-like payload bytes\n', 'utf8',);
            /**
             Gzip bytes the server serves.
             */
            const compressed = gzipSync(payload,);
            /**
             Serving directory for the gzip artifact.
             */
            const stage = join(
              fixture.root,
              'stage',
            );
            mkdirSync(
              stage,
              { recursive: true },
            );
            /**
             Artifact file name and path.
             */
            const asset = 'taplo-linux-x86_64.gz';
            writeFileSync(
              join(
                stage,
                asset,
              ),
              compressed,
            );
            /**
             Loopback server exposing the gzip artifact.
             */
            await using server = await serveDir(stage,);
            /**
             Entry whose artifact is a gzip single-file payload.
             */
            const entry = fixtureEntry({
              id: 'taplo',
              route: 'archive',
              version: '0.10.0',
              bins: ['taplo', 'taplo-linux-x86_64'],
              options: {
                binAliases: { 'taplo-linux-x86_64': 'taplo' },
              },
              artifacts: {
                [hostPlatformId()]: {
                  url: `${server.url}/${asset}`,
                  sha256: sha256Bytes(compressed,),
                },
              },
            },);
            /**
             Install directory the provider stages into.
             */
            const installDir = await installArchive({
              entry,
              paths: fixture.paths,
              env: fixture.env,
              extraDirs: [],
            },);
            /**
             Installed payload bytes.
             */
            const installed = readFileSync(join(
              installDir,
              'taplo',
            ),);
            expect(Buffer.compare(
              installed,
              payload,
            ),)
              .toBe(0,);
          },
        },),
      ],
    },),

    //endregion Archive payloads

    //region Plan data invariants

    describe({
      name: 'plan bin declarations',
      children: [
        it({
          name: 'single-file archive tools resolve every declared bin name through the copy or an alias',
          fn: async () => {
            for (const entry of ALL_TOOLS) {
              // Only archive-route entries use the single-file copy; the
              // mpm-uvx wheel installs its own console scripts.
              if (entry.route !== 'archive')
                continue;
              /**
               Artifact rows whose URL is a single-file payload.
               */
              const singleFileRows = Object.values(entry.artifacts,)
                .filter(
                function isSingleFile(row,): boolean {
                  return !ARCHIVE_SUFFIXES.some(
                    function endsTheUrl(suffix,): boolean {
                      return row.url
                        .endsWith(suffix,);
                    },
                  );
                },
                );
              if (singleFileRows.length === 0)
                continue;
              /**
               Names the single-file copy creates: the primary plus aliases.
               */
              const [copyName,] = entry.bins;
              /**
               Alias targets declared for this entry.
               */
              const aliasTargets = new Set(
                Object.values(entry.options.binAliases ?? {},),
              );
              for (const name of entry.bins) {
                if (name === copyName)
                  continue;
                expect(
                  (entry.options.binAliases?.[name] === copyName)
                    || aliasTargets.has(name,),
                )
                  .toBe(true,);
              }
            }
          },
        },),
      ],
    },),

    //endregion Plan data invariants
  ],
},);

//region Loopback file server

/**
 Loopback HTTP server exposing one directory's files.
 */
type FileServer = {
  /** Base URL of the server. */
  readonly url: string;
  /** Closes the server when the handle is disposed. */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Starts a loopback server that serves files from one directory.
 
 @param options - Absolute directory to serve
 
 @returns Server handle with its base URL
 */
async function serveDir(dir: string,): Promise<FileServer> {
  /**
   Server instance serving `/<name>` from the directory.
   */
  const server: Server = createHttpServer(
    function serve(request, response,) {
      /**
       Requested file path inside the served directory.
       */
      const requested = join(
        dir,
        (request.url ?? '/')
          .split('/')
          .at(-1) ?? '',
      );
      if (!existsSync(requested,)) {
        response.writeHead(404,);
        response.end();
        return;
      }
      response.writeHead(200,);
      response.end(readFileSync(requested,),);
    },
  );
  server.listen(
    0,
    '127.0.0.1',
  );
  await new Promise<void>(
    (resolve,) => {
      server.once(
        'listening',
        () => resolve(),
      );
    },
  );
  /**
   Bound address carrying the ephemeral port.
   */
  const address = server.address();
  if ((address === null) || ((typeof address) === 'string'))
    throw new Error('fixture server did not bind a TCP port',);
  return {
    url: `http://127.0.0.1:${
      address.port
    }`,
    async [Symbol.asyncDispose]() {
      await new Promise<void>(
        (resolve,) => {
          server.close(
            () => resolve(),
          );
        },
      );
    },
  };
}

/**
 SHA-256 hex digest of one buffer.
 
 @param bytes - Buffer to digest
 
 @returns Lowercase hex digest
 */
function sha256Bytes(bytes: Buffer,): string {
  return createHash('sha256',)
    .update(bytes,)
    .digest('hex',);
}

//endregion Loopback file server
