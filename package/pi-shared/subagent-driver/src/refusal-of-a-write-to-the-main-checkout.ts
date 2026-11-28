/**
 The refusal an `edit` or `write` call meets when it names the main checkout of the repository a run works in.

 Reading the main checkout stays allowed, and so does every path outside it, every path of some worktree of the repository, and
 every path of a session the driver did not start in a worktree. What is refused is one thing: a change of the checkout itself,
 whose copy belongs to another session and whose edits block merges for everyone.

 The path is read the way Pi's own tools read it (`resolveToCwd` in `dist/core/tools/path-utils.js` of
 `@earendil-works/pi-coding-agent` 1.6.0): a leading `@` and Unicode spaces are dropped, `~` is this machine's home folder, a
 `file://` URL is a path, and then every `..` segment and every symbolic link is resolved before the folders are compared.

 @module
 */

import { homedir, } from 'node:os';
import {
  isAbsolute,
  join,
  relative,
  sep,
} from 'node:path';
import { fileURLToPath, } from 'node:url';
import {
  isAWorktreesOwnPath,
  type MainCheckoutGuard,
} from './main-checkout-guard-of-a-run.ts';
import {
  isInside,
  resolveWithSymbolicLinks,
} from './path-with-symbolic-links-resolved.ts';
import { isRecord, } from './read-pi-fields.ts';

//region Sentinels

/**
 What this guard answers a call it refuses nothing of.

 A symbol rather than nothing, so a caller cannot take a decision this module never made for one it made.
 */
const NOT_REFUSED: unique symbol = Symbol('pi-shared-subagent-driver/main-checkout-refusal-not-refused',);

//endregion Sentinels

//region The paths a tool call may name

/**
 The Unicode spaces Pi's own tools read as an ordinary space, copied from `normalizePath` in `dist/utils/paths.js` of
 `@earendil-works/pi-coding-agent` 1.6.0.
 */
// oxlint-disable-next-line no-restricted-syntax/no-regex -- the character class Pi itself writes, matched against the one path a tool call names: its width is fixed, so no input can make it backtrack.
const UNICODE_SPACES = /[\u00A0\u2000-\u200A\u202F\u205F\u3000]/gu;

/**
 The prefix a tool call may spell a path with, which Pi's own tools strip before resolving.
 */
const FILE_REFERENCE_PREFIX = '@';

/**
 The scheme a path may be spelled with, which Pi's own tools read as a path.
 */
const FILE_URL_PREFIX = 'file://';

/**
 Reads the path a tool call names, the way Pi's own tools read it.

 @param given - the path as the tool call spells it

 @param cwd - the folder the session works in, which a relative path is read from

 @param home - the home folder a `~` path is read from

 @returns the absolute path, with every `..` segment and every symbolic link still to be read

 @example
 ```ts
 const path = absolutePathOfAPiToolPath({ given: '@~/repo/a.ts', cwd: '/repo', home: '/home/a', });
 ```
 */
export function absolutePathOfAPiToolPath({
  given,
  cwd,
  home,
}: {
  readonly given: string;
  readonly cwd: string;
  readonly home: string;
},): string {
  /**
   The path with the spaces Pi's tools normalize.
   */
  const spaced = given.replaceAll(
    UNICODE_SPACES,
    ' ',
  );
  /**
   The path with the file-reference prefix Pi's tools strip.
   */
  const unprefixed = spaced.startsWith(FILE_REFERENCE_PREFIX,)
    ? spaced.slice(FILE_REFERENCE_PREFIX.length,)
    : spaced;
  /**
   The path with `~` read as a home folder, which Pi's tools do before resolving.
   */
  const homed = (unprefixed === '~')
    ? home
    : (unprefixed.startsWith('~/',) ? `${home}/${unprefixed.slice('~/'.length,)}` : unprefixed);
  /**
   The path of a `file://` URL, which Pi's tools read as one.
   */
  const fromUrl = homed.startsWith(FILE_URL_PREFIX,)
    ? fileURLToPath(homed,)
    : homed;
  // Built onto the session's folder without collapsing anything: a `..` after a symbolic link means the folder above whatever
  // that link names, which only a walk through the file system can answer.
  return isAbsolute(fromUrl,)
    ? fromUrl
    : `${cwd}${sep}${fromUrl}`;
}

