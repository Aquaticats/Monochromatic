/**
 Tests for the answers-file schema and the line operations that record one block.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  AnswersError,
  insertBefore,
  parseAnswers,
  recordAnswers,
  removeBullet,
  removeRun,
  removeSection,
  type Answers,
} from '../dist/final/node/index.mjs';

/**
 Returns the error a call throws.
 
 @param action - Call expected to refuse.
 
 @returns Whatever the call threw.
 
 @throws {@link Error} when the call returns, which means it refused nothing.
 
 @example
 ```ts
 const refused = failureOf(() => parseAnswers({ text: '{}', at: 'answers.json' }));
 ```
 */
function failureOf(action: () => unknown,): unknown {
  try {
    action();
  }
  catch (error) {
    return error;
  }
  throw new Error('The call returned where the test expects a refusal.',);
}

/**
 Answers that record one block before a heading and remove one section.
 
 @returns Answers the tests rewrite.
 
 @example
 ```ts
 const answers = answersFor();
 ```
 */
function answersFor(): Answers {
  return {
    file: 'doc/handover/questions-for-the-owner.md',
    record: '## Answered, recorded 2026-11-10T05:40:16-05:00\n\n### A question\n\n"the owner\'s words"\n',
    anchor: '## Answered in the question tool, recorded',
    remove: {
      sections: ['### A question, asked 2026-11-09T01:16:11-05:00',],
      bullets: [],
      lineRuns: [],
    },
  };
}

/**
 A questions file holding one open question, one bullet, and the anchor heading.
 
 @returns The file's lines.
 
 @example
 ```ts
 const text = questionsText();
 ```
 */
function questionsText(): string {
  return [
    '# Questions for the owner',
    '',
    '## Open questions',
    '',
    '### A question, asked 2026-11-09T01:16:11-05:00',
    '',
    'The question body.',
    '',
    '### Another question, asked 2026-11-09T02:00:00-05:00',
    '',
    '- a bullet that stays',
    '- a bullet that goes',
    '  with its continuation',
    '- a third bullet',
    '',
    '## Answered in the question tool, recorded',
    '',
    '### An older answer',
    '',
  ].join('\n',);
}

