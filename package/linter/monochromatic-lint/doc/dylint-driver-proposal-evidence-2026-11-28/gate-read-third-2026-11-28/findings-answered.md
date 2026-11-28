# The third gate read, and the revision that answered it

What this is:
every finding of the three same-vendor gate reads of
`doc/planning/monochromatic-lint-who-provides-the-dylint-driver-and-what-a-skipped-rule-returns.md`,
answered with the line or the run each answer rests on,
and the report of the revision that ran the measurements.
It is unapproved and non-final;
the owner declares completion.

The reads,
all of them second passes by Claude instances and none an independent review:

- 2026-11-27,
  first:
  4 High,
  6 Medium,
  6 Low (`gate-read/review.md`).
- 2026-11-28,
  second:
  4 High,
  8 Medium,
  6 Low (`gate-read-second-2026-11-28/`).
- 2026-11-28,
  third:
  3 High,
  8 Medium,
  6 Low (`gate-read-third-2026-11-28/`,
  this folder's parent;
  the read itself is `review.md` beside this file,
  with its probes under `probes/`).

The third read is answered finding by finding in "The third read" below.
The first two are answered in "The first two reads" by naming the section of the proposal that holds
each answer now,
because the revision took the finding-by-finding answers out of the proposal
and into this file:
a reader answering the owner's questions does not need them.

Measurements named `M12` to `M17` are this revision's,
in `../third-round-2026-11-28/`.
`M1` to `M11` are the earlier rounds',
in their own folders beside it.

## The third read

### High 1: Answer I named a mechanism the repository's installer does not have

**What I checked.**
Read whole:
`package/mpm-shadow/cli/src/toolchain/model.rs` (`rustup_channel`,
`expand_version`),
`package/mpm-shadow/cli/src/toolchain/install_rust_android.rs` (`install_rust_toolchain`,
`install_rust_components`),
the `rust` row of `toolchain.plan.jsonc`,
and the `rust` entry of
`toolchain.lock.json`.
Ran:
`git grep --line-number 'rustup update' -- ':!doc' ':!*.md'` (no hit),
`rg --count-matches 'toolchain install nightly' .github` (17 in `cargo-publish.yml`,
1 in `crate-release.yml`),
and the reads saved in `../third-round-2026-11-28/probes/repository-plan-lock-and-installer-lines.txt`.

The finding is right in every part.
`rustup_channel()` returns the declaration's `packageId`,
the plan writes `"{version}"`,
the lock holds `nightly-2026-11-16`,
and both the toolchain install and every component add pass
`tool.rustup_channel()`.
So `toolchain:install` puts the components on the dated toolchain and on nothing else,
and no repository task moves the rustup channel named `nightly`.
The finding's second half is right too:
the record's "shared floating nightly" is the lock's dated entry,
as its own next sentence says.

**What the proposal changed.**
Answer I's "How it works" now says a new installer step is needed and names what a yes leaves undecided
(who adds the components,
who moves the channel);
its "What changes for good" no longer says nothing in the repository changes;
a new section,
"The plan row installs a dated toolchain,
and puts the components on it",
quotes the four sources;
the first screen states it before anything else.

### High 2: the first-ranked answer was ranked on a state a fresh machine cannot have

**What I ran.**
Fetched today's channel manifest and commit hash (08:04:48 -05:00):
dated `2026-11-28`,
compiler `c82ce1563 2026-11-27`.
Read Dylint's releases and pull request 2203 with `gh` (reads only):
latest release 6.4.0 of 2026-11-10,
pull request 2203 merged 2026-11-24,
`master` 51 commits past `v6.4.0`.
Read M2b's `E0603` log and M8's in-container script (the `cp -a` of the 2026-11-17 install over the
floating name).
Then measured the two states in one image and on one day:
M12 (Answer I on the channel's current compiler) and M15a (Answer K1 alone,
floating channel minimal),
and M16 (Answer K1 with its file naming today's compiler,
`nightly-2026-11-28`).

The finding is right.
On today's compiler,
Answer I returns one processing failure per package and status 2,
with no findings (M12).
Answer K1 returns the six findings with the variable (M15a arm C) and skips without it (M15a arms A and D).
K1 inherits I's failure exactly when the lock reaches a compiler past what 6.4.0 supports (M16).

