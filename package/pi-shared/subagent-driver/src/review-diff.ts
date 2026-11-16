/**
 The files one review's diff touches, and the part of a diff that touches only some of them.

 A review states which of these files it read whole, in part or not at all, so the checker needs the same list git wrote, and a
 second review of the files the first did not read whole needs a diff holding only those. Both are read from the diff file the
 brief wrote, so neither needs git, the checkout, or the branch the diff came from.

 @module
 */

import { pathOf, } from './git-quoted-path.ts';

/**
 A diff whose file headers cannot be read.

 The only headers this reader refuses are ones git wrote with a quoted path, because the quoted text is not the path a reviewer
 copies and comparing the two would say a file was missing when it was there.
 */
export class ReviewDiffError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'ReviewDiffError';
}

/**
 The files a diff touches, and the lines the whole diff holds.
 */
export type FilesInADiff = {
  /**
   The lines the diff holds, as `wc --lines` counts them: one per newline.
   */
  readonly lines: number;

  /**
   The paths the diff touches, each once, in the order git named them.
   */
  readonly files: readonly string[];
};

/**
 One file's part of a diff.
 */
type DiffSection = {
  /**
   The file's path, from the top of the checkout, which the `+++` or `---` line settles when the header alone is ambiguous.
   */
  path: string;

  /**
   The section's lines, from its `diff --git` header to the line before the next one.
   */
  readonly lines: string[];
};

/**
 A diff's parts, split at its file headers.
 */
type SplitDiff = {
  /**
   The lines before the first file header, which git writes only for a diff that touches no file.
   */
  readonly preamble: readonly string[];

  /**
   One section per file, in the order git named them.
   */
  readonly sections: readonly DiffSection[];
};

/**
 The words a file header starts with, for a path git does not have to quote.
 */
const DIFF_HEADER_START = 'diff --git a/';

/**
 The words between a header's two paths.
 */
const DIFF_HEADER_MIDDLE = ' b/';

/**
 The words every file header starts with, quoted or not.
 */
const DIFF_HEADER_PREFIX = 'diff --git ';

/**
 No path in a line, as a symbol so that no path can stand for it.
 */
const NOT_A_PATH: unique symbol = Symbol('diff line names no header path',);

/**
 A file's part of a diff, read for the files it names rather than for the lines it carries.
 */
type DiffSectionView = {
  /**
   The file's path.
   */
  readonly path: string;

  /**
   Its lines, which a reader only reads.
   */
  readonly lines: readonly string[];
};

/**
 Reads the paths a quoted header names, each unquoted.

 @param line - a `diff --git "a/..." "b/..."` header

 @returns the post-image path, or nothing when the line does not hold two quoted paths

 @example
 ```ts
 const path = quotedPathInHeader({ line: 'diff --git "a/r\\303\\251sum.ts" "b/r\\303\\251sum.ts"' });
 ```
 */
function quotedPathInHeader({ line, }: { readonly line: string },): string | typeof NOT_A_PATH {
  /**
   The quoted fields the line holds, each with its quotes.
   */
  const fields: string[] = [];
  /**
   Where the next character is read from.
   */
  const cursor = { at: DIFF_HEADER_PREFIX.length, };
  while ((fields.length < 2) && (cursor.at < line.length)) {
    if (line[cursor.at] !== '"') {
      cursor.at += 1;
      continue;
    }
    /**
     Where the field's closing quote is.
     */
    const start = cursor.at;
    cursor.at += 1;
    while (cursor.at < line.length) {
      if (line[cursor.at] === '\\') {
        cursor.at += 2;
        continue;
      }
      if (line[cursor.at] === '"') {
        cursor.at += 1;
        break;
      }
      cursor.at += 1;
    }
    fields.push(line.slice(
      start,
      cursor.at,
    ),);
  }
  if (fields.length < 2) {
    return NOT_A_PATH;
  }
  /**
   The post-image path, without its `b/` prefix.
   */
  const after = pathOf(fields[1] ?? '',);
  if (after === '/dev/null') {
    /**
     The pre-image path, for a header that names a deletion.
     */
    const before = pathOf(fields[0] ?? '',);
    return before.startsWith('a/',) ? before.slice(2,) : before;
  }
  return after.startsWith('b/',) ? after.slice(2,) : after;
}

