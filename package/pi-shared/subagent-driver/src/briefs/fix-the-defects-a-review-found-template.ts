/**
 The text of the brief every fix agent gets, with one `@NAME@` placeholder for each part a caller fills.

 The backticks around code in the prose are escaped, because the text is a template literal; every other character is the brief
 exactly as a fix agent reads it. `fix-brief.ts` fills it, and its test reads the placeholders out of this text so a placeholder
 nobody fills fails a test rather than a run.

 @module
 */

/**
 The fix brief's text, with its placeholders.
 */
export const FIX_THE_DEFECTS_A_REVIEW_FOUND_TEMPLATE: string = `Read \`@RULES@\` first and follow it.

Never generate synthetic host load in any session or subagent: no \`stress-ng\`, busy loops, \`--stress-count\`, or repeated full test runs; cap the test's own container (\`--cpus\` sized to the test) instead.

You are working on \`@PACKAGE@\` in the worktree \`@WORKTREE@\`, on branch \`@BRANCH@\`. It starts at \`@BASE@\`, the commit the review read. Run every command and make every edit inside that worktree.

## The task

A review of \`@REVIEW_SUBJECT@\` reported defects. Its text is \`@REVIEW_FILE@\`, the change it read is \`@DIFF_FILE@\` beside it, and its reproduction scripts are in \`@REPRODUCTIONS_FOLDER@\` beside it. Read all three.

The project's requirements the review judged against, which also bind your fix: never discard user data; a commit never fails where real Git would succeed; nothing waits forever; no command waits on another agent's command; the code cleans up after itself.

For each reported defect, in the review's order:

1.  Reproduce it yourself, on a disposable fixture with its own \`HOME\` and \`XDG_RUNTIME_DIR\`, against the code built from your worktree. A reviewer that is asked for defects reports some that do not hold: when one does not reproduce, write down exactly what you ran and what happened, and leave that code alone.
2.  When it reproduces, say in one sentence which requirement it breaks and how a real use would reach it. A defect that needs a deliberately hostile setup nobody would have still counts when it loses data; otherwise say so and weigh the fix against its cost.
3.  Fix it with a test shown failing first, in the package's own tests beside the code the defect lives in. Fix the cause in the code that owns it; do not add a special case that only makes the review's script pass. Look for the same mistake beside it and fix those places too, saying where.

Then:

4.  Run the package's own lint and the tests of the modules you changed (\`@PACKAGE@/mise.toml\` names the tasks), and read each summary. A green run counts only when its output names the tests you added.
5.  Commit as you go and let the wrapper push. Do not merge into \`main\` or into any integration branch.

Other agents are changing neighbouring code on other branches at the same time. Keep your change to what these defects need; a later merge joins the branches.

## Report

End with, for each defect of the review: reproduced or not (what you ran), the requirement it breaks, the fix and its commit, the test and its failing output before the fix, and any place beside it that had the same mistake. Then the summary lines of the lint and the tests. Take the time the work needs; the best result matters more than a fast one.`;
