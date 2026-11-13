/**
 The files one review's diff touches, and the part of a diff that touches only some of them.

 A review states which of these files it read whole, in part or not at all, so the checker needs the same list git wrote, and a
 second review of the files the first did not read whole needs a diff holding only those. Both are read from the diff file the
 brief wrote, so neither needs git, the checkout, or the branch the diff came from.

 @module
 */

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
   The file's path, from the top of the checkout.
   */
  readonly path: string;

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
 Reads the path a file header names.

 The quoted form is refused rather than half-read: git quotes a path holding a space, a quote or a control character, and the
 quoted text is not the path a reviewer copies from `git diff --name-only`.

 @param line - one line of a diff

 @returns the header's path, or {@link NOT_A_PATH} when the line is no header this reader can compare

 @example
 ```ts
 const path = pathInHeader({ line: 'diff --git a/src/a.rs b/src/a.rs' });
 ```
 */
function pathInHeader({ line, }: { readonly line: string },): string | typeof NOT_A_PATH {
  if (!line.startsWith(DIFF_HEADER_START,)) {
    return NOT_A_PATH;
  }
  /**
   Where the header's first path ends.
   */
  const middle = line.indexOf(
    DIFF_HEADER_MIDDLE,
    DIFF_HEADER_START.length,
  );
  if (middle === (-1)) {
    return NOT_A_PATH;
  }
  /**
   The header's second path, which is the one a reviewer copies.
   */
  const path = line.slice(middle + DIFF_HEADER_MIDDLE.length,);
  return (path === '') ? NOT_A_PATH : path;
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
  for (const line of diff.split('\n',)) {
    /**
     The path this line's file header names, when it is one.
     */
    const path = pathInHeader({ line, },);
    if (path !== NOT_A_PATH) {
      sections.push({
        path,
        lines: [line,],
      },);
      continue;
    }
    if (line.startsWith(DIFF_HEADER_PREFIX,)) {
      throw new ReviewDiffError(
        `the diff names a path this reader cannot compare, because git wrote it quoted or in a form this reader does not `
          + `know: ${line}`,
      );
    }
    /**
     The section this line belongs to, or the preamble before any section started.
     */
    const last = sections.at(-1,);
    if (last === undefined) {
      preamble.push(line,);
    }
    else {
      last.lines
        .push(line,);
    }
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
