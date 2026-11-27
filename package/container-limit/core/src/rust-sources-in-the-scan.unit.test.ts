/**
 The answers the scan must give for the Rust fixture sources beside this file.

 A Rust file the ledger meets is one that names `podman` or `docker`, so the ledger lists it and its role says what it
 does with the tool. The answer a role rests on is the starts the scan reports for the file, and each fixture here states
 one: a start with its verb, an image removal that starts nothing, a mention in a comment, and a mention in a message.

 The fixtures are a folder the ledger's own scan skips (`rust-source-fixtures`), because a fixture is not a program of
 this repository: they are read here alone, and a green here says the scan answers the same way for them as for the files
 the ledger lists.
 */

import { readFileSync, } from 'node:fs';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ContainerStartVerb, } from './container-start-vocabulary.ts';
import { containerStartInvocations, } from './container-starts-in-source-text.ts';
import {
  SOURCE_LANGUAGE_BY_EXTENSION,
  type SourceLanguage,
} from './source-text-without-comments.ts';

/**
 Folder the fixtures sit in, beside this file.
 */
const FIXTURE_FOLDER = join(
  import.meta.dirname,
  'rust-source-fixtures',
);

/**
 One fixture, the reason its answer is what it is, and the verbs of the starts the scan must report for it.
 */
type FixtureAnswer = {
  /**
   File name of the fixture, inside `rust-source-fixtures`.
   */
  readonly fixture: string;
  /**
   What the fixture writes, which is why the answer is this one.
   */
  readonly why: string;
  /**
   Verb of each start the scan must report, in the order the file writes them; none for a file that starts nothing.
   */
  readonly verbs: readonly ContainerStartVerb[];
};

/**
 Every fixture, with the answer the scan must give for it.
 */
const ANSWERS: readonly FixtureAnswer[] = [
  {
    fixture: 'starts-a-container.rs',
    why: 'names the program in `Command::new` and passes the verb as the argument array\'s first element',
    verbs: ['run',],
  },
  {
    fixture: 'starts-a-container-one-argument-at-a-time.rs',
    why: 'names the program in `Command::new` and passes the verb in the first of its `arg` calls',
    verbs: ['run',],
  },
  {
    fixture: 'removes-an-image-only.rs',
    why: 'runs the tool with `rmi`, which removes an image and starts nothing',
    verbs: [],
  },
  {
    fixture: 'mentions-the-tool-in-prose-only.rs',
    why: 'writes the start shape in a line comment, a block comment, a block doc comment and a nested block comment',
    verbs: [],
  },
  {
    fixture: 'words-the-tool-in-a-message-only.rs',
    why: 'writes `podman run` in a message and in a note, where the words are text and not a start',
    verbs: [],
  },
];

/**
 The comment syntax the scan blanked out of a Rust source, which a Rust fixture is.

 @returns the language row the scan holds for `.rs`

 @throws When the scan holds no language for `.rs`, which is the widening this file's fixtures ask for

 @example
 ```ts
 const language = rustLanguage();
 ```
 */
function rustLanguage(): SourceLanguage {
  /**
   The language the scan reads for a `.rs` path.
   */
  const language = SOURCE_LANGUAGE_BY_EXTENSION.get(
    '.rs',
  );
  if (language === undefined)
    throw new Error(
      'the scan holds no comment syntax for `.rs`, so a Rust source is read as no file at all',
    );
  return language;
}

/**
 Text of one fixture.

 @param fixture - file name inside `rust-source-fixtures`

 @returns the file's text

 @example
 ```ts
 const text = fixtureText('starts-a-container.rs');
 ```
 */
function fixtureText(fixture: string,): string {
  return readFileSync(
    join(
      FIXTURE_FOLDER,
      fixture,
    ),
    'utf8',
  );
}

/**
 What the scan reports for a start whose arguments a call builds rather than a verb written at the start itself.
 */
const ARGUMENTS_FROM_A_CALL = 'no verb: the arguments come from a call';

/**
 The verbs of the starts the scan reports for one fixture.

 @param fixture - file name inside `rust-source-fixtures`

 @returns one verb per start, in the order the file writes them

 @example
 ```ts
 const verbs = reportedVerbs('starts-a-container.rs');
 ```
 */
function reportedVerbs(fixture: string,): readonly string[] {
  return containerStartInvocations({
    text: fixtureText(fixture,),
    language: rustLanguage(),
  },)
    .map((start,) => start.verb ?? ARGUMENTS_FROM_A_CALL,);
}

await describe({
  name: 'the scan of Rust sources',
  children: [
    it({
      name: 'reads a `.rs` file as a Rust source',
      fn: async () => {
        expect(rustLanguage(),).toBe('rust',);
      },
    },),
    ...ANSWERS.map(function answersOfOneFixture({ fixture, why, verbs, }: FixtureAnswer) {
      return it({
        name: `reports ${String(verbs.length,)} start(s) for ${fixture}, which ${why}`,
        fn: async () => {
          expect(fixtureText(fixture,),).toContain('podman',);
          expect(reportedVerbs(fixture,),).toEqual(verbs,);
        },
      },);
    },),
  ],
},);
