import {
  isAsciiAlphanumeric,
  isWholeNumberText,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Cache account read
// WHAT THE PRE-LAUNCH CACHE CHECK READS (ledger M28). Every stage cache is
// stamped with the build's digest; the version constants are defence in
// depth, and each constant's TSDoc accounts for the commits that changed its
// stage's answers since the constant last moved. M28 found the check walking
// three of six constants, the ones remembered, so this reads every declaration
// ending in `CACHE_VERSION` out of the source instead of a list.
//
// PURE READING, SPLIT FROM THE ENTRY (`cache-account-audit.ts`), which asks
// git and prints. Nothing here runs a process or reads a file.
//
// THE COMMIT THAT SET A VALUE is the newest one that ADDED its declaration on
// balance. The scratch script this replaces took the oldest commit git's
// pickaxe named, which is wrong once a value is set, changed and set back,
// and a move between files adds and removes the line in one commit, which
// sets nothing.

/**
 Opening of a declaration line this reads.
 */
const EXPORTED_DECLARATION = 'export const ';

/**
 Opening of an unexported one.
 */
const LOCAL_DECLARATION = 'const ';

/**
 Ending every cache version constant's name carries.
 */
const VERSION_NAME_ENDING = 'CACHE_VERSION';

/**
 Assignment between a constant's name and its value, as the formatter writes it.
 */
const ASSIGNMENT = ' = ';

/**
 Statement end after the value.
 */
const STATEMENT_END = ';';

/**
 Characters of a commit hash the version accounts cite.
 */
export const CITED_HASH_LENGTH = 9;

/**
 A hash as the version accounts cite it.

 @param hash - full object id

 @returns Its first nine characters

 @example
 ```ts
 citedHash({ hash: 'f6e93ed5f0000000000000000000000000000000', },); // 'f6e93ed5f'
 ```
 */
export function citedHash({ hash, }: { readonly hash: string; },): string {
  return hash.slice(
    0,
    CITED_HASH_LENGTH,
  );
}

/**
 One cache version constant as its source declares it.

 @example
 ```ts
 const version: CacheVersion = { name: 'PAIRING_CACHE_VERSION', value: 3, path: 'src/pairing-cache-version.ts', declaration: 'PAIRING_CACHE_VERSION = 3', };
 ```
 */
export type CacheVersion = {
  /**
   Constant's name.
   */
  readonly name: string;

  /**
   Its current value.
   */
  readonly value: number;

  /**
   File declaring it, as the caller named it.
   */
  readonly path: string;

  /**
   Name and value as the source writes them, the text git's pickaxe looks for.
   */
  readonly declaration: string;
};

/**
 A declaration line naming a cache version that does not read as
 `NAME = digits;`, which the check would otherwise skip without a word.
 A STATED REFUSAL, since the message quotes one line of this package's own
 source and says how to write it: the command reports it as declined, with no
 frames, rather than as a fault of the command.

 @example
 ```ts
 throw new CacheAccountReadError({ path: 'src/a.ts', line: 'export const A_CACHE_VERSION: number = 1;', },);
 ```
 */
export class CacheAccountReadError extends StatedRefusalError {
  /**
   Declared here as well as inherited, so the source scan that keeps the
   marked-class inventory sees it: the message quotes one line of this
   package's own source, never corpus text.
   */
  override readonly messageNamesOnly: true = true;
  /**
   Builds the refusal naming the file and the line.

   @param path - file holding the line

   @param line - declaration as written

   @example
   ```ts
   new CacheAccountReadError({ path: 'src/a.ts', line: 'export const A_CACHE_VERSION: number = 1;', },);
   ```
   */
  constructor({
    path,
    line,
  }: {
    readonly path: string;
    readonly line: string;
  },) {
    super({
      says: `${path} declares a cache version the audit cannot read: "${line}". Write it as NAME = digits; so every constant is checked.`,
    },);
    this.name = 'CacheAccountReadError';
  }
}

/**
 Openings of the declaration lines this reads, exported first.
 */
const DECLARATION_KEYWORDS = [
  EXPORTED_DECLARATION,
  LOCAL_DECLARATION,
] as const;

/**
 Length of the identifier a declaration opens with.

 @param rest - declaration after its keywords

 @returns Units up to the first character no identifier here carries

 @example
 ```ts
 identifierLength({ rest: 'A_CACHE_VERSION = 1;', },); // 15
 ```
 */
function identifierLength({ rest, }: { readonly rest: string; },): number {
  /**
   Offset of the first character outside the identifier, or the whole length.
   */
  const end = Array.from(rest,)
    .findIndex(function outside(character,): boolean {
      return !((character === '_') || isAsciiAlphanumeric({ character, },));
    },);
  return (end === (-1)) ? rest.length : end;
}

/**
 What one line says about cache versions: nothing, a version, or a
 declaration naming one that does not read as `NAME = digits;`.

 @example
 ```ts
 const reading: DeclarationReading = { kind: 'version', name: 'A_CACHE_VERSION', digits: '1', };
 ```
 */
type DeclarationReading =
  | { readonly kind: 'none'; }
    | {
      readonly kind: 'version';

    /**
     Constant's name.
     */
      readonly name: string;

    /**
     Its value as written.
     */
      readonly digits: string;
    }
    | { readonly kind: 'unreadable'; };

/**
 Reads one source line as a cache version declaration, the one reading both
 the current source and the lines of a diff go through.

 A NAME IS READ WHOLE, never found inside a longer one: `SLICE_CACHE_VERSION`
 sits inside `TRANSLATE_SLICE_CACHE_VERSION`, and `= 3` at the start of
 `= 34`, so a substring would mistake one version for another (ledger B23).

 @param line - one line, trimmed

 @returns What the line declares

 @example
 ```ts
 readDeclaration({ line: 'export const PAIRING_CACHE_VERSION = 3;', },); // { kind: 'version', name: 'PAIRING_CACHE_VERSION', digits: '3' }
 ```
 */
function readDeclaration({ line, }: { readonly line: string; },): DeclarationReading {
  /**
   Keywords the line opens with, when it declares a constant.
   */
  const keyword = DECLARATION_KEYWORDS.find(function opens(candidate,): boolean {
    return line.startsWith(candidate,);
  },);
  if (keyword === undefined)
    return { kind: 'none', };
  /**
   Declaration after its keywords.
   */
  const rest = line.slice(keyword.length,);
  /**
   Constant's name.
   */
  const name = rest.slice(
    0,
    identifierLength({ rest, },),
  );
  if (!name.endsWith(VERSION_NAME_ENDING,))
    return { kind: 'none', };
  /**
   What follows the name, which has to be the assignment of a number.
   */
  const assigned = rest.slice(name.length,);
  /**
   Digits between the assignment and the statement end.
   */
  const digits = assigned.slice(
    ASSIGNMENT.length,
    -STATEMENT_END.length,
  );
  /**
   Whether the name is assigned a whole number and the statement ends there.
   */
  const readable = assigned.startsWith(ASSIGNMENT,)
    && assigned.endsWith(STATEMENT_END,)
    && isWholeNumberText({ text: digits, },);
  if (!readable)
    return { kind: 'unreadable', };
  return {
    kind: 'version',
    name,
    digits,
  };
}

/**
 Every cache version constant one source file declares.

 @param path - file's name, carried into each result

 @param text - file's contents

 @returns Constants in line order

 @throws CacheAccountReadError where a declaration naming a cache version
 does not read as `NAME = digits;`

 @example
 ```ts
 cacheVersionsIn({ path: 'src/pairing-cache-version.ts', text: 'export const PAIRING_CACHE_VERSION = 3;', },);
 ```
 */
export function cacheVersionsIn(
  {
    path,
    text,
  }: {
    readonly path: string;
    readonly text: string;
  },
): readonly CacheVersion[] {
  return text
    .split('\n',)
    .flatMap(function versionOf(raw,): readonly CacheVersion[] {
      /**
       Line without its indentation.
       */
      const line = raw.trim();
      /**
       What the line declares.
       */
      const reading = readDeclaration({ line, },);
      if (reading.kind === 'none')
        return [];
      if (reading.kind === 'unreadable')
        throw new CacheAccountReadError({
          path,
          line,
        },);
      return [{
        name: reading.name,
        value: Number(reading.digits,),
        path,
        declaration: `${reading.name}${ASSIGNMENT}${reading.digits}`,
      },];
    },);
}

/**
 Character a hunk header opens with, repeated once more than the diff has
 mark columns: `@@` over one parent, `@@@` over two.
 */
const HUNK_MARK = '@';

/**
 Mark columns of the hunks that follow a header line.

 @param line - hunk header, `@@ -1 +1 @@` or `@@@ -1 -1 +1 @@@`

 @returns One for a diff against one parent, one per parent for a merge's
 combined diff

 @example
 ```ts
 hunkMarkColumns({ line: '@@@ -1,2 -1,2 +1,2 @@@', },); // 2
 ```
 */
function hunkMarkColumns({ line, }: { readonly line: string; },): number {
  /**
   Offset of the first character past the header's opening run.
   */
  const end = Array.from(line,)
    .findIndex(function pastRun(character,): boolean {
      return character !== HUNK_MARK;
    },);
  return ((end === (-1)) ? line.length : end) - 1;
}

/**
 One hunk of a diff: how many mark columns open its lines, and the lines
 under its header. The lines before any header read as one column, as a
 diff written without headers does.
 */
type DiffHunk = {
  readonly columns: number;
  readonly lines: readonly string[];
};

/**
 Where one hunk header stands among a diff's lines, with the mark columns
 the hunk under it opens.
 */
type HunkHeader = {
  readonly index: number;
  readonly columns: number;
};

/**
 Splits a diff into its hunks by their header lines.

 @param diff - `git show` output, or lines written without a header

 @returns Hunks in diff order, the lines before the first header first

 @example
 ```ts
 diffHunksOf({ diff: '@@@ -1 -1 +1 @@@\n++a', },); // [{ columns: 1, lines: [] }, { columns: 2, lines: ['++a'] }]
 ```
 */
function diffHunksOf({ diff, }: { readonly diff: string; },): readonly DiffHunk[] {
  /**
   The diff's lines.
   */
  const lines = diff.split('\n',);
  /**
   Where each header stands, with the columns it opens.
   */
  const headers = lines.flatMap(function headerAt(
    line,
    index,
  ): readonly HunkHeader[] {
    return line.startsWith(`${HUNK_MARK}${HUNK_MARK}`,)
      ? [{
        index,
        columns: hunkMarkColumns({ line, },),
      },]
      : [];
  },);
  /**
   Lines before the first header.
   */
  const preamble: DiffHunk = {
    columns: 1,
    lines: lines.slice(
      0,
      headers
        .at(0,)
        ?.index
        ?? lines.length,
    ),
  };
  return [
    preamble,
    ...headers.map(function hunkUnder(
      header,
      order,
    ): DiffHunk {
      return {
        columns: header.columns,
        lines: lines.slice(
          header.index + 1,
          headers
            .at(order + 1,)
            ?.index
            ?? lines.length,
        ),
      };
    },),
  ];
}

/**
 Which way one line of a diff runs.
 */
type LineChange = 'added' | 'removed' | 'neither';

/**
 Which way one line of a diff runs: added where every mark column is `+`,
 removed where every one is `-`, neither otherwise. In a merge's combined
 diff a line marked `+` in some columns and not others stood in a parent
 already, so the merge brought it in and did not write it.

 @param marks - line's mark columns

 @returns Which way it runs

 @example
 ```ts
 lineChangeOf({ marks: '++', },); // 'added'
 ```
 */
function lineChangeOf({ marks, }: { readonly marks: string; },): LineChange {
  /**
   Each mark column.
   */
  const columns = Array.from(marks,);
  if (columns.every(function plus(mark,): boolean {
    return mark === '+';
  },))
    return 'added';
  if (columns.every(function minus(mark,): boolean {
    return mark === '-';
  },))
    return 'removed';
  return 'neither';
}

/**
 Lines a diff adds and removes that declare one version at its current
 value, read as {@link cacheVersionsIn} reads the source.

 A HISTORICAL LINE THAT DOES NOT READ is no declaration of this value and is
 left uncounted, not refused: the source may once have written one another
 way, and only the current source has to read.

 A MERGE'S COMBINED DIFF IS READ TOO (ledger B814): `git show` writes a merge
 with one mark column per parent, and a declaration its resolution wrote is
 marked added in every column. Such a line stands in no ordinary commit, so
 the merge is the commit that set it.

 NO LINE IS SET ASIDE AS A FILE HEADER: `--- a/path` and `+++ b/path` never
 read as a declaration, while a merge of three parents marks a line its
 resolution wrote `+++` and one it removed `---`, and those count.

 @param diff - `git show` output for one commit, a merge's combined diff
 included

 @param version - constant and value looked for

 @returns Counts of added and removed declarations of that value

 @example
 ```ts
 declarationLineCounts({ diff: '@@ -1 +1 @@\n+export const A_CACHE_VERSION = 2;', version, },); // { added: 1, removed: 0 }
 ```
 */
export function declarationLineCounts(
  {
    diff,
    version,
  }: {
    readonly diff: string;
    readonly version: Pick<CacheVersion, 'name' | 'value'>;
  },
): {
  readonly added: number;
  readonly removed: number;
} {
  /**
   Which way each line declaring this value runs, hunk by hunk.
   */
  const changes = diffHunksOf({ diff, },)
    .flatMap(function hunkChanges(hunk,): readonly LineChange[] {
      return hunk.lines
        .filter(function declaresVersion(line,): boolean {
          /**
           What the line declares, its marks removed.
           */
          const reading = readDeclaration({
            line: line
              .slice(hunk.columns,)
              .trim(),
          },);
          return (reading.kind === 'version')
            && (reading.name === version.name)
            && (Number(reading.digits,) === version.value);
        },)
        .map(function changeOf(line,): LineChange {
          return lineChangeOf({
            marks: line.slice(
              0,
              hunk.columns,
            ),
          },);
        },);
    },);
  return {
    added: changes
      .filter(function isAdded(change,): boolean {
        return change === 'added';
      },)
      .length,
    removed: changes
      .filter(function isRemoved(change,): boolean {
        return change === 'removed';
      },)
      .length,
  };
}

//endregion Cache account read
