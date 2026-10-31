#!/usr/bin/env node
/**
 Runs inside the conformance container: installs npm browserslist 4.29.5, swaps in the locked caniuse-lite and
 node-releases tarballs, and records what the real package answers for every case in `/work/cases.json`.

 The container has the `/work` directory mounted read-write.
 It holds `cases.json` (input), the verified tarballs in `tgz/` (input), and receives `out.json` (output).
 The container also has this `bin` directory and the `caught-value/core` package mounted read-only at their
 repository-relative paths under `/repo`, with `/repo/node_modules/@monochromatic-dev/caught-value-core` a symlink to
 the mounted package, so the workspace import below resolves to a real path outside any `node_modules` directory
 (Node.js does not strip types from files under one).
 */
import {
  mkdir,
  readdir,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { createRequire, } from 'node:module';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { runCommand, } from './run-command.ts';

/**
 Mounted work directory.
 */
const WORK_DIRECTORY = '/work';

/**
 Directory that receives the npm install and its `node_modules`.
 */
const INSTALL_DIRECTORY = `${WORK_DIRECTORY}/npm`;

/**
 The one browserslist release the Rust crate conforms to.
 */
const BROWSERSLIST_VERSION = '4.29.5';

/**
 Spaces per indent level in the written `out.json`.
 */
const JSON_INDENT = 2;

/**
 One case of `cases.json`.
 */
type Case = {
  readonly name: string;
  readonly supported: boolean;
  readonly config: string;
  readonly nodeVersions?: readonly string[];
};

/**
 What the real package answered for a case.
 */
type Answer = { readonly result: readonly string[]; } | { readonly error: string; };

/**
 The parts of the npm package this script uses.
 */
type Browserslist = {
  (
    queries: readonly string[],
    options: object,
  ): readonly string[];
  readonly parseConfig: (text: string,) => { readonly defaults: readonly string[]; };
  nodeVersions: readonly string[];
};

/**
 Check that a value is an object with named properties.

 @param value - parsed JSON or loaded module

 @returns true when the value is a non-null object

 @example
 ```ts
 isRecord({});
 ```
 */
function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null);
}

/**
 Check that a value is a list of strings.

 @param value - parsed JSON

 @returns true when the value is an array whose items are all strings

 @example
 ```ts
 isStringList(['a']);
 ```
 */
function isStringList(value: unknown,): value is readonly string[] {
  return Array.isArray(value,)
    && value.every(function isString(item: unknown,) {
      return (typeof item) === 'string';
    },);
}

/**
 Check that a value is a conformance case.

 @param value - parsed JSON

 @returns true when the value has the name, flag and config text, and a string list or nothing as node versions

 @example
 ```ts
 isCase({ name: 'a', supported: true, config: 'defaults' });
 ```
 */
function isCase(value: unknown,): value is Case {
  if (!isRecord(value,))
    return false;
  if (((typeof value.name) !== 'string') || ((typeof value.supported) !== 'boolean'))
    return false;
  if ((typeof value.config) !== 'string')
    return false;
  return (value.nodeVersions === undefined) || isStringList(value.nodeVersions,);
}

/**
 Check that a loaded module is the browserslist function with the members this script uses.

 @param value - the module `require` returned

 @returns true when the value is a function with `parseConfig` and `nodeVersions`

 @example
 ```ts
 isBrowserslist(createRequire(import.meta.url)('browserslist'));
 ```
 */
function isBrowserslist(value: unknown,): value is Browserslist {
  if ((typeof value) !== 'function')
    return false;
  return ('parseConfig' in value)
    && ((typeof value.parseConfig) === 'function')
    && ('nodeVersions' in value)
    && isStringList(value.nodeVersions,);
}

/**
 Check that parsed JSON is a `cases.json` document.

 @param value - parsed JSON

 @returns true when the value has a `cases` list of conformance cases

 @example
 ```ts
 isCasesFile({ cases: [] });
 ```
 */
function isCasesFile(value: unknown,): value is { readonly cases: readonly Case[]; } {
  return isRecord(value,)
    && Array.isArray(value.cases,)
    && value.cases
      .every(isCase,);
}

