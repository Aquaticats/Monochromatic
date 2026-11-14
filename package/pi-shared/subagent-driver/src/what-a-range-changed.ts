/**
 What a range changed, read from Git's own statuses: every path a range added, copied, deleted, modified, renamed or changed the
 type of, what each of those changes is to a review, and the paths a reviewer can still read at the commit.

 A review's kind is decided from every way a range can change a path, and not from added and modified paths alone: a range that
 deletes or renames code is a change of code even when every file the commit still holds at a touched path is a test, because a
 review of the tests would leave the removal unread. A status this module cannot read is refused rather than guessed at, because
 guessing would decide a review from part of the change.

 Git's `--name-status -z` is what this module reads: one record for each path, fields ended by a NUL byte, the status first (its
 letter, then the score a copy or a rename carries), the path, and then, for a copy or a rename, the path it came from and the path
 it became. `git-diff(1)` in this installation lists the statuses A, C, D, M, R, T, U and X, and C and R only when something asks
 for copy or rename detection.

 @module
 */

/**
 A path a range removed or renamed, which the reviewed commit no longer holds as a file a reviewer can read at that path.
 */
export type RemovedOrRenamedPath =
  | {
    /**
     The path was deleted.
     */
    readonly status: 'deleted';

    /**
     The path the range deleted, from the top of the checkout.
     */
    readonly path: string;
  }
    | {
      /**
       The path was renamed, so the commit holds its file under another name.
       */
      readonly status: 'renamed';

      /**
       The name the path had in the older side of the range, from the top of the checkout.
       */
      readonly was: string;

      /**
       The name the commit holds that file under now, from the top of the checkout.
       */
      readonly now: string;
    };

/**
 A status Git reported for a path that this writer cannot read as a change of code or of tests.

 A diff between two commits reports no unmerged path, and this writer asks for no broken pairing, so the only status that reaches
 this is one Git documents as an unknown change type or another status a later Git adds: it is refused rather than guessed at,
 because guessing would decide a review's kind from part of the change.
 */
export class ReviewDiffError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'ReviewDiffError';
}

/**
 How much of Git's answer a message shows when the answer is not whole records.
 */
const SHOWN_ANSWER_CHARACTERS = 400;

/**
 The statuses this writer reads for a path: the ones a diff between two commits can name, as Git's own `--diff-filter` and raw
 format document them. A copy and a rename carry a score after their letter, so only the letter is compared.
 */
const CHANGE_STATUSES_A_REVIEW_READS: ReadonlySet<string> = new Set<string>([
  'A',
  'C',
  'D',
  'M',
  'R',
  'T',
]);

/**
 The statuses that leave a file at the newer of the two names a record carries: an addition, a copy, a modification, a rename, or
 a change of type.
 */
const PATHS_WITH_A_FILE_AT_THE_NEWER_SIDE: ReadonlySet<string> = new Set<string>([
  'A',
  'C',
  'M',
  'R',
  'T',
]);

/**
 Reads the letter a status begins with, which is the status itself when Git printed no score after it.

 @param status - a status as Git printed it, such as `M`, `R100` or `C75`

 @returns the letter

 @example
 ```ts
 const letter = letterOfStatus('R100');
 ```
 */
function letterOfStatus(status: string,): string {
  return status.slice(
    0,
    1,
  );
}

/**
 One path a range changed, as `git diff --name-status` reports it: the letter of its status, the path, and for a rename or a copy
 the path it came from.
 */
export type ChangedPath = {
  /**
   The status, as Git printed it, score included: `M`, `A`, `D`, `R100`, `T`, `C100`, or one the writer must refuse.
   */
  readonly status: string;

  /**
   The path in the newer side of the range, from the top of the checkout.
   */
  readonly path: string;

  /**
   The path in the older side of the range, which a rename or a copy also names.
   */
  readonly from?: string;
};