**What the proposal changed.**
M12,
M15a and M16 are stated side by side with what state each stands for;
M8's stale-channel result is no longer the basis of the first place;
the Decision 1 criterion names Decision 2's answer T rather than excluding the variable;
the order is restated as measured results plus a preference asked separately;
the 91 s against 123 s figures are withdrawn as a ranking reason and stated as figures that cannot be
compared (different compilers,
one run each,
no spread).

### High 3: the ranking's real reason was a preference, read from words it does not hold

**What I checked.**
Read `doc/planning/toolchain-always-latest.md` section "Owner's words" whole,
`doc/planning/rust-nightly-always-latest.md` (the three options and their ranking),
and `doc/decision/rust-closure-lint-dylint.md` (the 2026-10-18 section,
the 2026-10-19 answers,
"Known limits").

The finding is right.
The words name "latest",
the plan implements latest as a dated name the daily update moves,
and nothing in the words picks the rustup channel over that.
"Wait for Dylint" answered a three-way question about the closure package's own red,
which that record ranked below "patch now";
it was not about pinning.
"No pin" is a default a session adopted and left open to veto,
and a pin was listed as not approved,
which is neither approved nor refused.

**What the proposal changed.**
The owner's words are quoted whole with what each answered and whose gloss the parentheses are;
the section on the record states what the record's "floating nightly" is;
"do not pin" is gone from every attribution;
the ranking says the words do not decide between K1 and I,
and the preference is asked in question 1
as a preference,
not produced as a result.

### Medium 1: what a breaking nightly returns, and who notices

**What I checked.**
Read `check_package` and the processing-failure message in `src/closure_step_package.rs`,
`driver_state` and `driver_gaps` in `src/closure_driver.rs`,
and the daily update's package check in `doc/planning/toolchain-always-latest.md`
("Checking every package after the update":
each package's `lint:clippy`,
a newly red package reported once,
"failed then failed is already red and is never reported again").
Measured the return itself:
status 2 and one record per package (M12 arm A,
M16 arm B).

The finding is right,
and this revision could now measure what it asked for.

**What the proposal changed.**
Answer I states the return (status 2 per package,
the message's shape,
the first compiler error,
M12),
that every linted package must also compile on the library's toolchain
(the owner record's "Known limits" and Dylint's own documentation),
that the daily update reports only the closure package and only once,
and that whoever runs the root lint
task meets the breakage.
The "not known" item it named is gone.

### Medium 2: reasons for I over K1 that are not delivered results, and a timing with no spread

**What I checked.**
Each reason the finding names,
against M7,
M8,
M12,
M15a and the source.
The finding is right on each:
the six findings are a tie where both deliver;
"needs neither the file nor the variable" is a count of parts;
the 91 s and 123 s are one run each on
different compilers;
the driver directory's name is a coincidence of how Dylint names a library with no file;
and K1's "cannot drift" is robustness,
not delivery.

**What the proposal changed.**
The reasons are restated as delivery or cost,
with the assumption (T) stated:
K1 over K2 on whether the run happens at all (nothing installs K2's date,
the cleaner keeps only what the lock
or a checkout-root file names),
K2 over I on measured delivery today,
I last on today's delivery with the
release that would change it named.
The timing is withdrawn as a reason.

### Medium 3: the decisions as a set

**What I checked.**
The criterion and its variable (`Decision 1`'s old criterion against Decision 2's subject),
`driver_state`'s branches,
`dylint-link`'s own read of `RUSTUP_TOOLCHAIN`
(`dylint-link/src/main.rs`,
and the measured `dylint-link --version` error in the M12 run log),
the plan's route list (`rg --only-matching '"route": "[a-z-]+"' toolchain.plan.jsonc`),
and the lock's exported `RUSTUP_TOOLCHAIN`.

The finding is right that the two decisions were not independent as ranked,
that option M and option E were
used in the combinations list without ever being described,
that four combinations were unnamed,
and that two of the "cannot be built" reasons do not hold.

