# Audit ledger: command lines read by each runner

How the runners read their command lines,
and ids a line names that its runner cannot reach.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B75: command lines read token by token, each runner for itself

Found beside B73,
which gave a flag's value one reader (`flagValue`)
and left each runner finding the flag itself.
Every runner found its flags with `indexOf` or `includes` on the exact token,
so a flag written any other way read as not written at all,
and nothing said which tokens a runner reads.

What the red commit (`6035e9b03`) shows each reader doing:

- `readOnlyIds` read `--only=Tabby_01`,
  and the mistyped `--olny Tabby_01`,
  as no filter,
  which `corpus-pass` runs as every pending entry;
- `readOnlyIds` and `flagValue` took the first of two `--only` or `--naps` values
  and dropped the second without a word;
- `flagValue` read `--naps=3` as unwritten;
- `readAuditArguments` read `--cap=0`,
  and `--cpa 0`,
  as no cap,
  which the settled audit runs as buying every subject in the archive;
- `readCandidateIds` read `--candidates=<ids>` as none,
  so the probe ran the seated roster and none of the models it was started for.

The named run of the four files printed exit 1 for each,
with 3,
2,
2 and 1 failing cases.
Read in the same sweep and recorded in the fix's message (`97aeee2ca`):
`sentinel-probe` dropped flags and probed the value after one as an entry id;
`coverage-control-probe` read `--only` written last as every entry;
`score-agreement` and `score-probe` read a flag written with an empty value as their default file;
`ledger-report` read `--model --x` as a seat named `--x`;
a runner that takes no flags ran as if nothing had been typed after it;
`--damage constructor` found `Object.prototype.constructor` in the fidelity probe's plain-object defect table
and handed a function on as the defects to build;
and the descriptions of five mise tasks left out flags their runners read
(`b78a90e4a`'s red case names them).
Found while closing it (`dd882e587` red):
`roster-card` answered an empty served id and a provider outside the four alike,
with the bare usage line,
and the provider and candidate readers named a refused id unquoted,
so a spaced one read as two words of the message.

Fixed (`97aeee2ca`,
`1f37aa07d`,
`b78a90e4a`,
`b8ba86137`,
`911d9afc3`,
`1241086aa`,
`5a3e38037`,
with the guards in `6f45620da`,
`c5d814d14` and `4dded7d92`):

- `corpus-run/command-line.ts` holds `readCommandLine`,
  the one reader of a runner's whole line.
  Node's `parseArgs` splits it,
  with `strict` off and its tokens kept,
  so the equals form,
  the `--` terminator and grouped short flags read as they do for every Node command,
  and every refusal is written there,
  all at once,
  ending with a usage line built from the declaration.
  It refuses a flag the runner does not read,
  a valued flag with nothing usable after it
  (nothing,
  an empty argument,
  or the next flag),
  a switch given a value,
  a once-only flag written twice,
  an argument past the last position read,
  and a required position left out.
- `corpus-run/command-lines.ts` declares,
  for each of the 42 runner entries,
  its valued flags,
  repeatable flags,
  switches and positions,
  typed key for key by `CommandLines`
  (the package's isolated declarations cannot infer the table).
- `reportingRefusals` takes `argv` and reads the line against the entry its `what` names before the body starts,
  handing the body the line it read;
  a refusal exits 6,
  as every stated refusal does.
- The value readers in `corpus-run/command-flags.ts` take a flag's value from that line:
  `flagValue` is gone,
  `writtenOr` reads the score reports' file flags,
  and `idListFlag` trims space around each id,
  as the pass's entry filter already did.
- The fidelity probe's defect table is a `Map`.
- `ledger-report`'s own exit 3 for `--model` with no seat is gone,
  answered by the shared refusal;
  `score-probe` refuses one of `--repair-sheet` and `--manifest` without the other,
  where it printed its unscored note and dropped the file named;
  `readAskedCount` refuses an empty count argument.
- The five task descriptions name every flag their runners read.
- `roster-card` names each wrong part of its ask,
  both at once when both are,
  and the provider and candidate refusals quote the id they refuse;
  `CARD_PROVIDERS` in `model-card-derive.ts` is the one list `CardProvider`,
  the ask reader and `roster-card`'s declaration are written from,
  where the reader and the declaration each wrote the four names by hand.

Calls made here are open to veto:

- a minus sign before ASCII digits reads as a position,
  as Python's argparse reads one when no flag looks like a number,
  so a count typed under zero still reaches the reader that answers it as under zero;
- `parseArgs` runs loose and the package writes every refusal,
  so all of a line's problems are named at once in the words the B73 readers used,
  where strict `parseArgs` stops at the first in its own words
  (`Unknown option '--olny'` for `--olny a --paln`,
  measured on node v26.10.0);
- a valued flag takes the argument after it whatever it is,
  and refuses one starting with `--` as the next flag,
  so no value can start with `--`;
  a path starting with `--` is written after the `--` terminator,
  where only positions are read;
- a once-only flag written twice is refused,
  where `parseArgs` keeps the last value and the old readers kept the first;
- a repeatable flag (`--source` of `coverage-census`) keeps every value,
  in the order written;
- a grouped short flag such as `-abc` is refused once,
  whole,
  as typed;
- `idListFlag` trims space around each id;
- `ledger-report`'s `--model` with no seat exits 6,
  where it exited 3;
- `score-probe` refuses `--repair-sheet` or `--manifest` alone;
- `readAskedCount` refuses an empty count argument,
  reversing a case that pinned it to the default;
  B73's reading of blanks as unset covers environment variables,
  and an argument typed empty is not unset;
- a refusal quotes what was typed through `JSON.stringify`,
  so an empty or spaced value shows;
- a task description spells exactly the flags its runner reads,
  and the living docs invoke a runner only with flags it reads;
  the audit ledger is left out of that check,
  since it records what was typed when it was typed.

The guards:

- `command-line.unit.test.ts` holds the reader's readings and refusals on real runners' declarations,
  through `command-line.test-fixture.ts`;
- `command-lines.unit.test.ts` holds the table to the runner entries the build makes,
  each declaration to flag names a long flag can carry and positions that can be written,
  each runner's mise task description to exactly the flags it reads,
  and every runner invocation in the living docs to flags the runner reads;
- `cli-refusal.unit.test.ts` holds a refused line to exit 6 before the body runs,
  and the body to the line as read;
- `command-line-reads.unit.test.ts`,
  which `source-scans` runs,
  reads the package's source for `argv` read off `process`
  or off a whole import of `node:process`
  (as a member,
  an indexed or optional member,
  or through a parenthesis or type assertion),
  `argv` destructured from `process` or imported from `node:process`,
  `parseArgs` imported from `node:util` or read off a whole import of it,
  dynamic imports of either module,
  and a `reportingRefusals` call handed another argv or naming a runner other than its own file;
  it allows only `command-line.ts`'s import of `parseArgs`,
  and requires a hand-off from every runner the build makes.
  Out of its reach:
  `process` reached through `globalThis` or handed on whole,
  and a property name built at run time.

Guard-offs,
one mutation per build,
each restored with `git diff` printing nothing after
(`~/temp/agent/audit-glossary-fix/b75-guard-offs.ts`):
an unknown flag read as a switch failed the reader's mistyped-flag,
flagless-runner and prototype-name cases and `cli-refusal`'s refused-line case;
a repeated once-only flag let through failed the repeat case;
an empty value let through,
and in another build the next flag taken as the value,
each failed the valueless-flag case,
the first also the fidelity probe's case for a flag written last;
a switch's value let through failed the switch case,
and the fidelity probe's again;
the once-only lookup made with `in` failed the prototype-name case
(`--constructor needs a value written after it`);
a minus number read as short flags failed the number case
(`-35 is not a flag this command reads`);
arguments past the last position kept failed the past-position,
mistyped-flag and flagless-runner cases and `cli-refusal`'s;
a required position let go failed the past-position case;
the defect table looked up as a plain object failed the fidelity probe's defect case;
a description without `--manifest` failed the description case;
and a runner renamed in the table failed the table and description cases.
The scan
(`~/temp/agent/audit-glossary-fix/b75-scan-guard-offs.ts`)
failed its fixture case with each detection removed in turn:
whole imports of `node:process`,
wrappers,
bracketed names,
destructuring,
`argv` imports,
`parseArgs` imports (its package case too),
`parseArgs` read off a whole import,
dynamic imports,
a hand-off naming another runner,
one handed another argv,
and a hand-off counted as the file's own whatever runner it names;
its package case failed with `spend-report` naming another runner,
and with it copying the line before handing it on.
With `roster-card`'s ask keeping only its first problem,
the case naming both failed;
with `--plan` misspelt in `mistake-prevention.md`,
the invocation case failed naming `mistake-prevention.md:57 corpus-pass --paln`.
Before the invocation case,
a scratch sweep over every tracked Markdown file under the package and `doc/`
read 69 invocations and found no undeclared flag,
and reported both invocations planted in a control file.
The full suite on `3d8776f9f`
(`mise run //package/module/translation-repair:buildAndTest` (moved on 2026-10-18 to
`//package/translation-repair/<package>:buildAndTest`))
printed 1,489 PASS lines and no FAIL line.
Later,
with `reportingRefusals` running the body once on a placeholder line before reading the real one,
`cli-refusal`'s refused-line case failed on `ran`
(`expected true to equal false`),
so its check that the body never starts carries weight of its own;
restored,
with `git diff` printing nothing after.

Found on the way and open:
an `--only` naming no entry runs over none without a word
(closed by B76);
and the package's other plain-object tables looked up or filled by text from outside it
(the family of prototype keys,
closed by B77;
the defect table is closed here).

Recurrence:
a runner declares what it reads in `corpus-run/command-lines.ts`,
reads only the line `reportingRefusals` hands it,
and reads a flag's value through `corpus-run/command-flags.ts`;
`command-lines.unit.test.ts` fails on a runner,
description or documented invocation out of step with the table,
and `command-line-reads.unit.test.ts` on a read of the line anywhere else.

## B76: ids a line names that its runner cannot reach, dropped without a word

Found closing B75.
`corpus-pass`,
`coverage-probe`,
`coverage-control-probe` and the settled rendering audit each kept the entries `--only` names with the same predicate,
and dropped any it could not reach:
an `--only` naming only a stray ran over nothing,
the settled audit printing `BUYING 0 of 0 selectable subjects`
and `coverage-control-probe` refusing as though no entry had offered a passage,
and a stray beside a real entry ran the real one alone.
`sentinel-probe` read its entry ids by position without a check,
so an unknown one printed an error line of its own while the probe spent on the rest.
`judge-fidelity-probe` already refused an entry its reviewed manifest lacks
(`selectReviewedFidelitySpecs`),
the first one only.

`28cbadd6a` moved the four predicates into one,
`askedAmong` in `corpus-run/command-flags.ts`,
with no change in what it keeps
(cases pin both readings).
The red commit (`31021fe29`) added what each refusal names to its parameters,
which every site passes,
and cases for a stray alone,
two strays beside a real id with one written twice and one spaced,
and the settled audit given `tabbby` beside `tabby`;
the named run printed exit 1 for both files,
the new case the only failing one in each.

Fixed (`4f9b79155`):
`askedAmong` refuses the line when any id asked for is missing from what the runner can reach,
naming every stray once,
in the order written and quoted:
`--only asks for "Tabby_0l", which the corpus at the pin does not hold`.
The four sites refuse before anything is spent,
and `sentinel-probe` holds its ids,
named or default,
to the corpus at the pin before it probes any.

Calls made here are open to veto:

- one stray refuses the whole line,
  where running the reachable ids and naming the strays would also have been possible;
  the line is refused because a mistyped id is a line the person did not mean;
- `sentinel-probe` probes in corpus order,
  as the other runners walk,
  where it probed in the order written;
- the settled audit's reachable entries are those the archive holds a reading for,
  so an entry with a reading and nothing to buy still runs,
  and reports `BUYING 0 of 0`;
- `judge-fidelity-probe` keeps its own refusal,
  which names the first stray only,
  since its reachable entries are a reviewed manifest's and its error class reports them.

Guard-offs,
one mutation per build,
each restored with `git diff` printing nothing after
(`~/temp/agent/audit-glossary-fix/b489-guard-offs.ts`):
strays let through failed both new cases;
a stray written twice named twice failed the `askedAmong` case;
strays named unquoted failed both.
The full suite on `e009f5d28`
(`mise run //package/module/translation-repair:buildAndTest` (moved on 2026-10-18 to
`//package/translation-repair/<package>:buildAndTest`),
with the worktree clean)
printed 1,490 PASS lines and no FAIL line.

Recurrence:
a runner that narrows its walk by id passes the ids asked for and the ids it can reach through `askedAmong`;
`command-flags.unit.test.ts` holds the refusal.
