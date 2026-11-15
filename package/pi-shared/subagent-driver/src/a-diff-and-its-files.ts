/**
 Reads one answer of `git diff --raw -z --patch` as both the diff a review is given and the paths that diff touches.

 One question answers both, so the list of files and the diff written beside it can never be of two moments: `--raw` names each file
 with the status Git gave it and `--patch` is the diff itself, and both come out of the same read. `-z` ends every field with a NUL,
 so a path holding a tab, a space or a newline arrives whole.

 The paths kept are the ones `--diff-filter=AM` names: a file the range added or modified. A file it deleted, renamed, copied, or
 changed the type of is not one a review reads as the change.

 @module
 */

/**
 The byte Git ends every field with when `-z` is given.
 */
const FIELD_END = '\0';

/**
 The statuses of the files a review reads as changed: one it added, or one it modified.
 */
const STATUSES_KEPT = 'AM';

/**
 The statuses of the entries that name two paths, the old and the new, which are not kept.
 */
const STATUSES_WITH_TWO_PATHS = 'RC';

/**
 The number of fields one raw entry holds beside its header: its path, and the destination a rename or a copy adds.
 */
const FIELDS_OF_A_PATH = 1;

/**
 The number of fields a rename's or a copy's raw entry adds beyond one path.
 */
const FIELDS_OF_A_SECOND_PATH = 2;

/**
 The number of fields that stand between the empty field and the patch: the empty field itself.
 */
const FIELDS_BEFORE_THE_PATCH = 1;

/**
 What one answer of `git diff --raw -z --patch` says.
 */
export type DiffAndFiles = {
  /**
   The patch itself, which is the diff a review reads.
   */
  readonly diff: string;

  /**
   The paths the range added or modified, in the order Git named them.
   */
  readonly paths: readonly string[];
};

/**
 Reads the status of one raw entry.

 @param header - the entry's header, which begins with a colon

 @returns its status letter and number, as `A`, `M` or `R100`, and empty text for a header that carries none

 @example
 ```ts
 const status = statusInHeader({ header: ':000000 100644 0000000 587be6b A', },);
 ```
 */
function statusInHeader({ header, }: { readonly header: string },): string {
  return header.split(' ',)
    .at(-1,)
    ?? '';
}

/**
 Reads one answer of `git diff --raw -z --patch` as the diff it holds and the paths the range added or modified.

 @param answer - what the question printed

 @returns the patch, which is empty when the range changed nothing, and the paths kept

 @example
 ```ts
 const read = diffAndFilesIn({ answer, },);
 ```
 */
export function diffAndFilesIn({ answer, }: { readonly answer: string },): DiffAndFiles {
  /**
   The answer's fields, which `-z` ends with a NUL: a raw entry's header, then one or two paths, then an empty field and the patch.
   */
  const fields = answer.split(FIELD_END,);
  /**
   The paths the range added or modified, in the order Git named them.
   */
  const paths: string[] = [];
  for (let at = 0; at < fields.length; at += 1) {
    /**
     The field this step reads.
     */
    const field = fields[at] ?? '';
    // The empty field after the last raw entry is where the patch begins.
    if (field === '') {
      return {
        diff: fields.slice(at + FIELDS_BEFORE_THE_PATCH,)
          .join(FIELD_END,),
        paths,
      };
    }
    // An answer that is neither a raw entry nor the empty field before its patch is not one this reads a diff out of.
    if (!field.startsWith(':')) {
      return {
        diff: '',
        paths,
      };
    }
    /**
     The status Git gave this entry.
     */
    const status = statusInHeader({ header: field, },);
    /**
     The path of this entry, which is the added or modified file unless the entry is a rename or a copy.
     */
    const path = fields[at + FIELDS_OF_A_PATH] ?? '';
    if (STATUSES_WITH_TWO_PATHS.includes(status.charAt(0,),)) {
      at += FIELDS_OF_A_SECOND_PATH;
      continue;
    }
    if (STATUSES_KEPT.includes(status.charAt(0,),)) {
      paths.push(path,);
    }
    at += FIELDS_OF_A_PATH;
  }
  return {
    diff: '',
    paths,
  };
}
