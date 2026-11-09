/**
 Which module each name a file imports comes from, so a builder an adjacent module exports can be read there.

 A start can spread a builder the module beside the file exports (`...buildBounds`), and following that name to where
 the arguments are written needs the module its import names.
 */

import {
  endOfQuotedString,
  isNameCharacter,
  isQuote,
  nameMatchesAt,
  statementEnd,
} from './positions-in-source-text.ts';
import {
  stripSourceComments,
  type SourceLanguage,
} from './source-text-without-comments.ts';

/**
 The module specifier an import clause's `from` names, when the clause names one.

 @param characters - the file's characters
 
 @param from - first character of the clause, past the `import` keyword
 
 @param to - index past the statement's semicolon

 @returns the specifier, and nothing when the clause names no module

 @example
 ```ts
 const { specifier } = moduleSpecifierOf({ characters, from: 6, to: 40 });
 ```
 */
function moduleSpecifierOf({
  characters,
  from,
  to,
}: {
  readonly characters: readonly string[];
  readonly from: number;
  readonly to: number;
},): { readonly specifier?: string; } {
  for (let index = from; index < to;) {
    if (!nameMatchesAt({
      characters,
      index,
      name: 'from',
    },)) {
      index += 1;
      continue;
    }
    /**
     Index of the quote that opens the module specifier.
     */
    let quote = index + 'from'.length;
    while ((quote < to) && (!isQuote(characters[quote],)))
      quote += 1;
    if (quote >= to)
      return {};
    /**
     Index past the quote that closes the module specifier.
     */
    const quoteEnd = endOfQuotedString({
      characters,
      quote,
      to,
    },);
    return { specifier: characters.slice(
      quote + 1,
      quoteEnd - 1,
    )
      .join('',), };
  }
  return {};
}

/**
 The local names an import clause binds.

 @param characters - the file's characters
 
 @param from - first character of the clause, past the `import` keyword
 
 @param to - index past the statement's semicolon
 
 @param specifier - the module the clause imports from

 @returns the names bound by this import, each mapped to the module specifier

 @example
 ```ts
 const bound = boundNamesOf({ characters, from: 6, to: 40, specifier: './x.ts' });
 ```
 */
function boundNamesOf({
  characters,
  from,
  to,
  specifier,
}: {
  readonly characters: readonly string[];
  readonly from: number;
  readonly to: number;
  readonly specifier: string;
},): ReadonlyMap<string, string> {
  /**
   The names this import binds.
   */
  const bound = new Map<string, string>();
  for (let index = from; index < to;) {
    if (!isNameCharacter(characters[index],)) {
      index += 1;
      continue;
    }
    /**
     Index past the word that starts at the scan's index.
     */
    let wordEnd = index;
    while ((wordEnd < to) && isNameCharacter(characters[wordEnd],))
      wordEnd += 1;
    /**
     The word itself, which binds a name unless it is a clause keyword.
     */
    const word = characters.slice(
      index,
      wordEnd,
    )
      .join('',);
    if ((word !== 'type') && (word !== 'as')
      && (word !== 'from'))
      bound.set(
        word,
        specifier,
      );
    index = wordEnd;
  }
  return bound;
}

/**
 Each name a text imports, with the module its import names.

 A start can spread a builder an adjacent module exports (`...buildBounds`), so following a name to where the arguments
 are written needs the module that name comes from. Package specifiers come back as they are, and the reader decides
 which of them this repository's own files answer.

 @param text - the file's text
 
 @param language - the language whose comment syntax the text holds

 @returns one entry per imported name, mapped to the specifier the import names

 @example
 ```ts
 const modules = importedModuleByName({ text, language: 'typescript' });
 ```
 */
export function importedModuleByName({
  text,
  language,
}: {
  readonly text: string;
  readonly language: SourceLanguage;
},): ReadonlyMap<string, string> {
  /**
   The text with its comments blanked, so a commented-out import binds nothing.
   */
  const characters = stripSourceComments({
    text,
    language,
  },)
    .split('',);
  /**
   Each imported name, mapped to the module its import names.
   */
  const modules = new Map<string, string>();
  for (let index = 0; index < characters.length;) {
    if (!nameMatchesAt({
      characters,
      index,
      name: 'import',
    },)) {
      index += 1;
      continue;
    }
    /**
     Index past the end of the import statement.
     */
    const end = statementEnd({
      characters,
      from: index,
    },);
    /**
     The module specifier this import names, when it names one.
     */
    const { specifier, } = moduleSpecifierOf({
      characters,
      from: index + 'import'.length,
      to: end,
    },);
    if (specifier !== undefined) {
      for (const [name, module,] of boundNamesOf({
        characters,
        from: index + 'import'.length,
        to: end,
        specifier,
      },))
        modules.set(
          name,
          module,
        );
    }
    index = end;
  }
  return modules;
}
