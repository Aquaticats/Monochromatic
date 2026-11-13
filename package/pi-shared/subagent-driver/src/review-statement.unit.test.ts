/**
 Reading a review's statement section: the section's place after the title, its one fenced block, and each field of the shape, with
 one test per way a field can be wrong.

 Nothing here reads a file, starts a run or touches a machine: each case is a string the test writes.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  REVIEW_STATEMENT_HEADING,
  REVIEW_STATEMENT_SKELETON,
  ReviewStatementError,
  commitIsTheSame,
  statementInReview,
} from '../dist/final/node/index.mjs';

/**
 The file name every message names.

 @returns the review file's path

 @example
 ```ts
 const file = aFile();
 ```
 */
function aFile(): string {
  return '/reviews/a/review.md';
}

/**
 The statement every test starts from.

 @returns the statement, as the JSON a review writes

 @example
 ```ts
 const statement = aStatement();
 ```
 */
function aStatement(): Readonly<Record<string, unknown>> {
  return {
    commit: 'abc1234',
    diff: {
      lines: 12,
      linesRead: 4,
    },
    files: [{
      path: 'package/a/src/one.rs',
      read: 'whole',
    },],
    commands: [{
      command: 'node repro.ts',
      exit: 0,
    },],
    findings: [{
      finding: '1',
      shownBy: 'reading',
    },],
  };
}

/**
 Builds the review file's text for one statement, placed as a review writes it.

 @param statement - the statement to write, or the text of the block to write as it is

 @returns the review file's text

 @example
 ```ts
 const shown = aReviewFile(aStatement());
 ```
 */
function aReviewFile(statement: Readonly<Record<string, unknown>> | string,): string {
  /**
   The block's text.
   */
  const block = (typeof statement) === 'string' ? statement : JSON.stringify(
    statement,
    undefined,
    2,
  );
  return `# Review: a change\n\n${REVIEW_STATEMENT_HEADING}\n\n\`\`\`review-statement\n${block}\n\`\`\`\n\n## Findings\n\n`
    + 'Nothing.\n';
}

/**
 Reads the error one review file answers with.

 @param shown - the review file's text

 @returns the error's message

 @example
 ```ts
 const message = messageOf(aReviewFile(aStatement()));
 ```
 */
function messageOf(shown: string,): string {
  try {
    statementInReview({
      shown,
      file: aFile(),
    },);
    return 'it read';
  }
  catch (error) {
    // A file this reader refuses is exactly what each case asserts on.
    return (error instanceof ReviewStatementError) ? error.message : String(error,);
  }
}

/**
 Reads a statement's text with one field replaced by a value the shape must refuse.

 @param field - the field's path, as the message names it

 @param value - the value to put there

 @returns the review file's text carrying it

 @example
 ```ts
 const shown = withTheField({ field: 'commit', value: 3 });
 ```
 */
function withTheField({
  field,
  value,
}: {
  readonly field: string;
  readonly value: unknown;
},): string {
  /**
   One level of the statement, as the field's path names it.
   */
  const statement = aStatement() as Record<string, unknown>;
  if (!field.includes('.',)) {
    return aReviewFile({
      ...statement,
      [field]: value,
    },);
  }
  const [outer, inner,] = field.split('.',);
  return aReviewFile({
    ...statement,
    [outer ?? '']: {
      ...(statement[outer ?? ''] as Readonly<Record<string, unknown>>),
      [inner ?? '']: value,
    },
  },);
}

