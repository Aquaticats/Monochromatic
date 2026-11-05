/**
 Guard: the words the end-to-end suite and the benchmark start the service with equal the words the executable dispatches on
 (`SERVICE_WORDS` in `src/native/service/service_main.rs`), so a rename in one place cannot leave the other starting nothing.
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
 The current spelling of the command word, read from its declaration (`COMMAND_WORD`).

 @param source - the Rust source text of the names

 @returns the current command word

 @throws Error when the declaration is not found
 */
function currentCommandWord(source: string): string {
  const match = (/pub const COMMAND_WORD: RenamedName = RenamedName \{\s*current: "([^"]+)"/u).exec(source,);
  if (match?.[1] === undefined)
    throw new Error('wrapper_names_old_and_new.rs no longer declares COMMAND_WORD with a current spelling');
  return match[1];
}

/**
 The quoted words of the `SERVICE_WORDS` declaration, read from the line that starts it.

 @param source - the Rust source text

 @returns the words, in order

 @throws Error when the declaration is not found
 */
function rustWords(source: string): readonly string[] {
  const marker = 'pub const SERVICE_WORDS: [&str; 3] = [';
  const start = source.indexOf(marker);
  if (start === (-1))
    throw new Error('service_main.rs no longer declares SERVICE_WORDS as three words');
  const list = source.slice(
    start + marker.length,
    source.indexOf(
      '];',
      start,
    ),
  );
  return list.split(',').map((word) => word.trim().replaceAll(
    '"',
    '',
  )).filter((word) => word !== '').map((word) => word === 'COMMAND_WORD.current' ? currentCommandWord(namesSource,) : word);
}

await describe({
  name: 'the service mode words',
  children: [
    it({
      name: 'the TypeScript launchers use the words the Rust dispatch uses',
      fn: async () => {
        expect([...SERVICE_MODE_WORDS,]).toEqual([...rustWords(rustSource),]);
      },
    }),
    it({
      name: 'a changed Rust word is a difference',
      fn: async () => {
        expect([...SERVICE_MODE_WORDS,]).not.toEqual([...rustWords(rustSource.replace(
          '"run"',
          '"serve"',
        )),]);
      },
    }),
  ],
});
