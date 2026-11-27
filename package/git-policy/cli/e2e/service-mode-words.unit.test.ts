/**
 Guard: the words the end-to-end suite and the benchmark start the service with equal the words the executable dispatches on
 (`SERVICE_WORDS` in `src/native/service/service_main.rs`), read from the declaration as it is written, so a rename in one
 place cannot leave the other starting nothing.
 */
import { readFileSync, } from 'node:fs';
import { resolve, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { SERVICE_MODE_WORDS, } from './service-launch-fixture.ts';

/**
 The Rust source that declares the words.
 */
const rustSource = readFileSync(
  resolve(
    import.meta.dirname,
    '../src/native/service/service_main.rs',
  ),
  'utf8',
);

/**
 The Rust source that declares the pair of names the first word comes from.
 */
const namesSource = readFileSync(
  resolve(
    import.meta.dirname,
    '../src/native/foundation/wrapper_names_old_and_new.rs',
  ),
  'utf8',
);

/**
 The declaration the words are read from, found by the name it declares rather than by its type.
 */
const SERVICE_WORDS_DECLARATION = 'pub const SERVICE_WORDS: ';

/**
 How the declaration's word list opens as this tree writes it: a slice of shared string references, `&[&str]`,
 whose list follows `= &[`.
 */
const SERVICE_WORDS_LIST_OPENS = ' = &[';

/**
 How the declaration's word list closes.
 */
const SERVICE_WORDS_LIST_CLOSES = '];';

/**
 The quote that stands on each side of a word in the declaration's list.
 */
const WORD_QUOTE = '"';

/**
 The current spelling of the command word, read from its declaration (`COMMAND_WORD`).

 @param source - the Rust source text of the names

 @returns the current command word

 @throws Error when the declaration is not found
 */
function currentCommandWord(source: string): string {
  const declaration = 'pub const COMMAND_WORD: RenamedName = RenamedName {';
  const declared = source.indexOf(declaration,);
  const keyword = 'current: "';
  const keywordAt = source.indexOf(
    keyword,
    declared,
  );
  if ((declared === (-1)) || (keywordAt === (-1)))
    throw new Error('wrapper_names_old_and_new.rs no longer declares COMMAND_WORD with a current spelling');
  const wordStart = keywordAt + keyword.length;
  return source.slice(
    wordStart,
    source.indexOf(
      '"',
      wordStart,
    ),
  );
}

/**
 The word one element of the word list stands for.

 @param element - one element of the list, whitespace trimmed
 @param names - the Rust source text of the names

 @returns the word the element stands for

 @throws Error when the element is neither a quoted word nor the command word's own name
 */
function wordOfElement(
  { element, names }: { readonly element: string; readonly names: string; },
): string {
  if (element === 'COMMAND_WORD.current')
    return currentCommandWord(names,);
  const opensAt = element.indexOf(WORD_QUOTE,);
  const closesAt = element.lastIndexOf(WORD_QUOTE,);
  const quoted = (opensAt === 0) && (closesAt === (element.length - WORD_QUOTE.length));
  const word = quoted ? element.slice(
    WORD_QUOTE.length,
    closesAt,
  ) : '';
  if ((word === '') || word.includes(WORD_QUOTE) || word.includes('\\'))
    throw new Error(`service_main.rs declares a SERVICE_WORDS element this guard does not read: ${element}`);
  return word;
}

/**
 The words of the `SERVICE_WORDS` declaration, read from the declaration as this tree writes it: the word list it opens
 with `= &[`, up to `];`, each element a quoted word or the command word's own name.

 @param source - the Rust source text of the service mode's module
 @param names - the Rust source text of the names

 @returns the words, in order

 @throws Error when the declaration's word list is not the shape this guard reads, or an element is neither a quoted
 word nor the command word's own name
 */
function rustWords(
  { source, names }: { readonly source: string; readonly names: string; },
): readonly string[] {
  const declared = source.indexOf(SERVICE_WORDS_DECLARATION,);
  const listOpensAt = declared === (-1) ? (-1) : source.indexOf(
    SERVICE_WORDS_LIST_OPENS,
    declared,
  );
  const listClosesAt = listOpensAt === (-1) ? (-1) : source.indexOf(
    SERVICE_WORDS_LIST_CLOSES,
    listOpensAt,
  );
  if ((listOpensAt === (-1)) || (listClosesAt === (-1)))
    throw new Error(`service_main.rs no longer declares SERVICE_WORDS as ${SERVICE_WORDS_LIST_OPENS}...${SERVICE_WORDS_LIST_CLOSES}`);
  const list = source.slice(
    listOpensAt + SERVICE_WORDS_LIST_OPENS.length,
    listClosesAt,
  );
  return list
    .split(',')
    .map((element,) => element.trim())
    .filter((element,) => element !== '')
    .map((element,) => wordOfElement({
      element,
      names,
    },));
}

await describe({
  name: 'the service mode words',
  children: [
    it({
      name: 'the TypeScript launchers use the words the Rust dispatch uses',
      fn: async () => {
        expect([...SERVICE_MODE_WORDS,]).toEqual([...rustWords({
          source: rustSource,
          names: namesSource,
        },),]);
      },
    }),
    it({
      name: 'a changed Rust word is a difference',
      fn: async () => {
        expect([...SERVICE_MODE_WORDS,]).not.toEqual([...rustWords({
          source: rustSource.replace(
            '"run"',
            '"serve"',
          ),
          names: namesSource,
        },),]);
      },
    }),
    it({
      name: 'an element the guard does not read is refused, never guessed',
      fn: async () => {
        expect(() => rustWords({
          source: rustSource.replace(
            '"service"',
            'SERVICE_NOUN',
          ),
          names: namesSource,
        },),).toThrow('does not read');
      },
    }),
  ],
});
