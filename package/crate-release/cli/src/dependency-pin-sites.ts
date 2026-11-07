import { CrateReleaseError } from './error.ts';
import {
  foundValue,
  type Lookup,
  missingValue,
} from './lookup.ts';
import { versionKey } from './package-version.ts';
import {
  blankCharacters,
  endsWithComment,
  locateValue,
  skipBlanks,
  type ValueLocation,
} from './toml-line.ts';

/**
 Where the quoted version requirement of one dependency on the crate sits in a manifest.
 */
export type PinSite = {
  /**
   Index of the line that holds the quoted requirement.
   */
  readonly index: number;
  /**
   That line without its carriage return.
   */
  readonly body: string;
  /**
   The requirement inside the line.
   */
  readonly location: ValueLocation;
};

/**
 Table names, before the crate's name, under which Cargo reads a dependency written as a table of its own.
 */
const dependencyTableKinds: readonly string[] = [
  'dependencies',
  'dev-dependencies',
  'build-dependencies',
];

/**
 Characters that may stand directly before a key inside an inline table: its opening brace or a comma.
 */
const inlineKeyIntroducers = '{,';

/**
 List where a word occurs in a text, without overlap.

 @param text - text under scan

 @param needle - non-empty word to find

 @returns the start index of each occurrence, in order, from one pass over the text

 @example
 ```ts
 occurrencesOf({ text: 'a version, version', needle: 'version' });
 ```
 */
function occurrencesOf({
  text,
  needle,
}: {
  readonly text: string;
  readonly needle: string;
}): readonly number[] {
  /**
   Pieces of the text between occurrences.
   */
  const pieces = text.split(needle);
  return pieces.slice(
    0,
    -1
  )
    .reduce<number[]>(
    function addStart(
      starts,
      piece
    ) {
      starts.push((starts.at(-1) ?? (-needle.length)) + needle.length
        + piece.length);
      return starts;
    },
    [],
    );
}

/**
 For each position of a text, find the closest character before it that is not a space or tab.

 @param text - text under scan

 @returns one entry per position; the entry is the empty text when only blanks, or nothing, come before the position

 @example
 ```ts
 significantBefore('a b');
 ```
 */
function significantBefore(text: string): readonly string[] {
  return Array.from(text)
    .reduce<{
      last: string;
      readonly entries: string[]
    }>(
    function extend(
      state,
      character
    ) {
      state.entries
        .push(state.last);
      state.last = blankCharacters.includes(character) ? state.last : character;
      return state;
    },
    {
      last: '',
      entries: []
    },
    )
    .entries;
}

/**
 Find the quoted `version` value inside the inline table of a dependency line.

 @param body - line body that starts a dependency of the wanted crate

 @param tableStart - index of the opening brace

 @returns the version location, or a miss when the inline table has no quoted `version` key

 @example
 ```ts
 findInlineVersion({ body: 'a = { version = "=1" }', tableStart: 4 });
 ```
 */
function findInlineVersion({
  body,
  tableStart,
}: {
  readonly body: string;
  readonly tableStart: number;
}): Lookup<ValueLocation> {
  /**
   Closest meaningful character before each position, so a key is only taken right after `{` or `,`.
   */
  const before = significantBefore(body);
  /**
   Places inside the table where a complete quoted `version` key stands.
   */
  const keys = occurrencesOf({
    text: body,
    needle: versionKey
  })
    .filter(function isInTable(index) {
      return index > tableStart;
    })
    .flatMap(function locateKey(index) {
      /**
       Character that introduces the key.
       */
      const introducer = before[index] ?? '';
      if ((introducer === '') || (!inlineKeyIntroducers.includes(introducer)))
        return [];
      /**
       The value, if the word is a complete key followed by a quoted string.
       */
      const location = locateValue({
        line: body,
        from: index,
        key: versionKey
      });
      return location.found ? [location.value] : [];
    });
  /**
   The first complete key.
   */
  const [first] = keys;
  return first === undefined ? missingValue('The inline table has no quoted version key.') : foundValue(first);
}