await describe({
  name: '',
  children: [
    describe({
      name: parseAnswers.name,
      children: [
        it({
          name: 'reads the block, the anchor, and every part to remove',
          fn: async () => {
            /**
             Valid answers text.
             */
            const text = JSON.stringify({
              file: 'doc/q.md',
              record: '## Answered',
              anchor: '## Recorded',
              remove: {
                sections: ['### Q',],
                bullets: ['- b',],
                lineRuns: [['a', 'b',],],
              },
            },);
            expect(parseAnswers({
              text,
              at: 'answers.json',
            },),).toEqual({
              file: 'doc/q.md',
              record: '## Answered',
              anchor: '## Recorded',
              remove: {
                sections: ['### Q',],
                bullets: ['- b',],
                lineRuns: [['a', 'b',],],
              },
            },);
          },
        },),
        it({
          name: 'treats every removal list as optional',
          fn: async () => {
            /**
             Answers text with no remove object.
             */
            const parsed = parseAnswers({
              text: JSON.stringify({
                file: 'doc/q.md',
                record: '## Answered',
                anchor: '## Recorded',
              },),
              at: 'answers.json',
            },);
            expect(parsed.remove,).toEqual({
              sections: [],
              bullets: [],
              lineRuns: [],
            },);
          },
        },),
        it({
          name: 'refuses text that is not JSON, naming the file',
          fn: async () => {
            /**
             Value the parse threw.
             */
            const refused = failureOf(() => parseAnswers({
              text: 'not json',
              at: 'answers.json',
            },),);
            expect(refused,).toBeInstanceOf(AnswersError,);
            expect((refused as Error).message,).toContain('answers.json is not valid JSON',);
          },
        },),
        it({
          name: 'refuses an answers file that is not an object',
          fn: async () => {
            /**
             Value the parse threw.
             */
            const refused = failureOf(() => parseAnswers({
              text: '[]',
              at: 'answers.json',
            },),);
            expect((refused as Error).message,).toContain('must be a JSON object',);
          },
        },),
        it({
          name: 'refuses a missing record block',
          fn: async () => {
            /**
             Value the parse threw.
             */
            const refused = failureOf(() => parseAnswers({
              text: JSON.stringify({ file: 'doc/q.md', anchor: '## Recorded', },),
              at: 'answers.json',
            },),);
            expect((refused as Error).message,).toContain("answers.json.record must be a non-empty string",);
          },
        },),
        it({
          name: 'refuses a removal list that is not a list of strings',
          fn: async () => {
            /**
             Value the parse threw.
             */
            const refused = failureOf(() => parseAnswers({
              text: JSON.stringify({
                file: 'doc/q.md',
                record: '## Answered',
                anchor: '## Recorded',
                remove: { sections: [7,], },
              },),
              at: 'answers.json',
            },),);
            expect((refused as Error).message,).toContain('answers.json.remove.sections[0]',);
          },
        },),
        it({
          name: 'refuses an empty line run',
          fn: async () => {
            /**
             Value the parse threw.
             */
            const refused = failureOf(() => parseAnswers({
              text: JSON.stringify({
                file: 'doc/q.md',
                record: '## Answered',
                anchor: '## Recorded',
                remove: { lineRuns: [[],], },
              },),
              at: 'answers.json',
            },),);
            expect((refused as Error).message,).toContain('at least one line',);
          },
        },),
        it({
          name: 'refuses a remove field that is not an object',
          fn: async () => {
            /**
             Value the parse threw.
             */
            const refused = failureOf(() => parseAnswers({
              text: JSON.stringify({
                file: 'doc/q.md',
                record: '## Answered',
                anchor: '## Recorded',
                remove: [],
              },),
              at: 'answers.json',
            },),);
            expect((refused as Error).message,).toContain('answers.json.remove must be a JSON object',);
          },
        },),
      ],
    },),
    describe({
      name: removeSection.name,
      children: [
        it({
          name: 'drops the heading through the line before the next heading',
          fn: async () => {
            /**
             Lines under test.
             */
            const lines = ['## A', '### Q', 'body', '### R', 'other',];
            expect(removeSection({
              lines,
              heading: '### Q',
            },),).toEqual(['## A', '### R', 'other',],);
          },
        },),
        it({
          name: 'refuses a heading the file does not hold',
          fn: async () => {
            /**
             Value the call threw.
             */
            const refused = failureOf(() => removeSection({
              lines: ['## A',],
              heading: '### Missing',
            },),);
            expect((refused as Error).message,).toContain('No section starts with: ### Missing',);
          },
        },),
        it({
          name: 'refuses a section at the end of the file with no heading after it',
          fn: async () => {
            /**
             Value the call threw.
             */
            const refused = failureOf(() => removeSection({
              lines: ['## A', '### Q', 'body',],
              heading: '### Q',
            },),);
            expect((refused as Error).message,).toContain('No heading follows the section',);
          },
        },),
      ],
    },),
    describe({
      name: removeBullet.name,
      children: [
        it({
          name: 'drops the bullet with its indented continuation',
          fn: async () => {
            /**
             Lines under test.
             */
            const lines = ['- a', '- b', '  more', '- c',];
            expect(removeBullet({
              lines,
              bullet: '- b',
            },),).toEqual(['- a', '- c',],);
          },
        },),
        it({
          name: 'refuses a bullet the file does not hold',
          fn: async () => {
            /**
             Value the call threw.
             */
            const refused = failureOf(() => removeBullet({
              lines: ['- a',],
              bullet: '- z',
            },),);
            expect((refused as Error).message,).toContain('No bullet starts with: - z',);
          },
        },),
        it({
          name: 'refuses a bullet that nothing follows',
          fn: async () => {
            /**
             Value the call threw.
             */
            const refused = failureOf(() => removeBullet({
              lines: ['- a', '  more',],
              bullet: '- a',
            },),);
            expect((refused as Error).message,).toContain('Nothing follows the bullet: - a',);
          },
        },),
      ],
    },),
    describe({
      name: removeRun.name,
      children: [
        it({
          name: 'drops exact consecutive lines',
          fn: async () => {
            /**
             Lines under test.
             */
            const lines = ['a', 'b', 'c', 'd',];
            expect(removeRun({
              lines,
              run: ['b', 'c',],
            },),).toEqual(['a', 'd',],);
          },
        },),
        it({
          name: 'refuses a run the file does not hold in that order',
          fn: async () => {
            /**
             Value the call threw.
             */
            const refused = failureOf(() => removeRun({
              lines: ['c', 'b',],
              run: ['b', 'c',],
            },),);
            expect((refused as Error).message,).toContain('No run of lines starts with: b',);
          },
        },),
      ],
    },),
    describe({
      name: insertBefore.name,
      children: [
        it({
          name: 'inserts the block before the anchor heading',
          fn: async () => {
            /**
             Insertion under test.
             */
            const inserted = insertBefore({
              lines: ['## Open', '## Recorded',],
              anchor: '## Recorded',
              block: ['## Answered',],
            },);
            expect(inserted.lines,).toEqual(['## Open', '## Answered', '## Recorded',],);
            expect(inserted.at,).toBe(1,);
          },
        },),
        it({
          name: 'refuses a file with no anchor heading',
          fn: async () => {
            /**
             Value the call threw.
             */
            const refused = failureOf(() => insertBefore({
              lines: ['## Open',],
              anchor: '## Recorded',
              block: [],
            },),);
            expect((refused as Error).message,).toContain('No heading starts with: ## Recorded',);
          },
        },),
      ],
    },),
    describe({
      name: recordAnswers.name,
      children: [
        it({
          name: 'removes the answered section, drops the trailing empty block line, and inserts the record',
          fn: async () => {
            /**
             Rewrite under test.
             */
            const result = recordAnswers({
              text: questionsText(),
              answers: answersFor(),
            },);
            expect(result.removedSections,).toBe(1,);
            expect(result.text,).toContain('## Answered, recorded 2026-11-10T05:40:16-05:00',);
            expect(result.text,).toContain('"the owner\'s words"',);
            expect(result.text,).not.toContain('### A question, asked 2026-11-09T01:16:11-05:00',);
            expect(result.text,).toContain('### Another question, asked 2026-11-09T02:00:00-05:00',);
            expect(result.text,).toContain('### An older answer',);
            expect(result.text.endsWith('\n',),).toBe(true,);
          },
        },),
        it({
          name: 'removes a bullet and a line run in one rewrite',
          fn: async () => {
            /**
             Answers that remove a bullet and a run of lines.
             */
            const answers = answersFor();
            /**
             Rewrite under test.
             */
            const result = recordAnswers({
              text: questionsText(),
              answers: {
                ...answers,
                remove: {
                  sections: answers.remove.sections,
                  bullets: ['- a bullet that goes',],
                  lineRuns: [['- a third bullet',],],
                },
              },
            },);
            expect(result.removedBullets,).toBe(1,);
            expect(result.removedRuns,).toBe(1,);
            expect(result.text,).not.toContain('- a bullet that goes',);
            expect(result.text,).not.toContain('with its continuation',);
            expect(result.text,).not.toContain('- a third bullet',);
            expect(result.text,).toContain('- a bullet that stays',);
          },
        },),
      ],
    },),
  ],
},);