/**
 What a path is to a review: the change itself, a test that checks it, or neither, which a review reads no part of.
 */
export type PathKind = 'code' | 'test' | 'unread';

/**
 What one path is to a review, as the module that knows this repository's extensions, test suffixes and build names decides it.
 */
export type PathKindOfAPath = (path: string) => PathKind;

/**
 Reads the paths a range changed from what `git diff --name-status -z` printed.

 @param printed - what git printed

 @returns the paths the range changed, in the order git named them

 @throws ReviewDiffError when the text is not a whole number of records, a record names no path, or a status is one this writer
 cannot read

 @example
 ```ts
 const changed = changedPathsIn({ printed: 'M\0src/gate.rs\0', },);
 ```
 */
export function changedPathsIn({ printed, }: { readonly printed: string },): readonly ChangedPath[] {
  /**
   The fields git ended with NUL bytes, of which the last is empty because the answer ends with one.
   */
  const fields = printed.split('\0',);
  if (fields.at(-1) === '') {
    fields.pop();
  }
  /**
   The paths read so far.
   */
  const changed: ChangedPath[] = [];
  for (let at = 0; at < fields.length;) {
    /**
     The status of this record, which begins with one letter and may carry a score after it.
     */
    const status = fields[at] ?? '';
    at += 1;
    /**
     Whether this record names the path it came from as well as the path it became.
     */
    const namesItsSource = status.startsWith('C',) || status.startsWith('R',);
    /**
     The path in the older side, which a copy or a rename names before the newer one.
     */
    const from = namesItsSource ? (fields[at] ?? '') : undefined;
    if (namesItsSource) {
      at += 1;
    }
    /**
     The path in the newer side of the range.
     */
    const path = fields[at] ?? '';
    at += 1;
    if ((status === '') || (path === '')) {
      /**
       The head of what git printed, so the message names what was read without becoming a page of it.
       */
      const shown = JSON.stringify(printed,);
      throw new ReviewDiffError(
        `a range's change names a path without a whole status and path around it: ${shown.slice(
          0,
          SHOWN_ANSWER_CHARACTERS,
        )}`,
      );
    }
    if (!CHANGE_STATUSES_A_REVIEW_READS.has(letterOfStatus(status,),)) {
      throw new ReviewDiffError(
        `git reported ${status} for ${path}, and this writer reads no such change of code or of tests: it reads ${
          [...CHANGE_STATUSES_A_REVIEW_READS,].join(', ',)
        }`,
      );
    }
    changed.push(
      (from === undefined)
        ? {
          status,
          path,
        }
        : {
          status,
          path,
          from,
        },
    );
  }
  return changed;
}

/**
 Reads what a changed path is to a review: the change itself when any of its names is code or a build decision, a test when any
 of them is a test, and neither otherwise.

 A rename is read from both names, because a rename of code into a test's name still removed code, and a rename of a test into
 another name still leaves the change a change to tests.

 @param changed - one path a range changed

 @param pathKindOf - what one path is to a review

 @returns the change itself, a test, or neither

 @example
 ```ts
 const kind = kindOfChangedPath({ changed: { status: 'D', path: 'src/gate.rs' }, pathKindOf },);
 ```
 */
export function kindOfChangedPath({
  changed,
  pathKindOf,
}: {
  readonly changed: ChangedPath;
  readonly pathKindOf: PathKindOfAPath;
},): PathKind {
  /**
   The kinds of every name this path has in the range.
   */
  const kinds = (changed.from === undefined)
    ? [pathKindOf(changed.path,),]
    : [
      pathKindOf(changed.from,),
      pathKindOf(changed.path,),
    ];
  if (kinds.includes('code',)) {
    return 'code';
  }
  return kinds.includes('test',) ? 'test' : 'unread';
}