/**
 Find where the inline table of a dependency on a crate begins in a line.

 @param body - line body

 @param crateName - crate the dependency is on

 @returns the index of the opening brace, or a miss when the line is not `<crateName> = {` (a longer name that merely
 starts with the crate name does not count)

 @example
 ```ts
 findDependencyTable({ body: 'a = { path = "x" }', crateName: 'a' });
 ```
 */
function findDependencyTable({
  body,
  crateName,
}: {
  readonly body: string;
  readonly crateName: string;
}): Lookup<number> {
  /**
   Index of the first non-blank character, where the crate name must begin.
   */
  const start = skipBlanks({
    line: body,
    from: 0
  });
  if (!body.startsWith(
    crateName,
    start
  ))
    return missingValue('The line does not begin with the crate name.');
  /**
   Index of the equals sign, after optional blanks.
   */
  const equals = skipBlanks({
    line: body,
    from: start + crateName.length
  });
  if (body.charAt(equals) !== '=')
    return missingValue('The crate name is not followed by an equals sign.');
  /**
   Index of the opening brace, after optional blanks.
   */
  const brace = skipBlanks({
    line: body,
    from: equals + 1
  });
  return body.charAt(brace) === '{' ? foundValue(brace) : missingValue('The dependency is not an inline table.');
}

/**
 Find the dependencies on a crate that are written as an inline table on one line, `<crate> = { ..., version = "=x" }`.

 @param bodies - manifest line bodies

 @param crateName - crate the dependency is on

 @returns one site per such line, in line order

 @throws CrateReleaseError when such a line has no quoted `version` key in its inline table

 @example
 ```ts
 inlinePinSites({ bodies: ['a = { version = "=1" }'], crateName: 'a' });
 ```
 */
export function inlinePinSites({
  bodies,
  crateName,
}: {
  readonly bodies: readonly string[];
  readonly crateName: string;
}): readonly PinSite[] {
  return bodies.flatMap(function inlineSite(
    body,
    index
  ): PinSite[] {
    /**
     Place of the inline table, when this line depends on the crate.
     */
    const table = findDependencyTable({
      body,
      crateName
    });
    if (!table.found)
      return [];
    /**
     The version requirement of the dependency.
     */
    const location = findInlineVersion({
      body,
      tableStart: table.value
    });
    if (!location.found)
      throw new CrateReleaseError(
        `The dependency line ${JSON.stringify(body)} has no quoted version key in its inline table.`,
      );
    return [
      {
        index,
        body,
        location: location.value
      }
    ];
  });
}

/**
 Check whether a line is a table header of any kind.

 @param body - line body

 @returns true when the first character that is not a blank opens a header; a line inside an array value that starts
 with a bracket counts too, which dependency tables never hold

 @example
 ```ts
 isTableHeader('[package]');
 ```
 */
function isTableHeader(body: string): boolean {
  return body.charAt(skipBlanks({
    line: body,
    from: 0
  })) === '[';
}

/**
 Check whether a line is the header of a dependency table of exactly one crate.

 @param body - line body

 @param crateName - crate the table must name

 @returns true for `[dependencies.<crate>]`, `[dev-dependencies.<crate>]` or `[build-dependencies.<crate>]`, each with
 optional blanks before the bracket and an optional comment after it; a crate whose name only starts with this one, an
 array of tables `[[...]]`, a nested table such as `[dependencies.<crate>.x]`, a table of another kind such as
 `[workspace.dependencies.<crate>]`, and a header inside a comment are not matched

 @example
 ```ts
 isDependencyTableHeader({ body: '[dependencies.a]', crateName: 'a' });
 ```
 */
