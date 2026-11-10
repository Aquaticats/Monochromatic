/**
 Every function declaration and named function expression in this package's library sources states its return type.

 The repository style asks for it, and `oxlint`'s `typescript/explicit-function-return-type` rule reports a function
 declaration and a function expression assigned to a variable. It reports neither a named function expression passed
 as a call argument nor a declaration nested inside a function body, so an inferred callback such as
 `combinations.filter(function isNotExcluded(combination,) {` passes every lint task while its contract changes with
 the surrounding call (issue #1420). The scan below reads this package's own sources and fails on one.

 Comments are blanked before the scan, because the TSDoc examples in `types.ts` show callback-shaped code a reader must
 not mistake for a declaration the rule covers.

 @module
 */

import { stripSourceComments, } from '@monochromatic-dev/container-limit-core/ts/source-text-without-comments.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

/**
 Suffix of a source file the rule covers.
 */
const LIBRARY_SOURCE_SUFFIX = '.ts';

/**
 Suffixes that mark a test, benchmark or declaration file rather than library code; `oxlint` turns the return-type
 rule off for the first two through its own override, so the scan leaves them out as that override does.
 */
const NON_LIBRARY_SOURCE_SUFFIXES: readonly string[] = [
  '.bench.ts',
  '.d.ts',
  '.test.ts',
];

/**
 The keyword every declaration under the rule starts with.
 */
const FUNCTION_KEYWORD = 'function';

/**
 Characters an identifier may be built from, which every name this package declares draws from.
 */
const IDENTIFIER_CHARACTERS = '$0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz';

/**
 Characters that separate a keyword, a name and a parameter list without changing which declaration they belong to.
 */
const WHITESPACE_CHARACTERS = ' \t\n\r';

/**
 Returned instead of a declaration when a `function` keyword starts no named function, such as an anonymous function
 expression or a `typeof` comparison against the keyword itself.

 A unique `Symbol` is the genuine-sentinel form the no-nullish-union rule allows: it lets a lookup signal absence
 without a `NamedDeclaration | undefined` return type, and the caller tests identity against it.
 */
const NOT_A_NAMED_FUNCTION = Symbol('keyword starts no named function',);

/**
 One declaration whose return type is inferred rather than stated.
 */
type MissingReturnTypeSite = {
  /** 1-based line of the `function` keyword in the scanned text */
  readonly line: number;
  /** Name the source gives the function */
  readonly name: string;
};

/**
 Named declaration the source writes at one `function` keyword.
 */
type NamedDeclaration = {
  /** Index of the first character after the parameter list */
  readonly afterParametersIndex: number;
  /** Name the source gives the function */
  readonly name: string;
};

/**
 One library source file of this package, with the text the scan reads.
 */
type LibrarySource = {
  /** Absolute path of the file */
  readonly path: string;
  /** The file's text with its comments blanked */
  readonly text: string;
};

//region Reading the source text

/**
 Tells whether a character may appear in an identifier.

 The empty string stands for the position before the text's first character, which never extends an identifier.

 @param character - one character of the scanned text

 @returns whether the character extends an identifier
 */
function isIdentifierCharacter(character: string,): boolean {
  return (character !== '') && IDENTIFIER_CHARACTERS.includes(character,);
}

/**
 Gives the index of the first character at or after `from` that is not whitespace.

 @param text - scanned text

 @param from - index to start reading at

 @returns index of the first non-whitespace character, or the text's length when only whitespace follows
 */
function firstCharacterOutsideWhitespace({
  text,
  from,
}: {
  readonly text: string;
  readonly from: number;
},): number {
  for (let index = from; index < text.length; index += 1) {
    if (!WHITESPACE_CHARACTERS.includes(text[index] ?? '',))
      return index;
  }

  return text.length;
}

/**
 Gives the index just past the identifier that starts at `from`.

 @param text - scanned text

 @param from - index the identifier starts at

 @returns index of the first character that does not extend the identifier
 */
function endOfIdentifierAt({
  text,
  from,
}: {
  readonly text: string;
  readonly from: number;
},): number {
  for (let index = from; index < text.length; index += 1) {
    if (!isIdentifierCharacter(text[index] ?? '',))
      return index;
  }

  return text.length;
}

