/**
 The check of an ended review against the files and run it left: each verdict, from a review file and a session this test writes
 itself, and what the check says it cannot see.

 Every fixture here is a folder the test makes and throws away, and every session is a handful of synthetic records: no real
 session, run folder or review is read, and nothing starts a run, a unit or a Pi.

 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  CHANGE_DIFF_FILE_NAME,
  CHECK_LIMITS_LINE,
  REVIEW_FILE_NAME,
  REVIEW_READING_FILE_NAME,
  REVIEW_STATEMENT_HEADING,
  REVIEW_STATEMENT_SKELETON,
  checkAReview,
  checkLineOf,
} from '../dist/final/node/index.mjs';

/**
 The change every fixture reviews.

 @returns the review's name

 @example
 ```ts
 const review = aReview();
 ```
 */
function aReview(): string {
  return 'a-round-1';
}

/**
 The commit every fixture reviews.

 @returns the commit's id

 @example
 ```ts
 const commit = aCommit();
 ```
 */
function aCommit(): string {
  return 'abc1234def5678';
}

/**
 The folder the fixture worktree stands for, which a built program's path is read against.

 @returns the folder's path

 @example
 ```ts
 const worktree = aWorktree();
 ```
 */
function aWorktree(): string {
  return '/worktrees/pi-review-a';
}

/**
 The program a review builds and runs, which is what makes a review one that ran something.

 @returns the program's path

 @example
 ```ts
 const program = aBuiltProgram();
 ```
 */
function aBuiltProgram(): string {
  return `${aWorktree()}/package/git-policy/cli/target/debug/monochromatic-git-policy-cli`;
}

/**
 One tool call a fixture session records.
 */
type FixtureCall = {
  /**
   The call's id, which its result names.
   */
  readonly id: string;

  /**
   The command the call ran.

   @default the build-and-run command of a review
   */
  readonly command?: string;

  /**
   The code the call's result records; absent for a result with no code line and no error.

   @default 0
   */
  readonly exit?: number;

  /**
   The tool the call came from.

   @default 'bash'
   */
  readonly tool?: 'bash' | 'process';
};

/**
 The build-and-run command every fixture's successful review runs.

 @returns the command

 @example
 ```ts
 const command = aBuiltRun();
 ```
 */
function aBuiltRun(): string {
  return `${aBuiltProgram()} commit -m a`;
}

/**
 Reads one line of a JSON-serialized record.

 @param record - the record to write

 @returns the line, with its newline

 @example
 ```ts
 const line = lineOf({ type: 'session' });
 ```
 */
function lineOf(record: Readonly<Record<string, unknown>>,): string {
  return `${JSON.stringify(record,)}\n`;
}

/**
 Builds the text of a synthetic session file.

 @param calls - the tool calls the session records, in order

 @returns the session's text, one JSON record per line

 @example
 ```ts
 const shown = sessionText([{ id: 'c1' }]);
 ```
 */
function sessionText({ calls, }: { readonly calls: readonly FixtureCall[] },): string {
  const lines: string[] = [lineOf({
    type: 'session',
    version: 3,
    id: 'fixture-session',
    cwd: aWorktree(),
  },),];
  for (const call of calls) {
    lines.push(lineOf({
      type: 'message',
      message: {
        role: 'assistant',
        content: [{
          type: 'toolCall',
          id: call.id,
          name: call.tool ?? 'bash',
          arguments: {
            command: call.command ?? aBuiltRun(),
            ...((call.tool === 'process') ? { action: 'start', } : {}),
          },
        },],
      },
    },),);
    lines.push(lineOf({
      type: 'message',
      message: {
        role: 'toolResult',
        toolCallId: call.id,
        ...(call.tool === 'process' ? {} : { toolName: 'bash', }),
        content: [{
          type: 'text',
          text: (call.exit === undefined) ? '(no output)' : `(no output)\n\nCommand exited with code ${String(call.exit,)}`,
        },],
        isError: (call.exit ?? 0) !== 0,
      },
    },),);
  }
  return lines.join('',);
}

/**
 The diff every fixture review was given.

 @returns the diff's text, touching two files

 @example
 ```ts
 const diff = aDiff();
 ```
 */
function aDiff(): string {
  return [
    'diff --git a/package/a/src/one.rs b/package/a/src/one.rs',
    '--- a/package/a/src/one.rs',
    '+++ b/package/a/src/one.rs',
    '@@ -1 +1 @@',
    '-old',
    '+new',
    'diff --git a/package/a/src/two.rs b/package/a/src/two.rs',
    '--- a/package/a/src/two.rs',
    '+++ b/package/a/src/two.rs',
    '@@ -1 +1 @@',
    '-old',
    '+new',
    '',
  ].join('\n',);
}