/**
 Reads the path a file header names.

 A path git quotes, for a name holding a non-ASCII byte, a quote or a control character, is unquoted so it compares with the path a
 reviewer copies from the brief.

 @param line - one line of a diff

 @returns the header's path, or {@link NOT_A_PATH} when the line is no header this reader can compare

 @example
 ```ts
 const path = pathInHeader({ line: 'diff --git a/src/a.rs b/src/a.rs' });
 ```
 */
function pathInHeader({ line, }: { readonly line: string },): string | typeof NOT_A_PATH {
  if (line.startsWith('diff --git "',)) {
    return quotedPathInHeader({ line, },);
  }
  if (!line.startsWith(DIFF_HEADER_START,)) {
    return NOT_A_PATH;
  }
  /**
   The header's two paths, with the words before the first one taken off.
   */
  const rest = line.slice(DIFF_HEADER_START.length,);
  /**
   Where a ` b/` separator may stand.
   */
  const separator = rest.indexOf(DIFF_HEADER_MIDDLE,);
  if (separator === (-1)) {
    return NOT_A_PATH;
  }
  // A file whose own path holds ` b/` makes the header's first separator look like a boundary. git writes both paths, and they are
  // the same path unless the file was renamed, so the boundary is the separator that leaves the same path on both sides.
  for (const [at, found,] of separatorPositionsIn(rest,)
    .entries()) {
    /**
     The path before this separator.
     */
    const before = rest.slice(
      0,
      found,
    );
    /**
     The path after it.
     */
    const after = rest.slice(found + DIFF_HEADER_MIDDLE.length,);
    if ((before !== '') && (before === after)) {
      return before;
    }
    // The last separator is the fallback: a rename's two paths differ, and the `---` and `+++` lines settle it afterwards.
    if (at === 0) {
      continue;
    }
  }
  /**
   The first path's own text, when no separator leaves the same path on both sides.
   */
  const first = rest.slice(
    0,
    separator,
  );
  return (first === '') ? NOT_A_PATH : first;
}

/**
 Reads where a ` b/` may separate a header's two paths.

 @param rest - a header's text after `diff --git a/`

 @returns each separator's position, in order

 @example
 ```ts
 const at = separatorPositionsIn('x b/x');
 ```
 */
function separatorPositionsIn(rest: string,): readonly number[] {
  /**
   The positions found so far.
   */
  const found: number[] = [];
  /**
   Where the next search starts.
   */
  const search = { from: 0, };
  while (search.from <= rest.length) {
    /**
     The next separator at or after this position.
     */
    const at = rest.indexOf(
      DIFF_HEADER_MIDDLE,
      search.from,
    );
    if (at === (-1)) {
      break;
    }
    found.push(at,);
    search.from = at + 1;
  }
  return found;
}

/**
 Splits a diff at its file headers.

 @param diff - the diff's text

 @returns its preamble and one section per file

 @throws ReviewDiffError when a header is one this reader cannot compare

 @example
 ```ts
 const split = splitDiffIn({ diff: await readFile('a/change.diff', 'utf8') });
 ```
 */