/**
 Gives the index of the parenthesis that closes the one at `openIndex`.

 Parentheses inside a string literal or a regular expression held in a parameter's default value would be counted as
 nesting; no declaration in this package carries one.

 @param text - source text whose parentheses are counted

 @param openIndex - index of the opening parenthesis

 @returns index of the matching closing parenthesis

 @throws Error when the text ends before the parenthesis closes
 */
function closingParenthesisIndex({
  text,
  openIndex,
}: {
  readonly text: string;
  readonly openIndex: number;
},): number {
  for (let depth = 1, index = openIndex + 1; index < text.length; index += 1) {
    if (text[index] === '(')
      depth += 1;
    else if (text[index] === ')') {
      depth -= 1;
      if (depth === 0)
        return index;
    }
  }

  throw new Error(`Unbalanced parentheses from index ${String(openIndex,)} on`,);
}

/**
 Gives the index just past the type parameter list that starts at `from`.

 An arrow function inside a type parameter list carries `=>`, whose `>` does not close the list.

 @param text - scanned text

 @param from - index of the opening angle bracket

 @returns index of the first character after the closing angle bracket

 @throws Error when the text ends before the list closes
 */
function endOfTypeParametersAt({
  text,
  from,
}: {
  readonly text: string;
  readonly from: number;
},): number {
  for (let depth = 1, index = from + 1; index < text.length; index += 1) {
    if ((text[index] === '=') && (text[index + 1] === '>')) {
      index += 1;
      continue;
    }
    if (text[index] === '<')
      depth += 1;
    else if (text[index] === '>') {
      depth -= 1;
      if (depth === 0)
        return index + 1;
    }
  }

  throw new Error(`Unbalanced angle brackets from index ${String(from,)} on`,);
}

/**
 Reads the named declaration the source writes at one `function` keyword.

 @param text - scanned text, comments already blanked

 @param keywordIndex - index of the `function` keyword

 @returns the declaration, or {@link NOT_A_NAMED_FUNCTION} when the keyword starts an anonymous function or no
 parameter list follows
 */
function namedDeclarationAt({
  text,
  keywordIndex,
}: {
  readonly text: string;
  readonly keywordIndex: number;
},): NamedDeclaration | typeof NOT_A_NAMED_FUNCTION {
  /**
   First character after the keyword and the whitespace that follows it; where a name would start.
   */
  const nameStart = firstCharacterOutsideWhitespace({
    text,
    from: keywordIndex + FUNCTION_KEYWORD.length,
  },);
  /**
   First character after the name.
   */
  const nameEnd = endOfIdentifierAt({
    text,
    from: nameStart,
  },);

  if (nameEnd === nameStart)
    return NOT_A_NAMED_FUNCTION;

  /**
   First character after the name and the whitespace that follows it; a type parameter list or the parameter list.
   */
  const afterName = firstCharacterOutsideWhitespace({
    text,
    from: nameEnd,
  },);
  /**
   Index of the opening parenthesis, past the type parameter list when the function is generic.
   */
  const openIndex = text[afterName] === '<'
    ? firstCharacterOutsideWhitespace({
      text,
      from: endOfTypeParametersAt({
        text,
        from: afterName,
      },),
    },)
    : afterName;

  if (text[openIndex] !== '(')
    return NOT_A_NAMED_FUNCTION;

  return {
    afterParametersIndex: closingParenthesisIndex({
      text,
      openIndex,
    },) + 1,
    name: text.slice(
      nameStart,
      nameEnd,
    ),
  };
}

//endregion Reading the source text

//region The scan

/**
 Lists the declarations in one source text whose return type is inferred.

 @param text - source text, comments already blanked

 @returns one entry per declaration that states no return type, in source order

 @example
 ```ts
 missingReturnTypeSites('[1].map(function doubled(value,) { return value * 2; },);');
 // [{ line: 1, name: 'doubled' }]
 ```
 */