**What the proposal changed.**
"How the decisions depend on each other" states every dependence,
including T's assumption and the lock's
exported dated value (which was not measured);
options M and E are removed rather than half-described,
and the facts they carried are kept
(`dylint-link`'s own read of the variable,
and that the library's build ignores its own `.cargo` `[env]`
because Dylint removes the variable first);
the four missing pairs are named in "Combinations,
and what cannot be built";
the K2-with-the-cleaner entry is no longer called impossible,
because the file installs its toolchain on first
use (M14 arm B,
M15b) and the real cost is a date nothing keeps;
the status-with-severity pair is called a conflict of intent,
not an impossibility.

### Medium 4: what rests on rustup installing what it is asked for

**What I ran.**
M14 (three arms) and M15b,
on rustup 1.29.3 (2026-11-11),
the version the image and this machine carry:

- an absent toolchain with auto-install on:
  the toolchain and the file's components are installed,
  and the components are itemised (8:
  the default profile's six plus the two declared);
- a present toolchain missing a declared component:
  the two components are downloaded and added;
- the supported step,
  `rustup toolchain install` with no argument:
  the same components are downloaded;
- auto-install off with an absent toolchain:
  the linter prints no record and exits 0.

So the second read's L5 is wrong as written for this rustup,
the earlier reply was right for the
absent-toolchain case and thinner than the log,
and the case neither covered (a present toolchain missing a
component) does add the component.
The deprecated behaviour is still deprecated,
and the supported step is the answer that does not depend on it.

**What the proposal changed.**
K1's first-use paragraph now says what the first use installs,
names the deprecation and the supported step,
and cites the itemised components;
a new "M15b and M14" section carries the measurements;
M11's folder name is stated to say the opposite of its log,
and M14 and M15b carry that measurement.
The case M11 could not answer,
a run with no auto-install,
became Decision 5's Part 5.

### Medium 5: Z over A resting on a pause, and question 7 not the owner's

**What I checked.**
`gh issue view 6316 --comments` (reads only):
both comments are under one account,
and the second says "the project answered",
not that the owner did.
The plan's routes (none compiles).
The review's own date evidence about the tool links is theirs and I did not
re-check it.

The finding is right that the attribution is not the session's to make and that half of question 7 is settled
by source.

**What the proposal changed.**
Z over A now rests on delivery today (Z delivers with parts that exist and is measured;
A has no route that
compiles),
not on the pause.
The 6316 comment is quoted with its own words.
The owner question about that comment is gone:
who answered it is a record question for the coordinating
session (named in the report below),
and question 7 is now about where the counts live.

### Medium 6: things read and not run, written as results

**What I checked.**
Mechanism T (the container set the variable by hand;
no `mise` task ran),
option F under I (no kept offline
driver build under I with a pre-filled registry),
Answer C's temporary directory (no kept run fails for it),
and K1's components (M11's count).

The finding is right on all four.

**What the proposal changed.**
Each is written as a design with the nearest measurement named,
and all four are in "Inferred".
K1's components are now measured rather than counted (M14,
M15b).

### Medium 7: the owner cannot decide from the first screen

**What the proposal changed.**
The file now opens with the seven questions,
each with the recommended answer,
its basis marked as measured,
line read or not measured,
and what a yes leaves undecided;
then the basis legend and the three facts that
would change the order.
The revision history and the finding-by-finding answers are out of the proposal and into this file.

### Medium 8: label-thin options, examples without results, decisions without rankings

**What the proposal changed.**
Every answer has "How it works",
"What it changes for good",
"What it gives up",
"Example and its result",
"Cost" and "What a yes leaves undecided",
including N,
T,
CL,
SH,
A,
C,
H,
J,
D1,
D2,
D3 and L.
Examples name the package and the visible result where a run exists and say "not measured" where none does.
Every decision carries a ranking with its criterion,
including Decisions 5,
6 and 7;
Designs CL and SH are ranked (as a tie on delivery) instead of being left outside the order.

### Low 1: M7 cited a log for Android targets that lists none

Checked:
`m7-k-with-minimal-floating-channel/log/dated-components.txt` has six lines and no Android target.
Right.
The sentence is gone with the old M7 description;
the Android targets belong to the host's dated toolchain,
not to the container's (`machine/machine-state.txt`).

### Low 2: M10's digest, its arm B directory, and `--silent`

