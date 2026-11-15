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

The change is @RANGES@. Its whole diff, of what it changes under the folder you review, is the file \`@DIFF_FILE@\` beside this brief, which is the diff you are given. The files listed below are the files the change touches that a review reads, which are the code and the files that build, check or run it; it also touches tests, fixtures and notes, and those are not listed. The last commit alone is not the change: a review of that commit's own diff reviews a part of it.

Never start a service manager on this machine (\`systemd --user\`, \`/usr/lib/systemd/systemd\` in any mode), and never request a well-known name on this machine's session bus.

Your review counts only if it is its own. Do not read anything under \`/var/home/user/temp/agent/kimi-reviews\`, \`/var/home/user/temp/agent/qwen-trials\`, or \`@REVIEWS_FOLDER@\` outside your own review folder, or any other run's folder under \`@RUNS_FOLDER@\`. Those hold other reviewers' answers. Do not run \`ps\` with \`args\`, or anything else that lists other processes' command lines. Look only at the code at the commit you start on, the diff you are given and the files listed below: no other branch or worktree, no GitHub (\`gh\`), no web search about this repository, no session logs under \`~/.claude\` or \`~/.pi\`. Every claim that you reproduced something must match a command you really ran. Your whole session can be read afterwards.

Change no file this checkout tracks in this worktree, and do not commit or push; a build writes its own files under the folders this checkout ignores, which is not a change to it. Scratch goes under \`@SCRATCH_FOLDER@\`, and every command's working folder stays inside \`@SCRATCH_FOLDER@\` or this worktree, so work you leave running is still yours and the launcher resumes you when it ends. You may build and run the code, but only on disposable repositories under a disposable \`HOME\` and \`XDG_RUNTIME_DIR\`, in a command of its own limited as \`systemd-run --user --scope -p CPUQuota=400% -p MemoryMax=8G -p TasksMax=1024 -p IOWeight=1 -- <command>\`, with the numbers sized to what the command does. A disposable \`HOME\` holds no key for this checkout's encrypted environment file, so mise stops on that file and every task with it: run a package's task as \`MISE_SOPS_STRICT=false mise --yes run --skip-tools <task>\` in the worktree, where \`<task>\` is one the package's own \`mise.toml\` names, such as \`//package/pi-shared/subagent-driver:test:unit\`. That setting tells mise to skip the file it cannot decrypt rather than fail, so no credential is read, and \`--skip-tools\` keeps mise from installing the toolchain into the disposable home. Never print the environment or read a credential file. If the model provider refuses or flags your content, stop and say so.

Review @SUBJECT@.
Read each of these files in full before you answer:
@FILES@
It must meet these requirements: "never discard user data", "Commits should never fail.", and "nothing should ever wait forever or indefinitely".
A process can be killed at any instruction, and other commands run at the same time.
Report only concrete defects. For each one give the file and line, the exact sequence of steps, and the wrong result, and say whether you reproduced it or found it by reading. Put each reproduction script in \`@REPRODUCTIONS_FOLDER@\`.
Leave out style, naming, and anything you cannot state as a sequence and a wrong result. Finding nothing is a good result once you have read every file listed above and the whole diff.
Look hardest at: @FOCUS@

Write your review to \`@REVIEW_FILE@\` as soon as your reading is done, marked at its top as not final, and with these two lines at its top, under that mark, so whoever reads the verdict can tell a whole review from a part of one:
Ranges reviewed: @RANGES_LIST@
Files read in full: <how many of the files above you read in full> of @FILE_COUNT@
Replace the file when the runs you started have finished; write your review file before anything else you do at the end. A review that ends with no file is lost, and nobody can read what it found. A review whose lines name another range than the ones above, or name no range at all, is read afterwards as a review that did not cover its change.`;
