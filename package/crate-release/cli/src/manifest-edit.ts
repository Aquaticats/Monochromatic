import { exactPinPrefix } from './constants.ts';
import { CrateReleaseError } from './error.ts';
import {
  foundValue,
  type Lookup,
  missingValue,
} from './lookup.ts';
import {
  blankCharacters,
  endsWithComment,
  joinManifest,
  locateValue,
  skipBlanks,
  splitManifest,
  spliceValue,
  type ValueLocation,
} from './toml-line.ts';

/**
 Result of editing a manifest text.
 */
export type ManifestEdit = {
  /**
   The manifest text after the edit; the input text itself when nothing changed.
   */
  readonly text: string;
  /**
   True when at least one line changed.
   */
  readonly changed: boolean;
  /**
   Number of lines that changed; zero when the manifest already held the target.
   */
  readonly replacedLines: number;
};

/**
 Key that holds the version in both a package table and an inline dependency table.
 */
const versionKey = 'version';

/**
 Header of the table that holds the crate's own version.
 */
const packageHeader = '[package]';

/**
 Characters that may stand directly before a key inside an inline table: its opening brace or a comma.
 */
const inlineKeyIntroducers = '{,';

/**
 Check whether a trimmed line is the header of the package table, with at most a comment after it.

 @param trimmed - line without surrounding blanks

 @returns true for `[package]` and `[package] # note`

 @example
 ```ts
 isPackageHeader('[package]');
 ```
 */
function isPackageHeader(trimmed: string): boolean {
  return trimmed.startsWith(packageHeader) && endsWithComment(trimmed.slice(packageHeader.length));
}

/**
 Mark each line that belongs to the package table: the lines after its header up to the next table header.

 @param bodies - line bodies

 @returns one flag per line

 @example
 ```ts
 packageLineFlags(['[package]', 'version = "1"', '[dependencies]']);
 ```
 */
function packageLineFlags(bodies: readonly string[]): readonly boolean[] {
  return bodies.reduce<boolean[]>(
    function mark(
      flags,
      body
    ) {
      /**
       Whether the line before this one belonged to the package table.
       */
      const previous = flags.at(-1) ?? false;
      flags.push(body.charAt(skipBlanks({
        line: body,
        from: 0
      })) === '[' ? isPackageHeader(body.trim()) : previous);
      return flags;
    },
    [],
  );
}

/**
 Decide what a located version value means for the release.

 @param found - value found in the manifest

 @param current - value a release replaces

 @param target - value a release writes

 @param description - what the value is, for the message

 @returns the action: replace it, or leave it because it already holds the target

 @throws CrateReleaseError when the value is neither the current one nor the target

 @example
 ```ts
 decideValue({ found: '0.4.2', current: '0.4.2', target: '0.4.3', description: 'x' });
 ```
 */
function decideValue({
  found,
  current,
  target,
  description,
}: {
  readonly found: string;
  readonly current: string;
  readonly target: string;
  readonly description: string;
}): 'replace' | 'keep' {
  if (found === current)
    return 'replace';
  if (found === target)
    return 'keep';
  throw new CrateReleaseError(
    `${description} is ${JSON.stringify(found)}, which is neither the current ${JSON.stringify(current)} nor the target ${JSON.stringify(target)}; fix the manifest or the arguments, then run again.`,
  );
}

/**
 A manifest line that starts a dependency on the crate, with where its inline table opens.
 */
type DependencyLine = {
  readonly index: number;
  readonly body: string;
  readonly tableStart: number;
};

/**
 Replace the crate's own version in the package table of a manifest.

 Only the `version` key of the `[package]` table counts: a `rust-version` key, a `version` key of another table, or a
 dependency's version is never touched. The current value must equal `from` exactly (no prefix or substring match);
 a value that already equals `to` is left alone, so a release interrupted after this edit can be run again.

 @param text - manifest text

 @param from - version the manifest must hold now

 @param to - version to write

 @returns the edited text, whether it changed, and how many lines changed

 @throws CrateReleaseError when the package table has no quoted `version`, holds text after the value that is not a
 comment, or holds a version that is neither `from` nor `to`

 @example
 ```ts
 replacePackageVersion({ text: '[package]\nversion = "0.4.2"\n', from: '0.4.2', to: '0.4.3' });
 ```
 */