//endregion The paths a tool call may name

//region The decision

/**
 A refusal of one tool call: the words Pi gives the model in place of the tool's own result.

 Pi runs no part of the tool when a `tool_call` handler answers with this, so the file is not written.
 */
export type MainCheckoutRefusal = {
  readonly block: true;
  readonly reason: string;
};

/**
 Decides whether one tool call changes the main checkout of the repository the run works in.

 @param toolName - the tool the model asked for, of which only `edit` and `write` are read

 @param input - the call's arguments, of any shape because they come from a model

 @param cwd - the folder the session works in

 @param guard - the guard of this run

 @param home - the home folder a `~` path is read from, which a test names for its own disposable home

 @returns the refusal, or {@link NOT_REFUSED} for every call this guard allows

 @example
 ```ts
 const refusal = await refusalOfAnEditOrWrite({ toolName: 'edit', input: { path: '/repo/a.ts' }, cwd: '/repo/pi-a', guard, });
 ```
 */
export async function refusalOfAnEditOrWrite({
  toolName,
  input,
  cwd,
  guard,
  home = homedir(),
}: {
  readonly toolName: string;
  readonly input: unknown;
  readonly cwd: string;
  readonly guard: MainCheckoutGuard;
  readonly home?: string;
},): Promise<MainCheckoutRefusal | typeof NOT_REFUSED> {
  /**
   Whether the tool is one whose path this guard reads.
   */
  const changesAFile = (toolName === 'edit') || (toolName === 'write');
  if (!guard.armed) {
    return NOT_REFUSED;
  }
  if (!changesAFile) {
    return NOT_REFUSED;
  }
  if (!isRecord(input,)) {
    return NOT_REFUSED;
  }
  /**
   The path this call names, as the call spells it.
   */
  const { path: given, } = input;
  // A call with no path is one Pi's own tool refuses, and it names nothing this guard could compare.
  if ((typeof given) !== 'string') {
    return NOT_REFUSED;
  }
  /**
   Where the path points once every `..` segment and every symbolic link is applied.
   */
  const resolved = await resolveWithSymbolicLinks({
    path: absolutePathOfAPiToolPath({
      given,
      cwd,
      home,
    },),
  },);
  /**
   Whether the path lies inside the main checkout at all.
   */
  const inTheMainCheckout = isInside({
    folder: guard.mainCheckout,
    path: resolved,
  },);
  /**
   Whether the path is some worktree's own copy, which no refusal covers.
   */
  const inAWorktree = isAWorktreesOwnPath({
    guard,
    path: resolved,
  },);
  if ((!inTheMainCheckout) || inAWorktree) {
    return NOT_REFUSED;
  }
  return {
    block: true,
    reason: refusalWords({
      toolName,
      given,
      guard,
      copy: join(
        guard.worktree,
        relative(
          guard.mainCheckout,
          resolved,
        ),
      ),
    },),
  };
}

//endregion The decision

//region The words

/**
 The words a refused call answers with: the path as the call gave it, that it is the main checkout's copy, the worktree's copy of
 the same path, and that reading the main checkout stays allowed.

 @param toolName - the tool the model asked for

 @param given - the path as the call gave it

 @param guard - the guard of this run

 @param copy - the worktree's copy of the same path

 @returns the refusal's words

 @example
 ```ts
 const words = refusalWords({ toolName: 'edit', given: '/repo/a.ts', guard, copy: '/repo/pi-a/a.ts', });
 ```
 */
export function refusalWords({
  toolName,
  given,
  guard,
  copy,
}: {
  readonly toolName: string;
  readonly given: string;
  readonly guard: Extract<MainCheckoutGuard, { readonly armed: true }>;
  readonly copy: string;
},): string {
  return `this run may not change the main checkout. The ${toolName} call's path "${given}" is the main checkout's copy of that path, under ${guard.mainCheckout}. This run works in the worktree ${guard.worktree}, so the worktree's copy of the same path is ${copy}: ${toolName} that copy instead. Reading the main checkout's copy of a path stays allowed. Nothing was written.`;
}

//endregion The words

export { NOT_REFUSED, };
