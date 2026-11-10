import { existsSync, } from 'node:fs';
import { readFile, } from 'node:fs/promises';
import { createRequire, } from 'node:module';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

//region Fixtures

/**
 Folder holding this test and the entry point it loads.
 */
const packageFolder = import.meta.dirname;

/**
 Repository root: this package sits at `package/pnpm-override/stub-throwing`, three folders below it.
 */
const repositoryRoot = join(
  packageFolder,
  '..',
  '..',
  '..',
);

/**
 Prefix the install hook prints on every substitution line, which the refusal repeats so an install log and a failed
 import name the same policy decision.
 */
const BLOCKLIST_PREFIX = '[blocked-dep]';

/**
 Repository file holding the per-package entry whose `throw` action substituted this package.
 The refusal names it so a reader searching for their blocked package lands on the table that lists it.
 */
const POLICY_TABLE_FILE = '.pnpmfile.policies.json';

/**
 Action whose entry in the policy table substitutes this package for the blocked dependency.
 */
const THROW_ACTION = 'throw';

/**
 Repository files that hold no per-package entry for a throwing substitution: the `readPackage` hook that applies
 the policy table, and the `overrides` block that owns removal cases.
 Searching either for a `throw`-policy package finds nothing, so the refusal must not send a reader there.
 */
const MECHANISM_FILES = [
  '.pnpmfile.mjs',
  'pnpm-workspace.yaml',
] as const;

/**
 Prefix of a repo-wide doc path, which repository rule DPL keeps under `doc/`.
 */
const REPOSITORY_DOC_PREFIX = 'doc/';

/**
 CommonJS loader a consumer uses, anchored at this test so a package name resolves through its own exports map.
 */
const requireFromHere = createRequire(import.meta.url,);

/**
 Loads a specifier through the CommonJS loader and hands back what it threw,
 so each case asserts on the error itself.

 @param specifier - Entry point path, or the package name a consumer imports

 @returns What loading threw, or `undefined` when it returned
 */
function loadError(specifier: string,): unknown {
  try {
    requireFromHere(specifier,);
  }
  catch (error) {
    return error;
  }
  return undefined;
}

/**
 Text of the refusal the entry point throws.

 @returns Message of the thrown error

 @throws Error when loading did not throw, so the stub no longer refuses the blocked package
 */
function refusalMessage(): string {
  const error = loadError('./index.cjs',);
  if (Error.isError(error,))
    return error.message;
  throw new Error('Loading ./index.cjs did not throw, so the stub no longer refuses the blocked package.',);
}

/**
 Fields of this package's manifest that the cases derive their expected values from.
 */
type PackageManifest = {
  /** Published name a consumer imports, which the refusal names as the substitute. */
  readonly name: string;

  /** Folder this package publishes from, which the policy reference cross-references. */
  readonly repository: { readonly directory: string; };
};

/**
 Reads this package's own manifest,
 so the expected name and folder come from the package instead of a copy that drifts from it.

 @returns Fields the cases assert on

 @example
 ```ts
 const manifest = await readManifest();
 ```
 */
async function readManifest(): Promise<PackageManifest> {
  return JSON.parse(await readFile(join(packageFolder, 'package.json',), 'utf8',),) as PackageManifest;
}

/**
 One entry of the policy table, keyed by the npm package name the install hook rewrites.
 */
type PolicyEntry = {
  /** Substitution the hook applies: `throw` lands this package, `silent` lands the silent stub. */
  readonly action: string;
};

/**
 Whether a policy-table key names a package rather than carrying a comment.

 @param key - Key as the table spells it

 @returns `true` for a package name, `false` for a table comment
 */
function isPackageKey(key: string,): boolean {
  return !(key.startsWith('//',));
}

/**
 Reads the repository's policy table, the data the install hook indexes by package name.
 Keys beginning with `//` are the table's own comments, which are not package entries.

 @returns Every named package's policy entry

 @example
 ```ts
 const table = await readPolicyTable();
 ```
 */
async function readPolicyTable(): Promise<Record<string, PolicyEntry>> {
  const raw = await readFile(join(repositoryRoot, POLICY_TABLE_FILE,), 'utf8',);
  return JSON.parse(raw,) as Record<string, PolicyEntry>;
}

/**
 Reads the repository-relative doc path the refusal sends a reader to,
 taken from the message so the case follows the reference a reader would follow.
 The path stands as its own whitespace-separated token in the message.

 @param message - Text of the refusal

 @returns Path as the message spells it

 @throws Error when the message names no doc path, quoting the message
 */
function namedDocPath(message: string,): string {
  const docPath = message.split(' ',).find((token) => token.startsWith(REPOSITORY_DOC_PREFIX,),);
  if (docPath === undefined)
    throw new Error(`The refusal names no doc path: ${message}`,);
  return docPath;
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    //region Loading

    describe({
      name: 'load',
      children: [
        it({
          name: 'throws an Error instead of exporting the blocked package',
          fn: async () => {
            expect(loadError('./index.cjs',),).toBeInstanceOf(Error,);
          },
        },),
        it({
          name: 'throws the same refusal when a consumer resolves the package name through the exports map',
          fn: async () => {
            const manifest = await readManifest();
            const byName = loadError(manifest.name,);
            expect(byName,).toBeInstanceOf(Error,);
            expect((byName as Error).message,).toBe(refusalMessage(),);
          },
        },),
      ],
    },),
    //endregion Loading

    //region Refusal message

    describe({
      name: 'refusal message',
      children: [
        it({
          name: 'starts with the blocklist prefix the install hook prints',
          fn: async () => {
            expect(refusalMessage().startsWith(BLOCKLIST_PREFIX,),).toBe(true,);
          },
        },),
        it({
          name: 'names the policy table holding the blocklist entry, which lists the throwing substitution',
          fn: async () => {
            const message = refusalMessage();
            expect(message,).toContain(POLICY_TABLE_FILE,);
            const policyTablePath = join(repositoryRoot, POLICY_TABLE_FILE,);
            expect(existsSync(policyTablePath,),).toBe(true,);
            const table = await readPolicyTable();
            const throwEntries = Object.entries(table,)
              .filter(([key,],) => isPackageKey(key,))
              .filter(([, entry],) => entry.action === THROW_ACTION);
            expect(throwEntries.length,).toBeGreaterThan(0,);
          },
        },),
        it({
          name: 'sends no reader to the hook or the overrides block, which carry no entry for this substitution',
          fn: async () => {
            const message = refusalMessage();
            for (const mechanismFile of MECHANISM_FILES)
              expect(message,).not.toContain(mechanismFile,);
          },
        },),
        it({
          name: 'describes itself the way the README does, naming the same policy table and the same doc',
          fn: async () => {
            const message = refusalMessage();
            const readme = await readFile(join(packageFolder, 'README.md',), 'utf8',);
            expect(readme,).toContain(POLICY_TABLE_FILE,);
            expect(readme,).toContain(namedDocPath(message,),);
          },
        },),
        it({
          name: 'names this package as the substitute',
          fn: async () => {
            const manifest = await readManifest();
            expect(refusalMessage(),).toContain(manifest.name,);
          },
        },),
        it({
          name: 'points at a doc under doc/ that exists and cross-references this package folder',
          fn: async () => {
            const docPath = namedDocPath(refusalMessage(),);
            const docFile = join(repositoryRoot, docPath,);
            expect(existsSync(docFile,),).toBe(true,);
            const manifest = await readManifest();
            expect(await readFile(docFile, 'utf8',),).toContain(manifest.repository.directory,);
          },
        },),
      ],
    },),
    //endregion Refusal message
  ],
},);
