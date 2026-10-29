/**
 Tests for reading an entry's `info.yml` at the pin.

 Every case asserts the whole observable value: the parsed record, the absence,
 the rethrown failure, or the one log line. Fixtures are cat-themed invention;
 no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { parse as parseYaml, } from 'yaml';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import { CorpusReadError, } from '@monochromatic-dev/translation-repair-corpus-source';
import {
  entryInfoFindingsOf,
  readEntryInfoRecord,
} from '../../dist/final/node/index.mjs';
import {
  capturingLogger,
  levelCapturingLogger,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';

/**
 Pin the cases read at; the reader never touches the clone itself, since the test reader answers.

 @example
 ```ts
 const pin = { cloneDir: '/nonexistent', commitSha: 'deadbeef', };
 ```
 */
const PIN = {
  cloneDir: '/nonexistent',
  commitSha: 'deadbeef',
};

/**
 Reader of a pin whose commit has no `info.yml` for the entry.

 @returns Never; raises the corpus reader's refusal for a missing object

 @throws {@link CorpusReadError} always, the way the corpus reader refuses a path the commit lacks

 @example
 ```ts
 await missingInfoFixture();
 ```
 */
async function missingInfoFixture(): Promise<string> {
  throw new CorpusReadError({
    detail: 'people/tabby/info.yml',
    cause: { stderr: 'fatal: path \'people/tabby/info.yml\' does not exist in \'deadbeef\'', },
    commit: 'held',
  },);
}

/**
 What the YAML parser says of a text it refuses, which is what a reader forwarding it would repeat.

 @param text - text handed to the parser

 @returns Parser's refusal as text, empty when it parses the text

 @example
 ```ts
 const said = parserRefusalOf({ text: 'info: [\n', },);
 ```
 */
function parserRefusalOf({ text, }: { readonly text: string; },): string {
  try {
    parseYaml(text,);
  }
  catch (error) {
    return String(error,);
  }
  return '';
}

