# Preventing mistakes: shell commands, edits, commits and agents

Rules for how the work is carried out:
shell commands,
lint and edits,
commit messages,
and work handed to agents.

Part of [Preventing this package's mistakes](mistake-prevention.md),
which lists every file of this record.
Its section "Questions to the owner" moved to
[Preventing mistakes: questions to the owner and rulings](mistake-prevention-questions-to-the-owner-and-rulings.md)
on 2026-10-29.

## Shell commands

What happened:
`;` joined commands dozens of times,
so a failed step's successor ran anyway and hid which failed;
heredocs and inline `python3 -c` scripts were chained to other commands;
foreground sleeps and do-nothing calls waited on background work;
git ran outside the repository root and cli-git refused it (M1,
M14,
M38,
M92);
a command ran in the same batch as the write it read,
and ran a stale file of the same name (M51).
A transcript census found 9,324 of 22,058 calls breaking the rule after it took its current wording,
and the rule itself misread as three `&&` rather than three steps (M1).
The slip recurred after every record of it,
four times,
then three more,
then three times after M116 was written,
across 2026-10-05 and 2026-10-06 (UTC) (ledger M107,
M116).
Each of those last three was a read of a log by two searches whose results were both wanted.
Five more followed around the merges of 2026-10-06 (UTC),
two of them a `;` after `rg --count` to get past its exit of 1 on a count of zero (ledger M116),
and two more after those,
each a read of a log between two steps of a merge (ledger M116).
An agent stopped its own census with `pkill -f` on a pattern that matched every agent's census on the machine
(ledger M126).
Loops,
a sleep,
polls of a running log and a heredoc chained to a second command came again,
from the lead and from agents (ledger M132).
The lead's came once more on 2026-10-06 (UTC):
calls chaining four steps with `&&`,
a heredoc with a second command,
a `git diff` run from a subdirectory,
and a count expected to be zero leading an `&&` chain it then stopped;
and agents chained three more reads or writes with `;` (ledger M116,
M139).
A background command of the lead's chained the source scans and the lint with `;` (ledger M140).
On 2026-10-10 (UTC) a `git add` of the lead's ran from inside the package directory,
and the git policy refused it as not at the root (ledger M141);
the agents writing the changes of ledger B365 to B378 chained six read-only calls with `;`,
one ran a heredoc and then `node` in one call,
and one edited files through inline `node --eval` scripts,
one of which lost its backticks to the shell.
On 2026-10-11 (UTC) the lead chained four `&&` steps in one call,
and created an agent worktree through the Git wrapper without `--no-worktree-copy`,
so it copied 888 MB of ignored logger files and held the settlement lock for over 45 minutes;
the script that sets up agent worktrees then ran under `mise exec`,
where `git` was plain Git and refused the flag (ledger M152).
On 2026-10-15 (UTC) the lead added a worktree from the keyed branch worktree without the flag once more,
and the wrapper exited 2 with `Permission denied` after the checkout;
the entry that was blamed first,
mode-locked test files,
was dangling symbolic links,
and the entry that stopped the copy was not traced (ledger M224).
Output to the terminal over 50 lines and Reads outside the limit of 50
came in six of the seven rounds from n162 to n168,
and several Edits to one file in one batch in five of them,
although every brief of those rounds stated each rule (ledger M227 to M233);
double-quoted patterns with a backtick or `$0` were interpreted by the shell in three of them (ledger M227,
M228,
M230).
In the work of B470 to B491 nine authors and reviewers and the lead broke the same rules once more:
`head`,
`;`,
a `for` loop,
a heredoc beside a second command,
`--eval` scripts,
short flags,
and an edit sent in one batch with the command that read it (ledger M177).
In the work of B492 to B500 the reviewer and both authors of the fixes broke them once more:
`head` and `tail`,
`node --eval`,
`sed --in-place` on package files,
large output printed to the terminal,
and edits sent in one batch with the commands that read them (ledger M182).
In the work of B501 to B521 the reviewers and the authors of five more rounds broke them again:
`;` and `head`,
output of more than 50 lines printed to the terminal,
`/dev/stdin` given as the path of an `rg`,
a shell loop and a `tail --follow` inside a unit,
and edits sent in one batch with the commands that read them (ledger M183 to M187).
In the work of B513 to B574 the authors and the reviewers of the rounds that followed broke them again,
and most of the rounds repeated the first two kinds:
two or three edits to one file,
or an edit and the command that reads it,
sent in one batch;
output of 50 to 650 lines printed to the terminal;
Reads of 54 to 347 lines at once of sources,
hand-backs and logs,
by authors and a reviewer who took the limit for a limit on outputs;
a listing or a search that recursed under the shared working tree;
a `;` chain;
a pipe into `rg` or `head`;
and a background unit whose output file was read three times before it finished (ledger M189 to M197).
The re-review polled the state of a unit about fifty times,
and the authors of the residue round and the fixes that followed it wrote a pipe,
a `;`,
and edits of one file in one batch again (ledger M201 to M206).

The rule:
a Bash call holds at most three steps joined by `&&`,
which is two `&&`,
and a leading `cd` is one of the three;
never `;`,
never a shell loop.
A report that must run after a failing command is `a || b`.
An edit script goes through the Write tool and runs in a call of its own,
in a later response than the write,
under a name no earlier script used.
A command that reads a file just edited is sent in a later call than the edit,
since parallel calls may run in either order (M177).
Edits to one file are sent one per call for the same reason (M189 to M197),
and the lead sent two Edits to one file in one block again on 2026-10-29 (UTC) (M306):
an edit that follows another edit of the same file waits for the next call.
Output that may pass 50 lines goes to a file in the scratch folder
and is read in ranges of at most 50 lines;
the limit binds a Read of any file as it binds the terminal,
source files,
hand-backs,
reviews and logs included,
and a brief says so (M193,
M202,
M205,
M209).
A listing or a search names the directory it needs
and never recurses under the shared working root (M191,
M197).
A search that may match widely is a count first and a listing second,
bounded by a directory or a file filter (M311).
A line number for an edit or a note is the one printed beside the line,
never an offset counted from the start of a range (M117,
M311).
Git runs from the repository root or with `git -C`.
An agent worktree is created with `--no-worktree-copy` from a shell whose `git` is the wrapper,
then set up.
Every worktree added from a worktree that holds ignored files passes the flag,
a keyless worktree most of all,
since the wrapper copies every ignored file of a linked source
(the `CopySelection` TSDoc in `package/git-policy/cli/src/native/worktree_copy_selection.rs`),
run keys and logger files included,
and `git clean --dry-run -d -X` in the new worktree lists nothing ignored (ledger M152,
M224).
A pattern that holds a backtick or a dollar sign is written in single quotes,
since the shell reads both inside double quotes,
and an `rg --replace` that uses `$0` is written in single quotes too (ledger M227,
M228,
M230).
A process is stopped by its own id,
or by the task id of the call that started it,
never by name or pattern.
A count that may be zero is a call of its own,
since `rg --count` exits 1 on zero.
A count of files or lines uses `--count` or a script over `--files-with-matches`:
`--count-matches '.'` printed 27,234 and 11,802,
counts of characters (ledger M245).
The path of an `rg` is a file or a directory:
the input of a pipe is written to a file in the scratch folder first,
and `/dev/stdin` is no path (ledger M186).
While a background task runs,
the turn ends;
a notification arrives when it finishes.
A permission refusal names an outcome,
so no other command reaching it is tried:
writing an older version over a tracked file was refused once
and attempted again through `git show` (M73);
an agent's read of the corpus clone was refused inside the agent,
and the count it then asked the lead to run stayed untaken and went to the owner (M118);
a before-and-after measurement of one module renders both versions from scratch copies
whose relative imports point at the worktree's sources.
A command that the repository's rules prescribe is run as written and from the folder the rule names:
the review call to `pi` was sent with two flags added,
from a scratch folder,
and failed on a missing `AGENTS.md` there (ledger M256).
A flag is added only after a call without it has failed for a reason the flag removes.

What enforces it:
nothing automated yet;
issue #579 asks the repository's guardrail hook to deny such calls.
The rate fell while slips were recorded by hand but never reached zero,
so until the hook lands,
count the `&&` and look for `;` before sending,
and rerun the M1 census before an entry claims a count.
The split lead's session chained with `;` five times (ledger M244),
and its notes conclude that the rule needs a mechanical check and not memory.
The agents of the sol round,
the test round and the seat round broke it again,
with pipes into `head` and `tail`,
`;`,
a script in a heredoc and a shell loop,
and each listed its own breaks (ledger M272).
The docs round,
the fix-mode fixture round and the follow-up round broke it after them,
with `for` loops,
`;`,
pipes into `head`,
`rg` piped into `rg`,
a chain of four `&&` steps,
short flags and a `python3` heredoc,
and the main session chained with `;` once to keep a lint log (ledger M273).
The notes say a brief alone has not held the rule,
and that the hook of issue 579 is the standing proposal for a mechanical stop.
A brief still repeats the rules and asks for that list,
but it is not the prevention:
the hand-back of every round is read for breaks before its work is accepted.
The rule to ask for the list was written by the filer of M272 on 2026-10-19 (UTC),
and the reading of each hand-back by the filer of M273,
and the notes did not give either.
The lead broke it again on 2026-10-19 (UTC):
most calls that kept an exit code ended in `; printf`,
one chained five `&&` steps,
and twice an edit went in one batch with the command that read the edited file (ledger M282).
An exit code is kept with `|| printf`,
which needs no `;`.

## Lint and edits

What happened:
edits attempted on files not read (M6);
lint run on changed files only (M10),
its warnings unread (M23),
or an autofix changing behaviour unreviewed (M26);
a file split at the line cap left comments naming the old file (M17);
every doc commit skipped the repository's Markdown linter,
so the package docs carried 3,344 of its findings (M54),
and the repository-level translation-repair docs 16,717 (D27);
the linter's `--fix` split 186 headings in a file with astral characters,
through an offset defect of its own (issue 559);
code spans closed early by a backtick inside them rendered as loose backticks,
which no linter rule reports (D30);
a page's link markup quoted as prose rendered as fifteen live links to files that do not exist (D32);
changes that stopped calling a function left it behind,
and nothing reported one (ledger B20);
the guard written for that counted a test,
or a barrel's re-export list,
as a use,
so 57 functions only tests reached passed it (ledger B30):
wrappers over parts production calls apart,
tested in place of the parts;
modules built beside the live path,
never wired,
and superseded;
functions whose callers changed course;
and test support shipped as package source,
23 values of the same kinds among them (ledger B31);
four type imports only the removed code read stayed,
and nothing reported them;
two error classes were marked safe to print and a third added
with only each commit's own tests run,
so the scan that reads every class failed two commits later (M59),
and the coverage census did the same with the rule written,
its four error classes and a copied helper failing two scans on the census's own first run;
and a scripted client moved from a unit test into a `.test-fixture.ts` file drew three lint errors and six warnings,
since the lint config relaxes the arrow-function and `require-await` rules
only for `*.test.ts` and `*.bench.ts` files (M87);
line numbers for in-place edits were worked out from offsets rather than read,
and garbled comments in three runner files before the diff showed it (M93);
and the scratch readers of lint output could miss a finding:
a location pattern written with a Unicode dash,
and a summary pattern that misses oxlint's singular "1 warning" (M95);
and 30 TSDoc blocks documented nothing,
each standing before a second block,
a `//` comment or a statement,
which the tsdoc lint plugin never reads,
since it walks from each declaration back to the nearest block (ledger B96);
four test declarations nothing read and 24 fixture exports no other file imported outlived their callers,
since the lint leaves `no-unused-vars` off and nothing else reads tests (ledger B118);
and 16,856 lines ended in whitespace,
nearly all TSDoc blank lines holding one space,
though `.editorconfig` forbids it,
since no configured check enforces that for TypeScript (ledger B119);
and a commit made on its named tests alone,
its message putting the lint off to a change that then went to agents,
shipped a lint warning to the twelve agent checkouts made from it (ledger M114);
and a line range for an in-place edit was counted off a print whose first line the tool had trimmed,
so the edit replaced the wrong lines of a TSDoc (ledger M117).

A red commit went out without the source scans,
and a scan failed at it on two test files that each built one body (ledger M123);
the fix commit for it went out without a lint run,
and the whole-suite run on that commit ended its lint with one warning (ledger M124).
The scans run before a red commit as before a fix,
and lint runs before any commit that adds a helper.
A fix commit that changed a doc went out without the Markdown lint,
which then found three findings in its paragraph (ledger M133):
the Markdown lint runs on every doc a commit touches,
a code commit's included.
A docs commit went out after its findings were fixed without the lint running again,
and one flagged line was left
(the 2026-10-11 recurrence of ledger M133):
the lint's last run before a commit is after the last edit,
and its output holds no finding.
The merge of a patch written at `f3c2bae13` ran its own files by name,
the scans and the lint,
and left the whole suite to the gate;
main had since gained cases pinning the provider's words in the line a stop writes,
which the merged change no longer writes,
and four of them failed at the gate though each change was green at its own base (ledger M140).
A patch whose base predates commits touching the modules it changes,
or their consumers' cases,
gets the whole suite on the merged tree before its fix commit,
not only its own files by name.
The port of the runner observations change onto a main that had rewritten its files
met three conflicts and four breaks with no conflict marker:
a case calling a builder without the parameter the change made required,
which only the type check found,
a fixture building an error without a field the change made required,
and cases expecting a kind and a remedy the change had split (ledger B366).
A port runs the type check and the cases of every user of each API it changes,
found by a search,
not only the files it changed.
Review of the combinator scan found declarations without TSDoc
and `@param` and `@returns` lines that said what a value is (ledger B356):
a destructuring or a loop's binding carries TSDoc as any declaration does,
and a parameter or a result line says why.
Review of the runner observations change found the same lines,
a `@returns` naming ids for a count,
and TSDoc describing a message language `LC_ALL=C` no longer lets through (ledger B377):
added TSDoc is read against the code it sits on after every merge.
The commits applying the owner's answers of 2026-10-22 added a hold to a model card,
a grace rule and a lint exemption,
and ran neither the linter nor the provider package's tests;
the gate found three files past `max-lines`,
two stage warnings and two failed seat-claim cases (ledger M287,
the fourth miss).
An owner's answer applied as a data edit
(a hold,
a list entry,
a constant)
is a code commit like any other,
and a file within a few lines of the cap goes over on the lines such an answer adds.

The rule:
read a region with the Read tool before editing it.
An applied patch that deletes code runs `mise run source-scans` before its named tests,
and an agent asked to delete code is asked to count the deleted names' remaining uses (ledger M75).
Lint,
type check and the named tests run after the final edit of a commit,
in that order,
and the commit follows only a clean run of all three (M46).
A commit that defers its whole-suite gates says so,
and starts them in the background in the call after it;
"with the next change" names no time (M114).
The named tests of a commit touching the package's source,
docs,
README or task file include the package-wide source scans,
run as one task,
`mise run source-scans`
(its description names each),
since a new class,
function,
export,
link,
sheet,
clock time or reference by position is read by those and not by its own file's tests;
a docs-only commit runs them too,
since four of them read the docs (M59);
a ledger entry is closed only after a full suite has passed on its last commit (M12,
M59).
Neither the type check nor the linter reports an unused import here
(issue #578 asks for a check),
and three of this audit's removals left ten imports unread in one day (M75),
so a removal counts the removed names' uses,
and a change that stops calling a function removes it in the same commit.
Code is live only when production reaches it,
never because a test or a re-export names it.
A wrapper over steps production runs apart,
with work between them,
is not written for a test:
the test calls the parts in production's order.
A module built beside the live path is wired in the change that builds it,
or goes in the change that supersedes it.
A helper only tests call lives in a `.test-fixture.ts` file,
which the package build never ships,
and which lints as source:
code moved there from a test follows an existing fixture's idiom
(`archive-selection.test-fixture.ts` for a scripted client).
A line-range `sed --in-place` names one file,
since it applies the range to every file it is given (M71),
and takes its line numbers from `rg --line-number '' <file> | sed --quiet 'A,Bp'` just before the edit,
read again after any insertion higher in the file (M93).
A replacement of more than one line is a Read and an Edit,
which fail where the text is not there;
a number for `sed` is printed beside its line,
never counted from a range's start (M117).
Lint's summary is read with `rg 'Found [0-9]+ warnings?'`,
beside the findings list,
each a check on the other (M95).
Commit before `--fix` and read the diff after it for anything but layout.
An author runs a fixer only on the files it changed,
or reports every file the fixer touched:
one author ran the package's auto-formatter twice over the shared tree and listed none (M178).
A split searches `src`,
tests and `doc` for the old file's name and repoints every hit in the same commit.
A TSDoc block sits directly before the declaration it documents:
a note on a statement is a `//` comment,
two blocks for one declaration are one,
and a lint suppression on a documented declaration opens before the TSDoc and closes after the declaration (ledger B96).
The line cap is met by splitting by concern,
never by reformatting or disabling the rule.
A Markdown change runs `mise run lint:monochromatic-lint <files>` from the repository root
(the task named `lint:markdown` until `0b49ae109e` replaced it),
and its `--fix` output is rendered against the committed version before it is staged,
block for block (`render-blocks.mjs` in the audit's scratch folder),
not by counting markers.
Until issue 559 is fixed,
`--fix` runs on a file holding an astral character
only with those characters mapped to single-unit private-use characters for the run
(`astral-safe-fix.mjs`).
A code span holding a backtick uses a longer backtick run as its delimiter,
since a backslash does not escape it,
and a rendered doc is scanned for stray backticks and asterisks (`stray-contexts.mjs`).
Markup quoted in prose,
a link above all,
goes in a code span,
or behind an escaped bracket inside a quotation that spans lines.
No line ends in whitespace,
except inside a template literal's text,
where the whitespace is the value:
a blank TSDoc line is an empty line,
the form the edit tool writes (ledger M103,
B119).
A change that stops calling a test helper deletes it,
and a fixture exports only what another file imports or re-exports,
since an export its own file alone reads offers nothing (ledger B118).
A round that adds or changes tests runs the `source-scan` task of the package family
beside `lint:types` and `lint` before it reports:
the test round of the survivors reported done without it,
and the scan found duplicate bodies in two of its test files (ledger M266).
This rule was written by the filer of M266 on 2026-10-19 (UTC),
and the notes did not give it.

What enforces it:
`mise run source-scans`,
one command for every scan a new file can fail;
the linter's `max-lines` and every other rule,
read in full before staging;
the Markdown linter,
which exits 0 only when no finding remains;
`package/translation-repair/source-scan/src/living-doc-links.unit.test.ts`,
which fails on a relative link,
image or link definition in the living docs,
the package's docs or its README whose file or heading does not exist;
`package/translation-repair/source-scan/src/dead-code.unit.test.ts`,
which fails on a private function or value its file never names
and on a top-level function,
class or value no production code reaches,
counting from the module-level statements of every non-test source file that declare nothing,
with a value named only in types counted as unreached
and the spend meter's reset its one allowed seam,
and on an allowed seam that production reaches or no file declares (ledger B30,
B31);
`package/translation-repair/source-scan/src/unused-imports.unit.test.ts`,
which fails on an import binding its file never names outside its imports,
type positions and export lists counted,
unless a TSDoc link names it (ledger B32);
`package/translation-repair/source-scan/src/tsdoc-attachment.unit.test.ts`,
which fails on a TSDoc block followed by anything but the declaration,
member or function it documents,
a lint suppression among them,
outside `@module` headers (ledger B96);
`package/translation-repair/source-scan/src/dead-test-declarations.unit.test.ts`,
which fails on a top-level declaration in a test file or fixture whose name its file never reads,
property and member names and other bindings not counted,
on a fixture export no other file imports or re-exports,
and on a default export,
namespace import or dynamic relative import,
which it cannot follow (ledger B118);
`package/translation-repair/source-scan/src/trailing-whitespace.unit.test.ts`,
which fails on a line ending in whitespace outside a template literal's text (ledger B119;
issue 585 asks for a repository-wide lint rule in its place).

A guard for a caught value takes it as `unknown` and returns a type predicate,
and the caller rethrows the narrowed value:
a type guard on a destructured catch binding tripped TS1230 and the lint rule `only-throw-error`
(ledger M292,
`isDryRefusal` in `stage/src/stage-run-end.ts`).

## Commit messages

What happened:
messages claimed records not yet written,
named hashes typed by hand,
chose `test` for a commit that changed production code,
named a task-list number as a GitHub issue,
and gave a cause that no command had yet shown (M16,
M39,
M49).

A red commit's message named three test files as failing and not the scan that failed beside them (ledger M123),
and a check of a message for task-list numbers matched the message's own words,
and stopped the commit without a word (ledger M130).
A message that names what fails names every failure of its tree,
and a check that stops a commit says that it stopped it.
A docs commit's body gave a package paragraph for a README the fix commit before it had changed (ledger M281):
in a multi-package body,
each package paragraph names a file the commit's stat lists.

The rule:
a message states only what `git show --stat` of that commit shows;
the type follows `git diff --cached --stat` (any production file under `src` makes it more than `test`);
every hash is resolved with `git rev-parse` in the command that uses it;
an issue number goes in only after `gh issue view` shows it is the one meant;
a cause goes in only after the command that shows it (blame,
log,
a probe) has run.
A count goes in only from a command run on the staged diff
(one commit said 44 and five where its diff held 40 and 6;
another counted its edit script's edits and called them the times they fixed).
A suite's count is its describe blocks,
one PASS line each,
never its cases:
one record called 1,384 PASS lines cases (M74).
A red run's findings are described from the arrays it printed,
each group counted against them,
never from the plan:
one red message summed eight findings as nine
and named a class its site's callee never throws (M77).
A batch's stretch counts are read from the baseline census's rows file by file,
each stretch placed in exactly one of reachable or removed,
and an edit the census never listed is named apart:
one record counted a narrowing the census had not listed as a removed stretch (M78).
A case is said to reach a stretch only once a census shows the stretch ran;
until then the message names the stretch the case is written for
(one message claimed two arms for a case that reached one,
M102).
A lint a message reports ran on the bytes the commit holds,
after the last edit to each file it reads
(one message reported a clean ledger the commit's ledger failed,
M106).
A lint run for a side checkout takes that checkout's root as an argument,
and its verdict is repeated only beside the root it read:
one message reported a clean lint that a script holding the main checkout's path had run on the main checkout's files,
whatever directory it was started from (M119).
A constant a message or a comment names is read from its declaration first,
and a correction is held to the same rule:
one message quoted a quorum of 3 where both quorums were 2,
and its correction's first wording repeated the slip (M84).
On 2026-10-11 (UTC) a merge message was drafted with call counts no source in front of the lead held,
and named the wrong file for the threshold it described;
both were rewritten from the diff before the commit ran (M150).
A message file takes a name no earlier message used,
checked with `ls` before any tool writes it (M72).
Before the commit runs,
and after the message's last edit (M46),
read the message for `#` followed by digits:
this audit's task list numbers its items like issues,
and two commits named a task as an issue within one hour.
A list of paths for a commit is generated from `git status --porcelain --untracked-files=all -z`
immediately before the commit and at no other time,
since the lists written at merge time went stale as soon as the next author added files (M173).
An inaccurate message is never amended:
a commit comment corrects it.
No commit is amended even when it is unpushed:
a docs agent and a merge agent of the split each amended one of their own (ledger M246),
and the briefs now say "never amend,
even unpushed".
A count in a commit body is read off the proof log it summarises,
or the body leaves the count out:
one body said six guards for the nine mutations it pins,
and a commit comment corrected it (ledger M267).
This rule was written by the filer of M267 on 2026-10-19 (UTC),
and the notes did not give it.
A body that counts failing cases names each one it counts:
a held fix said two cases "now fail on the error" and named neither,
and the redo listed every failing case of the test task by name for the base,
for the throw alone and for the result after the change (ledger M275).
A message said to be wrong is checked against the diff of its commit before a corrective comment is posted:
an agent called "nine places" a wrong count because the message names four cases,
and the diff removed nine spawns (ledger M274).
Once that check is done and the commit is on GitHub,
the corrective comment is posted at once and is not held until the branch lands:
the owner asked on 2026-10-29
"Why after landing?"
of a comment that waited (ledger M295).
A count that a message gives for a constant is read again after the constant changes,
and the message names the constant the count is for:
the message of `d599b87fab` said "none at the cap" of a cap it had just replaced,
which was true of the old cap of 13,082 only,
and a commit comment corrected it (ledger M312).

What enforces it:
habit,
which failed twice in one hour on the issue numbers.

## Work merged from parallel briefs

What happened:
the tense work and the consistency pass were built apart and both changed what artifact generation 15 holds,
so five replay fixtures of one broke on the other's section when merged (ledger M153).
Then the consistency review and the consistency modules' own tests were briefed together from one base,
kept apart by test file only,
and both changed the same production modules:
each fixed the reader's slot-offset and place-count rules its own way,
the merge held one rule twice,
and four test files pinned rules or messages the other had changed (ledger M156).
A three-way merge with no textual conflict said nothing about either.
The fold files were reserved to one of two rounds that ran at once,
and the lead's merge of that round onto the other kept two declarations of the folded links in one function,
a block of the earlier round beside the later rewrite of it,
so the merge commit did not build (ledger M238).
On 2026-10-20 (UTC) two commits of the seat-window census were written on an older base
and picked onto a tree that already carried the same `fanOut` change to `runPassContest` and its re-seat case.
Git reported no conflict and kept both copies,
since two identical additions placed at different lines are not a conflict:
a doubled `fanOut` parameter,
binding and spread in `pass-contest.ts`,
and a doubled `fanOut` key in the case (ledger B701).

The rule:
work that may change the same production files runs one after the other,
or one brief owns those files and the others only read them;
separating test files is not separating the work.
When two pieces of work did run apart,
the second merge runs every test file of both on the merged head before its commit is relied on,
and a rule both stated is kept once.
The first merged tree of the fourth review's fixes held three slips the gate found:
a case that pinned "unknown generation" at the number 17 after generation 17 became known,
a helper carried by two test files,
and two predicates that tested membership in a string (ledger M174).
A case about an unknown member reads the known list and names no number,
a helper two test files need lives in one fixture,
and membership is tested on a set.
A round's change merged onto a tree that held another round's change to the same function is compared
with the round's own change,
file by file and by a script,
before the commit,
and the build of the merged tree follows at once (ledger M238).
The same holds for commits written on an older base and picked onto a tree that already carries an overlapping
change:
the auto-merged hunks of each file both touched are read,
since git merges two identical additions placed at different lines without a conflict
(ledger B701).
A range is landed only after its base is checked to be an ancestor of the branch
and every earlier round from the same worktree is found in the branch's `git log`:
the first round of `w186` stood in the handover as done and was never landed,
and the second round's conflict on the ledger file the first had created was what showed it (ledger M264).
Two rounds that both add lines to the ledger indexes (`audit-ledger-entries-by-id.md` and `audit-ledger.md`)
collide at landing:
the landing script refused the caps round with "main 716016163e touches paths this branch changed"
after the real-clock round had landed in `422063bf09` (ledger M310).
When two rounds run at once,
one lands first,
then main is merged into the other,
every index line is kept in id order,
and `mise run //package/translation-repair/source-scan:test:unit` runs again before that round lands.

What enforces it:
nothing in the package;
the lead's briefs name the files each may change,
and the lead's named run after each merge covers both pieces of work.

## How an agent waits

What happened:
the brief for the consistency review's fixes forbade `sleep` and shell loops
but gave no way to wait for the whole unit suite.
The agent started the suite with the Bash tool's background mode,
armed a Monitor whose `ps` filter matched its own command line,
armed two more,
then counted the log's lines in about twenty separate calls while the suite ran;
one call also ended in `; echo done` (ledger M162).
Each count cost a turn and decided nothing.
The eighth review waited on a unit with a shell loop inside `systemd-run`,
a second unit that held a `tail --follow`,
and four `systemctl --user is-active` calls (ledger M185).

The rule:
a brief that forbids a way of waiting names the way to wait.
A command that ends inside ten minutes runs in the foreground with the Bash timeout at 600000.
A longer one runs as its own `systemd-run --user` unit with its output in a scratch log,
followed by one foreground call to `wait-unit.ts <unit>.service` in the lead's scratch folder,
which returns only when the unit ends and prints its result;
a timed-out wait is repeated once,
not replaced by polling.
No Monitor,
`pgrep` or `ps` loop,
or line count of a growing log stands in for it,
and neither does a loop in a unit,
a `tail --follow` in a unit,
or a repeated `systemctl` status call.
The output of a background command is read only after the harness reports that it finished:
on 2026-10-29 (UTC) a commit's result was read before the commit finished,
so that the head looked unchanged,
and a log of comment posts was read after one of three (ledger M301).
The lead's count for that day was three such reads:
a comment-post log,
a commit's output file and a worktree-retire log,
the last two empty when read (ledger M305).
An empty output file is the sign of a command still running,
never of a command that printed nothing,
and the turn ends until the harness reports that the command finished.

What enforces it:
the shared facts every brief points to carry the waiting rule and the exact unit form;
the lead checks a new brief for a forbidden form of waiting with no named replacement.

## Briefs and hand-backs

What happened:
a brief gave an example of a failing result that nobody had run,
and the reader accepted the section it said the reader would refuse (ledger M167).
A brief asked a reviewer to copy probes and repoint their imports,
and not the paths they write,
so the reviewer's reruns overwrote the output of the review before it (ledger M171).
A hand-back called the work green from 64 test files,
and the whole suite then found four failing cases (ledger M175).
An agent denied a step by the permission classifier ran the task the step wraps (ledger M176).
An author wrote the code of a fix before its tests,
and two rounds whose lists of files overlapped each edited files outside their lists (ledger M182).
Three more rounds wrote code before its tests (ledger M183,
M184,
M187),
and copied probes kept hard-coded output paths and overwrote the reviewer's outputs once more (ledger M187).
A brief required a result that contradicted a decision of the lead:
the check that no mark changes its reading was to reach 0 under an oracle that reads a lone `*` as emphasis,
when the lead had decided that a lone `*` is a character (ledger M188).
A brief asked for a shared letter test to be widened without a list of its callers,
and a later round had to confine the widening to the one pass that needed it (ledger M198).
A brief named grid copies that a later round had already updated,
and the review's runs on them gave counts that were no defects (ledger M199).
An author changed its copies of the oracle to its new rule,
and its zeros of the grids were the author's reading checked against itself;
an oracle written from the rule text by another author then found 3,555 failing pages on the same tree (ledger M194,
M196).
Numbers in hand-backs came from a red run,
from a build before the last refactors,
or from the review's build and not a rebuild,
and 36 more `replace` plans of a grid went unattributed for a round (ledger B532,
M194,
M195).
Probe copies from older rounds called a function the tree had since removed,
`textBetweenSlices`,
and failed (the replay copy of n143 and the examples probe of the ninth review),
so a newer copy had to be found for each (ledger M199).
A ruling stated the run between two marks as a list of named characters,
and each family around a name shipped a doubled mark until the re-review found it (ledger M208).
A brief gave the limit of 50 lines for outputs and did not say that it covers the files an agent reads,
and a reviewer read sources in ranges up to 347 lines (ledger M209).
Ten fix rounds in a row left their tests and code uncommitted in one worktree,
so that no build from before a change survived
and the reviews could not say which defects predate a round (ledger M218).
Four builds then gave identical oracle grids,
run sites,
replay rows and recounts of the elisions,
and each review of them still found new defects in code next to the changed code (ledger M219).
A brief asked for every other top index entry of the history archive to be checked and "reported",
and the agent applied six fixes,
read the verb,
and reverted them to proposals,
so the lead wrote the six clauses itself and no other author read them (ledger M225).
A brief said that the archive had not changed since a commit,
and the check that the brief gave the agent showed a commit that already held the fixes (ledger M229).
A review named the cause of a residue from reading the code and said it had not instrumented it,
and the brief that carried the pointer told the fix round to confirm it on an instrumented copy first,
which showed that refusing the case in that branch changed nothing (ledger M227,
B619).
A review claimed reads of heads and tails before they were done,
and about fifteen Edits rewrote it after its appended sections (ledger M230).
The next two rounds after those seven broke the same forms again,
the one that read empty links as their twins and the one that wrote the docs for the seven:
Reads over 50 lines,
an output of more than 50 lines to the terminal,
a checksum written in one batch with the `diff` that read it,
two Edits of a hand-back in one batch,
an Edit beside a search of the same file,
and,
for the first,
a search of its own session transcript for the commands of earlier rounds (ledger M234,
M235).
The brief of the docs round attributed to a review the cause that a fix round had found,
and named a file for the brief rules that the repository does not hold (ledger M236,
M237).

The rule:
an example in a brief that claims a failing result is run before the brief is sent,
or is written as a question.
A brief that asks for a copied probe names both the import and every path the probe writes,
and the author searches each copy for the folder of the review it copies before the first run (ledger M187).
Before a brief is sent,
each result it requires is checked against every decision that result depends on,
and each step it requires is checked against the rules of the repository that the step must pass,
the lint rule for the imports of tests among them (ledger M188,
M190).
The lead checks each brief against the decisions recorded in the notes,
and against the rules of this page for what a brief must say,
before the brief is launched (ledger M188,
M200,
M209).
A brief that changes a shared predicate carries the list of its callers and a measurement of each (ledger M198).
A brief names the newest copy of each probe it asks an agent to rerun and the round that updated it,
since a copy of an older round can measure another build or call code the tree has dropped (ledger M199).
The author of a change never edits or reads the oracle that judges it,
and a brief for a changed rule gives the oracle to another author (ledger M194,
and "Guards that cannot fail" in `mistake-prevention-guards-searches-and-claims.md`).
The numbers in a hand-back come from the final build only:
a suite that ended red is rerun whole after the fix,
and a grid is rerun after the last refactor.
A figure taken on another build or in a red run is labelled with that build
and is not offered as the result (ledger B532,
M194,
M195).
A figure that cannot be rerun on the final build says which build it came from
and why the final build would give the same,
for instance that its source files equal those of the build by checksum,
and a difference that two changes could have made is attributed to neither (ledger M201,
M204).
The run-to-run spread of a timing probe is measured on the build that is reported,
before a difference smaller than the spread is claimed (ledger M189,
M192).
A hand-back says which build each guard experiment ran on,
and every file a mutation touched is checked by checksum when it is restored (ledger M191).
A comparison written to a file is read for the size of the file before it is called identical,
and an expectation corrected after a green run is listed in the hand-back as having no red of its own (ledger M197).
"Green" in a hand-back means the whole package suite ran in that tree,
and a list of files is not green.
An agent denied a step stops and reports the denial,
even where the wrapped task would do the same harmless work.
A brief repeats the shell rules with each banned form named,
and a hand-back lists every rule its author broke;
the lead reads that list before relying on the results.
A brief's verb matches the ruling the lead will make:
a check whose findings will be fixed says "fix",
not "report",
and clauses that the lead writes itself get the reading that an agent's change gets (ledger M225).
A brief that states a premise about a tree,
such as "unchanged since" a commit,
is sent only after the lead has run the check that the brief gives the agent (ledger M229).
A brief that carries a cause from a report says whether the report instrumented it,
and keeps the instruction to confirm it on an instrumented copy before the fix,
which found the pointer wrong in the round that followed (ledger M227,
B619).
A brief that asks for a lint control names where the control goes and who removes it (ledger M228).
Rules that six of seven rounds broke while their briefs stated them are not stated again in the same words:
a brief opens its rules with the forms that were broken
(a Read with no limit,
a Read with a limit over 50,
a listing or a search printed to the terminal,
two Edits to one file in one batch,
and an Edit beside a read of the same file)
and the number of rounds that broke each,
and it asks for the author's own count of each form in the hand-back (ledger M227 to M233).
The brief of n173,
written after the breaches of n169,
did that,
with a `limit` of 50 or less on every Read "even when you need a longer stretch",
a checksum or a `diff` of a file counted as a read of it,
and a ban on searching session transcripts,
and the heading of its rules says "the last author broke the first three" (ledger M234).
The docs round for the seven broke the same forms in spite of its brief (ledger M235).
A search text in a brief is run against the thing it searches before the brief is sent:
a proof pattern for a bundle matched nothing in either bundle,
because the built code writes its digit between backticks (ledger M259),
and a pathspec `'package/translation-repair/*/src'` matched no file below `src`
and returned 10 commits all from before the split,
where 40 production commits were wanted (ledger M260).
Each carries a positive control,
a case known to be found,
and a proof pattern a negative control as well,
run on both bundles.
A report that a brief asks for is returned as text in the hand-back too,
since the tool has twice refused an agent's Write of its report file (ledger M226,
M261).
Where a brief wants a report file as well,
it names a heredoc inside the keyless wrapper as the way to write it,
since an agent cannot write a report file with the Write tool in this harness.
A brief that carries a cause into a doc cites the report that names it with the report's own limit,
and cites beside it the later finding that confirmed or refuted it (ledger M236).
A brief names a path only after the lead has listed it,
and a question that one listing answers is answered before the brief is sent (ledger M237).
A brief for a census over run directories names directories that no live pass writes to:
the brief of the caps round named all of `~/temp/agent` as the root,
so the census read the logs of the live pass `corpus-pass-XingZ6022`,
read-only and with nothing touched,
and the brief,
not the agent,
had named that root (ledger M312).
A hand-back states a read or a check as done only when it was done,
is written once and then appended to,
and a round that stops at a precondition names each read that its brief required and it did not finish
(ledger M229,
M230).
A change is built tests first,
item by item,
memos and reader changes included,
and an author who wrote the code first says so and shows that the red run was made without the change.
A temporary edit of a source file for an attribution run is checked by the file's checksum before and after,
and the hand-back lists the file (ledger M183).
A file outside an author's list is edited only for a reason the hand-back names,
and rounds whose lists overlap are merged by hand (ledger M182).
Each round's tests and code are committed before the next round starts,
even when a later round will change them,
and the brief of the next round names the commit it starts from (ledger M218).
A review loop stops when the checks that measure what ships
(oracle grids,
corpus counts,
replay)
stay identical across builds and each review finds defects in code next to the changed code:
the corpus reach of what is open is measured,
the work is landed,
and the rest is fixed after the launch,
each fix as its own commit (ledger M219).
The owner's stopping rule of 2026-10-25,
"Land on real impact",
says when a review loop ends:
a branch lands once a re-check finds no major issue
and nothing that changes behaviour on real sources,
runs or the corpus.
A gap that only invented inputs show becomes a stated limit and a separate later item,
and a guard evasion that only an adversarial input reaches is such a gap and not a mistake of the package.
A nit goes into the same docs round as the findings that came with it,
and never into a round of its own.
A branch is not held for a clean re-check:
the Claude route took 20 re-checks and 22 fix rounds,
and only re-checks 13 and 14 found a major issue (ledger M296).

What enforces it:
habit,
and the lead's check of each new brief against this list.
The accounts of the rounds from n162 to n169,
and of n172,
name no check that stopped a Read over 50 lines,
an output over 50 lines or two Edits to one file in one batch,
so the hand-back's list of broken rules is the only check on them (ledger M227 to M235).
Rounds n171 and n173 returned no hand-back file,
so no list of theirs was read.