/**
 Reads the paths one changed path removed or renamed, when it removed or renamed anything.

 @param changed - one path a range changed

 @returns the path it removed or renamed, or nothing when it left every file where it was

 @example
 ```ts
 const removed = removedOrRenamedBy({ changed: { status: 'D', path: 'src/gate.rs' }, },);
 ```
 */
export function removedOrRenamedBy({ changed, }: { readonly changed: ChangedPath },): readonly RemovedOrRenamedPath[] {
  if (letterOfStatus(changed.status,) === 'D') {
    return [{
      status: 'deleted',
      path: changed.path,
    },];
  }
  if ((letterOfStatus(changed.status,) === 'R') && (changed.from !== undefined)) {
    return [{
      status: 'renamed',
      was: changed.from,
      now: changed.path,
    },];
  }
  return [];
}

/**
 Reads the paths a range changed that the reviewed commit still holds as files a reviewer can read: the newer name of every path
 it added, copied, modified, renamed or changed the type of, when the commit holds that name.

 @param changed - the paths the ranges changed, in the order git named them

 @param held - the paths the reviewed commit holds

 @returns the paths a reviewer can read in full, in the order git named them, each once

 @example
 ```ts
 const readable = readablePathsOf({ changed, held },);
 ```
 */
export function readablePathsOf({
  changed,
  held,
}: {
  readonly changed: readonly ChangedPath[];
  readonly held: ReadonlySet<string>;
},): readonly string[] {
  /**
   The paths read so far.
   */
  const paths: string[] = [];
  for (const one of changed) {
    if (PATHS_WITH_A_FILE_AT_THE_NEWER_SIDE.has(letterOfStatus(one.status,),) && held.has(one.path,)
      && (!paths.includes(one.path,))) {
      paths.push(one.path,);
    }
  }
  return paths;
}

/**
 Says whether two removals or renames name the same path, comparing their parts and never text joined from them.

 A path can hold any byte but NUL, so no join of two paths is safe to compare; their parts are compared one by one instead.

 @param one - one removal or rename

 @param other - another removal or rename

 @returns whether both name the same path the same way

 @example
 ```ts
 const same = sameRemoval({ one: { status: 'deleted', path: 'x' }, other: { status: 'deleted', path: 'x' } },);
 ```
 */
function sameRemoval({
  one,
  other,
}: {
  readonly one: RemovedOrRenamedPath;
  readonly other: RemovedOrRenamedPath;
},): boolean {
  if (one.status === 'deleted') {
    return (other.status === 'deleted') && (one.path === other.path);
  }
  if (other.status !== 'renamed') {
    return false;
  }
  if (one.was !== other.was) {
    return false;
  }
  return one.now === other.now;
}

/**
 Reads the paths a range removed or renamed of one kind, which a brief names even though the commit holds no file to read in
 full at them.

 @param changed - the paths the ranges changed, in the order git named them

 @param kind - the kind of path whose removals and renames the caller wants

 @param pathKindOf - what one path is to a review

 @returns the removals and renames of that kind, in the order git named them, each once

 @example
 ```ts
 const removed = removedOrRenamedAmong({ changed, kind: 'code', pathKindOf },);
 ```
 */
export function removedOrRenamedAmong({
  changed,
  kind,
  pathKindOf,
}: {
  readonly changed: readonly ChangedPath[];
  readonly kind: PathKind;
  readonly pathKindOf: PathKindOfAPath;
},): readonly RemovedOrRenamedPath[] {
  /**
   The removals and renames read so far.
   */
  const removed: RemovedOrRenamedPath[] = [];
  for (const one of changed) {
    if (kindOfChangedPath({
      changed: one,
      pathKindOf,
    },) !== kind) {
      continue;
    }
    for (const entry of removedOrRenamedBy({ changed: one, },)) {
      if (!removed.some(function namesTheSame(seen,) {
        return sameRemoval({
          one: entry,
          other: seen,
        },);
      },)) {
        removed.push(entry,);
      }
    }
  }
  return removed;
}
