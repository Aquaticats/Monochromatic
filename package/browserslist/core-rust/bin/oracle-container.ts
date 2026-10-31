#!/usr/bin/env node
/**
 Runs inside the conformance container: installs npm browserslist 4.29.5, swaps in the locked caniuse-lite and
 node-releases tarballs, and records what the real package answers for every case in `/work/cases.json`.

 The container has the `/work` directory mounted read-write.
 It holds `cases.json` (input), the verified tarballs in `tgz/` (input), and receives `out.json` (output).
 */
import { execFileSync, } from 'node:child_process';
import { mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync, } from 'node:fs';
import { createRequire, } from 'node:module';

/** Mounted work directory. */
const WORK_DIRECTORY = '/work';

/** Directory that receives the npm install and its `node_modules`. */
const INSTALL_DIRECTORY = `${WORK_DIRECTORY}/npm`;

/** The one browserslist release the Rust crate conforms to. */
const BROWSERSLIST_VERSION = '4.29.5';

/** One case of `cases.json`. */
type Case = {
  readonly name: string;
  readonly supported: boolean;
  readonly config: string;
  readonly nodeVersions?: readonly string[];
};

/** What the real package answered for a case. */
type Answer = { readonly result: readonly string[]; } | { readonly error: string; };

/**
 Install browserslist into the install directory without running any lifecycle script.
 */
function installBrowserslist(): void {
  mkdirSync(INSTALL_DIRECTORY, { recursive: true, },);
  writeFileSync(`${INSTALL_DIRECTORY}/package.json`, '{"private":true}\n',);
  execFileSync(
    'npm',
    [
      'install',
      '--ignore-scripts',
      '--no-audit',
      '--no-fund',
      '--prefix',
      INSTALL_DIRECTORY,
      `browserslist@${BROWSERSLIST_VERSION}`,
    ],
    { stdio: 'inherit', },
  );
}

/**
 Replace an installed package directory with the contents of a verified tarball.

 @param packageName - installed package to replace.
 @param tarballPrefix - file name prefix of the tarball in `tgz/`.
 */
function swapInTarball({ packageName, tarballPrefix, }: { packageName: string; tarballPrefix: string; },): void {
  const tarballs = readdirSync(`${WORK_DIRECTORY}/tgz`,).filter((name,) => name.startsWith(`${tarballPrefix}-`,));
  if (tarballs.length !== 1) {
    throw new Error(`expected exactly one ${tarballPrefix} tarball in tgz/, found ${tarballs.length}`,);
  }
  const target = `${INSTALL_DIRECTORY}/node_modules/${packageName}`;
  rmSync(target, { recursive: true, force: true, },);
  mkdirSync(target, { recursive: true, },);
  execFileSync('tar', [
    '--extract',
    '--gzip',
    '--file',
    `${WORK_DIRECTORY}/tgz/${tarballs[0]}`,
    '--directory',
    target,
    '--strip-components=1',
  ],);
}

/**
 Ask the real package about one case, following the repository's call path.

 @param browserslist - the loaded package function.
 @param testCase - config text and optional Node.js version override.
 @returns the sorted target list, or the error message the package threw.
 */
function ask(
  { browserslist, testCase, }: {
    browserslist: ((queries: readonly string[], options: object,) => string[]) & {
      parseConfig: (text: string,) => { defaults: string[]; };
      nodeVersions: string[];
    };
    testCase: Case;
  },
): Answer {
  const originalNodeVersions = browserslist.nodeVersions;
  if (testCase.nodeVersions !== undefined) {
    browserslist.nodeVersions = [...testCase.nodeVersions,];
  }
  try {
    const config = browserslist.parseConfig(testCase.config,);
    return { result: browserslist(config.defaults, { path: false, stats: {}, },), };
  } catch (error) {
    return { error: error instanceof Error ? error.message : String(error,), };
  } finally {
    browserslist.nodeVersions = originalNodeVersions;
  }
}

process.env['BROWSERSLIST_DISABLE_CACHE'] = '1';
installBrowserslist();
swapInTarball({ packageName: 'caniuse-lite', tarballPrefix: 'caniuse-lite', },);
swapInTarball({ packageName: 'node-releases', tarballPrefix: 'node-releases', },);

const require = createRequire(`${INSTALL_DIRECTORY}/`,);
const browserslist = require('browserslist',);
const input = JSON.parse(readFileSync(`${WORK_DIRECTORY}/cases.json`, 'utf8',),) as { cases: Case[]; };
const versionOf = (name: string,): string => {
  const manifest = JSON.parse(readFileSync(`${INSTALL_DIRECTORY}/node_modules/${name}/package.json`, 'utf8',),) as {
    version: string;
  };
  return manifest.version;
};

const output = {
  browserslist: versionOf('browserslist',),
  caniuseLite: versionOf('caniuse-lite',),
  nodeReleases: versionOf('node-releases',),
  npmFirefoxEsr: (browserslist(['Firefox ESR',], { path: false, stats: {}, },) as string[])[0],
  cases: input.cases.map((testCase,) => ({ ...testCase, npm: ask({ browserslist, testCase, },), })),
};
writeFileSync(`${WORK_DIRECTORY}/out.json`, `${JSON.stringify(output, null, 2,)}\n`,);