function missingReturnTypeSites(text: string,): readonly MissingReturnTypeSite[] {
  /**
   Declarations found so far, in source order.
   */
  const sites: MissingReturnTypeSite[] = [];

  for (let index = 0, line = 1; index < text.length;) {
    if (text[index] === '\n') {
      line += 1;
      index += 1;
      continue;
    }
    if (!text.startsWith(FUNCTION_KEYWORD, index,)) {
      index += 1;
      continue;
    }
    /**
     Character before the keyword, empty at the text's start; an identifier character there makes this keyword the
     tail of a longer word.
     */
    const beforeKeyword = text[index - 1] ?? '';

    if (isIdentifierCharacter(beforeKeyword,)) {
      index += FUNCTION_KEYWORD.length;
      continue;
    }

    /**
     The declaration this keyword starts.
     */
    const declaration = namedDeclarationAt({
      text,
      keywordIndex: index,
    },);

    if (declaration !== NOT_A_NAMED_FUNCTION) {
      /**
       First character after the parameter list and the whitespace that follows it; a colon there begins the return
       type.
       */
      const afterParameters = firstCharacterOutsideWhitespace({
        text,
        from: declaration.afterParametersIndex,
      },);

      if (text[afterParameters] !== ':') {
        sites.push({
          line,
          name: declaration.name,
        },);
      }
    }

    index += FUNCTION_KEYWORD.length;
  }

  return sites;
}

/**
 Lists this package's library source files, each with the text the scan reads.

 @returns file paths and their comment-blanked text
 */
async function librarySources(): Promise<readonly LibrarySource[]> {
  /**
   Every file below this test's own directory, tests and declarations included so the filter below can drop them.
   */
  const entries = await readdir(
    import.meta.dirname,
    {
      recursive: true,
      withFileTypes: true,
    },
  );
  /**
   Paths of the library sources, in directory order.
   */
  const paths = entries
    .filter((entry,) => entry.isFile()
      && entry.name.endsWith(LIBRARY_SOURCE_SUFFIX,)
      && (!NON_LIBRARY_SOURCE_SUFFIXES.some((suffix,) => entry.name.endsWith(suffix,),)),)
    .map((entry,) => join(
      entry.parentPath,
      entry.name,
    ),);

  return await Promise.all(paths.map(async function readLibrarySource(path: string,): Promise<LibrarySource> {
    return {
      path,
      text: stripSourceComments({
        text: await readFile(
          path,
          'utf8',
        ),
        language: 'typescript',
      },),
    };
  },),);
}

//endregion The scan

await describe({
  name: 'function return type annotations',
  children: [
    it({
      name: 'reports a callback that states no return type',
      fn: async () => {
        expect(missingReturnTypeSites('[1,].map(function doubled(value: number,) {\n  return value * 2;\n},);',),)
          .toEqual([{ line: 1, name: 'doubled', },],);
      },
    },),

    it({
      name: 'reports a declaration nested in a function body',
      fn: async () => {
        expect(missingReturnTypeSites('function outer(): void {\n  function inner() {}\n}',),)
          .toEqual([{ line: 2, name: 'inner', },],);
      },
    },),

    it({
      name: 'accepts a callback that states its return type',
      fn: async () => {
        expect(missingReturnTypeSites('[1,].map(function doubled(value: number,): number {\n  return value * 2;\n},);',),)
          .toEqual([],);
      },
    },),

    it({
      name: 'accepts a generic declaration that states its return type',
      fn: async () => {
        expect(missingReturnTypeSites('function testMatcher<T,>({ value }: { value: T; },): boolean { return true; }',),)
          .toEqual([],);
      },
    },),

    it({
      name: 'ignores a callback shape shown inside a TSDoc example',
      fn: async () => {
        /**
         An example-only callback, as `types.ts` writes one inside a fenced code block.
         */
        const example = '/**\n ```ts\n { user: function notRoot(user) { return user !== \'root\'; } }\n ```\n */\n';
        /**
         The example as the scan reads it: comments blanked, so nothing is left to report.
         */
        const scannedText = stripSourceComments({
          text: example,
          language: 'typescript',
        },);

        expect(missingReturnTypeSites(scannedText,),)
          .toEqual([],);
      },
    },),

    it({
      name: 'states the return type of every function in this package\'s library sources',
      fn: async () => {
        /**
         Library sources this scan reads.
         */
        const sources = await librarySources();

        expect(sources.length,).toBeGreaterThan(0,);

        /**
         Every declaration that states no return type, named by file and line.
         */
        const reported = sources.flatMap(({ path, text, },) => missingReturnTypeSites(text,)
          .map((site,) => `${path}:${String(site.line,)}: ${site.name}`,),);

        expect(reported,).toEqual([],);
      },
    },),
  ],
},);