/**
 Check that parsed JSON is a package manifest with a version.

 @param value - parsed JSON

 @returns true when the value has a string `version`

 @example
 ```ts
 isVersionManifest({ version: '1.0.0' });
 ```
 */
function isVersionManifest(value: unknown,): value is { readonly version: string; } {
  return isRecord(value,) && ((typeof value.version) === 'string');
}

/**
 Install browserslist into the install directory without running any lifecycle script.

 @example
 ```ts
 await installBrowserslist();
 ```
 */
async function installBrowserslist(): Promise<void> {
  await mkdir(
    INSTALL_DIRECTORY,
    { recursive: true, },
  );
  await writeFile(
    `${INSTALL_DIRECTORY}/package.json`,
    '{"private":true}\n',
  );
  await runCommand({
    command: 'npm',
    args: [
      'install',
      '--ignore-scripts',
      '--no-audit',
      '--no-fund',
      '--prefix',
      INSTALL_DIRECTORY,
      `browserslist@${BROWSERSLIST_VERSION}`,
    ],
  },);
}

/**
 Replace an installed package directory with the contents of a verified tarball.

 @param packageName - installed package to replace.

 @param tarballPrefix - file name prefix of the tarball in `tgz/`.

 @throws Error when `tgz/` holds no tarball or more than one with the prefix

 @example
 ```ts
 await swapInTarball({ packageName: 'caniuse-lite', tarballPrefix: 'caniuse-lite' });
 ```
 */
async function swapInTarball({
  packageName,
  tarballPrefix,
}: {
  readonly packageName: string;
  readonly tarballPrefix: string;
},): Promise<void> {
  /**
   Names in `tgz/` that start with the prefix and a dash.
   */
  const tarballs = (await readdir(`${WORK_DIRECTORY}/tgz`,))
    .filter(function hasPrefix(name,) {
      return name.startsWith(`${tarballPrefix}-`,);
    },);
  /**
   The one matching tarball, when exactly one matches.
   */
  const [tarball,] = tarballs;
  if ((tarballs.length !== 1) || (tarball === undefined))
    throw new Error(`expected exactly one ${tarballPrefix} tarball in tgz/, found ${tarballs.length}`,);
  /**
   Installed package directory that the tarball replaces.
   */
  const target = `${INSTALL_DIRECTORY}/node_modules/${packageName}`;
  await rm(
    target,
    {
      recursive: true,
      force: true,
    },
  );
  await mkdir(
    target,
    { recursive: true, },
  );
  await runCommand({
    command: 'tar',
    args: [
      '--extract',
      '--gzip',
      '--file',
      `${WORK_DIRECTORY}/tgz/${tarball}`,
      '--directory',
      target,
      '--strip-components=1',
    ],
  },);
}

/**
 Whether a Node.js version has no semver pre-release part (no dash once any `+build` suffix is removed).

 @param version - a version from node-releases.

 @returns true when the version is a stable release

 @example
 ```ts
 isStable('26.0.0');
 ```
 */
function isStable(version: string,): boolean {
  return !(version.split('+',)[0] ?? '').includes('-',);
}

/**
 Message of a value the package threw.

 `BrowserslistError` sets `prototype = Error.prototype` without calling the `Error` constructor, so it has no
 internal error slot: `Error.isError` rejects it and string conversion would prefix the message with the class name.
 The recorded answers are the bare message, which is what the package documents for its own errors.

 @param error - the caught value

 @returns the `message` string when the value has one, otherwise the shared caught-value text

 @example
 ```ts
 thrownMessage(new Error('offline'));
 ```
 */
function thrownMessage(error: unknown,): string {
  if (isRecord(error,) && ((typeof error.message) === 'string'))
    return error.message;
  return caughtValueText(error,);
}

