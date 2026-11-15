/**
 The text of every review's brief, in the parts every review reads and the parts only a review of a change to tests alone adds.

 A change of code is judged by what it does to the data and the waits it holds, so its brief names the requirements the project
 states and leaves the tests out: a reviewer cannot judge a test as the change, and the package's own tests run it.

 A change of tests and fixtures alone has no code to judge, so its brief asks for the one thing a reviewer can show about a test:
 a break planted under it, and whether the test fails with that break in. A test that still passes with a break under it is what
 that brief exists to find, and a review that ran nothing has shown nothing. A change that holds a test is judged so even when it
 also holds code: both briefs then ask the project's requirements, because a reviewer reads the code as well.

 Every brief also names the files the change touched that it does not read, with the reason, so a line of prose beside a test is
 never the reason a test goes unreviewed, and the files the change removed, which are read in its diff because the commit no longer
 holds them.

 The backticks around code in the prose are escaped, because the text is a template literal; every other character is the brief
 exactly as a reviewer reads it. `review-brief.ts` fills it, and its test reads the placeholders out of this text so a
 placeholder nobody fills fails a test rather than a run.

 @module
 */

/**
 The opening every review reads before its own task: the rules file first, the ban on synthetic host load, the commit the review
 starts at, the ban on a service manager, the folders whose answers are another reviewer's, and where the review's own scratch
 goes.
 */
const THE_RULES_AND_WHERE_THE_REVIEW_WORKS: string = `Read \`@RULES@\` first and follow it.

Never generate synthetic host load in any session or subagent: no \`stress-ng\`, busy loops, \`--stress-count\`, or repeated full test runs; cap the test's own container (\`--cpus\` sized to the test) instead.

You review one change to the code at commit @COMMIT@. The folder you start in is at that commit.

Never start a service manager on this machine (\`systemd --user\`, \`/usr/lib/systemd/systemd\` in any mode), and never request a well-known name on this machine's session bus.

Your review counts only if it is its own. Do not read anything under \`/var/home/user/temp/agent/kimi-reviews\`, \`/var/home/user/temp/agent/qwen-trials\`, or \`@REVIEWS_FOLDER@\` outside your own review folder, or any other run's folder under \`@RUNS_FOLDER@\`. Those hold other reviewers' answers. Do not run \`ps\` with \`args\`, or anything else that lists other processes' command lines. Look only at the commit you start on and the diff you are given: no other branch or worktree, no GitHub (\`gh\`), no web search about this repository, no session logs under \`~/.claude\` or \`~/.pi\`. Every claim that you reproduced something must match a command you really ran. Your whole session can be read afterwards.

Change no file in this worktree, and do not commit or push. Scratch goes under \`@SCRATCH_FOLDER@\`, and every command's working folder stays inside \`@SCRATCH_FOLDER@\` or this worktree, so work you leave running is still yours and the launcher resumes you when it ends. You may build and run the code, but only on disposable repositories under a disposable \`HOME\` and \`XDG_RUNTIME_DIR\`, in a command limited with \`systemd-run --user --scope\`. Never print the environment or read a credential file. If the model provider refuses or flags your content, stop and say so.`;

/**
 The sentences that name the change under review and the files the reviewer reads, in every brief.
 */
const THE_CHANGE_AND_THE_FILES_TO_READ: string = `Review @SUBJECT@.
Read each of these files in full before you answer:
@FILES@`;

/**
 The paragraph that says what the change also touched and this review does not read, with the reason for each such file.
 */
const THE_FILES_LEFT_OUT: string = `@LEFT_OUT@`;

/**
 The paragraph that says which files the change removed, which the review reads in the change's diff because the commit no longer
 holds them.
 */
const THE_FILES_REMOVED: string = `@REMOVED@`;

/**
 The requirements the change is judged by, which `review-brief.ts` fills: the project's own for the code, whether the change is
 judged as code or as tests with code beside them, and, for a change to tests, that each changed test still fails under a break.
 */
const WHAT_THE_CHANGE_MUST_MEET: string = `@REQUIREMENTS@`;

/**
 The sentences that say how a defect is reported and what is left out, in every brief.
 */
const HOW_A_DEFECT_IS_REPORTED: string =
  'A process can be killed at any instruction, and other commands run at the same time.\n'
    + 'Report only concrete defects. For each one give the file and line, the exact sequence of steps, and the wrong result, and '
    + 'say whether you reproduced it or found it by reading. Put each reproduction script in `@REPRODUCTIONS_FOLDER@`.\n'
    + 'Leave out style, naming, and anything you cannot state as a sequence and a wrong result. Finding nothing is a good result.';

/**
 The demand only a change to tests alone carries: the breaks a reviewer plants, the runs that show whether each test catches
 one, and the report of both.
 */
const THE_BREAKS_TO_PLANT_UNDER_CHANGED_TESTS: string =
  'For each changed test above, plant a break under it, in a copy of the code it tests or in the value it expects, run that '
    + 'test by name, and say whether it fails: a test that still passes with the break in is a finding. For a changed fixture, '
    + 'name the tests that use it, plant a break in a copy of the fixture, run those tests by name, and say whether one fails. '
    + 'Say what you read and what you ran, because a review that ran nothing has shown nothing.';

/**
 The closing line that names what the reviewer looks hardest at, in every brief.
 */
const WHAT_TO_LOOK_HARDEST_AT: string = 'Look hardest at: @FOCUS@';

/**
 The last paragraph every review reads: the review file, written before anything else at the end.
 */
const THE_REVIEW_FILE: string = `Write your review to \`@REVIEW_FILE@\` as soon as your reading is done, marked at its top as not final, and replace it when the runs you started have finished; write your review file before anything else you do at the end. A review that ends with no file is lost, and nobody can read what it found.`;

/**
 The brief of a change to code: the code and the build decisions under the source folder, judged by the project's requirements.
 */
export const REVIEW_OF_A_CHANGE_TEMPLATE: string = `${THE_RULES_AND_WHERE_THE_REVIEW_WORKS}

${THE_CHANGE_AND_THE_FILES_TO_READ}
${THE_FILES_LEFT_OUT}
${THE_FILES_REMOVED}
${WHAT_THE_CHANGE_MUST_MEET}
${HOW_A_DEFECT_IS_REPORTED}
${WHAT_TO_LOOK_HARDEST_AT}

${THE_REVIEW_FILE}`;

/**
 The brief of a change to tests and fixtures alone: every changed test and fixture, judged by whether each test still fails
 under a break planted beneath it.
 */
export const REVIEW_OF_A_CHANGE_TO_TESTS_TEMPLATE: string = `${THE_RULES_AND_WHERE_THE_REVIEW_WORKS}

${THE_CHANGE_AND_THE_FILES_TO_READ}
${THE_FILES_LEFT_OUT}
${THE_FILES_REMOVED}
${WHAT_THE_CHANGE_MUST_MEET}
${HOW_A_DEFECT_IS_REPORTED}
${THE_BREAKS_TO_PLANT_UNDER_CHANGED_TESTS}
${WHAT_TO_LOOK_HARDEST_AT}

${THE_REVIEW_FILE}`;
