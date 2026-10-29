/**
 Container matrix test for the toolchain providers.
 Runs inside a container via `mise.container-test.ts`; never on the host.

 Covers the provider paths task-1 verified by hand:
 - `bin-sourced` install with its `-n` fallbacks,
 - `archive` install with SHA-256 verification and a rejected-checksum case,
 - `cargo-binstall` invocation shape,
 - exposure-directory linking (including alias names),
 - the postinstall `git` link's atomic replace.

 Every case runs against disposable fixtures under the container's temp
 directory and local fake CLIs or a loopback HTTP server: no external
 network state is mutated.
 */

import {
  chmodSync,
  existsSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readlinkSync,
  readdirSync,
  writeFileSync,
} from 'node:fs';
import { createServer, type Server, } from 'node:http';
import { tmpdir, } from 'node:os';
import { dirname, join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { run, sha256File, } from './exec.ts';
import { rebuildExposure, runPostinstallLinks, } from './exposure.ts';
import { hostPlatformId, } from './layout.ts';
import {
  resolveToolchainPaths,
  toolInstallDir,
  type ToolchainPaths,
} from './paths.ts';
import { BIN_BOOTSTRAP, } from './plan.ts';
import {
  installArchive,
  installBinSourced,
  installCargoBinstall,
  type ProviderContext,
} from './providers.ts';
import type { ToolPlanEntry, } from './types.ts';

//region Fixtures

/**
 Disposable fixture: home, toolchain paths, and a fake-CLI bin directory.
 */
type Fixture = {
  readonly root: string;
  readonly paths: ToolchainPaths;
  readonly env: Record<string, string>;
  readonly logFile: string;
};

/**
 Builds a disposable fixture with its own home and toolchain root.
 
 @returns The fixture
 */
function makeFixture(): Fixture {
  /**
   Disposable fixture root, used as the fixture home.
   */
  const root = mkdtempSync(join(
    tmpdir(),
    'toolchain-provider-',
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
 Writes an executable fake CLI that appends its argv to the fixture log and
 runs a small inline behavior script.
 
 @param options - Fixture, binary name, and the JavaScript body handling argv
 */
function writeFakeCli(
  {
    fixture,
    name,
    body,
    dir,
  }: {
    readonly fixture: Fixture;
    readonly name: string;
    readonly body: string;
    readonly dir?: string;
  },
): string {
  /**
   Absolute path of the fake CLI.
   */
  const cliPath = join(
    dir ?? fixture.paths.toolchainBinDir,
    name,
  );
  mkdirSync(
    dirname(cliPath,),
    { recursive: true },
  );
  writeFileSync(
    cliPath,
    [
      // The interpreter running this test, so the fixture works under any
      // matrix runtime.
      `#!${process.execPath}`,
      `const fs = require('node:fs');`,
      `fs.appendFileSync(${JSON.stringify(fixture.logFile,)}, JSON.stringify(process.argv.slice(2,)) + '\\n');`,
      body,
    ].join('\n',),
    'utf8',
  );
  chmodSync(
    cliPath,
    0o755,
  );
  return cliPath;
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
  readonly version?: ToolPlanEntry['version'];
  readonly route?: ToolPlanEntry['route'];
  readonly packageId?: ToolPlanEntry['packageId'];
  readonly bins?: ToolPlanEntry['bins'];
  readonly options?: ToolPlanEntry['options'];
  readonly artifacts?: ToolPlanEntry['artifacts'];
};

/**
 Builds a plan entry for one fixture case.
 
 @param overrides - Fields to set over the base entry
 
 @returns A plan entry shaped like the real plan
 */
function fixtureEntry(
  overrides: FixtureEntryOverrides,
): ToolPlanEntry {
  return {
    id: 'fixture-tool',
    request: { track: 'latest' },
    version: '1.2.3',
    route: 'archive',
    manager: null,
    packageId: null,
    bins: ['fixture-tool'],
    os: null,
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

await describe({
  name: 'toolchain providers (container)',
  children: [
    //region bin-sourced

    describe({
      name: installBinSourced.name,
      children: [
        it({
          name: 'installs each primary binary through the nested-file pattern',
          fn: async () => {
            /**
             Fixture with a fake `bin` that only matches the nested pattern.
             */
            const fixture = makeFixture();
            writeFakeCli({
              fixture,
              name: 'bin',
              dir: join(
                toolInstallDir({
                  toolchainRoot: fixture.paths.toolchainRoot,
                  id: BIN_BOOTSTRAP.id,
                  version: BIN_BOOTSTRAP.version,
                },),
              ),
              body: [
                'const args = process.argv.slice(2);',
                'const pattern = args[4] ?? \'\';',
                "if (!pattern.includes('/**/')) process.exit(1);",
                "const path = require('node:path');",
                `fs.writeFileSync(${  JSON.stringify(join(
                  fixture.paths.toolchainBinDir,
                  'alpha',
                ),)  }, '#!/bin/sh\\n');`,
                'process.exit(0);',
              ].join('\n',),
            },);
            /**
             Entry whose artifact URL pins the release tag.
             */
            const entry = fixtureEntry({
              id: 'fixture-bin',
              route: 'bin-sourced',
              bins: ['alpha', 'alpha-shorthand'],
              options: {
                binAliases: { 'alpha-shorthand': 'alpha' },
              },
              artifacts: {
                [hostPlatformId()]: {
                  url: 'https://github.com/example/demo/releases/download/v1.2.3/demo.tar.gz',
                  sha256: '0'.repeat(64,),
                },
              },
            },);
            /**
             Provider context for the fixture.
             */
            const context: ProviderContext = {
              entry,
              paths: fixture.paths,
              env: fixture.env,
              extraDirs: [],
            };
            await installBinSourced(context,);
            /**
             Argv log of the fake `bin`.
             */
            const log = readCliLog(fixture,);
            expect(log.length,)
              .toBe(1,);
            expect(log[0]?.[0],)
              .toBe('install',);
            expect(log[0]?.slice(
              1,
              3,
            ),)
              .toEqual(
              [
                '--force',
                'https://github.com/example/demo/releases/tag/v1.2.3',
              ],
              );
            expect(log[0]?.slice(
              3,
              5,
            ),)
              .toEqual(
              ['--name', 'demo.tar.gz/**/alpha'],
              );
            expect(
              existsSync(join(
                fixture.paths.toolchainBinDir,
                'alpha',
              ),),
            )
              .toBe(true,);
          },
        },),
        it({
          name: 'falls back to the root-level and raw-asset patterns in order',
          fn: async () => {
            /**
             Fixture with a fake `bin` that only matches the raw-asset form.
             */
            const fixture = makeFixture();
            writeFakeCli({
              fixture,
              name: 'bin',
              dir: join(
                toolInstallDir({
                  toolchainRoot: fixture.paths.toolchainRoot,
                  id: BIN_BOOTSTRAP.id,
                  version: BIN_BOOTSTRAP.version,
                },),
              ),
              body: [
                'const args = process.argv.slice(2);',
                'const pattern = args[4] ?? \'\';',
                'const rename = args[5];',
                "if (rename === undefined) process.exit(1);",
                "const path = require('node:path');",
                String.raw`fs.writeFileSync(path.join(path.dirname(process.argv[1]), rename), '#!/bin/sh\n');`,
                'process.exit(0);',
              ].join('\n',),
            },);
            /**
             Entry with a raw (non-archive) release asset.
             */
            const entry = fixtureEntry({
              id: 'fixture-raw',
              route: 'bin-sourced',
              bins: ['rawbin'],
              artifacts: {
                [hostPlatformId()]: {
                  url: 'https://github.com/example/demo/releases/download/v2.0.0/demo-linux-amd64',
                  sha256: '1'.repeat(64,),
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
             Argv log: the two nested patterns must have failed before the
             raw-asset form with the destination rename succeeded.
             */
            const log = readCliLog(fixture,);
            expect(log,)
              .toEqual(
              [
                [
                  'install',
                  '--force',
                  'https://github.com/example/demo/releases/tag/v2.0.0',
                  '--name',
                  'demo-linux-amd64/**/rawbin',
                ],
                [
                  'install',
                  '--force',
                  'https://github.com/example/demo/releases/tag/v2.0.0',
                  '--name',
                  'demo-linux-amd64/rawbin',
                ],
                [
                  'install',
                  '--force',
                  'https://github.com/example/demo/releases/tag/v2.0.0',
                  '--name',
                  'demo-linux-amd64',
                  'rawbin',
                ],
              ],
              );
          },
        },),
      ],
    },),

    //endregion bin-sourced

    //region Archive

    describe({
      name: installArchive.name,
      children: [
        it({
          name: 'downloads, verifies SHA-256, and extracts the payload',
          fn: async () => {
            /**
             Fixture whose fake tool ships one executable inside a tarball.
             */
            const fixture = makeFixture();
            /**
             Staging directory for the payload the server will serve.
             */
            const stage = join(
              fixture.root,
              'stage',
            );
            mkdirSync(join(
              stage,
              'demo-1.2.3',
            ), { recursive: true },);
            writeFileSync(
              join(
                stage,
                'demo-1.2.3',
                'demo-tool',
              ),
              '#!/bin/sh\nexit 0\n',
              'utf8',
            );
            chmodSync(join(
              stage,
              'demo-1.2.3',
              'demo-tool',
            ), 0o755,);
            await run({
              command: 'tar',
              args: [
                '--create',
                '--gzip',
                '--file',
                join(
                  fixture.root,
                  'demo.tar.gz',
                ),
                '--directory',
                stage,
                'demo-1.2.3',
              ],
            },);
            /**
             SHA-256 of the fixture archive.
             */
            const digest = await sha256File(join(
              fixture.root,
              'demo.tar.gz',
            ),);
            /**
             Loopback server exposing the fixture archive.
             */
            await using server = await createArchiveServer({
              archivePath: join(
                fixture.root,
                'demo.tar.gz',
              ),
            },);
            /**
             Entry pinned to the served artifact and its real digest.
             */
            const entry = fixtureEntry({
              id: 'fixture-archive',
              route: 'archive',
              bins: ['demo-tool'],
              artifacts: {
                [hostPlatformId()]: {
                  url: `${server.url}/demo.tar.gz`,
                  sha256: digest,
                },
              },
            },);
            /**
             Install directory the provider extracts into.
             */
            const installDir = await installArchive({
              entry,
              paths: fixture.paths,
              env: fixture.env,
              extraDirs: [],
            },);
            expect(
              existsSync(join(
                installDir,
                'demo-1.2.3',
                'demo-tool',
              ),),
            )
              .toBe(true,);
          },
        },),
        it({
          name: 'rejects a download whose SHA-256 does not match the lock',
          fn: async () => {
            /**
             Fixture serving bytes whose digest will not match.
             */
            const fixture = makeFixture();
            writeFileSync(
              join(
                fixture.root,
                'demo.tar.gz',
              ),
              'not the expected bytes',
              'utf8',
            );
            /**
             Loopback server exposing the mismatching artifact.
             */
            await using server = await createArchiveServer({
              archivePath: join(
                fixture.root,
                'demo.tar.gz',
              ),
            },);
            /**
             Entry whose recorded digest is deliberately wrong.
             */
            const entry = fixtureEntry({
              id: 'fixture-bad',
              route: 'archive',
              bins: ['demo-tool'],
              artifacts: {
                [hostPlatformId()]: {
                  url: `${server.url}/demo.tar.gz`,
                  sha256: 'a'.repeat(64,),
                },
              },
            },);
            /**
             Rejected download: the digest check must refuse.
             */
            let caught: unknown;
            try {
              await installArchive({
                entry,
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
            // Refusal wording from the download verifier.
            expect((caught as Error).message,)
              .toContain('SHA-256 mismatch',);
            /**
             The rejected bytes must not have been left as the target.
             */
            expect(
              existsSync(join(
                fixture.paths.toolchainRoot,
                'downloads',
              ),)
                && (readdirSync(join(
                  fixture.paths.toolchainRoot,
                  'downloads',
                ),)
                  .filter(
                  (name,) => name.endsWith('.download',),
                  )
                  .length === 0),
            )
              .toBe(true,);
          },
        },),
      ],
    },),

    //endregion Archive

    //region cargo-binstall

    describe({
      name: installCargoBinstall.name,
      children: [
        it({
          name: 'drives cargo binstall with the locked version and features',
          fn: async () => {
            /**
             Fixture with a fake `cargo` capturing the invocation.
             */
            const fixture = makeFixture();
            writeFakeCli({
              fixture,
              name: 'cargo',
              body: 'process.exit(0);',
            },);
            /**
             Entry with a locked version and one cargo feature.
             */
            const entry = fixtureEntry({
              id: 'cargo:fixture',
              route: 'cargo-binstall',
              packageId: 'fixture-crate',
              version: '9.9.9',
              bins: ['fixture-crate'],
              options: { cargoFeatures: ['unix', 'extra'], },
              artifacts: {},
            },);
            // The provider resolves the cargo binary from PATH, so the
            // fixture prepends its fake cargo home the way a real machine would.
            using pathOverride = overridePath(`${fixture.paths.toolchainBinDir}:${process.env.PATH ?? ''}`,);
            await installCargoBinstall({
              entry,
              paths: fixture.paths,
              env: fixture.env,
              extraDirs: [fixture.paths.toolchainBinDir],
            },);
            expect(readCliLog(fixture,),)
              .toEqual(
              [
                [
                  'binstall',
                  '--no-confirm',
                  '--version',
                  '9.9.9',
                  '--features=unix',
                  '--features=extra',
                  'fixture-crate',
                ],
              ],
              );
          },
        },),
      ],
    },),

    //endregion cargo-binstall

    //region Exposure and postinstall

    describe({
      name: 'exposure directory',
      children: [
        it({
          name: 'links every declared bin atomically, aliases sharing one target',
          fn: async () => {
            /**
             Fixture with one installed binary behind two declared names.
             */
            const fixture = makeFixture();
            /**
             Installed target the links will resolve to.
             */
            const target = join(
              fixture.paths.toolchainBinDir,
              'demo-tool',
            );
            writeFileSync(
              target,
              '#!/bin/sh\nexit 0\n',
              'utf8',
            );
            chmodSync(
              target,
              0o755,
            );
            /**
             Entry declaring the primary name and one alias.
             */
            const entry = fixtureEntry({
              route: 'bin-sourced',
              bins: ['demo-tool', 'demo'],
              options: { binAliases: { demo: 'demo-tool' }, },
            },);
            /**
             Exposure rebuild report.
             */
            const report = rebuildExposure({
              paths: fixture.paths,
              env: fixture.env,
              entries: [entry],
              extraDirs: [fixture.paths.toolchainBinDir],
            },);
            expect(report.linked.toSorted(),)
              .toEqual(['demo', 'demo-tool'],);
            expect(report.missing,)
              .toEqual([],);
            expect(
              readlinkSync(join(
                fixture.paths.exposureDir,
                'demo-tool',
              ),),
            )
              .toBe(target,);
            expect(
              readlinkSync(join(
                fixture.paths.exposureDir,
                'demo',
              ),),
            )
              .toBe(target,);
            /**
             No temporary link names may survive the atomic replace.
             */
            expect(
              readdirSync(fixture.paths.exposureDir,)
                .filter(
                (name,) => name.includes('.tmp',),
                ),
            )
              .toEqual([],);
          },
        },),
        it({
          name: 'reports declared bins that do not resolve',
          fn: async () => {
            /**
             Fixture with an uninstalled declared bin.
             */
            const fixture = makeFixture();
            /**
             Entry declaring a binary that is not installed.
             */
            const entry = fixtureEntry({ bins: ['absent-tool'], },);
            expect(rebuildExposure({
              paths: fixture.paths,
              env: fixture.env,
              entries: [entry],
              extraDirs: [],
            },),)
              .toEqual({
                linked: [],
                missing: ['absent-tool'],
              },);
          },
        },),
      ],
    },),

    describe({
      name: 'postinstall git link',
      children: [
        it({
          name: 'creates the git link atomically and replaces it in place',
          fn: async () => {
            /**
             Fixture with the git-policy binary installed.
             */
            const fixture = makeFixture();
            /**
             Source binary the postinstall step links as `git`.
             */
            const source = join(
              fixture.paths.toolchainBinDir,
              'cli-git-native',
            );
            writeFileSync(
              source,
              '#!/bin/sh\nexit 0\n',
              'utf8',
            );
            chmodSync(
              source,
              0o755,
            );
            /**
             Entry carrying the cli-git-bin postinstall step.
             */
            const entry = fixtureEntry({
              route: 'bin-sourced',
              bins: ['cli-git-native'],
              options: {
                postinstall: {
                  when: 'always',
                  link: {
                    sourceBin: 'cli-git-native',
                    linkName: 'git',
                    linkDir: '.cache/cli-git-bin',
                  },
                },
              },
            },);
            /**
             Link directory and path the step owns.
             */
            const linkDir = join(
              fixture.root,
              '.cache',
              'cli-git-bin',
            );
            runPostinstallLinks({
              paths: fixture.paths,
              env: fixture.env,
              entries: [entry],
              extraDirs: [fixture.paths.toolchainBinDir],
            },);
            expect(
              readlinkSync(join(
                linkDir,
                'git',
              ),),
            )
              .toBe(source,);
            expect(readdirSync(linkDir,),)
              .toEqual(['git'],);
            expect(lstatSync(join(
              linkDir,
              'git',
            ),)
              .isSymbolicLink(),)
              .toBe(true,);
            // A second run must replace the link in place, leaving one entry.
            runPostinstallLinks({
              paths: fixture.paths,
              env: fixture.env,
              entries: [entry],
              extraDirs: [fixture.paths.toolchainBinDir],
            },);
            expect(readdirSync(linkDir,),)
              .toEqual(['git'],);
          },
        },),
      ],
    },),

    //endregion Exposure and postinstall
  ],
},);

//region Loopback archive server

/**
 Loopback HTTP server exposing one file as a downloadable archive.
 */
type ArchiveServer = {
  /** Base URL of the server. */
  readonly url: string;
  /** Closes the server and waits for it to stop when the handle is disposed. */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Starts a loopback server that serves one file under its base name.
 
 @param options - Absolute archive path to serve
 
 @returns Server handle with its base URL
 */
async function createArchiveServer(
  { archivePath, }: { readonly archivePath: string; },
): Promise<ArchiveServer> {
  /**
   Server instance; assigned synchronously so `url` can be built below.
   */
  const server: Server = createServer(
    function serve(_request, response,) {
      response.writeHead(
        200,
        { 'content-type': 'application/octet-stream', },
      );
      response.end(readFileSync(archivePath,),);
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
   Address the server bound to (port is ephemeral).
   */
  const address = server.address();
  if ((address === null) || ((typeof address) === 'string'))
    throw new Error('archive fixture server did not bind a TCP port',);
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

//endregion Loopback archive server