//region Entry info record tests

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readEntryInfoRecord.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS an entry info.yml at the pin and parses its nested info record',
          fn: async () => {
            /**
             Log lines the reader wrote, none on this path.
             */
            const messages: string[] = [];
            expect(await readEntryInfoRecord({
              pin: PIN,
              entryId: 'mittens',
              readFile: async function readFixture() {
                return 'info:\n  born: \'2019-05-02\'\n  died: \'2025-02-26\'\n';
              },
              l: capturingLogger({ messages, },),
            },),).toEqual({
              kind: 'read',
              info: {
                info: {
                  born: '2019-05-02',
                  died: '2025-02-26',
                },
              },
            },);
            expect(messages,).toEqual([],);
          },
        },),

        it({
          name: 'READS AN UNQUOTED DATE as the text it is written as, so a death written without quotes still sets lifeEnded on the prepared pair',
          fn: async () => {
            /**
             The record an info.yml with an unquoted date parses to.
             */
            const record = await readEntryInfoRecord({
              pin: PIN,
              entryId: 'mittens',
              readFile: async function readUnquoted() {
                return 'info:\n  born: 2019-05-02\n  died: 2025-02-26\n';
              },
              l: capturingLogger({ messages: [], },),
            },);
            expect(record,).toEqual({
              kind: 'read',
              info: {
                info: {
                  born: '2019-05-02',
                  died: '2025-02-26',
                },
              },
            },);
            if (record.kind !== 'read')
              throw new Error('the unquoted info.yml was recorded as absent',);
            expect(prepareDocumentPair({
              sourceText: '猫睡觉。\n',
              targetText: 'The cat naps.\n',
              sourceEntryInfo: record.info,
            },).lifeEnded,).toBe(true,);
          },
        },),

        it({
          name: 'READS A BARE YEAR as the number YAML makes of it, and a prepared pair built from it still records the death',
          fn: async () => {
            /**
             The record an info.yml with a bare year parses to.
             */
            const record = await readEntryInfoRecord({
              pin: PIN,
              entryId: 'mittens',
              readFile: async function readBareYear() {
                return 'info:\n  born: 2019\n  died: 2025\n';
              },
              l: capturingLogger({ messages: [], },),
            },);
            expect(record,).toEqual({
              kind: 'read',
              info: {
                info: {
                  born: 2_019,
                  died: 2_025,
                },
              },
            },);
            if (record.kind !== 'read')
              throw new Error('the bare-year info.yml was recorded as absent',);
            /**
             The pair prepared from that record.
             */
            const prepared = prepareDocumentPair({
              sourceText: '猫睡觉。\n',
              targetText: 'The cat naps.\n',
              sourceEntryInfo: record.info,
            },);
            expect(prepared.lifeEnded,).toBe(true,);
            expect(prepared.identityContext,).toBe('- died: the date this person died, as the corpus records it: "2025".',);
          },
        },),

        it({
          name: 'RECORDS an absence, and logs it, when the entry has no info.yml at the pin',
          fn: async () => {
            /**
             Log lines the reader wrote for the absence.
             */
            const messages: string[] = [];
            expect(await readEntryInfoRecord({
              pin: PIN,
              entryId: 'tabby',
              readFile: missingInfoFixture,
              l: capturingLogger({ messages, },),
            },),).toEqual({ kind: 'absent', },);
            expect(messages.map(function startsWithEntry(line,): boolean {
              return line.startsWith('ENTRY INFO entry=tabby: no info.yml at the pin',);
            },),).toEqual([true,],);
          },
        },),

        it({
          name: 'RECORDS an absence in silence when no logger is given, as a command with pinned terminal output reads it',
          fn: async () => {
            expect(await readEntryInfoRecord({
              pin: PIN,
              entryId: 'tabby',
              readFile: missingInfoFixture,
            },),).toEqual({ kind: 'absent', },);
          },
        },),

        it({
          name: 'REFUSES any other read failure by rethrowing the same error, which is no absence',
          fn: async () => {
            /**
             The failure the reader is given.
             */
            const failure = new CorpusReadError({
              detail: 'people/tabby/info.yml',
              cause: { stderr: 'fatal: unable to read tree', },
              commit: 'held',
            },);
            /**
             What the reader threw, if anything.
             */
            let caught: unknown = undefined;
            try {
              await readEntryInfoRecord({
                pin: PIN,
                entryId: 'tabby',
                readFile: async function brokenFixture() {
                  throw failure;
                },
                l: capturingLogger({ messages: [], },),
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught === failure,).toBe(true,);
          },
        },),

        it({
          name: 'RECORDS AN info.yml THAT IS NOT YAML as unreadable, carrying the refusal that names the entry and where the '
            + 'parser stopped, and warns that no death is recorded for it, so a run ships the entry (the owner\'s rule)',
          fn: async () => {
            /**
             Lines the reader wrote at their levels.
             */
            const lines: string[] = [];
            /**
             What the reader answered for the damaged file.
             */
            const record = await readEntryInfoRecord({
              pin: PIN,
              entryId: 'mittens',
              readFile: async function unreadableFixture() {
                return 'info: [\n';
              },
              l: levelCapturingLogger({ lines, },),
            },);
            /**
             Sentence the refusal carries.
             */
            const refusal = 'The info.yml of entry mittens at the pin refused to parse at line 2 column 1 (BAD_INDENT); '
              + 'corpus metadata parses upstream, so this signals corruption.';
            expect(record,).toEqual({
              kind: 'unreadable',
              refusal,
            },);
            expect(lines,).toEqual([
              `warn ENTRY INFO entry=mittens: ${refusal} No death is recorded for the entry, and it goes on as one whose `
                + 'info.yml records none',
            ],);
          },
        },),

        it({
          name: 'QUOTES NONE OF A DAMAGED info.yml in the record or in the warning, since the parser\'s own message embeds a '
            + 'code frame of the file, which holds a person\'s dates and handles',
          fn: async () => {
            /**
             File whose failing line holds a handle.
             */
            const damaged = 'info:\n  born: [whiskers-secret\n';
            /**
             What the parser says of it, which is what the reader must not repeat.
             */
            const parserSaid = parserRefusalOf({ text: damaged, },);
            // POSITIVE CONTROL: the parser's own message does quote the handle, so a reader that forwarded it would show.
            expect(parserSaid.includes('whiskers-secret',),).toBe(true,);
            /**
             Lines the reader wrote at their levels.
             */
            const lines: string[] = [];
            /**
             What the reader answered for the damaged file.
             */
            const record = await readEntryInfoRecord({
              pin: PIN,
              entryId: 'mittens',
              readFile: async function unreadableWithHandle() {
                return damaged;
              },
              l: levelCapturingLogger({ lines, },),
            },);
            expect(record.kind,).toBe('unreadable',);
            expect(JSON.stringify([record, lines,],).includes('whiskers-secret',),).toBe(false,);
            expect(lines.length,).toBe(1,);
          },
        },),

        it({
          name: 'RECORDS AN info.yml THAT IS NOT YAML as unreadable in silence when no logger is given, as a command with '
            + 'pinned terminal output reads it',
          fn: async () => {
            expect(await readEntryInfoRecord({
              pin: PIN,
              entryId: 'mittens',
              readFile: async function unreadableFixture() {
                return 'info: [\n';
              },
            },),).toEqual({
              kind: 'unreadable',
              refusal: 'The info.yml of entry mittens at the pin refused to parse at line 2 column 1 (BAD_INDENT); '
                + 'corpus metadata parses upstream, so this signals corruption.',
            },);
          },
        },),

        it({
          name: 'WARNS, naming the entry, the field and the kind, of a death YAML reads as a truth value, a list, a record or '
            + 'a number that is not finite, and still records the file as read',
          fn: async () => {
            await Promise.all([
              [
                'true',
                'a truth value',
              ],
              [
                '[2025, 2026]',
                'a list',
              ],
              [
                '{ year: 2025 }',
                'a record',
              ],
              [
                '.nan',
                'a number that is not finite',
              ],
              [
                '-.inf',
                'a number that is not finite',
              ],
            ].map(async function readUnreadableKind([written, kind,],): Promise<void> {
              /**
               Lines the reader wrote at their levels.
               */
              const lines: string[] = [];
              /**
               What the reader answered for this kind.
               */
              const record = await readEntryInfoRecord({
                pin: PIN,
                entryId: 'mittens',
                readFile: async function readKind() {
                  return `info:\n  died: ${written}\n`;
                },
                l: levelCapturingLogger({ lines, },),
              },);
              expect(record.kind,).toBe('read',);
              expect(lines,).toEqual([
                `warn ENTRY INFO entry=mittens: info.died is ${kind}, which is neither text nor a finite number, `
                  + 'so no death is recorded for it',
              ],);
            },),);
          },
        },),

        it({
          name: 'SAYS NOTHING of a death that is text, a finite number, a blank value or absent, since each is a record or an '
            + 'absence and no unreadable kind',
          fn: async () => {
            await Promise.all([
              '\'2025-02-26\'',
              '2025',
              'null',
              '\'\'',
            ].map(async function readQuietKind(written,): Promise<void> {
              /**
               Lines the reader wrote at their levels.
               */
              const lines: string[] = [];
              await readEntryInfoRecord({
                pin: PIN,
                entryId: 'mittens',
                readFile: async function readKind() {
                  return `info:\n  died: ${written}\n`;
                },
                l: levelCapturingLogger({ lines, },),
              },);
              expect(lines,).toEqual([],);
            },),);
            /**
             Lines the reader wrote for an info record that holds no death at all.
             */
            const absentLines: string[] = [];
            await readEntryInfoRecord({
              pin: PIN,
              entryId: 'mittens',
              readFile: async function readNoDeath() {
                return 'info:\n  born: \'2019-05-02\'\n';
              },
              l: levelCapturingLogger({ lines: absentLines, },),
            },);
            expect(absentLines,).toEqual([],);
          },
        },),

        it({
          name: 'READS an unreadable death in silence when no logger is given, as a command with pinned terminal output reads it',
          fn: async () => {
            expect(await readEntryInfoRecord({
              pin: PIN,
              entryId: 'mittens',
              readFile: async function readTruthValue() {
                return 'info:\n  died: true\n';
              },
            },),).toEqual({
              kind: 'read',
              info: { info: { died: true, }, },
            },);
          },
        },),
      ],
    },),
    describe({
      name: entryInfoFindingsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SAYS THE RECORD WAS UNREADABLE and that no death is recorded for the entry, in the refusal\'s own words, so '
            + 'a reader of the artifact can tell it from an entry that records no death',
          fn: async () => {
            expect(entryInfoFindingsOf({
              entryInfo: {
                kind: 'unreadable',
                refusal: 'The info.yml of entry mittens at the pin refused to parse at line 2 column 1 (BAD_INDENT); '
                  + 'corpus metadata parses upstream, so this signals corruption.',
              },
            },),).toEqual([
              'entry info: The info.yml of entry mittens at the pin refused to parse at line 2 column 1 (BAD_INDENT); '
                + 'corpus metadata parses upstream, so this signals corruption. No death is recorded for the entry.',
            ],);
          },
        },),

        it({
          name: 'SAYS THE KEY AND THE KIND of a death that is a truth value, a list, a record or a number that is not finite, '
            + 'quoting no value, so a reader of the artifact can tell it from an entry that records no death',
          fn: async () => {
            /**
             Each `died` the feature cannot read, with the kind the finding names.
             */
            const unreadableKinds: readonly (readonly [unknown, string,])[] = [
              [
                true,
                'a truth value',
              ],
              [
                ['whiskers-secret', 2_026,],
                'a list',
              ],
              [
                { handle: 'whiskers-secret', },
                'a record',
              ],
              [
                Number.NaN,
                'a number that is not finite',
              ],
              [
                Number.NEGATIVE_INFINITY,
                'a number that is not finite',
              ],
            ];
            await Promise.all(unreadableKinds.map(async function findUnreadableKind([died, kind,],): Promise<void> {
              expect(entryInfoFindingsOf({
                entryInfo: {
                  kind: 'read',
                  info: { info: { died, }, },
                },
              },),).toEqual([
                `entry info: info.died is ${kind}, which is neither text nor a finite number. No death is recorded for the entry.`,
              ],);
            },),);
          },
        },),

        it({
          name: 'SAYS NOTHING OF A DEATH THAT IS TEXT, A FINITE NUMBER, A BLANK VALUE OR ABSENT, which each record a death or '
            + 'an entry with none and no unreadable kind',
          fn: async () => {
            await Promise.all([
              { info: { died: '2025-02-26', }, },
              { info: { died: 2_025, }, },
              { info: { died: null, }, },
              { info: { died: '', }, },
              { info: { born: '2019-05-02', }, },
              { died: true, },
              'cats',
            ].map(async function findQuietKind(info,): Promise<void> {
              expect(entryInfoFindingsOf({ entryInfo: { kind: 'read', info, }, },),).toEqual([],);
            },),);
          },
        },),

        it({
          name: 'SAYS NOTHING OF AN ABSENT FILE or of a parsed one, which each record what they hold and no unreadable record',
          fn: async () => {
            expect(entryInfoFindingsOf({ entryInfo: { kind: 'absent', }, },),).toEqual([],);
            expect(entryInfoFindingsOf({
              entryInfo: {
                kind: 'read',
                info: { info: { died: '2025-02-26', }, },
              },
            },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);

//endregion Entry info record tests