function splitDiffIn({ diff, }: { readonly diff: string },): SplitDiff {
  /**
   The lines before the first file header.
   */
  const preamble: string[] = [];
  /**
   One section per file, as they are read.
   */
  const sections: DiffSection[] = [];
  /**
   The section being read, and what its `---` line said about a deletion.
   */
  const state = {
    current: [] as DiffSection[],
    fromPath: '',
    resolved: false,
    inHunk: false,
  };
  for (const line of diff.split('\n',)) {
    /**
     The path this line's file header names, when it is one.
     */
    const path = pathInHeader({ line, },);
    if (path !== NOT_A_PATH) {
      /**
       A fresh section, whose path the `---` and `+++` lines may settle.
       */
      const section: DiffSection = {
        path,
        lines: [line,],
      };
      sections.push(section,);
      state.current = [section,];
      state.fromPath = '';
      state.resolved = false;
      state.inHunk = false;
      continue;
    }
    if (line.startsWith(DIFF_HEADER_PREFIX,)) {
      throw new ReviewDiffError(
        `the diff names a path this reader cannot compare, because git wrote it quoted or in a form this reader does not `
          + `know: ${line}`,
      );
    }
    if (state.current
      .length
      > 0) {
      // The section being read is the only one in this list; every line after a header belongs to it until the next header.
      /**
       The section being read, which is the only one in the list.
       */
      const current = state.current
        .at(0,);
      if (current === undefined) {
        continue;
      }
      current.lines
        .push(line,);
      // A line the change adds can itself start `+++ `, so only a line before the first hunk settles the path.
      if (line.startsWith('@@ ',)) {
        state.inHunk = true;
        continue;
      }
      // The post-image line names the file git compares as new; a deletion names no file there and keeps the pre-image's name.
      if ((line.startsWith('+++ ',)) && (!state.resolved)
        && (!state.inHunk)) {
        /**
         What the post-image line names, without the quotes git may put around it.
         */
        const after = pathOf(line.slice('+++ '.length,),);
        if (after === '/dev/null') {
          if (state.fromPath !== '') {
            current.path = state.fromPath;
          }
        }
        else if (after.startsWith('b/',)) {
          current.path = after.slice('b/'.length,);
        }
        state.resolved = true;
      }
      else if ((line.startsWith('--- ',)) && (!state.resolved)
        && (!state.inHunk)) {
        /**
         What the pre-image line names, without the quotes git may put around it.
         */
        const before = pathOf(line.slice('--- '.length,),);
        if (before.startsWith('a/',)) {
          state.fromPath = before.slice('a/'.length,);
        }
      }
      continue;
    }
    preamble.push(line,);
  }
  return {
    preamble,
    sections,
  };
}

/**
 Reads the files a diff touches and the lines it holds.

 @param diff - the diff's text

 @returns the diff's lines and the files it touches

 @throws ReviewDiffError when a file header cannot be read

 @example
 ```ts
 const reading = filesInDiff({ diff: 'diff --git a/src/a.rs b/src/a.rs\n' });
 ```
 */
export function filesInDiff({ diff, }: { readonly diff: string },): FilesInADiff {
  /**
   The diff split at its file headers.
   */
  const split = splitDiffIn({ diff, },);
  /**
   The paths already named, so a diff repeating one is listed once.
   */
  const seen = new Set<string>();
  /**
   The files the diff touches, in the order git named them.
   */
  const files: string[] = [];
  for (const section of split.sections) {
    if (seen.has(section.path,)) {
      continue;
    }
    seen.add(section.path,);
    files.push(section.path,);
  }
  return {
    lines: diff.split('\n',)
      .length
      - 1,
    files,
  };
}

/**
 Reads the part of a diff that touches only some of its files.

 @param diff - the diff's text

 @param files - the files the part keeps, from the top of the checkout

 @returns the diff's preamble and the kept files' sections, as they were written

 @throws ReviewDiffError when the diff holds none of the files, because a diff of nothing is no brief

 @example
 ```ts
 const part = diffOfOnlyFiles({ diff, files: ['src/a.rs'] });
 ```
 */
export function diffOfOnlyFiles({
  diff,
  files,
}: {
  readonly diff: string;
  readonly files: readonly string[];
},): string {
  /**
   The files the part keeps.
   */
  const kept = new Set(files,);
  /**
   The diff split at its file headers.
   */
  const split = splitDiffIn({ diff, },);
  /**
   The sections the part keeps.
   */
  const sections = split.sections
    .filter(function isKept(section: DiffSectionView,) {
      return kept.has(section.path,);
    },);
  if (sections.length === 0) {
    throw new ReviewDiffError(
      `the diff touches none of the files a review of them would read: ${files.join(', ')}`,
    );
  }
  return [
    ...split.preamble,
    ...sections.flatMap(function linesOf(section: DiffSectionView,) {
      return section.lines;
    },),
  ].join('\n',);
}