/**
 The number of lines of {@link aDiff}.

 @returns the diff's lines, as `wc --lines` counts them

 @example
 ```ts
 const lines = aDiffLines();
 ```
 */
function aDiffLines(): number {
  return aDiff()
    .split('\n',)
    .length - 1;
}

/**
 One fixture: a reviews folder, a runs folder and the session file a run names, all under one folder the test throws away.
 */
type Fixture = {
  /**
   The folder everything is written under.
   */
  readonly root: string;

  /**
   The folder that holds one folder per review.
   */
  readonly reviewsFolder: string;

  /**
   The folder the runs live in.
   */
  readonly runsFolder: string;

  /**
   The review's own folder.
   */
  readonly reviewFolder: string;

  /**
   How to remove the whole folder.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Makes a fixture folder.

 @returns the folder, and how to remove it

 @example
 ```ts
 await using fixture = await makeFixture();
 ```
 */
async function makeFixture(): Promise<Fixture> {
  const root = await mkdtemp(join(scratchRoot(), 'pi-review-check-',),);
  const reviewsFolder = join(root, 'reviews',);
  const runsFolder = join(root, 'runs',);
  const reviewFolder = join(reviewsFolder, aReview(),);
  await mkdir(reviewFolder, { recursive: true, },);
  await mkdir(runsFolder, { recursive: true, },);
  return {
    root,
    reviewsFolder,
    runsFolder,
    reviewFolder,
    async [Symbol.asyncDispose]() {
      await rm(
        root,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Writes what every review records it read.

 @param fixture - the fixture to write into

 @example
 ```ts
 await writeTheReading({ fixture });
 ```
 */
async function writeTheReading({ fixture, }: { readonly fixture: Fixture },): Promise<void> {
  await writeFile(
    join(fixture.reviewFolder, REVIEW_READING_FILE_NAME,),
    `${JSON.stringify({
      review: aReview(),
      commit: aCommit(),
      source: 'package/a/src',
      ranges: ['main...fix/a',],
    },)}\n`,
  );
  await writeFile(
    join(fixture.reviewFolder, CHANGE_DIFF_FILE_NAME,),
    aDiff(),
  );
}

/**
 Writes a finished run of the review, with the session it names.

 @param fixture - the fixture to write into

 @param endReason - why the summary says the run ended

 @param calls - the tool calls the session records

 @param attempt - the run's number

 @returns the session file's path

 @example
 ```ts
 await writeARun({ fixture, endReason: 'idle', calls: [{ id: 'c1' }] });
 ```
 */
async function writeARun({
  fixture,
  endReason,
  calls,
  attempt = 1,
}: {
  readonly fixture: Fixture;
  readonly endReason: string;
  readonly calls: readonly FixtureCall[];
  readonly attempt?: number;
},): Promise<string> {
  const runFolder = join(
    fixture.runsFolder,
    `review-${aReview()}-${String(attempt,)}`,
  );
  const sessionFile = join(
    fixture.root,
    `session-${String(attempt,)}.jsonl`,
  );
  await mkdir(runFolder, { recursive: true, },);
  await writeFile(sessionFile, sessionText({ calls, },),);
  await writeFile(
    join(runFolder, 'summary.json',),
    `${JSON.stringify({
      endReason,
      turns: 40,
      sessionStats: { sessionFile, },
      lastAssistantText: 'a review',
    },)}\n`,
  );
  return sessionFile;
}

/**
 Reads the statement a fixture review writes.

 @param overrides - the fields to replace in the statement

 @returns the statement, as the review file's block carries it

 @example
 ```ts
 const statement = aStatement({ diff: { lines: 6, linesRead: 2 } });
 ```
 */
function aStatement(overrides: Readonly<Record<string, unknown>> = {},): Readonly<Record<string, unknown>> {
  return {
    commit: aCommit(),
    diff: {
      lines: aDiffLines(),
      linesRead: 2,
    },
    files: [
      {
        path: 'package/a/src/one.rs',
        read: 'whole',
      },
      {
        path: 'package/a/src/two.rs',
        read: 'part',
      },
    ],
    commands: [
      {
        command: aBuiltRun(),
        exit: 0,
        ranBuiltProgram: aBuiltProgram(),
      },
    ],
    findings: [
      {
        finding: '1',
        shownBy: 'run',
      },
    ],
    ...overrides,
  };
}

/**
 Writes the review file a fixture review writes.

 @param fixture - the fixture to write into

 @param section - the statement section's text, which the heading and its fences are added around

 @example
 ```ts
 await writeTheReview({ fixture, section: '```review-statement\n{}\n```' });
 ```
 */
async function writeTheReview({
  fixture,
  section,
}: {
  readonly fixture: Fixture;
  readonly section: string;
},): Promise<void> {
  await writeFile(
    join(fixture.reviewFolder, REVIEW_FILE_NAME,),
    `# Review: a change\n\n${REVIEW_STATEMENT_HEADING}\n\n${section}\n\n## What I found\n\nNothing.\n`,
  );
}

/**
 Reads the statement section a fixture review writes for one statement.

 @param statement - the statement to write

 @returns the section's text, fences included

 @example
 ```ts
 const section = aSection(aStatement());
 ```
 */
function aSection(statement: Readonly<Record<string, unknown>>,): string {
  return `\`\`\`review-statement\n${JSON.stringify(
    statement,
    undefined,
    2
  )}\n\`\`\``;
}

await describe({
  name: '',
  children: [
    describe({
      name: 'checkAReview',
      children: [
        //region The five verdicts
        it({
          name: 'answers complete when the review file, its statement and its session agree and a command ran a program the worktree built',
          fn: async () => {
            await using fixture = await makeFixture();
            await writeTheReading({ fixture, },);
            await writeARun({
              fixture,
              endReason: 'idle',
              calls: [{ id: 'call-build', },],
            },);
            await writeTheReview({
              fixture,
              section: aSection(aStatement(),),
            },);
            const check = await checkAReview({
              review: aReview(),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
            },);
            expect(check.verdict,).toBe('complete',);
            expect(check.run,).toBe(`review-${aReview()}-1`,);
            expect(check.builtPrograms,).toEqual([aBuiltProgram(),],);
            expect(check.coverage?.filesNotReadWhole,).toEqual(['package/a/src/two.rs',],);
            expect(checkLineOf({ check, },),).toContain(`${aBuiltProgram()}`,);
            expect(check.reasons.join('\n',),).toContain(`read 2 of the diff's ${String(aDiffLines(),)} lines`,);
          },
        },),
        it({
          name: 'answers ran nothing when the statement names no built program and no session call ran one',
          fn: async () => {
            await using fixture = await makeFixture();
            await writeTheReading({ fixture, },);
            await writeARun({
              fixture,
              endReason: 'idle',
              calls: [{ id: 'call-read', command: 'rg --count change package/a/src', },],
            },);
            await writeTheReview({
              fixture,
              section: aSection(aStatement({
                commands: [{
                  command: 'rg --count change package/a/src',
                  exit: 0,
                },],
              },),),
            },);
            const check = await checkAReview({
              review: aReview(),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
            },);
            expect(check.verdict,).toBe('ran nothing',);
            expect(check.reasons[0],).toContain('ran a program built from commit',);
            // The coverage is still named, so a second review knows what is left even when the first ran nothing.
            expect(check.coverage?.filesNotReadWhole,).toEqual(['package/a/src/two.rs',],);
          },
        },),
        it({
          name: 'answers statement missing or not matching its session when the section is absent, naming the heading',
          fn: async () => {
            await using fixture = await makeFixture();
            await writeTheReading({ fixture, },);
            await writeARun({
              fixture,
              endReason: 'idle',
              calls: [{ id: 'call-build', },],
            },);
            await writeFile(
              join(fixture.reviewFolder, REVIEW_FILE_NAME,),
              '# Review: a change\n\n## Findings\n\nNothing.\n',
            );
            const check = await checkAReview({
              review: aReview(),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
            },);
            expect(check.verdict,).toBe('statement missing or not matching its session',);
            expect(check.reasons[0],).toContain('## Findings',);
            expect(check.reasons[0],).toContain(REVIEW_STATEMENT_HEADING,);
          },
        },),
        it({
          name: 'answers no review file when the run ended by itself and wrote none',
          fn: async () => {
            await using fixture = await makeFixture();
            await writeTheReading({ fixture, },);
            await writeARun({
              fixture,
              endReason: 'idle',
              calls: [{ id: 'call-build', },],
            },);
            const check = await checkAReview({
              review: aReview(),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
            },);
            expect(check.verdict,).toBe('no review file',);
            expect(check.reasons[0],).toContain('ended idle',);
            expect(check.reasons[0],).toContain('wrote no review.md',);
          },
        },),
        it({
          name: 'answers cut when the run ended on a cap rather than by itself, even though a review file is there',
          fn: async () => {
            await using fixture = await makeFixture();
            await writeTheReading({ fixture, },);
            await writeARun({
              fixture,
              endReason: 'turn-cap',
              calls: [{ id: 'call-build', },],
            },);
            await writeTheReview({
              fixture,
              section: aSection(aStatement(),),
            },);
            const check = await checkAReview({
              review: aReview(),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
            },);
            expect(check.verdict,).toBe('cut',);
            expect(check.reasons[0],).toContain('ended turn-cap',);
            expect(check.reasons[0],).toContain('was cut',);
          },
        },),
        it({
          name: 'answers cut when no run of the review ever finished, so no session says what it ran',
          fn: async () => {
            await using fixture = await makeFixture();
            await writeTheReading({ fixture, },);
            await writeTheReview({
              fixture,
              section: aSection(aStatement(),),
            },);
            const check = await checkAReview({
              review: aReview(),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
            },);
            expect(check.verdict,).toBe('cut',);
            expect(check.reasons[0],).toContain('no finished run',);
          },
        },),
        //endregion The five verdicts

        //region What the check compares
        it({
          name: 'refuses a statement whose command its session never records, quoting the command',
          fn: async () => {
            await using fixture = await makeFixture();
            await writeTheReading({ fixture, },);
            await writeARun({
              fixture,
              endReason: 'idle',
              calls: [{ id: 'call-build', },],
            },);
            await writeTheReview({
              fixture,
              section: aSection(aStatement({
                commands: [{
                  command: `${aBuiltProgram()} push --anything`,
                  exit: 0,
                  ranBuiltProgram: aBuiltProgram(),
                },],
              },),),
            },);
            const check = await checkAReview({
              review: aReview(),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
            },);
            expect(check.verdict,).toBe('statement missing or not matching its session',);
            expect(check.reasons.join('\n',),).toContain('the session records no call whose text holds it');
          },
        },),
        it({
          name: 'refuses a statement whose exit code is not the one the session result records',
          fn: async () => {
            await using fixture = await makeFixture();
            await writeTheReading({ fixture, },);
            await writeARun({
              fixture,
              endReason: 'idle',
              calls: [{
                id: 'call-build',
                exit: 3,
              },],
            },);
            await writeTheReview({
              fixture,
              section: aSection(aStatement({
                commands: [{
                  command: aBuiltRun(),
                  exit: 0,
                  ranBuiltProgram: aBuiltProgram(),
                },],
              },),),
            },);
            const check = await checkAReview({
              review: aReview(),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
            },);
            expect(check.verdict,).toBe('statement missing or not matching its session',);
            expect(check.reasons.join('\n',),).toContain('exited 0 and the session records 3');
          },
        },),
        it({
          name: 'refuses a statement that says it read whole a file its session never read, when it also leaves a diff file out',
          fn: async () => {
            await using fixture = await makeFixture();
            await writeTheReading({ fixture, },);
            await writeARun({
              fixture,
              endReason: 'idle',
              calls: [{ id: 'call-build', },],
            },);
            await writeTheReview({
              fixture,
              section: aSection(aStatement({
                files: [{
                  path: 'package/a/src/one.rs',
                  read: 'whole',
                },],
              },),),
            },);
            const check = await checkAReview({
              review: aReview(),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
            },);
            expect(check.verdict,).toBe('statement missing or not matching its session',);
            expect(check.reasons.join('\n',),).toContain('does not say how package/a/src/two.rs was read');
          },
        },),
        it({
          name: 'refuses a statement whose line count is not the diff file\'s',
          fn: async () => {
            await using fixture = await makeFixture();
            await writeTheReading({ fixture, },);
            await writeARun({
              fixture,
              endReason: 'idle',
              calls: [{ id: 'call-build', },],
            },);
            await writeTheReview({
              fixture,
              section: aSection(aStatement({
                diff: {
                  lines: aDiffLines() + 1,
                  linesRead: 2,
                },
              },),),
            },);
            const check = await checkAReview({
              review: aReview(),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
            },);
            expect(check.verdict,).toBe('statement missing or not matching its session',);
            expect(check.reasons.join('\n',),).toContain('the statement says the diff holds');
          },
        },),
        it({
          name: 'refuses a session call that ran a built program the statement does not name as one that did',
          fn: async () => {
            await using fixture = await makeFixture();
            await writeTheReading({ fixture, },);
            await writeARun({
              fixture,
              endReason: 'idle',
              calls: [
                { id: 'call-build', },
                {
                  id: 'call-push',
                  command: `${aBuiltProgram()} push --dry-run`,
                },
              ],
            },);
            await writeTheReview({
              fixture,
              section: aSection(aStatement(),),
            },);
            const check = await checkAReview({
              review: aReview(),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
            },);
            expect(check.verdict,).toBe('statement missing or not matching its session',);
            expect(check.reasons.join('\n',),).toContain('and the statement does not name that command as one that ran a built program');
          },
        },),
        it({
          name: 'confirms a built program a command names only through a script the command runs',
          fn: async () => {
            await using fixture = await makeFixture();
            await writeTheReading({ fixture, },);
            await writeFile(
              join(fixture.reviewFolder, 'repro.ts',),
              `const WRAPPER = '${aBuiltProgram()}';\n`,
            );
            await writeARun({
              fixture,
              endReason: 'idle',
              calls: [{
                id: 'call-script',
                command: 'node repro.ts',
              },],
            },);
            await writeTheReview({
              fixture,
              section: aSection(aStatement({
                commands: [{
                  command: 'node repro.ts',
                  exit: 0,
                  ranBuiltProgram: aBuiltProgram(),
                },],
              },),),
            },);
            const check = await checkAReview({
              review: aReview(),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
            },);
            expect(check.verdict,).toBe('complete',);
            expect(check.builtPrograms,).toEqual([aBuiltProgram(),],);
          },
        },),
        it({
          name: 'says a process start\'s exit code cannot be compared rather than calling it a match',
          fn: async () => {
            await using fixture = await makeFixture();
            await writeTheReading({ fixture, },);
            await writeARun({
              fixture,
              endReason: 'idle',
              calls: [{
                id: 'call-process',
                tool: 'process',
              },],
            },);
            await writeTheReview({
              fixture,
              section: aSection(aStatement(),),
            },);
            const check = await checkAReview({
              review: aReview(),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
            },);
            expect(check.verdict,).toBe('complete',);
            expect(check.reasons.join('\n',),).toContain('records no exit code');
          },
        },),
        it({
          name: 'says a commit its record does not name cannot be compared, and a different commit as a mismatch',
          fn: async () => {
            await using fixture = await makeFixture();
            await writeTheReading({ fixture, },);
            await writeARun({
              fixture,
              endReason: 'idle',
              calls: [{ id: 'call-build', },],
            },);
            await writeTheReview({
              fixture,
              section: aSection(aStatement(),),
            },);
            await rm(join(fixture.reviewFolder, REVIEW_READING_FILE_NAME,),);
            const withoutRecord = await checkAReview({
              review: aReview(),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
            },);
            expect(withoutRecord.verdict,).toBe('complete',);
            expect(withoutRecord.reasons.join('\n',),).toContain('so the commit the statement names cannot be compared');
            await writeTheReading({ fixture, },);
            await writeTheReview({
              fixture,
              section: aSection(aStatement({ commit: 'ffffffff', },),),
            },);
            const other = await checkAReview({
              review: aReview(),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
            },);
            expect(other.verdict,).toBe('statement missing or not matching its session',);
            expect(other.reasons.join('\n',),).toContain('the statement names commit ffffffff');
          },
        },),
        it({
          name: 'names in the verdict what it cannot see, and the line says so before any review is read',
          fn: async () => {
            expect(CHECK_LIMITS_LINE,).toContain('codemode',);
            expect(CHECK_LIMITS_LINE,).toContain('process start',);
          },
        },),
        it({
          name: 'says the skeleton copies into a section is a statement nobody wrote, naming the field that still holds a placeholder',
          fn: async () => {
            await using fixture = await makeFixture();
            await writeTheReading({ fixture, },);
            await writeARun({
              fixture,
              endReason: 'idle',
              calls: [{ id: 'call-build', },],
            },);
            await writeTheReview({
              fixture,
              section: `\`\`\`review-statement\n${REVIEW_STATEMENT_SKELETON}\n\`\`\``,
            },);
            const check = await checkAReview({
              review: aReview(),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
            },);
            expect(check.verdict,).toBe('statement missing or not matching its session',);
            expect(check.reasons.join('\n',),).toContain('diff.lines is no whole number from zero');
          },
        },),
        //endregion What the check compares
      ],
    },),
  ],
},);
