# The gate reads, and the revisions that answered them

What this is:
every finding of the four same-vendor gate reads of
`doc/planning/monochromatic-lint-who-provides-the-dylint-driver-and-what-a-skipped-rule-returns.md`,
answered with the line or the run each answer rests on,
and the reports of the revisions that answered them.
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
- 2026-11-28,
  fourth:
  1 High,
  6 Medium,
  6 Low (`../gate-read-fourth-2026-11-28/`,
  `review.md` with its probes under `probes/`).

The third read is answered finding by finding in "The third read" below,
and the fourth in "The fourth read,
and the fourth revision that answered it" at the end.
The first two are answered in "The first two reads" by naming the section of the proposal that holds
each answer now,
because the revision took the finding-by-finding answers out of the proposal
and into this file:
a reader answering the owner's questions does not need them.

Measurements named `M12` to `M17` are the third revision's,
in `../third-round-2026-11-28/`.
`M1` to `M11` are the earlier rounds',
in their own folders beside it.
The fourth revision ran no measurement.

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

## The fourth read, and the fourth revision that answered it

The fourth read came back on 2026-11-28 with one High,
six Medium and six Low findings,
and the verdict "Not ready to put to the owner as written;
it needs a handful of text edits and no new run."
It is kept whole,
with its twelve probes,
in `../gate-read-fourth-2026-11-28/`.
Its `:NNN` citations are lines of the proposal as `git show 88980bc4bb:<path>` held it,
which is the tree this revision started from.

This revision checked every finding against the sources,
the records and the kept logs,
ran no container,
no build and no test,
and changed the proposal wherever a finding was right.
It found none of them wrong.
The section names below are the proposal's.

### High 1: question 1's recommendation holds in two states the first screen did not name

**The finding.**
"K1's lead over I in M12 against M15a exists only because the lock's compiler (`5f4effd57`,
2026-11-13) is older than the channel's",
and the second half:
"what adding the components to this host's stale channel does to the channel" is not measured.

**What I checked.**

- `../third-round-2026-11-28/m16/run.txt`:
  the library's file names `nightly-2026-11-28`,
  `arm A status 1 after 23s` with `error[E0603]`,
  and `arm B status 2` with one record at `pkg/Cargo.toml`,
  `core/processing-failure`.
- `../machine/machine-state.txt` lines 6,
  12 to 17 and 31 to 32:
  the floating channel `nightly-x86_64-unknown-linux-gnu` holds `cargo`,
  `clippy`,
  `rust-std`,
  `rustc` and `rustfmt`
  and its compiler is `5df8c02af 2026-11-16`.
- `../third-round-2026-11-28/m12/run.txt`,
  the `floating rustc` line:
  `c82ce1563 2026-11-27`.
- `toolchain.lock.json`,
  the `rust` entry:
  `version` and `packageId` are `nightly-2026-11-16`.
- `../scripts/m8-in-container.sh` lines 20 to 21:
  M8 copies `nightly-2026-11-17` over the floating name,
  so it does not show what the installer's component command does to an installed older channel.
- `doc/planning/toolchain-always-latest.md` lines 3 to 9:
  the timer was enabled on 2026-11-07;
  the second read's probe `../gate-read-second-2026-11-28/probe-daily-update-timer-and-lock-state.txt`
  shows it `disabled; inactive (dead)` now.

**What holds.**
Both halves.
M16's arm B is the expiry:
a file naming the channel's compiler returns the same `E0603` and status 2 as I.
And the compiler a fresh install resolves is not this host's channel's:
`c82ce1563` against `5df8c02af`.
No kept log shows what
`rustup toolchain install --no-self-update nightly` and
`rustup component add rustc-dev llvm-tools-preview rust-src --toolchain nightly`
do to `5df8c02af`.

**What changed.**

- Question 1 on the first screen carries the expiry in the recommended sentence,
  the two measured compilers,
  and the arm that would close the second half.
- "What would change the recommended order":
  the second bullet now names the lock moving,
  with M16 and the timer's state.
- "What is true today about the compiler and Dylint":
  a new paragraph states this host's channel,
  M8's substitution,
  and the one container arm that would settle it.
- "Measured,
  inferred,
  and not known":
  the unmeasured question is in "Not known".

### Medium 1: the order K1 over K2 over I does not follow from M12, M15a and M16

**The finding.**
"What M12,
M15a and M16 deliver" is "K1 over I while the lock lags,
and K1 equal to I once it does not";
"Nothing in the three says anything about K2";
and the reason that a stale K1 file is safe,
"which rustup keeps",
is wrong because the lean cleaner removes it.

