/**
 Root markers shipped with the package: the three repository kinds this
 monorepo uses as presets, and factories for the common shapes (a file, a
 directory, a `package.json` with a given name). Each is a value for
 `findRoot`; a new kind of root is a new value, not a new function.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { matchesValidGitMarker, } from './git-marker.ts';
import { ABSENT, } from './root-filesystem-contract.ts';
import type {
  RootMarker,
  RootMatcherArgs,
} from './root-marker-contract.ts';

//region Constants

/**
 Tagged logger for marker probes that skip a candidate for a reason worth
 seeing (unreadable manifest, malformed JSON).
 */
const rootMarkerLogger = tagged({ tag: 'rootMarker', },);

/**
 TOML table header that marks the monorepo root `mise.toml`.
 */
const MONOREPO_TABLE_HEADER = '[monorepo]';

/**
 TOML comment introducer, the only thing allowed after a table header on
 its line.
 */
const TOML_COMMENT_START = '#';

/**
 Manifest name read by {@link packageNamed}.
 */
const PACKAGE_MANIFEST = 'package.json';

//endregion Constants

//region mise monorepo

/**
 Whether one line of a TOML file is the `[monorepo]` table header.

 Trimming drops the carriage return of a CRLF file and trailing spaces;
 the remainder must be the header alone or the header followed by a
 comment, so a header quoted inside a value never matches.

 @param line - one line of `mise.toml`, line ending included or not

 @returns `true` when the line opens the `[monorepo]` table

 @example
 ```ts
 isMonorepoHeaderLine('[monorepo] # workspace root\r'); // true
 isMonorepoHeaderLine('description = "[monorepo] later"'); // false
 ```
 */
function isMonorepoHeaderLine(line: string,): boolean {
  /**
   Line without surrounding whitespace, carriage return included.
   */
  const bare = line.trim();
  if (!bare.startsWith(MONOREPO_TABLE_HEADER,))
    return false;
  /**
   Text after the header; empty, or a comment, on a header line.
   */
  const rest = bare.slice(MONOREPO_TABLE_HEADER.length,)
    .trimStart();
  return (rest === '') || rest.startsWith(TOML_COMMENT_START,);
}

/**
 Checks whether a directory holds a `mise.toml` with a `[monorepo]` table.

 @param dir - candidate directory

 @param fs - filesystem the walk runs over

 @returns `true` when the candidate is the mise monorepo root

 @example
 ```ts
 await matchesMiseMonorepo({ dir: '/repo', fs });
 ```
 */
async function matchesMiseMonorepo({
  dir,
  fs,
}: RootMatcherArgs,): Promise<boolean> {
  /**
   Candidate `mise.toml` content; {@link ABSENT} when the file is missing.
   */
  const content = await fs.readTextFile(`${dir}/mise.toml`,);
  if (content === ABSENT)
    return false;
  return content.split('\n',)
    .some(isMonorepoHeaderLine,);
}

/**
 Nearest ancestor whose `mise.toml` contains a `[monorepo]` table header on
 its own line (CRLF endings and a trailing comment tolerated).

 @example
 ```ts
 const root = await findRootCached({ marker: MISE_MONOREPO });
 ```
 */
export const MISE_MONOREPO: RootMarker = {
  matches: matchesMiseMonorepo,
  name: 'mise monorepo',
};

//endregion mise monorepo

//region Git repository

/**
 Nearest ancestor with a structurally usable `.git` directory or gitfile:
 HEAD, objects, refs, relative gitfile targets, and linked-worktree
 `commondir` pointers are validated, and an invalid nearer marker is
 skipped in favour of a valid outer one.

 @example
 ```ts
 const root = await findRoot({ cwd, marker: GIT_REPOSITORY });
 ```
 */
export const GIT_REPOSITORY: RootMarker = {
  matches: matchesValidGitMarker,
  name: 'git repository',
};

//endregion Git repository

//region Factories

/**
 Marker for the nearest ancestor holding a regular file with this name.

 @param name - file name probed at each ancestor

 @returns marker named after the file

 @example
 ```ts
 const root = await findRoot({ marker: fileNamed('deno.json') });
 ```
 */
export function fileNamed(name: string,): RootMarker {
  return {
    matches: function matchesFile({
      dir,
      fs,
    }: RootMatcherArgs,): Promise<boolean> {
      return fs.isFile(`${dir}/${name}`,);
    },
    name: `file ${name}`,
  };
}

/**
 Marker for the nearest ancestor holding a directory with this name.

 @param name - directory name probed at each ancestor

 @returns marker named after the directory

 @example
 ```ts
 const root = await findRoot({ marker: directoryNamed('node_modules') });
 ```
 */
export function directoryNamed(name: string,): RootMarker {
  return {
    matches: function matchesDirectory({
      dir,
      fs,
    }: RootMatcherArgs,): Promise<boolean> {
      return fs.isDirectory(`${dir}/${name}`,);
    },
    name: `directory ${name}`,
  };
}

/**
 Marker for the nearest ancestor whose `package.json` declares this `name`.

 Matching by name is defensive: a missing or corrupted local manifest is
 skipped instead of silently landing on a parent workspace manifest, and
 the walk from `src/` and from a bundled `dist/` subdirectory ends at the
 same root.

 @param name - expected `name` field of the manifest

 @returns marker named after the package

 @example
 ```ts
 const root = await findRootCached({
   cwd: import.meta.dirname,
   marker: packageNamed('@monochromatic-dev/fs-path-core'),
 });
 ```
 */
export function packageNamed(name: string,): RootMarker {
  return {
    matches: async function matchesPackage({
      dir,
      fs,
    }: RootMatcherArgs,): Promise<boolean> {
      /**
       Manifest path tested at this ancestor.
       */
      const manifestPath = `${dir}/${PACKAGE_MANIFEST}`;
      /**
       Manifest text; {@link ABSENT} when this ancestor has none.
       */
      const content = await fs.readTextFile(manifestPath,);
      if (content === ABSENT)
        return false;
      try {
        /* oxlint-disable typescript-eslint/no-unsafe-type-assertion -- JSON.parse returns `any`; only the optional `name` field is read, and `===` tolerates its absence. */
        /**
         Parsed manifest exposing the optional `name` field.
         */
        const parsed = JSON.parse(content,) as { name?: string; };
        /* oxlint-enable typescript-eslint/no-unsafe-type-assertion */
        return parsed.name === name;
      }
      catch (error: unknown) {
        rootMarkerLogger.debug(`skipping unparsable manifest ${manifestPath}: ${caughtValueText(error,)}`,);
        return false;
      }
    },
    name: `package ${name}`,
  };
}

//endregion Factories

//region pnpm workspace

/**
 Nearest ancestor holding a `pnpm-workspace.yaml` file.

 @example
 ```ts
 const root = await findRoot({ marker: PNPM_WORKSPACE });
 ```
 */
export const PNPM_WORKSPACE: RootMarker = fileNamed('pnpm-workspace.yaml',);

//endregion pnpm workspace
