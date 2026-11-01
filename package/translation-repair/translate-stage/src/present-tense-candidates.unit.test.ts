/**
 Tests for present-tense candidates on memorial pages.

 Every case asserts the WHOLE candidate list rather than one entry, because the
 defect this guards against is a rule that fires on the wrong sentence or misses
 one, and that shows up as a changed list rather than a changed field. Fixtures
 are invented cats; no corpus text appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  presentTenseCandidates,
} from '../dist/final/node/index.mjs';

/**
 Candidates for a memorial page that records a death, the case every rule applies to.

 @param text - body of the page
 @param subjectName - declared name of the subject

 @returns Candidates the detector reports for the body

 @example
 ```ts
 const found = candidatesOf({ text: 'Mittens naps.', subjectName: 'Mittens', },);
 ```
 */
function candidatesOf(
  {
    text,
    subjectName,
  }: {
    readonly text: string;
    readonly subjectName: string;
  },
) {
  return presentTenseCandidates({
    text,
    subjectNames: [subjectName,],
    died: true,
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: presentTenseCandidates.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reports a contraction of she or he followed by a present state',
          fn: async () => {
            expect(candidatesOf({
              text: 'She\'s a cat of great patience.',
              subjectName: 'Mittens',
            },),).toEqual([
              {
                sentence: 'She\'s a cat of great patience.',
                matched: 'She\'s',
                rule: 'contraction',
              },
            ],);
          },
        },),

        it({
          name: 'returns no candidate for a contraction followed by been, had, got, gone, done or left',
          fn: async () => {
            expect(candidatesOf({
              text: 'She\'s been napping since noon.',
              subjectName: 'Mittens',
            },),).toEqual([],);
          },
        },),

        it({
          name: 'reports they are contracted as present',
          fn: async () => {
            expect(candidatesOf({
              text: 'They\'re the reason the box stays open.',
              subjectName: 'Mittens',
            },),).toEqual([
              {
                sentence: 'They\'re the reason the box stays open.',
                matched: 'They\'re',
                rule: 'contraction',
              },
            ],);
          },
        },),

        it({
          name: 'reports a copula after the subject name as a copula or auxiliary hit',
          fn: async () => {
            expect(candidatesOf({
              text: 'Mittens is a tabby with a white chin.',
              subjectName: 'Mittens',
            },),).toEqual([
              {
                sentence: 'Mittens is a tabby with a white chin.',
                matched: 'Mittens is',
                rule: 'copula-or-auxiliary',
              },
            ],);
          },
        },),

        it({
          name: 'reports an -s verb after a pronoun as an s-ending verb hit',
          fn: async () => {
            expect(candidatesOf({
              text: 'She naps by the window every afternoon.',
              subjectName: 'Mittens',
            },),).toEqual([
              {
                sentence: 'She naps by the window every afternoon.',
                matched: 'She naps',
                rule: 's-ending-verb',
              },
            ],);
          },
        },),

        it({
          name: 'skips was as an -s word, so a past copula never fires the s-ending rule',
          fn: async () => {
            expect(candidatesOf({
              text: 'She was small when the box arrived.',
              subjectName: 'Mittens',
            },),).toEqual([],);
          },
        },),

        it({
          name: 'reports a bare present verb after a pronoun as a bare verb hit',
          fn: async () => {
            expect(candidatesOf({
              text: 'They love the sun on the step.',
              subjectName: 'Mittens',
            },),).toEqual([
              {
                sentence: 'They love the sun on the step.',
                matched: 'They love',
                rule: 'bare-verb',
              },
            ],);
          },
        },),

        it({
          name: 'does not take the subject name as a bare verb subject, only pronouns',
          fn: async () => {
            expect(candidatesOf({
              text: 'Mittens love the sun on the step.',
              subjectName: 'Mittens',
            },),).toEqual([],);
          },
        },),

        it({
          name: 'gives one candidate for a copula sentence and no second hit from another rule',
          fn: async () => {
            expect(candidatesOf({
              text: 'She is napping on the sill.',
              subjectName: 'Mittens',
            },),).toEqual([
              {
                sentence: 'She is napping on the sill.',
                matched: 'She is',
                rule: 'copula-or-auxiliary',
              },
            ],);
          },
        },),

        it({
          name: 'matches the subject name in regular expression metacharacters as plain text',
          fn: async () => {
            expect(candidatesOf({
              text: 'Tom (Jr) naps by the door.',
              subjectName: 'Tom (Jr)',
            },),).toEqual([
              {
                sentence: 'Tom (Jr) naps by the door.',
                matched: 'Tom (Jr) naps',
                rule: 's-ending-verb',
              },
            ],);
          },
        },),

        it({
          name: 'returns no candidate for a subject that follows an inverted auxiliary',
          fn: async () => {
            expect(candidatesOf({
              text: 'Even less did she want to leave the box.',
              subjectName: 'Mittens',
            },),).toEqual([],);
          },
        },),

        it({
          name: 'returns no candidate for a sentence that ends in a question mark',
          fn: async () => {
            expect(candidatesOf({
              text: 'Wherever Mittens is, does she still nap?',
              subjectName: 'Mittens',
            },),).toEqual([],);
          },
        },),

        it({
          name: 'gives each sentence of a paragraph its own candidate and skips sentences that do not hit',
          fn: async () => {
            expect(candidatesOf({
              text: 'The vet came in May. She naps by the window. Did she nap there? Mittens is loud.',
              subjectName: 'Mittens',
            },),).toEqual([
              {
                sentence: 'She naps by the window.',
                matched: 'She naps',
                rule: 's-ending-verb',
              },
              {
                sentence: 'Mittens is loud.',
                matched: 'Mittens is',
                rule: 'copula-or-auxiliary',
              },
            ],);
          },
        },),

        it({
          name: 'reads the link text of a Markdown link as the sentence words and drops the URL',
          fn: async () => {
            expect(candidatesOf({
              text: '[Mittens](https://cats.example/mittens) naps by the door.',
              subjectName: 'Mittens',
            },),).toEqual([
              {
                sentence: 'Mittens naps by the door.',
                matched: 'Mittens naps',
                rule: 's-ending-verb',
              },
            ],);
          },
        },),

        it({
          name: 'returns no candidate for a straight-quoted sentence, which is speech and not narration',
          fn: async () => {
            expect(candidatesOf({
              text: '"She naps all day," said the vet.',
              subjectName: 'Mittens',
            },),).toEqual([],);
          },
        },),

        it({
          name: 'returns no candidate for a curly-quoted sentence, which is speech and not narration',
          fn: async () => {
            expect(candidatesOf({
              text: 'The vet said, “She naps all day.” Then she left.',
              subjectName: 'Mittens',
            },),).toEqual([],);
          },
        },),

        it({
          name: 'returns no candidate for a sentence quoted in single curly marks, which is speech and not narration (n166, Z10)',
          fn: async () => {
            expect(candidatesOf({
              text: 'The vet said, ‘She naps all day.’ Then she left.',
              subjectName: 'Mittens',
            },),).toEqual([],);
            expect(candidatesOf({
              text: 'The vet said, ‘She naps all day, and the cat’s bowl is full.’ Then she left.',
              subjectName: 'Mittens',
            },),).toEqual([],);
          },
        },),

        it({
          name: 'drops the quoted sentence and keeps the narration beside a quotation in single curly marks and an apostrophe after a '
            + 'plural (n166, Z10)',
          fn: async () => {
            expect(candidatesOf({
              text: 'She naps by the window. ‘She naps.’ Mittens’ bowl is empty.',
              subjectName: 'Mittens',
            },),).toEqual([
              {
                sentence: 'She naps by the window.',
                matched: 'She naps',
                rule: 's-ending-verb',
              },
            ],);
          },
        },),

        it({
          name: 'keeps reading a sentence whose apostrophe inside a word is the character a closing single mark is, since no mark opened '
            + 'a quotation it could close (n166, Z10 controls)',
          fn: async () => {
            expect(candidatesOf({
              text: 'The cat’s bowl is full. She naps by the window.',
              subjectName: 'Mittens',
            },),).toEqual([
              {
                sentence: 'She naps by the window.',
                matched: 'She naps',
                rule: 's-ending-verb',
              },
            ],);
            expect(candidatesOf({
              text: 'The vet called it ‘the cat’s bowl and she naps by the window.',
              subjectName: 'Mittens',
            },),).toEqual([
              {
                sentence: 'The vet called it ‘the cat’s bowl and she naps by the window.',
                matched: 'she naps',
                rule: 's-ending-verb',
              },
            ],);
          },
        },),

        it({
          name: 'returns no candidate when a comma separates the subject from the verb',
          fn: async () => {
            expect(candidatesOf({
              text: 'She, naps by the window.',
              subjectName: 'Mittens',
            },),).toEqual([],);
          },
        },),

        it({
          name: 'returns no candidate when a bracket separates the subject from the verb',
          fn: async () => {
            expect(candidatesOf({
              text: 'She (naps) by the window.',
              subjectName: 'Mittens',
            },),).toEqual([],);
          },
        },),

        it({
          name: 'returns no candidate for a verb-like word with a hyphen, which is not a present verb',
          fn: async () => {
            expect(candidatesOf({
              text: 'She half-naps by the window.',
              subjectName: 'Mittens',
            },),).toEqual([],);
          },
        },),

        it({
          name: 'reads a final sentence that has no closing mark as a sentence of its own',
          fn: async () => {
            expect(candidatesOf({
              text: 'She naps by the window',
              subjectName: 'Mittens',
            },),).toEqual([
              {
                sentence: 'She naps by the window',
                matched: 'She naps',
                rule: 's-ending-verb',
              },
            ],);
          },
        },),

        it({
          name: 'returns no candidate for a blockquote line',
          fn: async () => {
            expect(candidatesOf({
              text: '> She naps by the window.',
              subjectName: 'Mittens',
            },),).toEqual([],);
          },
        },),

        it({
          name: 'returns no candidate for a heading',
          fn: async () => {
            expect(candidatesOf({
              text: '## Mittens is a tabby',
              subjectName: 'Mittens',
            },),).toEqual([],);
          },
        },),

        it({
          name: 'returns no candidate for a fenced code block',
          fn: async () => {
            expect(candidatesOf({
              text: '```\nShe naps by the window.\n```',
              subjectName: 'Mittens',
            },),).toEqual([],);
          },
        },),

        it({
          name: 'returns no candidate for an HTML block',
          fn: async () => {
            expect(candidatesOf({
              text: '<div>\nMittens naps by the window.\n</div>',
              subjectName: 'Mittens',
            },),).toEqual([],);
          },
        },),

        it({
          name: 'returns no candidates for a page that records no death, whatever its text',
          fn: async () => {
            expect(presentTenseCandidates({
              text: 'She naps by the window. Mittens is loud.',
              subjectNames: ['Mittens',],
              died: false,
            },),).toEqual([],);
          },
        },),

        it({
          name: 'matches only pronoun subjects when no subject name is declared',
          fn: async () => {
            expect(candidatesOf({
              text: 'Mittens is loud. They love the sun.',
              subjectName: '',
            },),).toEqual([
              {
                sentence: 'They love the sun.',
                matched: 'They love',
                rule: 'bare-verb',
              },
            ],);
          },
        },),

        it({
          name: 'matches a sentence that names the subject by a declared alias, as it matches the declared name',
          fn: async () => {
            expect(presentTenseCandidates({
              text: 'Mittens naps by the door. Whiskers is loud. Tom naps too.',
              subjectNames: [
                'Mittens',
                'Whiskers',
              ],
              died: true,
            },),).toEqual([
              {
                sentence: 'Mittens naps by the door.',
                matched: 'Mittens naps',
                rule: 's-ending-verb',
              },
              {
                sentence: 'Whiskers is loud.',
                matched: 'Whiskers is',
                rule: 'copula-or-auxiliary',
              },
            ],);
          },
        },),

        it({
          name: 'matches the longest declared name where one declared name begins another',
          fn: async () => {
            expect(presentTenseCandidates({
              text: 'Mittens Jr naps by the door.',
              subjectNames: [
                'Mittens',
                'Mittens Jr',
              ],
              died: true,
            },),).toEqual([
              {
                sentence: 'Mittens Jr naps by the door.',
                matched: 'Mittens Jr naps',
                rule: 's-ending-verb',
              },
            ],);
          },
        },),

        it({
          name: 'matches a declared name written in a script without letter case, as it matches a Latin one',
          fn: async () => {
            expect(presentTenseCandidates({
              text: '猫猫 is loud. Mittens is loud.',
              subjectNames: [
                '猫猫',
                'Mittens',
              ],
              died: true,
            },),).toEqual([
              {
                sentence: '猫猫 is loud.',
                matched: '猫猫 is',
                rule: 'copula-or-auxiliary',
              },
              {
                sentence: 'Mittens is loud.',
                matched: 'Mittens is',
                rule: 'copula-or-auxiliary',
              },
            ],);
          },
        },),

        it({
          name: 'matches a declared name written beyond the first plane, as it matches a Latin one',
          fn: async () => {
            /**
             A cat name in Deseret letters, built from code points so the source holds no such character.
             */
            const deseretName = String.fromCodePoint(
              0x1_04_28,
              0x1_04_29,
              0x1_04_2A,
            );
            expect(presentTenseCandidates({
              text: `${deseretName} naps by the door. Mittens naps by the door.`,
              subjectNames: [
                deseretName,
                'Mittens',
              ],
              died: true,
            },),).toEqual([
              {
                sentence: `${deseretName} naps by the door.`,
                matched: `${deseretName} naps`,
                rule: 's-ending-verb',
              },
              {
                sentence: 'Mittens naps by the door.',
                matched: 'Mittens naps',
                rule: 's-ending-verb',
              },
            ],);
          },
        },),
      ],
    },),
  ],
},);
