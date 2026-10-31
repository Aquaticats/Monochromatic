import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
  resolve,
} from 'node:path';

import nanoSpawn from 'nano-spawn';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  assertPnprPackageName,
  checkPnprConfig,
  describeDrift,
  describeFirstDifference,
  generatePnprConfig,
  PNPR_CONFIG_PATH,
  PNPR_EXCLUDED_PACKAGES,
  pnprManifestHasEntryPoint,
  PnprConfigDriftError,
  readPnprPublishTarget,
  readWorkspaceEntries,
  renderPnprConfig,
  renderPnprConfigForWorkspace,
  selectPnprPublishedNames,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Repository root, four levels above this source file (the bundled artifact sits at a different depth).
 */
const REPOSITORY_ROOT = resolve(import.meta.dirname, '../../../..',);

/**
 Path of the generator command-line entry, run as a subprocess.
 */
const GENERATOR_CLI = join(import.meta.dirname, 'generate-config.ts',);

/**
 Workspace package fixtures: directory and manifest.
 */
type FixturePackage = {
  readonly name: string;
  readonly directory: string;
  readonly manifest: Readonly<Record<string, unknown>>;
};

/**
 Packages every fixture workspace carries so the checked-in exclusions still name workspace packages.
 */
const EXCLUDED_FIXTURE_PACKAGES: readonly FixturePackage[] = Object.keys(PNPR_EXCLUDED_PACKAGES,)
  .map(function toFixture(name,): FixturePackage {
    return {
      name,
      directory: `package/fixture-excluded/${name.slice('@monochromatic-dev/'.length,)}`,
      manifest: {
        name,
        version: '1.0.0',
        exports: { '.': './dist/index.js', },
      },
    };
  },);

/**
 Publishable fixture package whose name sorts last.
 */
const PUBLISHED_Z: FixturePackage = {
  name: '@monochromatic-dev/z-pkg',
  directory: 'package/fixture/z',
  manifest: {
    name: '@monochromatic-dev/z-pkg',
    version: '1.0.0',
    exports: { '.': './dist/index.js', },
  },
};

/**
 Publishable fixture package whose name sorts first.
 */
const PUBLISHED_A: FixturePackage = {
  name: '@monochromatic-dev/a-pkg',
  directory: 'package/fixture/a',
  manifest: {
    name: '@monochromatic-dev/a-pkg',
    version: '1.0.0',
    main: 'dist/index.js',
  },
};

/**
 Writes fixture manifests under a root.

 @param rootDir - Disposable workspace root.

 @param packages - Fixture packages to write.

 @example
 ```ts
 await writeWorkspace({ rootDir, packages: [PUBLISHED_A] });
 ```
 */
async function writeWorkspace(
  {
    rootDir,
    packages,
  }: {
    readonly rootDir: string;
    readonly packages: readonly FixturePackage[];
  },
): Promise<void> {
  await Promise.all(packages.map(async function writeOne(fixture,) {
    await mkdir(join(rootDir, fixture.directory,), { recursive: true, },);
    await writeFile(join(rootDir, fixture.directory, 'package.json',), JSON.stringify(fixture.manifest,), 'utf8',);
  },),);
}

/**
 Disposable fixture workspace; removed on scope exit.
 */
type DisposableWorkspace = {
  readonly rootDir: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates a fresh fixture workspace holding the exclusion packages plus the given ones.

 @param packages - Fixture packages to write.

 @returns Handle to use with `await using`.

 @example
 ```ts
 await using workspace = await makeWorkspace({ packages: [PUBLISHED_A] });
 ```
 */
async function makeWorkspace(
  { packages, }: { readonly packages: readonly FixturePackage[]; },
): Promise<DisposableWorkspace> {
  const rootDir = await mkdtemp(join(scratchRoot(), 'pnpr-config-test-',),);
  await writeWorkspace({
    rootDir,
    packages: [
      ...EXCLUDED_FIXTURE_PACKAGES,
      ...packages,
    ],
  },);
  return {
    rootDir,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(rootDir, {
        recursive: true,
        force: true,
      },);
    },
  };
}