/**
 Ask the real package about one case, following the repository's call path.

 @param browserslist - the loaded package function.

 @param testCase - config text and optional Node.js version override.

 @param dropNodePrereleases - when true, pre-release Node.js versions are removed from `browserslist.nodeVersions`
 first, which is the Rust crate's deliberate divergence (see doc/troubleshooting/browserslist-node-major-prerelease.md).

 @returns the sorted target list, or the error message the package threw.

 @mutates browserslist - `nodeVersions` holds the case's versions while the package answers and the original list afterwards.

 @example
 ```ts
 ask({ browserslist, testCase, dropNodePrereleases: false });
 ```
 */
function ask(
  {
    browserslist,
    testCase,
    dropNodePrereleases,
  }: {
    readonly browserslist: Browserslist;
    readonly testCase: Case;
    readonly dropNodePrereleases: boolean;
  },
): Answer {
  /**
   The package's own list, restored when the answer is ready.
   */
  const originalNodeVersions = browserslist.nodeVersions;
  /**
   The versions the case starts from.
   */
  const startingVersions = testCase.nodeVersions ?? originalNodeVersions;
  browserslist.nodeVersions = dropNodePrereleases ? startingVersions.filter(isStable,) : [...startingVersions,];
  /**
   Puts the package's own list back when the function ends, however it ends.
   */
  using restoration = {
    [Symbol.dispose](): void {
      browserslist.nodeVersions = originalNodeVersions;
    },
  };
  try {
    /**
     The parsed config, whose defaults are the queries.
     */
    const config = browserslist.parseConfig(testCase.config,);
    return { result: browserslist(
      config.defaults,
      {
        path: false,
        stats: {},
      },
    ), };
  } catch (error) {
    return { error: thrownMessage(error,), };
  }
}

/**
 Read the installed version of a package from its manifest.

 @param name - package directory name under the install's `node_modules`

 @returns the manifest's version

 @throws Error when the manifest has no string version

 @example
 ```ts
 await versionOf('caniuse-lite');
 ```
 */
async function versionOf(name: string,): Promise<string> {
  /**
   The parsed manifest.
   */
  const manifest: unknown = JSON.parse(await readFile(
    `${INSTALL_DIRECTORY}/node_modules/${name}/package.json`,
    'utf8',
  ),);
  if (!isVersionManifest(manifest,))
    throw new Error(`${name} has no version in its manifest`,);
  return manifest.version;
}

/**
 Install browserslist with the locked data and write `out.json`.

 @throws Error when `cases.json` is not a list of cases, or the installed package lacks the expected members

 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  process.env
    .BROWSERSLIST_DISABLE_CACHE = '1';
  await installBrowserslist();
  await swapInTarball({
    packageName: 'caniuse-lite',
    tarballPrefix: 'caniuse-lite',
  },);
  await swapInTarball({
    packageName: 'node-releases',
    tarballPrefix: 'node-releases',
  },);
  /**
   The installed package, loaded through a require function rooted in the install directory.
   */
  const browserslist: unknown = createRequire(`${INSTALL_DIRECTORY}/`,)('browserslist',);
  if (!isBrowserslist(browserslist,))
    throw new Error('the installed browserslist lacks parseConfig or nodeVersions',);
  /**
   The parsed `cases.json`.
   */
  const input: unknown = JSON.parse(await readFile(
    `${WORK_DIRECTORY}/cases.json`,
    'utf8',
  ),);
  if (!isCasesFile(input,))
    throw new Error('cases.json is not an object with a list of cases',);
  /**
   What the document records.
   */
  const output = {
    browserslist: await versionOf('browserslist',),
    caniuseLite: await versionOf('caniuse-lite',),
    nodeReleases: await versionOf('node-releases',),
    npmFirefoxEsr: browserslist(
      ['Firefox ESR',],
      {
        path: false,
        stats: {},
      },
    )[0],
    cases: input.cases
      .map(function answerCase(testCase,) {
        return {
          ...testCase,
          npm: ask({
            browserslist,
            testCase,
            dropNodePrereleases: false,
          },),
          npmPrereleaseFiltered: ask({
            browserslist,
            testCase,
            dropNodePrereleases: true,
          },),
        };
      },),
  };
  await writeFile(
    `${WORK_DIRECTORY}/out.json`,
    `${JSON.stringify(
      output,
      null,
      JSON_INDENT,
    )}\n`,
  );
}

await main();
