#!/usr/bin/env node
/**
 Pre-install toml-edit scope command. Only Node built-ins and local source are loaded.
 The checked-in TypeScript entry is the deployed artifact, not a bundled library.

 @module
 */

import { appendFile, } from 'node:fs/promises';

import { ScopeError, } from './ci-scope-error.ts';
import { scopeGit, } from './ci-scope-git.ts';

//region Input and path contracts

/**
 GitHub merge-group commit identifiers are full SHA-1 object names.
 */
const SHA_LENGTH = 40;
/**
 Accepted hexadecimal alphabet excludes revision operators and command options.
 */
const SHA_DIGITS = '0123456789abcdef';
/**
 Package roots of the toml-edit workflow: what its pull-request trigger lists, used when the calling workflow supplies no
 list of its own.
 */
const TOML_EDIT_PACKAGE_PREFIXES: readonly string[] = [
  'package/toml-edit/core/',
  'package/toml-edit/core.fuzz/',
  'package/test-fixture/toml-edit/',
];
/**
 Exact paths outside those package roots that change the toml-edit verification contract.
 */
const TOML_EDIT_CONTRACT_PATHS: readonly string[] = [
  'doc/decision/toml-edit-fuzzing.md',
  '.github/workflows/toml-edit-fuzz.yml',
];

/**
 A path list a calling workflow supplies, or the toml-edit default.

 The form is the environment variable's value, one path per line, which keeps a path holding a space in one piece. This is
 how a second workflow reaches the same fail-closed decision without a second copy of this program
 (doc/decision/toml-edit-ci-scope.md).

 @param name - environment key holding the list

 @param fallback - paths to match when the key is unset or empty

 @returns the paths to match

 @example
 ```ts
 pathList('SCOPE_PACKAGE_PREFIXES', TOML_EDIT_PACKAGE_PREFIXES);
 ```
 */
function pathList({
  name,
  fallback,
}: {
  readonly name: string;
  readonly fallback: readonly string[];
},): readonly string[] {
  /**
   Value the calling workflow supplied, if it supplied one.
   */
  const supplied = process.env[name];
  if ((supplied === undefined) || (supplied.trim() === ''))
    return fallback;
  return supplied
    .split('\n',)
    .map(function trim(line: string,): string {
      return line.trim();
    },)
    .filter(function nonBlank(line: string,): boolean {
      return line !== '';
    },);
}

/**
 Package roots whose additions, modifications, removals, or renames require verification.
 */
const PACKAGE_PREFIXES = pathList({
  name: 'SCOPE_PACKAGE_PREFIXES',
  fallback: TOML_EDIT_PACKAGE_PREFIXES,
},);
/**
 Exact paths outside the package roots that change the verification contract.
 */
const CONTRACT_PATHS = pathList({
  name: 'SCOPE_CONTRACT_PATHS',
  fallback: TOML_EDIT_CONTRACT_PATHS,
},);
/**
 Provided workflow name, when the calling workflow sets one.
 */
const providedSubject = process.env
  .SCOPE_SUBJECT;
/**
 Name the workflow prints, so a caller's log line says which workflow it decided for.
 */
const SUBJECT = providedSubject ?? 'toml-edit';

/**
 Require an immutable event revision rather than accepting a moving branch name.

 @param name - Environment key supplied from GitHub's merge-group payload.

 @returns Validated commit identifier.

 @throws {@link ScopeError} When the event revision is missing or malformed.

 @example
 ```ts
 const base = eventRevision('SCOPE_BASE_SHA');
 ```
 */
function eventRevision(name: string,): string {
  /**
   Event input, kept out of command construction until validated.
   */
  const value = process.env[name] ?? '';
  if (value.length !== SHA_LENGTH)
    throw new ScopeError(`${name} must contain the merge-group's full lowercase commit SHA.`,);
  for (let index = 0; index < value.length; index += 1) {
    if (!SHA_DIGITS.includes(value.charAt(index,),))
      throw new ScopeError(`${name} contains a non-hexadecimal commit character.`,);
  }
  return value;
}

/**
 Match package boundaries and exact files without interpreting filename bytes as syntax.

 @param path - One NUL-delimited Git path, including any embedded whitespace.

 @returns Whether this changed path requires toml-edit verification.

 @example
 ```ts
 relevantPath('package/toml-edit/core/src/index.ts'); // true
 ```
 */
function relevantPath(path: string,): boolean {
  return PACKAGE_PREFIXES.some(function matchesPackage(prefix: string,): boolean {
    return path.startsWith(prefix,);
  },) || CONTRACT_PATHS.some(function matchesContract(file: string,): boolean {
    return path === file;
  },);
}

//endregion Input and path contracts

//region Fail-closed scope decision

/**
 Compare the exact merge-group trees, preserving failures before any output is written.

 @returns Whether verification must run for this workflow event.

 @throws {@link ScopeError} When the event, checkout, ancestry, or comparison is unavailable.

 @example
 ```ts
 const run = await shouldRun();
 ```
 */
async function shouldRun(): Promise<boolean> {
  /**
   GitHub-owned event discriminator, never interpolated into executable source.
   */
  const event = process.env
    .GITHUB_EVENT_NAME;
  console.log(`Determining ${SUBJECT} scope for event ${String(event,)}`,);
  if ((event === 'push') || (event === 'pull_request'))
    return true;
  if (event !== 'merge_group')
    throw new ScopeError(`Unsupported scope event: ${String(event,)}`,);

  /**
   Immutable base of the queued merge group, not today's origin/main.
   */
  const base = eventRevision('SCOPE_BASE_SHA',);
  /**
   Immutable merge-group head that checkout and the comparison must agree on.
   */
  const head = eventRevision('SCOPE_HEAD_SHA',);
  await scopeGit([
    'cat-file',
    '-e',
    `${base}^{commit}`,
  ],);
  await scopeGit([
    'cat-file',
    '-e',
    `${head}^{commit}`,
  ],);
  /**
   Checked-out commit whose code subsequent verification will exercise.
   */
  const checkoutHead = (await scopeGit([
    'rev-parse',
    '--verify',
    'HEAD^{commit}',
  ],)).trim();
  if (checkoutHead !== head)
    throw new ScopeError(`Checkout HEAD ${checkoutHead} does not match merge-group head ${head}.`,);
  await scopeGit([
    'merge-base',
    '--is-ancestor',
    base,
    head,
  ],);

  // Disabling rename detection exposes both removed and added paths, including moves out of scope.
  // No diff-filter excludes deletions. NUL separators preserve tabs, quotes, and newlines in names.
  /**
   Complete path list, available only when Git reports successful comparison.
   */
  const changed = await scopeGit([
    'diff',
    '--name-only',
    '--no-renames',
    '-z',
    base,
    head,
    '--',
  ],);
  return changed.split('\0',)
    .some(relevantPath,);
}

/**
 Output location is mandatory even when this event always runs verification.
 */
const outputPath = process.env
  .GITHUB_OUTPUT;
if ((outputPath === undefined) || (outputPath === ''))
  throw new ScopeError('GITHUB_OUTPUT is required to publish the scope decision.',);
/**
 Final decision; no output file is touched before all required evidence succeeds.
 */
const run = await shouldRun();
console.log(run
  ? `Running ${SUBJECT} verification.`
  : `No ${SUBJECT} paths changed in the verified merge group; skipping the heavy steps.`,);
await appendFile(
  outputPath,
  `run=${String(run,)}\n`,
  'utf8',
);

//endregion Fail-closed scope decision