Checked:
M10's `run.txt` prints no digest;
its arm B's skip names the floating driver directory;
`--silent` is source only (`src/run_output.rs`),
and M10 ran `--quiet`.
Right.
Every third-round run prints the linter's digest,
and M10's binary is the same 0.5.2-alpha.25
(sha256 `34ca8f26d6...`);
the proposal no longer states arm B's directory as a claim about the variable,
and it says `--silent` is read and not run.

### Low 3: M11's folder name, its missing log folder, its missing section

Checked:
right on all three.
`third-round-2026-11-28/` keeps `run.txt` and `log/` for every measurement,
and M14 and M15b carry what M11
measured,
with names that say what they show.
M11 is cited only for the rustup version and the warning text.
A section of its own was not added,
because its measurement is superseded and its log is kept.

### Low 4: stale letters and colliding labels

Checked:
the old text had "Design S" for SH,
"Answers F and P",
and "F skips (M7 arm A)",
and its measurement names and finding names both used M7 and M8.
Right.
This revision cites findings as "the third read's Medium 4" and measurements as M12 to M17,
and every measurement name in the proposal is defined in `third-round-2026-11-28/README.md`.

### Low 5: figures and corrections the replies claimed in the body

Checked:
the 2.09 GiB figure appeared only in a reply,
and Answer C's `TMPDIR` sentence was unchanged.
Right.
The figure is now beside the local providers and in Answer H's cost,
and Answer C's temporary-directory
trap is labelled a design.

### Low 6: M7's setup is not kept

Checked:
M7's folder held only the active-toolchain line.
Right.
`m7-k-with-minimal-floating-channel/inputs/rust-toolchain.toml` now holds option K's file,
copied from `m5-option-k-end-to-end/library-rust-toolchain.toml` (identical content);
M7's linter configuration was not kept and is not reconstructed.
The K1 measurement the ranking now uses,
M15a,
keeps its file's content in `log/library-file.txt` and its
configuration in the host script that wrote it (`third-round-2026-11-28/scripts/run-measurements.sh`).

### The third read's "what it still does not say"

Its closing list is answered by:

- the plan row installing a dated toolchain,
  and nothing moving the floating channel:
  "The plan row installs a dated toolchain,
  and puts the components on it" and
  "Nothing in the repository moves the floating channel";
- today's `nightly` being `c82ce1563`,
  on which 6.4.0 fails,
  and the merged unreleased fix:
  "What is true today about the compiler and Dylint";
- what a library failure returns and who meets it:
  Answer I's "What it gives up";
- the daily update's own report:
  the same paragraph;
- K1's first use and what the linter says without it:
  Answer K1 and Decision 5's Part 5;
- the lock's exported `RUSTUP_TOOLCHAIN` and I:
  "How the decisions depend on each other";
- a way to get the fix before a release:
  "What is true today",
  with its inference labelled;
- the disk cost:
  Decision 3's cost paragraph and Answer H's cost.

## The first two reads

The proposal no longer carries their answers;
each answer is in a body section now.
Where this revision changed an earlier answer,
it says so.

The first read's sixteen findings (2026-11-27) are answered by the sections named beside them:

- its F1 and F2 (the proxy arm and the shadow):
  "What `cargo dylint` itself does" and
  "The linter's message,
  the warm command,
  the tasks and the workflows",
  with M15a measuring the proxy state.
