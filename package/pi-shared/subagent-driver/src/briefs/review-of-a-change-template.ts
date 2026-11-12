/**
 The text of every review's brief, with one `@NAME@` placeholder for each part a review fills.

 The backticks around code in the prose are escaped, because the text is a template literal; every other character is the brief
 exactly as a reviewer reads it. `review-brief.ts` fills it, and its test reads the placeholders out of this text so a
 placeholder nobody fills fails a test rather than a run.

 @module
 */

/**
 The review brief's text, with its placeholders.
 */
export const REVIEW_OF_A_CHANGE_TEMPLATE: string = `Read \`@RULES@\` first and follow it.

Never generate synthetic host load in any session or subagent: no \`stress-ng\`, busy loops, \`--stress-count\`, or repeated full test runs; cap the test's own container (\`--cpus\` sized to the test) instead.

You review one change to the code at commit @COMMIT@. The folder you start in is at that commit.

Never start a service manager on this machine (\`systemd --user\`, \`/usr/lib/systemd/systemd\` in any mode), and never request a well-known name on this machine's session bus.

Your review counts only if it is its own. Do not read anything under \`/var/home/user/temp/agent/kimi-reviews\`, \`/var/home/user/temp/agent/qwen-trials\`, or \`@REVIEWS_FOLDER@\` outside your own review folder, or any other run's folder under \`@RUNS_FOLDER@\`. Those hold other reviewers' answers. Do not run \`ps\` with \`args\`, or anything else that lists other processes' command lines. Look only at the commit you start on and the diff you are given: no other branch or worktree, no GitHub (\`gh\`), no web search about this repository, no session logs under \`~/.claude\` or \`~/.pi\`. Every claim that you reproduced something must match a command you really ran. Your whole session can be read afterwards.

Change no file in this worktree, and do not commit or push. Scratch goes under \`@SCRATCH_FOLDER@\`, and every command's working folder stays inside \`@SCRATCH_FOLDER@\` or this worktree, so work you leave running is still yours and the launcher resumes you when it ends. You may build and run the code, but only on disposable repositories under a disposable \`HOME\` and \`XDG_RUNTIME_DIR\`, in a command limited with \`systemd-run --user --scope\`. Never print the environment or read a credential file. If the model provider refuses or flags your content, stop and say so.

Review @SUBJECT@.
Read each of these files in full before you answer:
@FILES@
It must meet these requirements: "never discard user data", "Commits should never fail.", and "nothing should ever wait forever or indefinitely".
A process can be killed at any instruction, and other commands run at the same time.
Report only concrete defects. For each one give the file and line, the exact sequence of steps, and the wrong result, and say whether you reproduced it or found it by reading. Put each reproduction script in \`@REPRODUCTIONS_FOLDER@\`.
Leave out style, naming, and anything you cannot state as a sequence and a wrong result. Finding nothing is a good result.
Look hardest at: @FOCUS@

Write your review to \`@REVIEW_FILE@\` as soon as your reading is done, marked at its top as not final, and replace it when the runs you started have finished; write your review file before anything else you do at the end. A review that ends with no file is lost, and nobody can read what it found.`;