function isDependencyTableHeader({
  body,
  crateName,
}: {
  readonly body: string;
  readonly crateName: string;
}): boolean {
  /**
   Index of the opening bracket, after optional blanks.
   */
  const open = skipBlanks({
    line: body,
    from: 0
  });
  if (body.charAt(open) !== '[')
    return false;
  /**
   Index of the first closing bracket.
   */
  const close = body.indexOf(
    ']',
    open + 1
  );
  if (close === (-1))
    return false;
  /**
   The text between the brackets.
   */
  const name = body.slice(
    open + 1,
    close
  );
  return dependencyTableKinds.some(function namesTheCrate(kind) {
    return name === `${kind}.${crateName}`;
  })
    && endsWithComment(body.slice(close + 1));
}

/**
 Find the quoted `version` value among the lines of one table.

 @param bodies - manifest line bodies

 @param header - index of the table's header line

 @returns the site, or a miss when the lines up to the next header hold no `version = "..."` key at the start of a line

 @example
 ```ts
 findTableVersion({ bodies: ['[dependencies.a]', 'version = "=1"'], header: 0 });
 ```
 */
function findTableVersion({
  bodies,
  header,
}: {
  readonly bodies: readonly string[];
  readonly header: number;
}): Lookup<PinSite> {
  /**
   Index of the first line after the header.
   */
  const bodyStart = header + 1;
  /**
   Lines after the header.
   */
  const following = bodies.slice(bodyStart);
  /**
   How many of those lines belong to the table: the ones before the next header.
   */
  const nextHeader = following.findIndex(isTableHeader);
  /**
   The first line of the table that holds a complete quoted `version` key.
   */
  const [first] = (nextHeader === (-1) ? following : following.slice(
    0,
    nextHeader
  ))
    .flatMap(function tableSite(
      body,
      offset
    ): PinSite[] {
      /**
       The value, if the line is `version = "..."`.
       */
      const location = locateValue({
        line: body,
        from: skipBlanks({
          line: body,
          from: 0
        }),
        key: versionKey
      });
      return location.found ? [
        {
          index: bodyStart + offset,
          body,
          location: location.value
        }
      ] : [];
    });
  return first === undefined ? missingValue('The table has no quoted version key.') : foundValue(first);
}

/**
 Find the dependencies on a crate that are written as a table of their own, `[dependencies.<crate>]` with the
 requirement on a line of its own.

 @param bodies - manifest line bodies

 @param crateName - crate the table is for

 @returns one site per such table, in line order; the sites of a `[dev-dependencies.<crate>]` and a
 `[build-dependencies.<crate>]` table are included

 @throws CrateReleaseError when such a table has no quoted `version` key before the next header, or text that is not
 a comment follows its value

 @example
 ```ts
 tablePinSites({ bodies: ['[dependencies.a]', 'version = "=1"'], crateName: 'a' });
 ```
 */
export function tablePinSites({
  bodies,
  crateName,
}: {
  readonly bodies: readonly string[];
  readonly crateName: string;
}): readonly PinSite[] {
  return bodies.flatMap(function tableSite(
    body,
    header
  ): PinSite[] {
    if (!isDependencyTableHeader({
      body,
      crateName
    }))
      return [];
    /**
     The requirement inside the table.
     */
    const found = findTableVersion({
      bodies,
      header
    });
    if (!found.found)
      throw new CrateReleaseError(
        `The table ${JSON.stringify(body.trim())} has no quoted version key before the next table header (an inherited version.workspace key cannot be released here).`,
      );
    /**
     The line that holds the requirement, and the requirement inside it.
     */
    const {
      body: versionLine,
      location
    } = found.value;
    /**
     Text after the closing quote of the requirement.
     */
    const rest = versionLine.slice(location.valueEnd + 1);
    if (!endsWithComment(rest))
      throw new CrateReleaseError(
        `The version line ${JSON.stringify(versionLine)} of the table ${JSON.stringify(body.trim())} has text after the value that is not a comment.`,
      );
    return [found.value];
  });
}