- its F3 to F8 (the mechanism,
  the options,
  the skip's shape):
  "How the rule runs",
  "What each absence produces",
  "What a skipped rule returns" and Decisions 1 to 5.
- its F9 (precedent):
  the rankings carry their own reasons;
  no answer refers to the ranking of an earlier
  draft.
- its F10 to F13:
  "The machine's state,
  and the four toolchain names" and Decision 3;
  the issue-6316 comment is quoted with its own attribution and is no longer an owner question.
- its F14 to F16 (citations,
  the evidence folder,
  the labels):
  the whole file cites by symbol and section,
  and the evidence folder list is at its end.

The second read's eighteen findings (2026-11-28) and where they stand:

- High 1 (the pin question):
  closed in the owner-record section,
  which quotes the words whole and separates
  "not approved" from "refused".
- High 2 (K never measured alone;
  I's disqualification):
  closed by M7 and M8 for their states,
  and superseded
  by M15a and M12 for today's state.
  **Changed:**
  the earlier answer called M8 the measurement of I;
  this revision says M8 measured I on a stale channel and M12 measures it on today's compiler.
- High 3 (sets not separable):
  answered by "How the decisions depend on each other".
- High 4 (F's last place):
  F is a complement and in no ranking.
- Medium 1 (what changed since the rule last ran):
  the section of that name.
  **Changed:**
  its item 3 attributed the components to a plan row that puts them on the dated toolchain;
  the plan-row section now says so.
- Medium 2 (statements with no kept line):
  M3's counts are recounted from its kept files in this revision
  (`probes/m3-counts-recounted.txt`),
  and M10 and M15a keep their status lines.
- Medium 3 (which `cargo`,
  the shadow):
  "The machine's state" and Mechanism T and Design SH.
- Medium 4 (options half-described):
  **changed.**
  Its answer for Answer I was "the plan's row installs the
  components";
  the plan row installs them on the dated toolchain,
  and Answer I now needs a new step.
- Medium 5 (pairs):
  the rankings rest on delivery,
  and the one state-based reason is gone.
- Medium 6 (the seven questions):
  the questions are seven again,
  each answerable alone.
  **Changed:**
  the old question 7 (what the 6316 answer covers) is not the owner's;
  it is a record question,
  and the owner's seventh question is where the counts live.
- Medium 7 (records not cited):
  the four records are cited where they bear,
  including the daily update's
  package check.
- Medium 8 (M3's facts):
  the counts and the wall times are in "The earlier rounds' measurements".
  **Changed:**
  the disk cost is now stated beside the provider answers,
  not only in a reply.
- Low 1 to Low 6:
  the citation style,
  the file names,
  the text defects,
  the word "reproduced",
  the tool statements and the letters are all addressed in the body.
  **Changed:**
  its L5 dispute is settled by measurement (M14,
  M15b):
  the second read was wrong about rustup
  1.28.0 and later,
  and right that the components had not been shown.

## The report of this revision

### What was run

- One image,
  `localhost/dylint-probe-2026-11-28c`,
  built at 2026-11-28T07:54:59-05:00
  (`third-round-2026-11-28/Containerfile`),
  digest `822e32a11951...`.
- Seven container runs,
  each detached,
  each capped at `--cpus=4 --memory=8g --pids-limit=1024
  --blkio-weight=10`,
  inside a `systemd-run --user --scope --slice=agent-work.slice` with limits:
  M12,
  M13,
  M14,
  M15a,
  M15b,
  M16,
  M17.
  All were removed at the end of their run,
  and the image was removed after the round.
- Reads:
  the four sources of the installer's mechanism,
  the plan and the lock,
  the linter's closure sources,
  the owner records,
  the daily update's section,
  the workflow,
  the plan's route list;
  `gh` reads of Dylint's releases,
  pull request 2203 and issue 6316;
  `curl` of today's channel manifest and commit hash;
  and the recount of M3 from its kept files.
- Nothing on this account was built,
  installed or warmed outside a container.

### The decisions as the proposal lists them

1. Which toolchain the lint library names:
   K1,
   K2,
   I.
2. How the toolchain variable reaches Dylint's children:
   T,
   N,
   CL,
   SH.
3. Who provides the driver on this machine:
   B,
   Z,
   A,
   with F as a complement.
4. Whether a pull request gets the rule:
   C,
   H,
   neither.
5. What a skip returns:
   five parts (severity,
   marker field,
   exit status,
   closing count,
   and a report for a run
   that checked nothing).
6. Whether the two Dylint tools stay at `@latest`:
   `@latest` or L.
7. Where the counts live,
   and whether the run repeats.

Combinations that cannot be built as stated:

- K1 or K2 with option F and no network at all (the dated build clones `rust-clippy`).
- A severity keyed by cause with a status keyed on any skip (a conflict of intent).
- `MONOCHROMATIC_LINT_CARGO` with the component check (the check is skipped while the variable is set).

Buildable with extra work,
and named so:

- I with C or H (different install steps and a different driver name).
- A with I (the asset goes stale at the next nightly move).
- CL or SH with K1 (the linter's own probe still reads the floating compiler).
- F with any provider (it fills caches and provides neither driver nor compiler).

Not measured,
and named so:

- T with I when the lint task inherits the lock's exported `RUSTUP_TOOLCHAIN`.
- A with the lean cleaner.

### The rankings and their criteria

- Decision 1:
  **K1 > K2 > I**,
  on findings returned on the proxies with T's variable,
  on the compiler each
  answer names.
  K1 over K2:
  whether the run happens at all (the lock names K1's toolchain,
  nothing installs
  K2's date,
  the cleaner keeps only what the lock or a checkout-root file names).
  K2 over I:
  measured delivery today.
  I last on today's delivery,
  with the release that would change it named.
- Decision 2:
  **T > CL = SH > N**,
  on the rule's runs that reach findings counting both needs.
  T meets both needs and is measured;
  CL and SH meet the first and leave the second unmet;
  N needs a person.
  CL and SH tie,
  and the tie is not broken on cost.
- Decision 3:
  **B > Z > A**,
  on reaching findings without anyone remembering a step,
  with the parts that exist
  today.
  A's reversal when a route exists is written down.
- Decision 4:
  **C > H > neither**,
  on how much of the job's own state must be right.
- Decision 5,
  part by part:
  **J > keep `warn`** (what a gate can distinguish);
  **D2 > none** (what a record-only consumer can tell);
  **by cause > any skip > none** (the reds that are true);
  **D3 > none** (what a reader sees without parsing JSONL);
  **report > leave** (whether a clean run can be told from a run that checked nothing).
- Decision 6:
  **`@latest` > L**,
  on how soon the rule picks up an upstream fix.
- Decision 7:
  **the existing record > its own record**,
  on whether the two counts of one quantity can be
  compared.

### The questions, and what a yes leaves undecided

1. Which compiler builds the library and which name the driver is filed under.
   K1:
   who writes the file,
   which lock field,
   when it is rewritten.
   K2:
   which date,
   who moves it.
   I:
   who installs the components,
   who moves the channel,
   what to do on a breaking nightly.
2. How the variable reaches the lint's `cargo` tree.
   T:
   whether the shadow gets a rule for that tree.
   CL or SH:
   what replaces the shadow's checks,
   and how the linter's own probe reads a compiler with
   `rustc-dev`.
3. Who builds the driver.
   B:
   what a lint run does without the network,
   which package warms the driver.
   Z:
   nothing further.
   A:
   the new route type,
   the driver root,
   whether a run outside `mise` reads it.
4. Whether a pull request gets the rule.
   C:
   how the checkout enters the image,
   whether the run has the network,
   where the image lives.
   H:
   which of its four steps change,
   how the cache is keyed,
   what the job costs.
5. What a run says when the rule did not check a package.
   J:
   the option's name and its causes.
   The field:
   its name.
   A status:
   the number.
   A count:
   the wording.
6. Whether the two tools stay at `@latest`.
   L:
   which version to hold.
7. Where the counts live.
   Whether the host run is repeated,
   and when.

### What I chose that the brief did not decide

- **The recommendation order,
  K1 first.**
  The brief said to restate the ranking as results plus a preference;
  it did not say which answer should lead.
  I put K1 first because it is the only answer that returns findings on the compiler the channel carries today
  and because the repository's own mechanism for "always latest" is the lock the plan moves.
  The way not taken:
  leading with I on the owner's recorded wish.
  Its cost:
  the owner is asked to weigh a preference against a measured failure,
  and the proposal states that
  as the preference to confirm rather than as a result.
- **Removing options M and E from the combinations list rather than describing them.**
  The way not taken:
  describe M (cargo's `[env]` in the library's configuration) and E (a shipped driver asset)
  and rank them.
  Its cost:
  two mechanisms the owner might have wanted are no longer offered;
  the facts they carried (that `dylint-link` reads the variable itself,
  and that an asset built for one
  toolchain carries that toolchain's RUNPATH) are kept in the text,
  so a reader can ask for them again.
- **Dropping the owner question about the 2026-11-23 comment.**
  The way not taken:
  keep it and ask the owner to interpret a text attributed to "the project".
  Its cost:
  a record question for the coordinating session now stands open in this file instead of in the
  owner's list.
- **Adding a fifth part to Decision 5 (a run that checked nothing).**
  The record has four;
  the measurement (M14 arm A) showed a shape none of them catches.
  The way not taken:
  leave it as a "not known" limit.
  Its cost:
  the owner is asked about one more part than
  the record proposed.
- **Keeping M13 in the evidence although its intended state failed.**
  The component removal failed inside the container,
  so it measures K1's file with the components also on the
  channel.
  The way not taken:
  delete it and keep only M15a.
  Its cost:
  one more folder to read,
  with its state stated in `third-round-2026-11-28/README.md`.
- **Not creating an issue for the silent pass.**
  M14 arm A shows a run that checks nothing and exits 0;
  that is a defect of the linter's `Unknown` branch,
  not of this proposal.
  The way not taken:
  open an issue for it now.
  Its cost:
  it is recorded here and in
  Decision 5,
  and the coordinating session decides whether to file it.
- **Not repeating the 30-package M3 run.**
  The way not taken:
  re-run it today so the counts stand in one state.
  Its cost:
  the counts are the
  2026-11-27 run's,
  recounted from its kept files,
  and the proposal says so.

### What was not run, and why

- No `mise` task,
  no `cargo dylint` and no driver build on this account;
  every one ran in a container,
  because the round may not write `~/.cargo`,
  `~/.rustup` or `~/.dylint_drivers`.
- No `mise run toolchain:install`,
  no `pnpm install`,
  no release,
  no workflow:
  forbidden by the brief.
- The ignored host test `host_real_run_over_wg_quicker_exempt`:
  it needs the host's driver and would write
  under the home directory.
- `cargo-shadow` in a container:
  the shadow's hand-off is read,
  not run.
- Designs CL and SH,
  Answer A's route,
  Answer C's image,
  Answer H's job:
  designs with no implementation to run.
- The daily update's own report of a broken package:
  read from its planning record,
  not run.
- A `cargo-dylint` release carrying upstream's fix:
  none exists.
- rustup 1.30:
  not installed here;
  the supported step is measured instead.

### Nothing of this round is left

- Containers:
  none (`podman ps --all` lists none of this round's names).
- Units:
  none of this round's scopes (each `systemd-run --scope` ended with its command).
- Processes:
  none.
- Guests:
  none.
- The image was removed with `podman image rm localhost/dylint-probe-2026-11-28c`.
- The worktree holds only the files this revision wrote,
  and the branch holds them in commits.

## The closing checks

- The repository's linter was run from the worktree root on every Markdown file this revision wrote:
  the proposal,
  this file,
  `../README.md`,
  `../third-round-2026-11-28/README.md` and `../m7-k-with-minimal-floating-channel/inputs/README.md`.
  After the repository's own `--fix` pass,
  each reports no finding.
- The copied third read,
  `review.md`,
  is kept verbatim and carries 779 findings of the formatter's kinds
  (774 `markdown/semantic-line-breaks`,
  5 `markdown/single-h1`).
  It is not edited,
  because it is another session's text and a reader comparing it with its scratch copy
  should find it unchanged;
  the two earlier read copies in this folder carry 1326,
  8 and 3 of the same kinds.
- `scripts/check-quotations.ts` reports `92 of 92 quotations matched`.
  Two rows that quoted `doc/planning/github-hosted-runners-for-package-checks-and-a-guest.md` were removed:
  the sentences are no longer in that record,
  and this revision does not quote it.
- Each of the five Markdown files was rendered once with `pandoc --from=gfm --to=html5` and read.
  The proposal renders as one `h1`,
  21 `h2`,
  38 `h3`,
  six ordered lists,
  39 unordered lists,
  11 blockquotes and three fenced code blocks,
  with no table and no literal emphasis or backtick left over.
- `origin/main` was merged again after the commits (66 commits),
  and the checks were run again on the merged tree.
  The merge left this round's facts standing:
  `toolchain.lock.json` still records `nightly-2026-11-16`
  and `cargo-dylint` and `dylint-link` 6.4.0.
- Nothing of this round is left:
  no container named `dylint-*`,
  no `systemd-run` unit of this round,
  no process of this round,
  and no guest.
  The image was removed with `podman image rm localhost/dylint-probe-2026-11-28c`,
  by the digest `822e32a11951a20abd04720ca0811b6b6058c45a715b3c96c2449f4ea772677d` its runs printed,
  so a rerun starts with `podman build` from `../third-round-2026-11-28/Containerfile`.
