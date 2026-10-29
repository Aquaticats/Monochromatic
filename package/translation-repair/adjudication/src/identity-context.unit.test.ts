/**
 Tests for the identity context: declared names, the lines built from them,
 and the pronoun the original uses for its subject.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  collectIdentityLines,
  extractDeclaredIdentity,
  sourcePronounLines,
  unreadableDeathKindOf,
} from '@monochromatic-dev/translation-repair-house-rule';
import {
  buildCriticMessages,
  prepareDocumentPair,
} from '../dist/final/node/index.mjs';

/**
 Clause the pronoun line carries where the original writes more than one
 form (class eighty-one, one entry's slice 14, 2026-09-22).
 */
const AS_WRITTEN = 'each form is rendered as written where it stands (她 is she, 他 is he, TA is singular they), and';

//region Identity context tests
// Fixtures are cat-themed inventions, never real corpus entries, but they
// mirror the exact shapes the pinned corpus uses: a top-level `name` with
// `alias` and `location` nested under `info`.

await describe({
  name: '',
  children: [
    describe({
      name: extractDeclaredIdentity.name,
      children: [
        it({
          name: 'reads name from the top level and alias and location from info',
          fn: async () => {
            /**
             Declaration in the shape the pinned corpus uses.
             */
            const identity = extractDeclaredIdentity({
              data: {
                name: 'Mittens Pawsworth',
                info: {
                  alias: 'Mittens, MTNS',
                  location: 'Catbury',
                },
              },
            },);

            expect(identity,).toEqual({
              name: 'Mittens Pawsworth',
              alias: 'Mittens, MTNS',
              location: 'Catbury',
            },);
          },
        },),

        it({
          name: 'returns an empty identity for non-record front matter',
          fn: async () => {
            expect(extractDeclaredIdentity({ data: null, },),).toEqual({},);
            expect(extractDeclaredIdentity({ data: 'whiskers', },),).toEqual({},);
            expect(extractDeclaredIdentity({ data: undefined, },),).toEqual({},);
          },
        },),

        it({
          name: 'skips non-string and blank declarations rather than coercing them',
          fn: async () => {
            /**
             A coerced value would enter the prompt as an authoritative
             correspondence, which is worse than declaring nothing.
             */
            const identity = extractDeclaredIdentity({
              data: {
                name: 42,
                info: {
                  alias: '   ',
                  location: [ 'Catbury', ],
                },
              },
            },);

            expect(identity,).toEqual({},);
          },
        },),

        it({
          name: 'tolerates info missing entirely',
          fn: async () => {
            expect(
              extractDeclaredIdentity({ data: { name: 'Mittens', }, },),
            ).toEqual({ name: 'Mittens', },);
          },
        },),
      ],
    },),

    describe({
      name: collectIdentityLines.name,
      children: [
        it({
          name: 'renders both sides of a transliterated name, the graded false-positive shape',
          fn: async () => {
            /**
             Mirrors a graded entry shape: the two sides declare
             names matching neither phonetically nor semantically.
             */
            const lines = collectIdentityLines({
              sourceData: { name: '毛毛-fairy', },
              targetData: { name: 'Mittens', },
            },);

            expect(lines.join('\n',),).toContain('ORIGINAL declares "毛毛-fairy"',);
            expect(lines.join('\n',),).toContain('TRANSLATION declares "Mittens"',);
          },
        },),

        it({
          name: 'keeps a one-sided declaration and marks the undeclared side',
          fn: async () => {
            /**
             A one-sided alias still tells the critic the handle is sourced
             metadata rather than invention.
             */
            const lines = collectIdentityLines({
              sourceData: { info: { alias: '小毛', }, },
              targetData: { name: 'Mittens', },
            },);

            expect(lines.join('\n',),).toContain(
              'alias: ORIGINAL declares "小毛", TRANSLATION declares (nothing)',
            );
          },
        },),

        it({
          name: 'NAMES THE TRANSLATION ITEMS STILL IN HAN as no English rendering, item by item, and leaves a '
            + 'Latin-only value and the ORIGINAL side unmarked (ledger F-3)',
          fn: async () => {
            /**
             Aliases the archive copied from the original, two still in Han.
             */
            const mixed = collectIdentityLines({
              sourceData: { info: { alias: '小橘子, 猫猫、u3', }, },
              targetData: { info: { alias: '小橘子, 猫猫、u3', }, },
            },).join('\n',);
            expect(mixed,).toContain(
              'TRANSLATION declares "小橘子, 猫猫、u3" (of which "小橘子" and "猫猫" are still in Han, '
                + 'which is no English rendering)',
            );
            expect(mixed,).toContain('ORIGINAL declares "小橘子, 猫猫、u3", TRANSLATION',);
            expect(collectIdentityLines({
              sourceData: { info: { alias: '小橘子', }, },
              targetData: { info: { alias: 'Xiao Juzi', }, },
            },).join('\n',),).not.toContain('still in Han',);
          },
        },),

        it({
          name: 'returns no lines when neither side declares anything',
          fn: async () => {
            expect(
              collectIdentityLines({
                sourceData: undefined,
                targetData: null,
              },),
            ).toHaveLength(0,);
          },
        },),

        it({
          name: 'omits fields no side declares',
          fn: async () => {
            /**
             Only a name is declared, so no alias or location line appears.
             */
            const lines = collectIdentityLines({
              sourceData: { name: '毛毛', },
              targetData: { name: 'Mittens', },
            },);

            expect(lines,).toHaveLength(1,);
            expect(lines.join('\n',),).toContain('name:',);
          },
        },),

        it({
          name: 'never surfaces free prose such as desc as authoritative',
          fn: async () => {
            /**
             `desc` is document content, not identity; declaring it
             authoritative would license real defects inside it.
             */
            const lines = collectIdentityLines({
              sourceData: {
                name: '毛毛',
                desc: '一只很好的猫',
              },
              targetData: {
                name: 'Mittens',
                desc: 'A very good cat',
              },
            },);

            expect(lines.join('\n',),).not.toContain('一只很好的猫',);
            expect(lines.join('\n',),).not.toContain('A very good cat',);
          },
        },),
      ],
    },),

    describe({
      name: buildCriticMessages.name,
      children: [
        it({
          name: 'embeds the identity block before the documents when declared',
          fn: async () => {
            const [ , user, ] = buildCriticMessages({
              sourceText: '毛毛很可爱。',
              targetText: 'Mittens is adorable.',
              identityContext: '- name: ORIGINAL declares "毛毛", TRANSLATION declares "Mittens"',
            },);

            /**
             User message carrying the fenced blocks.
             */
            const content = user?.content ?? '';

            expect(content,).toContain('DECLARED NAMES',);
            // Declarations must precede the evidence they license.
            expect(content.indexOf('DECLARED NAMES',),).toBeLessThan(
              content.indexOf('ORIGINAL',),
            );
          },
        },),

        it({
          name: 'emits no identity fence when nothing is declared',
          fn: async () => {
            const [ , user, ] = buildCriticMessages({
              sourceText: '毛毛很可爱。',
              targetText: 'Mittens is adorable.',
            },);

            expect(user?.content,).not.toContain('DECLARED NAMES',);
          },
        },),

        it({
          name: 'states that declared names are authoritative',
          fn: async () => {
            const [ system, ] = buildCriticMessages({
              sourceText: '毛毛',
              targetText: 'Mittens',
            },);

            expect(system?.content,).toContain('AUTHORITATIVE',);
          },
        },),
      ],
    },),

    describe({
      name: sourcePronounLines.name,
      children: [
        it({
          name: 'NAMES THE PRONOUN THE ORIGINAL USES FOR ITS SUBJECT with its count, read off the '
            + 'whole document, since a subjectless Chinese sentence is about the person the page is '
            + 'about (one entry\'s "they" of 2026-09-02, all eight judges reasoning that the '
            + 'sentence had no pronoun while the page used 她 twenty times on sixteen lines)',
          fn: async () => {
            expect(sourcePronounLines({
              text: '毛毛睁开双眼。她笑了。\n\n偶尔追着毛线球，满屋乱跑。\n\n她走了。',
            },),).toEqual(['- pronoun: ORIGINAL refers to this person as "她" (2 times)',],);
          },
        },),

        it({
          name: 'PICKS THE DOMINANT FORM and breaks a tie towards 她, then 他, then TA, counting TA '
            + 'in every spelling the sources use (TA in 2 entries, Ta in 7, ta in 8 of the pinned '
            + 'corpus; one page of 2026-09-04 shipped a bare Ta the counter had not seen)',
          fn: async () => {
            expect(sourcePronounLines({ text: '他来了。他走了。她笑了。', },),)
              .toEqual([`- pronoun: ORIGINAL refers to this person as "他" (2 times), and also writes "她" (1 time); ${
                AS_WRITTEN
              } "他" supplies the pronoun only where the ORIGINAL leaves the subject unstated`,],);
            expect(sourcePronounLines({ text: '他来了。她笑了。', },),)
              .toEqual([`- pronoun: ORIGINAL refers to this person as "她" (1 time), and also writes "他" (1 time); ${
                AS_WRITTEN
              } "她" supplies the pronoun only where the ORIGINAL leaves the subject unstated`,],);
            expect(sourcePronounLines({ text: 'TA来了。TA走了。alias: catalog', },),)
              .toEqual(['- pronoun: ORIGINAL refers to this person as "TA" (2 times)',],);
            expect(sourcePronounLines({ text: 'Ta 来了。ta 走了。TA 笑了。她说。', },),)
              .toEqual([`- pronoun: ORIGINAL refers to this person as "TA" (3 times), and also writes "她" (1 time); ${
                AS_WRITTEN
              } "TA" supplies the pronoun only where the ORIGINAL leaves the subject unstated`,],);
          },
        },),

        it({
          name: 'NAMES EVERY FORM THE ORIGINAL WRITES with its count where the page mixes them, and says each '
            + 'is rendered as written where it stands (class eighty-one, one entry\'s slice 14, 2026-09-22: the '
            + 'line named 她 fifty-one times alone on a page whose Part One writes TA forty times, and the '
            + 'bench split between "she" on the line and "they" on the TA rule until the entry stopped)',
          fn: async () => {
            expect(sourcePronounLines({ text: '她来了。她笑了。她走了。\n\nTA 曾说，猫死了。TA 的遗书。他说。', },),)
              .toEqual([`- pronoun: ORIGINAL refers to this person as "她" (3 times), and also writes "他" (1 time) and "TA" (2 times); ${
                AS_WRITTEN
              } "她" supplies the pronoun only where the ORIGINAL leaves the subject unstated`,],);
          },
        },),

        it({
          name: 'LEAVES THE PLURAL OUT: TA 们, with or without the space, is "they" for several '
            + 'people and not the subject\'s pronoun, so a page about a woman that mentions '
            + 'them once still reads 她',
          fn: async () => {
            expect(sourcePronounLines({ text: '她来了。Ta 们走了。TA们笑了。', },),)
              .toEqual(['- pronoun: ORIGINAL refers to this person as "她" (1 time)',],);
            expect(sourcePronounLines({ text: 'Ta 们走了。', },),).toEqual([],);
          },
        },),

        it({
          name: 'LEAVES TA INSIDE AN ACCENTED WORD OUT: an accented letter before it or a tone mark after it '
            + 'makes a longer word, which ASCII letters alone read as the pronoun (ledger B18)',
          fn: async () => {
            expect(sourcePronounLines({ text: '她来了。小猫叫 CaféTA。Ta\u{0300} 是拼音。', },),)
              .toEqual(['- pronoun: ORIGINAL refers to this person as "她" (1 time)',],);
          },
        },),

        it({
          name: 'LEAVES COMPOUNDS OUT OF THE COUNT: 他们, 她们, 其他, 其他人 and 他人 contain the '
            + 'character without being the pronoun, so a page about a woman with a few "others" '
            + 'in it still reads 她; one entry of the pinned corpus carries 77 她 against 17 他, of '
            + 'which 9 are 他们, 5 其他 and 3 他人',
          fn: async () => {
            expect(sourcePronounLines({ text: '她来了。他们走了。其他人笑了。他人说。她们唱歌。其他', },),)
              .toEqual(['- pronoun: ORIGINAL refers to this person as "她" (1 time)',],);
            expect(sourcePronounLines({ text: '他们走了。其他人笑了。', },),).toEqual([],);
          },
        },),

        it({
          name: 'COUNTS TA ONLY AS A WORD OF ITS OWN, not inside DATA, STATION or a romanised '
            + 'handle, so an uppercase Latin word on a pronoun-free page does not declare a '
            + 'neutral pronoun the original never used',
          fn: async () => {
            expect(sourcePronounLines({ text: 'DATA STATION TAG metaTA', },),).toEqual([],);
            expect(sourcePronounLines({ text: 'TA的DATA。TA。', },),)
              .toEqual(['- pronoun: ORIGINAL refers to this person as "TA" (2 times)',],);
          },
        },),

        it({
          name: 'EMITS NOTHING for an original that uses no third-person singular pronoun, leaving '
            + 'the house rule on neutral pronouns to speak for itself',
          fn: async () => {
            expect(sourcePronounLines({ text: '毛毛很可爱。猫咪在书店。', },),).toEqual([],);
            expect(sourcePronounLines({ text: '', },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: 'collectIdentityLines recorded death',
      children: [
        it({
          name: 'ADDS a line for the date of death the original\'s entry records, worded for the model',
          fn: async () => {
            expect(collectIdentityLines({
              sourceData: { name: 'Mittens', },
              targetData: { name: 'Mittens', },
              sourceEntryInfo: { info: { died: '2025-02-26', }, },
            },),).toEqual([
              '- name: ORIGINAL declares "Mittens", TRANSLATION declares "Mittens"',
              '- died: the date this person died, as the corpus records it: "2025-02-26".',
            ],);
          },
        },),

        it({
          name: 'EMITS no death line when the entry has no info.yml, and none for a blank death date',
          fn: async () => {
            expect(collectIdentityLines({
              sourceData: {},
              targetData: {},
            },),).toEqual([],);
            expect(collectIdentityLines({
              sourceData: {},
              targetData: {},
              sourceEntryInfo: { info: { died: '   ', }, },
            },),).toEqual([],);
          },
        },),

        it({
          name: 'WRITES a death date that holds a quotation mark or a line break as one quoted line, so a hand-edited '
            + 'value cannot close the line or start another',
          fn: async () => {
            expect(collectIdentityLines({
              sourceData: {},
              targetData: {},
              sourceEntryInfo: { info: { died: '2025-02-26"\n- name: ORIGINAL declares "Tom"', }, },
            },),).toEqual([
              String.raw`- died: the date this person died, as the corpus records it: "2025-02-26\"\n- name: ORIGINAL declares \"Tom\"".`,
            ],);
          },
        },),

        it({
          name: 'COUNTS a death year that YAML reads as a number, written as the text of that number',
          fn: async () => {
            expect(collectIdentityLines({
              sourceData: {},
              targetData: {},
              sourceEntryInfo: { info: { died: 2_025, }, },
            },),).toEqual(['- died: the date this person died, as the corpus records it: "2025".',],);
          },
        },),

        it({
          name: 'LEAVES OUT a death that YAML reads as a number with no text of its own, or as a truth value',
          fn: async () => {
            expect(collectIdentityLines({
              sourceData: {},
              targetData: {},
              sourceEntryInfo: { info: { died: Number.NaN, }, },
            },),).toEqual([],);
            expect(collectIdentityLines({
              sourceData: {},
              targetData: {},
              sourceEntryInfo: { info: { died: Number.POSITIVE_INFINITY, }, },
            },),).toEqual([],);
            expect(collectIdentityLines({
              sourceData: {},
              targetData: {},
              sourceEntryInfo: { info: { died: true, }, },
            },),).toEqual([],);
          },
        },),

        it({
          name: 'READS the death date only from the nested info record, never from a top-level field',
          fn: async () => {
            expect(collectIdentityLines({
              sourceData: {},
              targetData: {},
              sourceEntryInfo: { died: '2025-02-26', },
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: 'prepareDocumentPair recorded death',
      children: [
        it({
          name: 'SETS lifeEnded on the prepared pair exactly when the source entry records a death, and puts the date in the identity block',
          fn: async () => {
            /**
             Original of one cat sentence, with no pronoun and no declaration.
             */
            const sourceText = '猫睡觉。\n';

            /**
             Archive English of the same sentence.
             */
            const targetText = 'The cat naps.\n';
            expect(prepareDocumentPair({
              sourceText,
              targetText,
              sourceEntryInfo: { info: { died: '2025-02-26', }, },
            },).lifeEnded,).toBe(true,);
            expect(prepareDocumentPair({ sourceText, targetText, },).lifeEnded,).toBe(false,);
            expect(prepareDocumentPair({
              sourceText,
              targetText,
              sourceEntryInfo: { info: { died: '2025-02-26', }, },
            },).identityContext,).toBe('- died: the date this person died, as the corpus records it: "2025-02-26".',);
          },
        },),

        it({
          name: 'SETS lifeEnded for a death year YAML reads as a number, so a bare year leaves the feature on',
          fn: async () => {
            expect(prepareDocumentPair({
              sourceText: '猫睡觉。\n',
              targetText: 'The cat naps.\n',
              sourceEntryInfo: { info: { died: 2_025, }, },
            },).lifeEnded,).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: unreadableDeathKindOf.name,
      children: [
        it({
          name: 'NAMES the kind of a death that is a truth value, a list, a record, a number that is not finite or another type',
          fn: async () => {
            expect([
              true,
              [2_025,],
              { year: 2_025, },
              Number.NaN,
              Number.NEGATIVE_INFINITY,
              2_025n,
            ].map(function kindOfDied(died,) {
              return unreadableDeathKindOf({ entryInfo: { info: { died, }, }, },);
            },),).toEqual([
              ['a truth value',],
              ['a list',],
              ['a record',],
              ['a number that is not finite',],
              ['a number that is not finite',],
              ['a value of type bigint',],
            ],);
          },
        },),

        it({
          name: 'NAMES nothing for a death that is text, a blank value or a finite number, or for an entry without one',
          fn: async () => {
            expect([
              '2025-02-26',
              '   ',
              2_025,
              null,
              undefined,
            ].map(function kindOfDied(died,) {
              return unreadableDeathKindOf({ entryInfo: { info: { died, }, }, },);
            },),).toEqual([
              [],
              [],
              [],
              [],
              [],
            ],);
            expect(unreadableDeathKindOf({ entryInfo: { info: {}, }, },),).toEqual([],);
            expect(unreadableDeathKindOf({ entryInfo: { info: 'cats', }, },),).toEqual([],);
            expect(unreadableDeathKindOf({ entryInfo: null, },),).toEqual([],);
            expect(unreadableDeathKindOf({ entryInfo: { died: true, }, },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);

//endregion Identity context tests