/**
 Runs a body inside a fresh disposable workspace and removes it afterwards.

 @param packages - Fixture packages to write before the body runs.

 @param body - Receives the workspace root.

 @example
 ```ts
 await withWorkspace({ packages: [PUBLISHED_A], body: async (rootDir) => { void rootDir; } });
 ```
 */
async function withWorkspace(
  {
    packages,
    body,
  }: {
    readonly packages: readonly FixturePackage[];
    readonly body: (rootDir: string,) => Promise<void>;
  },
): Promise<void> {
  await using workspace = await makeWorkspace({ packages, },);
  await body(workspace.rootDir,);
}

/**
 Captures the error an async call rejects with.

 @param call - Operation expected to reject.

 @returns The rejection value, or `undefined` when the call resolved.

 @example
 ```ts
 const caught = await caughtBy(async () => { throw new Error('x'); });
 ```
 */
async function caughtBy(call: () => Promise<unknown>,): Promise<unknown> {
  try {
    await call();
  }
  catch (error) {
    return error;
  }
  return undefined;
}

/**
 Reads the generated config of a fixture workspace.

 @param rootDir - Workspace root.

 @returns File text.

 @example
 ```ts
 await readConfig(rootDir);
 ```
 */
async function readConfig(rootDir: string,): Promise<string> {
  return readFile(join(rootDir, PNPR_CONFIG_PATH,), 'utf8',);
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    //region Real workspace

    describe({
      name: 'real workspace',
      children: [
        it({
          name: 'renders the checked-in config byte for byte',
          fn: async () => {
            const { content, } = await renderPnprConfigForWorkspace({ rootDir: REPOSITORY_ROOT, },);
            const checkedIn = await readFile(join(REPOSITORY_ROOT, PNPR_CONFIG_PATH,), 'utf8',);
            expect(content,).toBe(checkedIn,);
          },
        },),
        it({
          name: 'selects the package names the checked-in trust list holds',
          fn: async () => {
            const { publishedNames, } = await renderPnprConfigForWorkspace({ rootDir: REPOSITORY_ROOT, },);
            const checkedIn = await readFile(join(REPOSITORY_ROOT, PNPR_CONFIG_PATH,), 'utf8',);
            expect(publishedNames,).toEqual([...readPnprPublishTarget(checkedIn,).packageNames,].toSorted(),);
            expect(publishedNames.length,).toBeGreaterThan(0,);
          },
        },),
        it({
          name: 'counts every workspace manifest',
          fn: async () => {
            const { workspacePackageCount, } = await renderPnprConfigForWorkspace({ rootDir: REPOSITORY_ROOT, },);
            const entries = await readWorkspaceEntries({ rootDir: REPOSITORY_ROOT, },);
            expect(workspacePackageCount,).toBe(entries.length,);
          },
        },),
        it({
          name: 'check passes without writing',
          fn: async () => {
            const before = await readFile(join(REPOSITORY_ROOT, PNPR_CONFIG_PATH,), 'utf8',);
            await checkPnprConfig({ rootDir: REPOSITORY_ROOT, },);
            const after = await readFile(join(REPOSITORY_ROOT, PNPR_CONFIG_PATH,), 'utf8',);
            expect(after,).toBe(before,);
          },
        },),
      ],
    },),

    //endregion Real workspace

    //region Selection

    describe({
      name: pnprManifestHasEntryPoint.name,
      children: [
        {
          label: 'a string exports field',
          manifest: { exports: './dist/index.js', },
          expected: true,
        },
        {
          label: 'an empty string exports field',
          manifest: { exports: '', },
          expected: false,
        },
        {
          label: 'exports with a published subpath',
          manifest: { exports: { '.': './dist/index.js', './ts': './src/index.ts', }, },
          expected: true,
        },
        {
          label: 'exports with only the ./ts subpath',
          manifest: { exports: { './ts': './src/index.ts', }, },
          expected: false,
        },
        {
          label: 'exports with only ./ts/ nested subpaths',
          manifest: { exports: { './ts/a.ts': './src/a.ts', }, },
          expected: false,
        },
        {
          label: 'exports using root conditions',
          manifest: { exports: { import: './dist/index.mjs', }, },
          expected: true,
        },
        {
          label: 'an empty exports map',
          manifest: { exports: {}, },
          expected: false,
        },
        {
          label: 'a null exports field',
          manifest: { exports: null, },
          expected: false,
        },
        {
          label: 'a loadable main without exports',
          manifest: { main: 'dist/index.js', },
          expected: true,
        },
        {
          label: 'a loadable module without exports',
          manifest: { module: 'dist/index.mjs', },
          expected: true,
        },
        {
          label: 'main shadowed by exports that publish nothing',
          manifest: { main: 'dist/index.js', exports: { './ts': './src/index.ts', }, },
          expected: false,
        },
        {
          label: 'a TypeScript source main',
          manifest: { main: 'src/index.ts', },
          expected: false,
        },
        {
          label: 'a string bin to JavaScript',
          manifest: { bin: 'dist/cli.js', },
          expected: true,
        },
        {
          label: 'a string bin to TypeScript source',
          manifest: { bin: 'src/cli.ts', },
          expected: false,
        },
        {
          label: 'a bin map with one loadable target',
          manifest: { bin: { a: 'src/a.ts', b: 'dist/b.js', }, },
          expected: true,
        },
        {
          label: 'a bin map of TypeScript source only',
          manifest: { bin: { a: 'src/a.ts', }, },
          expected: false,
        },
        {
          label: 'a null bin',
          manifest: { bin: null, },
          expected: false,
        },
        {
          label: 'no entry fields',
          manifest: { name: 'x', },
          expected: false,
        },
      ].map(function entryPointCase({
        label,
        manifest,
        expected,
      },) {
        return it({
          name: `${expected ? 'accepts' : 'rejects'} ${label}`,
          fn: async () => {
            expect(pnprManifestHasEntryPoint({ manifest, },),).toBe(expected,);
          },
        },);
      },),
    },),
    describe({
      name: assertPnprPackageName.name,
      children: [
        it({
          name: 'accepts a scoped name with npm characters',
          fn: async () => {
            expect(() => {
              assertPnprPackageName({ name: '@monochromatic-dev/or-throw-core', },);
            },).not.toThrow();
          },
        },),
        ...[
          {
            label: 'a name outside the scope',
            name: 'left-pad',
          },
          {
            label: 'a bare scope',
            name: '@monochromatic-dev/',
          },
          {
            label: 'a single quote that would break the YAML string',
            name: "@monochromatic-dev/a'b",
          },
          {
            label: 'an uppercase letter',
            name: '@monochromatic-dev/Abc',
          },
          {
            label: 'a newline',
            name: '@monochromatic-dev/a\nb',
          },
          {
            label: 'a non-ASCII character',
            name: '@monochromatic-dev/café',
          },
        ].map(function invalidNameCase({
          label,
          name,
        },) {
          return it({
            name: `rejects ${label}`,
            fn: async () => {
              expect(() => {
                assertPnprPackageName({ name, },);
              },).toThrow('is not a @monochromatic-dev/ name',);
            },
          },);
        },),
      ],
    },),
    describe({
      name: selectPnprPublishedNames.name,
      children: [
        it({
          name: 'sorts names and drops fixtures, versionless, entry-less, and excluded packages',
          fn: async () => {
            const names = selectPnprPublishedNames({
              workspacePackages: [
                {
                  name: PUBLISHED_Z.name,
                  directory: PUBLISHED_Z.directory,
                  manifest: PUBLISHED_Z.manifest,
                },
                {
                  name: PUBLISHED_A.name,
                  directory: PUBLISHED_A.directory,
                  manifest: PUBLISHED_A.manifest,
                },
                {
                  name: '@monochromatic-dev/in-test-fixture',
                  directory: 'package/test-fixture/x',
                  manifest: { name: '@monochromatic-dev/in-test-fixture', version: '1.0.0', main: 'a.js', },
                },
                {
                  name: '@monochromatic-dev/no-version',
                  directory: 'package/fixture/no-version',
                  manifest: { name: '@monochromatic-dev/no-version', main: 'a.js', },
                },
                {
                  name: '@monochromatic-dev/no-entry',
                  directory: 'package/fixture/no-entry',
                  manifest: { name: '@monochromatic-dev/no-entry', version: '1.0.0', },
                },
                {
                  name: '@monochromatic-dev/excluded',
                  directory: 'package/fixture/excluded',
                  manifest: { name: '@monochromatic-dev/excluded', version: '1.0.0', main: 'a.js', },
                },
              ],
              excludedPackages: { '@monochromatic-dev/excluded': 'reviewed', },
            },);
            expect(names,).toEqual([
              '@monochromatic-dev/a-pkg',
              '@monochromatic-dev/z-pkg',
            ],);
          },
        },),
        it({
          name: 'throws when an exclusion names no workspace package',
          fn: async () => {
            expect(() => {
              selectPnprPublishedNames({
                workspacePackages: [],
                excludedPackages: { '@monochromatic-dev/gone': 'reviewed', },
              },);
            },).toThrow('exclusions name no workspace package: @monochromatic-dev/gone',);
          },
        },),
        it({
          name: 'throws when a published name is outside the scope',
          fn: async () => {
            expect(() => {
              selectPnprPublishedNames({
                workspacePackages: [
                  {
                    name: 'left-pad',
                    directory: 'package/fixture/left-pad',
                    manifest: { name: 'left-pad', version: '1.0.0', main: 'a.js', },
                  },
                ],
                excludedPackages: {},
              },);
            },).toThrow('is not a @monochromatic-dev/ name',);
          },
        },),
        it({
          name: 'defaults to the checked-in exclusions',
          fn: async () => {
            expect(() => {
              selectPnprPublishedNames({ workspacePackages: [], },);
            },).toThrow('exclusions name no workspace package',);
          },
        },),
      ],
    },),

    //endregion Selection

    //region Rendering

    describe({
      name: renderPnprConfig.name,
      children: [
        it({
          name: 'lists names and exclusion reasons',
          fn: async () => {
            const text = renderPnprConfig({
              publishedNames: [
                '@monochromatic-dev/a-pkg',
                '@monochromatic-dev/z-pkg',
              ],
              excludedPackages: { '@monochromatic-dev/skipped': 'cannot build', },
            },);
            expect(text,).toContain('# Published package count: 2\n# Reviewed exclusions:\n#   @monochromatic-dev/skipped: cannot build\n',);
            expect(text,).toContain("          packages:\n            - '@monochromatic-dev/a-pkg'\n            - '@monochromatic-dev/z-pkg'\n\nui:\n",);
            expect(text,).toContain(`secret: \${PNPR_SECRET}\n`,);
            expect(text.endsWith('  level: info\n',),).toBe(true,);
          },
        },),
        it({
          name: 'says none when nothing is excluded',
          fn: async () => {
            const text = renderPnprConfig({
              publishedNames: [],
              excludedPackages: {},
            },);
            expect(text,).toContain('# Reviewed exclusions: none\n',);
            expect(text,).toContain('# Published package count: 0\n',);
          },
        },),
        ...['\n', '\r',].map(function lineBreakCase(lineBreak,) {
          return it({
            name: `rejects an exclusion reason containing ${JSON.stringify(lineBreak,)}`,
            fn: async () => {
              expect(() => {
                renderPnprConfig({
                  publishedNames: [],
                  excludedPackages: { '@monochromatic-dev/skipped': `a${lineBreak}b`, },
                },);
              },).toThrow('exclusion reason for @monochromatic-dev/skipped must be one line',);
            },
          },);
        },),
        it({
          name: 'renders the checked-in exclusions by default',
          fn: async () => {
            const text = renderPnprConfig({ publishedNames: [], },);
            for (const name of Object.keys(PNPR_EXCLUDED_PACKAGES,))
              expect(text,).toContain(`#   ${name}: `,);
          },
        },),
      ],
    },),

    //endregion Rendering

    //region Diagnostics

    describe({
      name: describeFirstDifference.name,
      children: [
        it({
          name: 'names the first unequal line',
          fn: async () => {
            expect(describeFirstDifference({ expected: 'a\nb\nc', actual: 'a\nx\nc', },),).toBe(
              'The first difference is on line 2: expected "b", found "x".',
            );
          },
        },),
        it({
          name: 'notes when the file ends early',
          fn: async () => {
            expect(describeFirstDifference({ expected: 'a\nb', actual: 'a', },),).toBe(
              'The first difference is on line 2: expected "b", found end of file.',
            );
          },
        },),
        it({
          name: 'notes when the file runs longer',
          fn: async () => {
            expect(describeFirstDifference({ expected: 'a', actual: 'a\nb', },),).toBe(
              'The first difference is on line 2: expected end of file, found "b".',
            );
          },
        },),
      ],
    },),
    describe({
      name: describeDrift.name,
      children: [
        it({
          name: 'reports an unreadable trust list',
          fn: async () => {
            expect(describeDrift({
              expected: 'a',
              actual: 'b',
              listsDiffer: false,
              listUnreadable: true,
            },),).toContain('no readable OIDC trust list',);
          },
        },),
        it({
          name: 'points at the first difference when package lists differ',
          fn: async () => {
            expect(describeDrift({
              expected: 'a',
              actual: 'b',
              listsDiffer: true,
              listUnreadable: false,
            },),).toBe('The first difference is on line 1: expected "a", found "b".',);
          },
        },),
        it({
          name: 'says the lists match when only other content differs',
          fn: async () => {
            expect(describeDrift({
              expected: 'a',
              actual: 'b',
              listsDiffer: false,
              listUnreadable: false,
            },),).toContain('The package lists match; other content differs.',);
          },
        },),
      ],
    },),

    //endregion Diagnostics

    //region Generate and check in a disposable workspace

    describe({
      name: generatePnprConfig.name,
      children: [
        it({
          name: 'writes the config, then reports it up to date',
          fn: async () => {
            await withWorkspace({
              packages: [
                PUBLISHED_Z,
                PUBLISHED_A,
              ],
              body: async (rootDir,) => {
                expect(await generatePnprConfig({ rootDir, },),).toBe(true,);
                const first = await readConfig(rootDir,);
                expect(first,).toBe((await renderPnprConfigForWorkspace({ rootDir, },)).content,);
                expect(readPnprPublishTarget(first,).packageNames,).toEqual([
                  '@monochromatic-dev/a-pkg',
                  '@monochromatic-dev/z-pkg',
                ],);
                expect(await generatePnprConfig({ rootDir, },),).toBe(false,);
              },
            },);
          },
        },),
        it({
          name: 'rewrites a drifted config',
          fn: async () => {
            await withWorkspace({
              packages: [PUBLISHED_A,],
              body: async (rootDir,) => {
                await mkdir(dirname(join(rootDir, PNPR_CONFIG_PATH,),), { recursive: true, },);
                await writeFile(join(rootDir, PNPR_CONFIG_PATH,), 'stale\n', 'utf8',);
                expect(await generatePnprConfig({ rootDir, },),).toBe(true,);
                await checkPnprConfig({ rootDir, },);
              },
            },);
          },
        },),
        it({
          name: 'rethrows read failures other than absence',
          fn: async () => {
            await withWorkspace({
              packages: [PUBLISHED_A,],
              body: async (rootDir,) => {
                // A directory where the file belongs makes the read fail with EISDIR.
                await mkdir(join(rootDir, PNPR_CONFIG_PATH,), { recursive: true, },);
                const caught = await caughtBy(async () => generatePnprConfig({ rootDir, },),);
                expect(caught,).toBeInstanceOf(Error,);
                expect(caught,).not.toBeInstanceOf(PnprConfigDriftError,);
              },
            },);
          },
        },),
      ],
    },),
    describe({
      name: checkPnprConfig.name,
      children: [
        it({
          name: 'passes on a freshly generated config',
          fn: async () => {
            await withWorkspace({
              packages: [PUBLISHED_A,],
              body: async (rootDir,) => {
                await generatePnprConfig({ rootDir, },);
                await checkPnprConfig({ rootDir, },);
              },
            },);
          },
        },),
        it({
          name: 'names the file and each package a new workspace package would add',
          fn: async () => {
            await withWorkspace({
              packages: [PUBLISHED_A,],
              body: async (rootDir,) => {
                await generatePnprConfig({ rootDir, },);
                await writeWorkspace({ rootDir, packages: [PUBLISHED_Z,], },);
                const caught = await caughtBy(async () => checkPnprConfig({ rootDir, },),);
                expect(caught,).toBeInstanceOf(PnprConfigDriftError,);
                const drift = caught as InstanceType<typeof PnprConfigDriftError>;
                expect(drift.configPath,).toBe(PNPR_CONFIG_PATH,);
                expect(drift.missingPackages,).toEqual(['@monochromatic-dev/z-pkg',],);
                expect(drift.unexpectedPackages,).toEqual([],);
                expect(drift.message,).toContain(`${PNPR_CONFIG_PATH} is not what the generator renders`,);
                expect(drift.message,).toContain('Packages the file lacks (1): @monochromatic-dev/z-pkg',);
                expect(drift.message,).toContain('mise run //package/config/pnpr:generate:config',);
                expect(drift.message,).not.toContain('should not publish',);
              },
            },);
          },
        },),
        it({
          name: 'names each package the file trusts but the workspace no longer publishes',
          fn: async () => {
            await withWorkspace({
              packages: [
                PUBLISHED_A,
                PUBLISHED_Z,
              ],
              body: async (rootDir,) => {
                await generatePnprConfig({ rootDir, },);
                await rm(join(rootDir, PUBLISHED_Z.directory,), { recursive: true, force: true, },);
                const caught = await caughtBy(async () => checkPnprConfig({ rootDir, },),);
                expect(caught,).toBeInstanceOf(PnprConfigDriftError,);
                const drift = caught as InstanceType<typeof PnprConfigDriftError>;
                expect(drift.missingPackages,).toEqual([],);
                expect(drift.unexpectedPackages,).toEqual(['@monochromatic-dev/z-pkg',],);
                expect(drift.message,).toContain(
                  'Packages the file lists that should not publish (1): @monochromatic-dev/z-pkg',
                );
                expect(drift.message,).not.toContain('Packages the file lacks',);
              },
            },);
          },
        },),
        it({
          name: 'reports a difference outside the package list',
          fn: async () => {
            await withWorkspace({
              packages: [PUBLISHED_A,],
              body: async (rootDir,) => {
                await generatePnprConfig({ rootDir, },);
                const text = await readConfig(rootDir,);
                await writeFile(join(rootDir, PNPR_CONFIG_PATH,), text.replace('enabled: false', 'enabled: true',), 'utf8',);
                const caught = await caughtBy(async () => checkPnprConfig({ rootDir, },),);
                expect(caught,).toBeInstanceOf(PnprConfigDriftError,);
                const drift = caught as InstanceType<typeof PnprConfigDriftError>;
                expect(drift.message,).toContain('The package lists match; other content differs.',);
                expect(drift.message,).toContain('expected "  enabled: false", found "  enabled: true"',);
              },
            },);
          },
        },),
        it({
          name: 'reports an absent file',
          fn: async () => {
            await withWorkspace({
              packages: [PUBLISHED_A,],
              body: async (rootDir,) => {
                const caught = await caughtBy(async () => checkPnprConfig({ rootDir, },),);
                expect(caught,).toBeInstanceOf(PnprConfigDriftError,);
                const drift = caught as InstanceType<typeof PnprConfigDriftError>;
                expect(drift.message,).toContain('The file does not exist.',);
                expect(drift.missingPackages,).toEqual(['@monochromatic-dev/a-pkg',],);
                const readCaught = await caughtBy(async () => readConfig(rootDir,),);
                expect(readCaught,).toBeInstanceOf(Error,);
              },
            },);
          },
        },),
        it({
          name: 'reports a file that is not valid YAML as unreadable rather than failing on the parser',
          fn: async () => {
            await withWorkspace({
              packages: [PUBLISHED_A,],
              body: async (rootDir,) => {
                await mkdir(dirname(join(rootDir, PNPR_CONFIG_PATH,),), { recursive: true, },);
                await writeFile(join(rootDir, PNPR_CONFIG_PATH,), 'a: [b\n', 'utf8',);
                const caught = await caughtBy(async () => checkPnprConfig({ rootDir, },),);
                expect(caught,).toBeInstanceOf(PnprConfigDriftError,);
                const drift = caught as InstanceType<typeof PnprConfigDriftError>;
                expect(drift.message,).toContain('no readable OIDC trust list',);
              },
            },);
          },
        },),
        it({
          name: 'reports a file without a readable trust list',
          fn: async () => {
            await withWorkspace({
              packages: [PUBLISHED_A,],
              body: async (rootDir,) => {
                await mkdir(dirname(join(rootDir, PNPR_CONFIG_PATH,),), { recursive: true, },);
                await writeFile(join(rootDir, PNPR_CONFIG_PATH,), 'not a config\n', 'utf8',);
                const caught = await caughtBy(async () => checkPnprConfig({ rootDir, },),);
                expect(caught,).toBeInstanceOf(PnprConfigDriftError,);
                const drift = caught as InstanceType<typeof PnprConfigDriftError>;
                expect(drift.message,).toContain('no readable OIDC trust list',);
                expect(drift.missingPackages,).toEqual(['@monochromatic-dev/a-pkg',],);
              },
            },);
          },
        },),
      ],
    },),

    //endregion Generate and check in a disposable workspace

    //region Command line

    describe({
      name: 'generate-config command line',
      children: [
        it({
          name: 'writes the config, then --check exits 0',
          fn: async () => {
            await withWorkspace({
              packages: [PUBLISHED_A,],
              body: async (rootDir,) => {
                await nanoSpawn('node', [GENERATOR_CLI, '--root', rootDir,],);
                await nanoSpawn('node', [GENERATOR_CLI, '--check', '--root', rootDir,],);
                expect(await readConfig(rootDir,),).toBe((await renderPnprConfigForWorkspace({ rootDir, },)).content,);
              },
            },);
          },
          timeout: 60_000,
        },),
        it({
          name: '--check exits non-zero naming the file and the differing package, and leaves the file alone',
          fn: async () => {
            await withWorkspace({
              packages: [PUBLISHED_A,],
              body: async (rootDir,) => {
                await nanoSpawn('node', [GENERATOR_CLI, '--root', rootDir,],);
                await writeWorkspace({ rootDir, packages: [PUBLISHED_Z,], },);
                const before = await readConfig(rootDir,);
                const caught = await caughtBy(async () => nanoSpawn('node', [GENERATOR_CLI, '--check', '--root', rootDir,],),);
                expect(caught,).toBeInstanceOf(Error,);
                const failure = caught as Error & { readonly exitCode?: number; readonly stderr?: string; };
                expect(failure.exitCode,).toBe(1,);
                expect(failure.stderr,).toContain(PNPR_CONFIG_PATH,);
                expect(failure.stderr,).toContain('@monochromatic-dev/z-pkg',);
                expect(await readConfig(rootDir,),).toBe(before,);
              },
            },);
          },
          timeout: 60_000,
        },),
      ],
    },),

    //endregion Command line
  ],
},);
