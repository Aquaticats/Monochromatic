import { realpath, } from 'node:fs/promises';
import { devNull, } from 'node:os';
import {
  dirname,
  resolve,
} from 'node:path';

import { childEnvironment, } from '@monochromatic-dev/translation-repair-typography/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Intrinsic corpus Git context
// A pin names physical objects in its own clone, never inherited repository state.

/**
 Native read flags shared by corpus text, bytes, listings and naming history.
 Lazy object retrieval would mutate the clone and contact a remote.

 @example
 ```ts
 const args = [...CORPUS_GIT_FLAGS, '-C', pin.cloneDir, 'show', spec];
 ```
 */
export const CORPUS_GIT_FLAGS: readonly string[] = [
  '--no-replace-objects',
  '--no-lazy-fetch',
  '--literal-pathspecs',
];

/**
 Repository-routing variables identified by Git's environment.c local_repo_env,
 plus namespace and pathspec overrides that can change literal read scope.
 */
const REPOSITORY_ENVIRONMENT: ReadonlySet<string> = new Set([
  'GIT_ALTERNATE_OBJECT_DIRECTORIES',
  'GIT_CONFIG',
  'GIT_CONFIG_PARAMETERS',
  'GIT_CONFIG_COUNT',
  'GIT_OBJECT_DIRECTORY',
  'GIT_DIR',
  'GIT_WORK_TREE',
  'GIT_IMPLICIT_WORK_TREE',
  'GIT_GRAFT_FILE',
  'GIT_INDEX_FILE',
  'GIT_NO_REPLACE_OBJECTS',
  'GIT_REPLACE_REF_BASE',
  'GIT_PREFIX',
  'GIT_SHALLOW_FILE',
  'GIT_COMMON_DIR',
  'GIT_NAMESPACE',
  'GIT_LITERAL_PATHSPECS',
  'GIT_GLOB_PATHSPECS',
  'GIT_NOGLOB_PATHSPECS',
  'GIT_ICASE_PATHSPECS',
],);

/**
 Whether a variable's name routes git to another repository or injects
 configuration.

 @param name - variable's name

 @returns Whether the corpus child must not inherit it

 @example
 ```ts
 const routed = routesRepository({ name: 'GIT_DIR', },); // true
 ```
 */
function routesRepository({ name, }: { readonly name: string; },): boolean {
  if (REPOSITORY_ENVIRONMENT.has(name,))
    return true;
  if (name.startsWith('GIT_CONFIG_KEY_',))
    return true;
  return name.startsWith('GIT_CONFIG_VALUE_',);
}

/**
 Codes with which resolving a clone directory's real path failed where git's
 own `-C` into the same path was measured to fail too (git 2.55, exit status
 128, `cannot change to`): a missing directory, a path through a file, a
 parent without search permission and a link to itself.
 */
const UNRESOLVED_CLONE_CODES: ReadonlySet<string> = new Set([
  'ENOENT',
  'ENOTDIR',
  'EACCES',
  'ELOOP',
],);

/**
 Whether resolving a clone directory failed in one of the ways git's own `-C`
 into it fails too.

 @param error - what resolving the clone directory's real path threw

 @returns Whether git then stops at the directory itself, so no ceiling is
 read

 @throws Error naming the broken invariant where what resolving threw is no
 error carrying a code, which every rejection of node's `realpath` carries
 (measured on node 26.10.0: a missing path, a path through a file, a link to
 itself, a name too long and a NUL byte each rejected with an `Error` whose
 `code` is a string)

 @example
 ```ts
 if (!gitStopsAtTheClone({ error, },)) throw error;
 ```
 */
function gitStopsAtTheClone({ error, }: { readonly error: unknown; },): boolean {
  if (!(Error.isError(error,) && ('code' in error)))
    throw new Error(
      'unreachable: resolving a clone directory\'s real path failed with something other than an error carrying a code, which every rejection of node\'s realpath carries',
      { cause: error, },
    );

  /**
   The system's code for the failure, which is all that is read of it.
   */
  const { code, } = error;
  return ((typeof code) === 'string') && UNRESOLVED_CLONE_CODES.has(code,);
}

/**
 What resolving a clone directory came to: its real path, or that git cannot
 enter it either.
 */
type ResolvedClone =
  | {
    readonly kind: 'real';
    readonly path: string;
  }
    | { readonly kind: 'unresolved'; };

/**
 Resolves a clone directory to its real path.

 @param cloneDir - clone directory as the pin names it

 @returns Its real path, or that it has none for a reason git's own `-C`
 meets too

 @throws Whatever resolving the real path threw, unchanged, for a failure git
 was not measured to meet at its own `-C`

 @example
 ```ts
 const resolved = await realCloneDir({ cloneDir: pin.cloneDir, },);
 ```
 */
async function realCloneDir({ cloneDir, }: { readonly cloneDir: string; },): Promise<ResolvedClone> {
  try {
    return {
      kind: 'real',
      path: await realpath(cloneDir,),
    };
  }
  catch (error) {
    if (!gitStopsAtTheClone({ error, },))
      throw error;
    return { kind: 'unresolved', };
  }
}