await describe({
  name: '',
  children: [
    describe({
      name: statementInReview.name,
      children: [
        it({
          name: 'reads the statement a review writes after its title, keeping every field as the review wrote it',
          fn: async () => {
            expect(statementInReview({
              shown: aReviewFile(aStatement(),),
              file: aFile(),
            },),).toEqual(aStatement(),);
          },
        },),
        it({
          name: 'reads a block the skeleton was copied from, refusing it by the first field still holding a placeholder',
          fn: async () => {
            expect(messageOf(aReviewFile(REVIEW_STATEMENT_SKELETON,),),).toContain('diff.lines is no whole number from zero');
          },
        },),
        //region The section's place
        it({
          name: 'refuses a review file with no title line',
          fn: async () => {
            expect(messageOf('A review with no title.\n\n## Findings\n',),).toContain('has no title line starting "# "');
          },
        },),
        it({
          name: 'refuses a review file whose first section is not the statement',
          fn: async () => {
            expect(messageOf('# Review\n\n## Findings\n\nNothing.\n',),).toContain('as its first section, not');
          },
        },),
        it({
          name: 'refuses a statement section with no fenced block',
          fn: async () => {
            expect(messageOf(`# Review\n\n${REVIEW_STATEMENT_HEADING}\n\njust prose\n`,),).toContain('holds no fenced block tagged');
          },
        },),
        it({
          name: 'refuses a statement section whose only block is tagged something else, naming that language',
          fn: async () => {
            expect(messageOf(`# Review\n\n${REVIEW_STATEMENT_HEADING}\n\n\`\`\`json\n{}\n\`\`\`\n`,),).toContain(
              'holds a fenced block tagged json and none tagged',
            );
          },
        },),
        it({
          name: 'refuses a statement section holding two statement blocks, so which one states the review is unreadable',
          fn: async () => {
            expect(messageOf(
              `# Review\n\n${REVIEW_STATEMENT_HEADING}\n\n\`\`\`review-statement\n{}\n\`\`\`\n\n\`\`\`review-statement\n{}\n\`\`\`\n`,
            ),).toContain('holds 2 blocks tagged');
          },
        },),
        it({
          name: 'refuses a fenced block that is never closed',
          fn: async () => {
            expect(messageOf(`# Review\n\n${REVIEW_STATEMENT_HEADING}\n\n\`\`\`review-statement\n{}\n`,),).toContain(
              'a fenced block that is never closed',
            );
          },
        },),
        //endregion The section's place

        //region The block
        it({
          name: 'refuses a block that is no JSON',
          fn: async () => {
            expect(messageOf(aReviewFile('{ this is not json',),),).toContain('is no JSON');
          },
        },),
        it({
          name: 'refuses a block that is no JSON object',
          fn: async () => {
            expect(messageOf(aReviewFile('[]',),),).toContain('the statement is no JSON object');
          },
        },),
        it({
          name: 'refuses a field the shape does not carry, naming it and the fields the reader reads',
          fn: async () => {
            expect(messageOf(withTheField({
              field: 'verdict',
              value: 'clean',
            },),),).toContain('the statement names a field this reader does not read: verdict');
          },
        },),
        //endregion The block

        //region Each field
        it({
          name: 'refuses a commit that is missing, blank, or not text',
          fn: async () => {
            expect(messageOf(withTheField({
              field: 'commit',
              value: '  ',
            },),),).toContain('commit is no text');
            expect(messageOf(withTheField({
              field: 'commit',
              value: 7,
            },),),).toContain('commit is no text');
          },
        },),
        it({
          name: 'refuses a diff that is no object, or that names a field the shape does not carry',
          fn: async () => {
            expect(messageOf(withTheField({
              field: 'diff',
              value: 12,
            },),),).toContain('diff is no JSON object');
            expect(messageOf(withTheField({
              field: 'diff.files',
              value: 12,
            },),),).toContain('diff names a field this reader does not read: files');
          },
        },),
        it({
          name: 'refuses a line count that is no whole number from zero',
          fn: async () => {
            expect(messageOf(withTheField({
              field: 'diff.lines',
              value: -1,
            },),),).toContain('diff.lines is no whole number from zero');
            expect(messageOf(withTheField({
              field: 'diff.linesRead',
              value: 1.5,
            },),),).toContain('diff.linesRead is no whole number from zero');
          },
        },),
        it({
          name: 'refuses a line count read that is more than the diff holds',
          fn: async () => {
            expect(messageOf(withTheField({
              field: 'diff.linesRead',
              value: 13,
            },),),).toContain('diff.linesRead says 13 lines read of a diff of 12 lines');
          },
        },),
        it({
          name: 'refuses files that are no array, or an entry that is no object',
          fn: async () => {
            expect(messageOf(withTheField({
              field: 'files',
              value: {},
            },),),).toContain('files is no JSON array');
            expect(messageOf(withTheField({
              field: 'files',
              value: ['package/a/src/one.rs',],
            },),),).toContain('files entry 1 is no JSON object');
          },
        },),
        it({
          name: 'refuses a file entry naming a field the shape does not carry',
          fn: async () => {
            expect(messageOf(withTheField({
              field: 'files',
              value: [{
                path: 'package/a/src/one.rs',
                read: 'whole',
                lines: 3,
              },],
            },),),).toContain('files entry 1 names a field this reader does not read: lines');
          },
        },),
        it({
          name: 'refuses a file entry with no path, and one whose read word is not whole, part or none',
          fn: async () => {
            expect(messageOf(withTheField({
              field: 'files',
              value: [{
                path: '',
                read: 'whole',
              },],
            },),),).toContain('files entry 1.path is no text');
            expect(messageOf(withTheField({
              field: 'files',
              value: [{
                path: 'package/a/src/one.rs',
                read: 'half',
              },],
            },),),).toContain('files entry 1.read is "half", which is not one of whole, part, none');
          },
        },),
        it({
          name: 'refuses a file named more than once',
          fn: async () => {
            expect(messageOf(withTheField({
              field: 'files',
              value: [
                {
                  path: 'package/a/src/one.rs',
                  read: 'whole',
                },
                {
                  path: 'package/a/src/one.rs',
                  read: 'part',
                },
              ],
            },),),).toContain('files names package/a/src/one.rs more than once');
          },
        },),
        it({
          name: 'refuses a command that is no text, and an exit code that is no whole number from zero',
          fn: async () => {
            expect(messageOf(withTheField({
              field: 'commands',
              value: [{
                command: '',
                exit: 0,
              },],
            },),),).toContain('commands entry 1.command is no text');
            expect(messageOf(withTheField({
              field: 'commands',
              value: [{
                command: 'node repro.ts',
                exit: -2,
              },],
            },),),).toContain('commands entry 1.exit is no whole number from zero');
          },
        },),
        it({
          name: 'refuses a command entry naming a field the shape does not carry',
          fn: async () => {
            expect(messageOf(withTheField({
              field: 'commands',
              value: [{
                command: 'node repro.ts',
                exit: 0,
                stderr: 'nothing',
              },],
            },),),).toContain('commands entry 1 names a field this reader does not read: stderr');
          },
        },),
        it({
          name: 'refuses a carried built-program path that is blank, and reads one that is not',
          fn: async () => {
            expect(messageOf(withTheField({
              field: 'commands',
              value: [{
                command: 'node repro.ts',
                exit: 0,
                ranBuiltProgram: ' ',
              },],
            },),),).toContain('commands entry 1.ranBuiltProgram is no text');
            expect(statementInReview({
              shown: withTheField({
                field: 'commands',
                value: [{
                  command: 'node repro.ts',
                  exit: 0,
                  ranBuiltProgram: '/w/target/debug/w',
                },],
              },),
              file: aFile(),
            },).commands,).toEqual([{
              command: 'node repro.ts',
              exit: 0,
              ranBuiltProgram: '/w/target/debug/w',
            },],);
          },
        },),
        it({
          name: 'refuses a finding whose shownBy is neither run nor reading, and a finding entry with no name',
          fn: async () => {
            expect(messageOf(withTheField({
              field: 'findings',
              value: [{
                finding: '1',
                shownBy: 'guess',
              },],
            },),),).toContain('findings entry 1.shownBy is "guess", which is not one of run, reading');
            expect(messageOf(withTheField({
              field: 'findings',
              value: [{
                finding: '',
                shownBy: 'run',
              },],
            },),),).toContain('findings entry 1.finding is no text');
          },
        },),
        it({
          name: 'reads a review that closed with no finding at all, which is a good result',
          fn: async () => {
            expect(statementInReview({
              shown: aReviewFile({
                ...aStatement(),
                findings: [],
              },),
              file: aFile(),
            },).findings,).toEqual([],);
          },
        },),
        //endregion Each field
      ],
    },),
    describe({
      name: commitIsTheSame.name,
      children: [
        it({
          name: 'reads an abbreviated commit as the same one, and two different commits as different',
          fn: async () => {
            expect(commitIsTheSame({
              stated: 'abc1234',
              recorded: 'abc1234def5678',
            },),).toBe(true,);
            expect(commitIsTheSame({
              stated: 'abc1234def5678',
              recorded: 'abc1234',
            },),).toBe(true,);
            expect(commitIsTheSame({
              stated: 'abc1234',
              recorded: 'ffffffff',
            },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