export function replacePackageVersion({
  text,
  from,
  to,
}: {
  readonly text: string;
  readonly from: string;
  readonly to: string;
}): ManifestEdit {
  /**
   Line bodies and their carriage returns.
   */
  const {
    bodies,
    hadReturn
  } = splitManifest(text);
  /**
   Flag per line for membership in the package table.
   */
  const inPackage = packageLineFlags(bodies);
  /**
   Lines of the package table that carry a quoted `version` key; TOML allows one, so the first is the one.
   */
  const candidates = bodies.flatMap(function packageVersion(
    body,
    index
  ) {
    if (inPackage[index] !== true)
      return [];
    /**
     The version value, when this line is the version key.
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
        index,
        body,
        location: location.value
      }
    ] : [];
  });
  /**
   The line that holds the version.
   */
  const [first] = candidates;
  if (first === undefined)
    throw new CrateReleaseError(
      'The manifest has no quoted version = "..." line in its [package] table (an inherited version.workspace key cannot be released here).',
    );
  if (!endsWithComment(first.body
    .slice(first.location
      .valueEnd
      + 1)))
    throw new CrateReleaseError(
      `The [package] version line ${JSON.stringify(first.body)} has text after the value that is not a comment.`,
    );
  if (decideValue({
    found: first.location
      .value,
    current: from,
    target: to,
    description: 'The [package] version'
  }) === 'keep')
    return {
      text,
      changed: false,
      replacedLines: 0
    };
  return {
    text: joinManifest({
      bodies: bodies.with(
        first.index,
        spliceValue({
          body: first.body,
          location: first.location,
          replacement: to
        })
      ),
      hadReturn,
    }),
    changed: true,
    replacedLines: 1,
  };
}

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
 Move the exact pin of a dependency on a crate from one version to another.

 A pin is the requirement `"=<version>"` in the dependency's inline table. Every line that depends on the crate is
 checked, and each must hold `=<from>` (changed) or `=<to>` (kept); any other requirement is refused, so a loose
 requirement or a stale pin is never rewritten silently. Only the quoted pin changes: the crate's own version, other
 crates' pins, and a crate whose name merely starts with this one are left alone.

 @param text - manifest text

 @param crateName - crate whose pin moves

 @param from - version the pin must hold now

 @param to - version to pin

 @returns the edited text, whether it changed, and how many lines changed

 @throws CrateReleaseError when no line depends on the crate with an inline table, a line has no quoted `version`
 key, or a pin is neither `=<from>` nor `=<to>`

 @example
 ```ts
 replaceDependencyPin({ text: 'a = { version = "=1.0.0" }', crateName: 'a', from: '1.0.0', to: '1.0.1' });
 ```
 */
export function replaceDependencyPin({
  text,
  crateName,
  from,
  to,
}: {
  readonly text: string;
  readonly crateName: string;
  readonly from: string;
  readonly to: string;
}): ManifestEdit {
  /**
   Line bodies and their carriage returns.
   */
  const {
    bodies,
    hadReturn
  } = splitManifest(text);
  /**
   Lines that depend on the crate with an inline table.
   */
  const dependencies: readonly DependencyLine[] = bodies.flatMap(function dependencyLine(
    body,
    index
  ): DependencyLine[] {
    /**
     Place of the inline table, when this line depends on the crate.
     */
    const table = findDependencyTable({
      body,
      crateName
    });
    return table.found ? [
      {
        index,
        body,
        tableStart: table.value
      }
    ] : [];
  });
  if (dependencies.length === 0)
    throw new CrateReleaseError(
      `No dependency line of the form ${crateName} = { ... } exists in the manifest; table-style [dependencies.${crateName}] entries are not supported.`,
    );
  /**
   Edited text of each line whose pin moves, by line index.
   */
  const replacements = new Map(dependencies.flatMap(function planPin({
    index,
    body,
    tableStart,
  }): [
    number,
    string
  ][] {
    /**
     The version requirement of the dependency.
     */
    const location = findInlineVersion({
      body,
      tableStart
    });
    if (!location.found)
      throw new CrateReleaseError(
        `The dependency line ${JSON.stringify(body)} has no quoted version key in its inline table.`,
      );
    return decideValue({
      found: location.value
        .value,
      current: `${exactPinPrefix}${from}`,
      target: `${exactPinPrefix}${to}`,
      description: `The pin of ${crateName}`
    }) === 'replace' ? [
      [
        index,
        spliceValue({
          body,
          location: location.value,
          replacement: `${exactPinPrefix}${to}`
        })
      ]
    ] : [];
  }));
  if (replacements.size === 0)
    return {
      text,
      changed: false,
      replacedLines: 0
    };
  return {
    text: joinManifest({
      bodies: bodies.map(function applyReplacement(
        body,
        index
      ) {
        return replacements.get(index) ?? body;
      }),
      hadReturn,
    }),
    changed: true,
    replacedLines: replacements.size,
  };
}