/**
 The directory git's search for a repository stops at, for one clone, which
 every corpus read resolves once before it starts git and hands to
 {@link corpusGitEnvironment}.

 THE PARENT OF THE CLONE'S REAL PATH, since git compares the ceiling with its
 real working directory. Where the real path cannot be resolved for a reason
 git's own `-C` meets too, the parent of the path as named stands in: git
 then stops at the directory before reading any ceiling.

 A PARENT WHOSE PATH HOLDS A COLON IS REFUSED. `GIT_CEILING_DIRECTORIES` is a
 colon-separated list with no escape (git 2.55 manual, `git help git`), so
 such a parent splits into two paths, neither of them the parent: measured
 with git 2.55, a plain directory under `a:b` inside a repository read that
 repository's page, and a page absent there read as one absent at a held
 commit, which a walker steps past. A real clone there reads its own pages
 whether or not the ceiling applies, but nothing tells the two apart without
 the ceiling, so both are refused. A link from a path free of colons does
 not help, since the ceiling is taken from the real path.

 @param cloneDir - clone directory as the pin names it

 @returns The parent of the clone's real path, or of the path as named where
 git cannot enter it

 @throws {@link StatedRefusalError} when that parent's path holds a colon,
 naming the clone and the parent and saying to move the clone

 @throws Whatever resolving the real path threw, unchanged, for a failure git
 was not measured to meet at its own `-C`

 @example
 ```ts
 const searchCeiling = await gitSearchCeiling({ cloneDir: pin.cloneDir, },);
 ```
 */
export async function gitSearchCeiling({ cloneDir, }: { readonly cloneDir: string; },): Promise<string> {
  /**
   The clone's real path, or that git cannot enter it.
   */
  const resolved = await realCloneDir({ cloneDir, },);
  if (resolved.kind === 'unresolved')
    return dirname(resolve(cloneDir,),);

  /**
   The real parent, which git's ceiling list must carry whole.
   */
  const parent = dirname(resolved.path,);
  if (parent.includes(':',)) {
    throw new StatedRefusalError({
      says: `corpus clone ${cloneDir} lies under ${parent}, a path holding a colon, which git's ceiling list `
        + 'cannot carry since git splits it at colons, so git could read a repository around the clone in its '
        + 'place: move the clone to a directory whose path holds no colon.',
    },);
  }
  return parent;
}

/**
 Isolates every corpus subprocess from inherited repository routing and grafts,
 and from every credential and setting of the package.
 Guards are assigned after inheritance, so callers cannot override them.
 No process-global environment is mutated or logged.

 EVERY REMOVED NAME IS STATED AS `undefined`, NOT LEFT OUT: `nano-spawn` merges
 this object over the process's own environment, so a name left out came back
 from the parent and `GIT_DIR` reached the child of its listing, while node's
 `execFile` read the same object as replacing the environment.

 GIT SPEAKS THE C LOCALE. A failed corpus read is classified by the English
 words git prints (`corpus-source.ts`), and git translates its messages into
 the caller's language: measured with git 2.55 on a throwaway repository, a
 path absent at a held commit printed `Schwerwiegend: Pfad ... existiert nicht
 in ...` under `LANGUAGE=de` with an English `LANG`, and under `LANG` or
 `LC_MESSAGES` set to German, which the reader classed as another failure.
 `LC_ALL=C` overrides all three, so git prints its own English whatever the
 caller's locale says; the caller's other locale variables are kept as they
 are.

 GIT NEVER CLIMBS OUT OF THE CLONE: `GIT_CEILING_DIRECTORIES` names the
 parent of the directory the clone resolves to ({@link gitSearchCeiling}),
 so a directory
 that is no repository is refused rather than read as whatever repository
 encloses it. Git checks the directory it starts in whatever the list says,
 so the ceiling is the parent and never the clone. Git compares the list with
 its real working directory, so the parent is taken from the clone's real
 path: measured with git 2.55, a clone directory that is itself a link to a
 plain directory inside a repository read that repository's page under a
 ceiling taken from the link's own parent, and was refused under one taken
 from its real path, while a link to a real clone still read.

 @param searchCeiling - directory git's search for a repository stops at,
 as {@link gitSearchCeiling} resolved it for the clone the child reads

 @param environment - inherited process context, injectable for verification

 @returns Owned environment with intrinsic-object, no-fetch, bounded-search
 and C-locale semantics

 @example
 ```ts
 const env = corpusGitEnvironment({ searchCeiling: await gitSearchCeiling({ cloneDir: pin.cloneDir, },), },);
 ```
 */
export function corpusGitEnvironment({
  searchCeiling,
  environment = process.env,
}: {
  readonly searchCeiling: string;
  readonly environment?: Readonly<NodeJS.ProcessEnv>;
},): NodeJS.ProcessEnv {
  /**
   Preserve unrelated process settings without exposing their values in logs.
   */
  const inherited = childEnvironment({ parent: environment, },);
  for (const name of Object.keys(inherited,)) {
    if (routesRepository({ name, },))
      inherited[name] = undefined;
  }
  return {
    ...inherited,
    GIT_CEILING_DIRECTORIES: searchCeiling,
    GIT_GRAFT_FILE: devNull,
    GIT_NO_REPLACE_OBJECTS: '1',
    GIT_NO_LAZY_FETCH: '1',
    LC_ALL: 'C',
  };
}

//endregion Intrinsic corpus Git context