**What I checked.**

- The three logs,
  as in High 1's list.
- `package/cargo-shadow/cli/src/lean_rustup_toolchains.rs`,
  the header lines 1 to 17,
  `pins_in` and `pinned_by`:
  the cleaner keeps a toolchain a checkout pins (the lock's `rust` version,
  `.cache/toolchain.env.json`'s `RUSTUP_TOOLCHAIN`,
  a checkout-root `rust-toolchain` file),
  one rustup's settings name as default or an override,
  one a process runs or has in its environment,
  and one younger than the grace period;
  it does not read a package-level file.
- `../third-round-2026-11-28/m17/run.txt`:
  with `RUSTUP_TOOLCHAIN` naming an absent toolchain,
  the linter's version probe fails with one record at the configuration file and status 2.

**What holds.**
Every reason the read lists is a line read or a prediction,
not a delivered result,
and "which rustup keeps" is contradicted by the cleaner's own header.

**What changed.**
The ranking is restated as what is measured (three results),
what that supports (K1 over I only while the lock lags;
equal after),
and what is not measured (K2 on its own date;
this host's channel with the components).
K1 over K2 now rests on what installs and records each date,
with M17 for an uninstalled date under T,
and on the cleaner's actual rule;
"with a compiler that moves on its own" is gone,
and the order after a release is question 8's preference instead.

### Medium 2: the preference is not asked as a preference

**The finding.**
the read found the preference named only inside Decision 1's ranking;
Question 1 said the wish "is implemented by the lock's daily move",
while the timer is disabled;
and K1 and K2 "put a package-local `rust-toolchain.toml` back" without saying so.

**What I checked.**

- `mise.toml`:
  `[tasks."toolchain:update"]` runs mpm-shadow `update`;
  `[tasks."toolchain:update:daily"]` is the timer's command.
- The second read's timer probe:
  `disabled; inactive (dead)`.
- `git log --format='%cI %h %s' -8 -- toolchain.lock.json`:
  the newest lock commits name other tools (the newest at `2026-11-28T08:20:18-05:00`),
  and the `rust` entry stands at `nightly-2026-11-16`.
- `doc/decision/rust-closure-lint-dylint.md` lines 63 to 65 and 66 to 71:
  "The package therefore has no `rust-toolchain.toml` and no `RUSTUP_TOOLCHAIN` override"
  and "Still not approved:
  ... a pin of the nightly".
- `package/linter/monochromatic-lint-closure/README.md` lines 185 to 192:
  the same arrangement,
  and "the package has no `rust-toolchain.toml` and no toolchain override of its own".

**What holds.**
All three.
The lock moves only when a person runs `toolchain:update` today.

**What changed.**

- New question 8 and Decision 8 ask the preference,
  with the mechanism,
  cost,
  example and undecided list of each side,
  and a ranking with a reason for the adjacent pair.
- Question 1's sentence now says what moves the lock today,
  and the first screen's question 8 says the same.
- K1 and K2 say the file reverses the 2026-10-18 arrangement;
  K2 says it is the pin the record lists as not approved;
  question 1 says the README's sentence changes with a yes.

### Medium 3: "needs an installer step nobody has built" is stronger than the sources bear

**The finding.**
"The two functions act on whatever channel `packageId` names,
and a test in the same package asserts exactly that for `nightly`".

**What I checked.**

- `package/mpm-shadow/cli/src/toolchain/install_tests.rs`,
  `rust_components_and_targets_use_the_package_id_channel_that_was_installed`:
  with `package_id = Some("nightly")` it asserts
  `rustup toolchain install --no-self-update nightly` and
  `rustup component add clippy --toolchain nightly`.
- `package/mpm-shadow/cli/src/toolchain/model.rs`,
  `rustup_channel`:
  the declaration's `packageId` when it has one,
  else the locked version.
- `toolchain.plan.jsonc`,
  the `rust` row:
  `packageId` is `{version}` and `env.RUSTUP_TOOLCHAIN` is `{version}`.

**What holds.**
The functions exist and take the channel;
what is missing is a plan row that names it,
a decision on what the lock and `env` record for it
(a dated `{version}` there would be installed by nothing,
which is M17's shape),
and an owner for moving the channel.

**What changed.**
Answer I's "How it works" names the two functions and the test,
its cost and undecided list say what remains,
and the plan-row section's sentence no longer says "a new installer step".

### Medium 4: the first screen does not let the owner answer, and four of five labels carry an error

**The finding.**
"no question lists its alternatives";
and the four labels:
question 1 joins arm C's findings to arm B's 87 s and says "for every package" where one package was measured,
question 2 claims the variable state of every measurement,
question 3's "other answers" covers only I,
question 5 counts four parts and then five with no recommendation per part.

**What I checked.**

- `../third-round-2026-11-28/m15a/run.txt`:
  `arm B status 0 after 87s`,
  and arm C's six records with no time line.
- `../third-round-2026-11-28/m12/run.txt`:
  arm A prints one record,
  status 2.
- `../scripts/m12-in-container.sh`,
  `m14-in-container.sh`,
  `m15a-in-container.sh`,
  `m16-in-container.sh`:
  the variable is unset in M12 arm A,
  M14 arm A and M15a arms A,
  B and D,
  and set in M15a arm C and M16 (M17 sets it too).
- `../third-round-2026-11-28/m12/run.txt`:
  the arm B failure is I's,
  at the library build.
- `package/linter/monochromatic-lint/src/closure_step_package.rs`,
  `check_package`:
  the skip and the processing failure are distinct branches,
  so a single "record and an exit status only for the causes the owner names" did not describe Decision 5's four recommendations.

**What holds.**
All four.

**What changed.**
Every item on the first screen names the answers it was chosen over,
its basis as measured or not,
and what a yes leaves undecided;
question 5 is four choices plus a defect,
each with its recommendation;
the four labels are corrected.

### Medium 5: M14 arm A is a defect, not a choice

**The finding.**
offering a "Leave it" option "asks the owner to choose that harm";
"the state it ran in is not the recommended one,
and the proposal does not say so";
and "checked nothing" rests on a positive control.

**What I checked.**

- `../third-round-2026-11-28/m14/run.txt` and `../scripts/m14-in-container.sh`:
  arm A runs `env -u RUSTUP_TOOLCHAIN RUSTUP_AUTO_INSTALL=0`,
  `a linter status 0`,
  and `a-record.txt` is empty.
- `../third-round-2026-11-28/Containerfile`:
  the image's floating channel carries `rustc-dev`,
  `llvm-tools-preview` and `rust-src`.
- `package/linter/monochromatic-lint/src/closure_step_package.rs`,
  `check_package`:
  a failed `run_dylint` and a `!run.success` each add a processing failure,
  so a `cargo dylint` that failed cannot produce an empty record.
- `../third-round-2026-11-28/m15a/run.txt` arm C:
  the positive control.
- `../third-round-2026-11-28/m17/run.txt`:
  an absent toolchain under T fails at the version probe.
- `doc/decision/rust-closure-lint-dylint.md` lines 246 to 254:
  a missing prerequisite is a warning skip,
  and "Failures that are not a missing prerequisite keep exiting with status 2".
- `gh issue view 6667 --json number,title,state,body`:
  open,
  titled "monochromatic-lint:
  a run whose Dylint driver check cannot be made checks a package,
  gets no report from the compiler,
  prints no record and exits 0".

**What holds.**
All of it.
The record points to the warning-level skip for a toolchain the machine lacks,
because it is a prerequisite with a fix rustup itself prints,
and keeps status 2 for a failure that is not a missing prerequisite.

**What changed.**

- Part 5 is a defect the work fixes without asking;
  "Leave it" is gone.
- The arm's state is stated,
  and the route to the silent pass in K1's recommended state is labelled inference.
- "Checked nothing" is labelled inferred,
  with the positive control named.
- The two shapes are stated with their mechanism,
  cost and undecided list,
  ranked,
  with the record's own position beside the recommendation and the departure named.
- The round on issue 6667 is named as the one building the fix.
- The K2 example no longer presents the silent pass as K2's result in its recommended state:
  under T it is M17's loud failure.
- The absence list and "What the measurements imply" item 8 carry the arm's state.

### Medium 6: the file does not say which questions come first

**The finding.**
"It says Decisions 5 and 7 are independent and that 3 and 4 depend on 1,
but never says which to answer first";
and one sentence says every question "is answerable without the others".

**What I checked.**
The proposal's "How the decisions depend on each other":
Decision 1's ranking assumes Decision 2's T;
Decision 3 and 4 deliver only under a compiler Dylint builds on;
Decision 5 and 7 are independent.
The owner-facing list's sentence at `:2224-2226`.

**What I found.**
The read's sorting holds,
with one addition:
question 8's preference belongs with questions 1 and 2,
because it decides which of question 1's answers to prefer.
Question 4 can wait for the read's reason,
and it also waits on question 1's compiler.

**What changed.**
A new "Which questions come first" section states the order;
the "answerable without the others" sentence is gone;
"How the decisions depend on each other" gives the reasons.

### Low 1: M15a cited for a RUNPATH entry its logs do not hold

`rg --line-number 'RUNPATH|readelf' ../third-round-2026-11-28/m15a` returns nothing;
`../third-round-2026-11-28/m13/log/b-driver-readelf.txt` line 1 holds
`Library runpath: [/root/.rustup/toolchains/nightly-2026-11-16-x86_64-unknown-linux-gnu/lib]`.
Changed:
the M15a arm B list says the entry is in M13's log,
and the two citations now name M13.

### Low 2: the `Containerfile` path

`ls ../third-round-2026-11-28/scripts/Containerfile` fails;
`../third-round-2026-11-28/Containerfile` is the file.
Changed.

### Low 3: "printed in every run log"

`rg --count-matches 'linter digest'` on the run logs:
one in M12,
M13,
M14,
M15a and M17,
none in M15b,
M16 and M10.
Changed:
the sentence names the logs that print it and says which do not,
and M10's binary is said to be the same one the host script copied,
which its own log does not show.

### Low 4: M12 arm A's record does not carry the compiler's error header

`rg --count-matches 'error\[E0603\]' ../third-round-2026-11-28/m12/log/a-linter.jsonl` is 0;
the record begins "The end of cargo's messages was:"
and `../third-round-2026-11-28/m12/log/b-warm.err` line 3 holds
``error[E0603]: enum `ErrorOutputType` is private``.
Changed:
the three places say "the tail of" the error,
and arm B is named for the header and the `dylint_linting-6.4.0/src/lib.rs:794` path.

### Low 5: M16's script comment

`../scripts/run-m16.sh` runs `podman run --detach ... "$img"`,
so M16's floating channel is the image's,
with the components its `Containerfile` installs,
not the minimal one M15a left.
Changed:
the kept script's comment,
and no other line of it.

### Low 6: M12 has no arm with the variable set

`../scripts/m12-in-container.sh` arms A and B never export `RUSTUP_TOOLCHAIN`,
and arm A runs the linter with `env -u RUSTUP_TOOLCHAIN`.
The source removes it for Dylint's children (`internal/src/rustup.rs`),
so the library build does not depend on it.
Changed:
M12's "What M12 settles" and the ranking's measured list say so.

### One thing outside the five points: question 6's reason

The proposal had "`@latest` carries it to the machine on the next daily update with no person involved".
The timer is `disabled; inactive (dead)` (the second read's probe),
so nothing moves the rows without a person.
Changed:
Decision 6's ranking and question 6 now say `@latest` picks the fix up on the next `toolchain:update` a person runs,
with no edit of the plan or the lock,
against a pin that needs an edit;
the recommendation stands.

## The fourth revision's report

### What was run

- Read whole:
  the fourth read with its twelve probes,
  the proposal,
  the third round's `run.txt` for M12,
  M14,
  M15a and M16,
  M17's `run.txt`,
  the third round's `Containerfile` and in-container scripts,
  `machine/machine-state.txt`,
  the `rust` row of `toolchain.plan.jsonc` and the `rust` entry of `toolchain.lock.json`,
  `package/cargo-shadow/cli/src/lean_rustup_toolchains.rs`,
  `package/mpm-shadow/cli/src/toolchain/model.rs` (`rustup_channel`),
  the installer test named under Medium 3,
  `package/linter/monochromatic-lint/src/closure_step_package.rs` (`check_package`),
  `doc/decision/rust-closure-lint-dylint.md` (lines 40 to 100 and 227 to 258),
  `package/linter/monochromatic-lint-closure/README.md` (lines 180 to 205),
  and `doc/planning/toolchain-always-latest.md` (lines 1 to 40).
- Ran:
  `git show`,
  `git log --format=%cI -8 -- toolchain.lock.json`,
  `rg`,
  `awk`,
  `sed --quiet`,
  `diff`,
  `node <the quotation checker>`,
  `monochromatic-lint` and `monochromatic-lint --fix` on the Markdown files,
  `pandoc --from=gfm --to=html5`,
  `gh issue view 6667 --json ...`.
- No container,
  no `cargo`,
  no `rustup`,
  no driver build,
  no test,
  no `mise` task,
  no network fetch,
  no editor,
  no `sudo`.

### The decisions as the proposal lists them

- Decision 1:
  K1,
  K2,
  I.
- Decision 2:
  T,
  N,
  CL,
  SH.
- Decision 3:
  B,
  Z,
  A,
  with option F as a complement to any of them.
- Decision 4:
  C,
  H,
  neither.
- Decision 5:
  four choices (severity,
  marker,
  status,
  closing count) and one defect whose report shape is a choice.
- Decision 6:
  keep `@latest`,
  L.
- Decision 7:
  the existing record,
  a record of their own;
  and whether the host run repeats.
- Decision 8:
  the lock's dated name,
  the rustup channel.

Not buildable as stated:

- K1 or K2 with option F and no network at all.
- A severity keyed by cause with a status keyed on any skip.
- `MONOCHROMATIC_LINT_CARGO` with the component check.

Not offered:
leaving part 5's silent pass as it is.

### The rankings and their criteria

- Decision 1,
  by findings the rule returns with T and the driver built:
  K1 > K2 > I,
  measured for K1 over I while the lock lags,
  inferred for K2.
- Decision 2,
  by the runs that reach findings and what each takes from a program it does not own:
  T > CL = SH > N.
- Decision 3,
  by whether a lint run reaches findings without anyone remembering a step:
  B > Z > A.
- Decision 4,
  by how much of the job's own state has to be right:
  C > H > neither.
- Decision 5,
  part 1 by what a gate can distinguish (J > warn),
  part 2 by what a consumer that reads records can tell (D2 > none),
  part 3 by the reds a gate gets that are true (by cause > any skip > none),
  part 4 by what a person sees without parsing JSONL (D3 > none),
  part 5 by whether a gate can tell a checked-nothing run from a clean one (processing failure > warning skip).
- Decision 6,
  by how soon the rule picks up an upstream fix:
  keep `@latest` > L.
- Decision 7,
  by whether a reader can compare the two counts:
  the existing record > its own.
- Decision 8,
  by whether a move of the compiler is recorded and repeatable:
  the lock's dated name > the channel.

### The questions, and what a yes leaves undecided

1. K1,
   K2 or I:
   the generator and its field;
   K2's date and the pin's approval;
   I's lock record and the channel's owner.
2. T,
   N,
   CL or SH:
   the shadow's rule;
   the second need for the two designs.
3. B,
   Z or A:
   the network and the warming package;
   A's route and driver root,
   whether a run outside `mise` reads it.
4. C,
   H or neither:
   the checkout's way in,
   the network,
   the image's home;
   H's four steps,
   cache key and cost.
5. The four choices and part 5's shape:
   the names and numbers;
   which other `Unknown` causes keep status 2;
   whether the record is amended.
6. `@latest` or L:
   which version to hold.
7. Where the counts live,
   and whether to repeat the run.
8. The lock's name or the channel:
   who runs the update and whether the timer returns;
   who moves the channel.

### What this revision chose that the brief did not decide

- **Question 8 numbered last.**
  The brief named the preference as a question of its own but not where it sits.
  Numbering it 1 would have renumbered all seven questions and their cross-references;
  it is numbered 8 and the first screen says it is answered with questions 1 and 2.
  The cost is that the number does not follow the reading order;
  the way not taken was a full renumber,
  which would have touched every "Decision N" and "question N" reference.
- **The preference's recommendation.**
  The lock's dated name,
  on whether a move is recorded and repeatable.
  The way not taken was to recommend the channel on the owner's "always latest" words;
  that costs the record of a move,
  and the words do not say which mechanism.
- **Part 5's recommendation.**
  A processing failure,
  with the record's own answer (a warning skip for a missing prerequisite)
  stated beside it as a departure.
  The way not taken was to recommend the record's shape;
  that costs the red a gate would otherwise get,
  and keeps a `--quiet` run looking clean.
- **The M16 script comment corrected** rather than a rerun that would print the floating components;
  a rerun is a new measurement this round may not make.
- **The fourth read kept in the evidence folder**,
  because the proposal's preamble says every read is kept whole and scratch expires.

### What was not run, and why

- One container arm would close question 1's second half
  (a floating channel frozen at `5df8c02af`,
  the components added by the installer's commands,
  then the linter).
  The brief forbids a new measurement.
- No `cargo`,
  `rustup`,
  driver build,
  `cargo dylint`,
  test or `mise` task:
  forbidden,
  and none is needed for a text revision.
- The ignored host test `host_real_run_over_wg_quicker_exempt`:
  it would write under the home directory.
- No release,
  no workflow,
  no `gh` write.

### Nothing left

No container,
no `systemd-run` unit,
no process and no guest of this revision is left.
It started none.
