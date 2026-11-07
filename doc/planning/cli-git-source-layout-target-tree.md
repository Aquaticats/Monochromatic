# Target source tree of the Git wrapper (issue 6261), proposal for the owner

Status:
proposal only,
nothing moved,
renamed or edited in the wrapper's source.
The owner rejected the crate split on 2026-11-07T02:03:01-05:00
("Not good enough, assume every crate must stand on its own
and no one else is going to use our crates to build a git wrapper.");
section "The crate split" is being redone against that rule,
and no file moves until the owner confirms the redone tree.
Drafted 2026-11-04 for the owner's review,
on top of `doc/planning/cli-git-source-layout.md`
(the owner's words and the group order live there).
The per-file tree is in [`cli-git-source-layout-target-tree-files.md`](cli-git-source-layout-target-tree-files.md);
the machine-readable map is [`cli-git-source-layout-groups.json`](cli-git-source-layout-groups.json) (key `target`).

## What to look at and how to answer

- Purpose:
  decide whether this tree,
  these crates and this move order are what the owner wants before any file moves.
- What changed since the planning doc:
  the lint split added files,
  so the group map was rescanned and extended;
  the groups were then cut into crates;
  every folder and file got a plain name;
  every change the tree needs is named.
- Inspect first:
  the crate graph (section "The crate split"),
  the open points (section "Open points for the owner"),
  and the rejected crates (section "Crates considered and rejected").
- Respond by:
  accepting or vetoing each open point;
  a veto on a crate or a name can be answered in one line,
  because the tree is generated from a table (`spec/spec3.txt` and `spec/renames.txt` in the scratch folder,
  summarised by `cli-git-source-layout-groups.json`).

## The owner's words and the rules that bind this

The owner,
2026-11-04,
verbatim:

- "Principle 2:
  every name of a dirent must be understandable to a monkey.
  If jargon is unavoidable,
  the file's top doc
  must explain everything,
  in a way that would make any monkey understand."
- "Principle 3:
  Who depends on who must be immediately visible from looking at the file structure."
- "Consider monkey someone who only knows what git commit and git push are."
- "Layout + linting fixes are the biggest priority."
- On crates:
  "We should not put everything in one crate."
  and
  "Crates obviously have to be reusable,
  otherwise there's no point.
  Also I refuse to believe C means splitting into fewer crates."
- Where crates live:
  each is its own package `package/git-policy/<name>` (crate `monochromatic-git-policy-<name>`),
  its own Cargo workspace root,
  depending on the others by `path` with an exact version pin;
  "A:
  Cargo.toml between crates,
  uses/ inside (Recommended)".
- 2026-11-07,
  on the drafted split of 46 libraries:
  "Not good enough,
  assume every crate must stand on its own
  and no one else is going to use our crates to build a git wrapper."
  So a crate whose only other user would be a Git wrapper,
  a front end for the commit service,
  a push service or a commit-landing service does not count as standing on its own.
- 2026-11-07,
  on one shared build folder (`CARGO_TARGET_DIR`) for the split packages:
  "One shared build folder,
  but first,
  I'd like you to confirm with me each pkg can stand on its own itself."
- 2026-11-07,
  where releases go:
  "deploy all releases on pnpr,
  but only publish rc and stable releases on crates.io"
  (versions ending `-alpha.N` or `-beta.N` go to pnpr only;
  `-rc.N` and plain numbers go to pnpr and crates.io;
  owner's pick "Label decides").

`AGENTS.md`,
verbatim:

- MKY:
  "Name every file and folder so someone who knows only `git commit` and `git push` understands it alone;
  long names are fine;
  jargon nothing can replace is explained in that file's top doc."
- VGW:
  "Each file and folder name says what its contents do;
  a word that fits any code (`native`,
  `core`,
  `helpers`)
  never stands alone:
  `helpers_for_git_push`,
  not `helpers`.
  Package folders are exempt."
- LYD:
  "Show who depends on whom in the tree:
  every source folder inside a crate holds a `uses/` folder of links to the
  folders its code uses."
- LYL:
  "Two source folders never use each other,
  and the package's lint fails when a folder's `uses/` folder of links and
  its code disagree."
- LYC:
  "A crate lists the crates it uses in its `Cargo.toml` dependencies,
  never in a `uses/` folder;
  cargo refuses loops
  between crates."
- LYF:
  "Files inside one folder never use each other in a loop;
  the folder's top doc lists them in dependency order,
  and
  the package's lint fails any loop."

Decision records that bear on it:

- `doc/decision/rust-dependency-catalog.md`,
  line 15:
  "Every Rust crate is its own Cargo workspace root".
  So no workspace spans the new packages,
  and no `[workspace.dependencies]` is shared;
  each new package carries its own `Cargo.toml`,
  `Cargo.lock` and `target` folder.
- `.out-of-scope/cargo-workspace.md`:
  profiles belong to each package,
  which is why a repository-wide Cargo workspace
  is out of scope.
- Since 2026-11-05 the split crates were to be `publish = false` (`doc/planning/cli-git-source-layout.md`,
  end of section
  "Splitting into crates",
  and `doc/planning/crates-io-footprint.md`).
  Superseded 2026-11-07:
  every release of every crate goes to pnpr,
  and only `-rc.N` and plain-number releases also go to crates.io.
  The wrapper's one crates.io release still happens before it depends on a crate crates.io does not hold.

## Rescan of main

Measured 2026-11-04T20:45:14-05:00 on `main` at `f44d921e24` (contains the lint merge `977e697ad5`),
with the planning doc's scanners copied unchanged into the scratch folder
(`~/temp/agent/wrapper-target-tree/scan/`:
`refs.ts`,
`crossings.ts`,
`inside.ts`,
`modules.ts`)
plus three new ones for this proposal (`engine.ts` for the crate,
folder and file graph,
`testrefs.ts` for test references,
`reexports.ts` for named re-exports).

- Production modules scanned:
  627 files;
  2607 module-to-module references;
  5456 item-level references.
  The test side:
  604 `*_tests.rs` sidecars and 29 test-only modules.
- Every file the lint split added was placed by the name of the file it was split from
  (25 files),
  except four placed by what they hold.
  Before that,
  35 references pointed upward between groups;
  after it,
  zero.
  Four decisions came out of it:
  - `push_deadline_and_patience_settings` is a basic tool (child watching needs it),
    not a push file;
  - `conflict_side_state` moves down to the readers of Git's files (the merge code needs `conflicted_paths`);
  - `editor_would_open` and `stdin_and_stdout_are_one_terminal` move from `merge_isolation_git_reads` to
    `merge_second_pass_git_runner`;
  - the group order is corrected to the planning doc's text (`push` before `worktree`);
    the old JSON had them swapped.
- The group map is updated in place (`cli-git-source-layout-groups.json`):
  157 module placements,
  the item moves of the planning doc re-keyed to the files that now define them,
  the new moves of this proposal,
  and the new key `target` that gives every production module its crate,
  folder and new file name.
- File loops between files of one group:
  17 loops touching 53 files (the planning doc said 27 and 79 before the
  lint split).
  Every one is listed with its edges in section "Loops between files,
  as they stand".

### Lint baseline

The repository's installed `monochromatic-lint` does not know the two layout rules yet,
so the linter was built from
`package/linter/monochromatic-lint` on `main` into the scratch folder
(`mise run //package/linter/monochromatic-lint:build:debug` with `CARGO_TARGET_DIR` set there),
and run on the wrapper with a throwaway configuration that switches both rules on
(`package/git-policy/cli/**/*.rs`,
severity error),
2026-11-04T20:47:48-05:00,
`main` at `f44d921e24`:

- `rust/folder-uses-links`:
  317 findings:
  288 folders use another folder without a link,
  28 flat `pub use folder::*;` glob re-exports,
  and 1 folder loop (the crate root and every folder).
- `rust/file-order-in-folder`:
  41 findings:
  27 folders without a file list in their top doc,
  and 14 file loops.
- Total 358.

A fact the owner may trip on:
the linter's README shows the file-list marker split over two lines,
but the rule reads
the exact single line `Files in this folder, each using only files listed before it:`
(`package/linter/monochromatic-lint/src/layout_top_doc.rs`,
`LIST_MARKER`).
Measured on a throwaway crate (scratch folder `rootcheck/`):
the two-line form reports "no file list",
the one-line form
passes.
The same experiment shows a crate root `lib.rs` that holds only `pub mod x;` lines needs no `uses/` folder,
and adding `pub use x::Thing;` to it makes the root use `x` and demands `src/uses/x`.
So every `lib.rs` in the target tree holds only `mod` lines and its top doc,
with no re-exports.

## The crate split

### Method

Each crate is a union of whole target folders;
the group order of the planning doc makes any union of consecutive
groups acyclic,
and cutting inside a group was checked with a script,
not by eye:
`engine.ts` reads the table,
applies the named changes of section "Changes the tree needs before files can move",
and reports loops between crates,
between folders of one crate,
and between any files (global).
With the changes applied it reports zero at all three levels.
Without them (same placement,
no named change) it reports one crate loop and 16 global file loops,
so the changes are what makes the cut valid,
not the cut alone.
Two checks were added after an early version looked clean for the wrong reasons:
named re-exports (`pub use crate::a::X;` in `b.rs`) are resolved so a use through `b` counts as a use of `a`
(284 re-export names found),
and every moved item's own uses are cut out of its source and charged to its new file.

### The crates

Forty-six libraries,
the test-fixtures library,
and the program.
Each line:
the one job,
then a plausible outside consumer.

- `git-environment-variables`:
  Says which environment variables the wrapper adds when it starts the real Git,
  and reads them back.
  Another program that could depend on it:
  any program that starts Git with a controlled environment.
- `git-index-file`:
  Reads and writes the index file that Git keeps in the hidden `.git` folder (the list of files in the next commit),
  with its extensions.
  Another program that could depend on it:
  any tool that edits a Git index without starting Git.
- `text-time-and-log-line-formats`:
  Writes text,
  time stamps,
  JSON records and warning lines the way JavaScript and the old TypeScript tools write them.
  Another program that could depend on it:
  the TypeScript twin and any tool that must print byte-identical text.
- `safe-file-saving`:
  Saves a file so a crash never leaves half of it,
  makes private scratch places,
  and cleans up paths.
  Another program that could depend on it:
  any program that saves files.
- `read-git-files-without-git`:
  Answers questions about a repository by reading the files in `.git` directly,
  which is faster than starting Git.
  Another program that could depend on it:
  any tool that wants commit,
  tree,
  ref and setting data without starting Git.
- `linux-filesystem-primitives`:
  Calls the Linux disk functions the wrapper needs:
  mount tables,
  disk kind,
  btrfs folder copies,
  file cloning,
  swapping two paths,
  and file operations that never follow a link.
  Another program that could depend on it:
  any Linux file tool.
- `starting-and-watching-programs`:
  Starts another program,
  tells which programs are running,
  and ends one that stops making progress.
  Another program that could depend on it:
  any program that runs children under a progress deadline.
- `git-lock-files`:
  Finds who holds a lock file and clears the reference locks that a crashed Git left behind.
  Another program that could depend on it:
  any tool that must wait for or clear Git locks.
- `typed-command-reader`:
  Reads the words typed after `git` the way Git reads them,
  plus the words meant for this wrapper.
  Another program that could depend on it:
  any Git wrapper or shell hook that needs to know what a command line means.
- `start-the-real-git`:
  Finds the real `git` program on the computer,
  starts it,
  and answers some of its questions without starting it.
  Another program that could depend on it:
  any Git wrapper.
- `wrapper-settings`:
  Reads `cli-git.config.jsonc`,
  Git's own settings,
  and the hooks a repository configures.
  Another program that could depend on it:
  the sibling tools that read the same settings file.
- `commit-candidate-content`:
  Loads the exact bytes a commit would save and scans them for forbidden strings.
  Another program that could depend on it:
  any pre-commit checker.
- `commit-service-messages`:
  Defines the messages the wrapper and the background commit service send each other,
  one JSON object per line,
  and names a worktree's service.
  Another program that could depend on it:
  another front end for the commit service.
- `system-journal-entries`:
  Sends log entries to the systemd journal over its native socket.
  Another program that could depend on it:
  any Linux service that logs to the journal.
- `commit-service-log-lines`:
  Writes the lines that say where time went in a commit and which Git runs the service started.
  Another program that could depend on it:
  a log reader.
- `crash-points-for-tests`:
  Lets a test stop the program at a named step by setting an environment variable.
  Another program that could depend on it:
  any crash-safety test suite.
- `durable-work-items`:
  Keeps small files that say "this landed commit still needs a push" so a crash loses nothing.
  Another program that could depend on it:
  a job queue on files.
- `rules-about-git-command-lines`:
  Decides,
  from the words typed,
  whether a command may run here and what words to add.
  Another program that could depend on it:
  any command-line guard.
- `policy-engine`:
  Runs the configured policies in order for one point in a commit or push,
  and settles their corrections.
  Another program that could depend on it:
  a policy runner.
- `standards-review-log`:
  Reviews landed commits against the written standards with one HTTPS call and keeps the log of the answers.
  Another program that could depend on it:
  a review runner.
- `systemd-user-manager-client`:
  Talks to the systemd user manager over D-Bus to start and look up units.
  Another program that could depend on it:
  any program that starts user units without libsystemd.
- `commit-service-client`:
  Asks the background commit service to save a commit,
  starting it first when nothing answers.
  Another program that could depend on it:
  another Git front end.
- `isolated-merge-recovery`:
  Records an isolated merge and finishes or discards it if the process died.
  Another program that could depend on it:
  a merge tool.
- `parallel-work-within-a-budget`:
  Runs independent jobs side by side on as many threads as the computer can spare.
  Another program that could depend on it:
  any batch tool.
- `sha-1-and-sha-256-digests`:
  Computes SHA-1 and SHA-256 of bytes,
  written out in full.
  Another program that could depend on it:
  any program (a published crate could replace it).
- `commit-service-index-lock`:
  Takes and holds Git's `index.lock` for the background commit service,
  and waits for any program's hold on it.
  Another program that could depend on it:
  another writer that must share the index with Git.
- `three-way-text-merge`:
  Merges text three ways by line and,
  where both sides changed a line,
  word by word.
  Another program that could depend on it:
  any merge tool.
- `merge-conflicts-word-by-word`:
  Tries again word by word on the files a Git merge left conflicted.
  Another program that could depend on it:
  a merge driver.
- `push-with-a-deadline`:
  Runs `git push` with its own deadline,
  stops it when it shows no sign of work,
  and runs one push at a time.
  Another program that could depend on it:
  any program that pushes.
- `upload-large-files-before-a-push`:
  Uploads Git LFS files (large pictures and videos) before a push.
  Another program that could depend on it:
  any tool that pushes LFS repositories.
- `background-push-after-commit`:
  Pushes a landed commit in the background,
  retries,
  and merges when someone else pushed first.
  Another program that could depend on it:
  a push service.
- `hidden-git-copy-for-preparing-commits`:
  Makes a hidden second Git folder that shares the saved commits,
  so a commit can be prepared without touching your work.
  Another program that could depend on it:
  a tool that prepares commits aside.
- `keep-copies-of-discarded-work`:
  Saves a copy of work that a command would throw away,
  records it,
  and removes old copies.
  Another program that could depend on it:
  a safety net for destructive Git commands.
- `land-prepared-commits`:
  Lands a batch of prepared commits on a branch and updates the index without starting Git.
  Another program that could depend on it:
  a commit-landing service.
- `markdown-autofix-check`:
  Fixes Markdown files in the same process,
  with a guard against a stuck linter.
  Another program that could depend on it:
  a Markdown policy.
- `convert-a-checkout-to-a-btrfs-subvolume`:
  Turns a project folder into a btrfs subvolume,
  which can be copied instantly,
  and can undo it.
  Another program that could depend on it:
  any btrfs migration.
- `fix-packages-after-copying-a-folder`:
  Repairs installed packages (`node_modules`,
  Cargo build state,
  toolchain launchers) after a project folder is copied.
  Another program that could depend on it:
  any worktree copy tool.
- `new-worktrees-quickly`:
  Makes `git worktree add` fast by copying,
  cloning or snapshotting,
  copies the ignored files,
  and removes or converts worktrees.
  Another program that could depend on it:
  a worktree manager.
- `dependent-version-bump-check`:
  Plans a version bump for every package that depends on a package whose version was raised.
  Another program that could depend on it:
  the pnpm wrapper.
- `shipped-policies`:
  Holds the policies the wrapper ships and the one pass that runs them all.
  Another program that could depend on it:
  a policy bundle.
- `commit-command`:
  The `git commit` command:
  prepares the commit aside,
  asks the service to land it,
  and reports.
  Another program that could depend on it:
  none above the program.
- `service-after-commit-work`:
  Does the work left after a landing:
  push,
  tidy-up and review.
  Another program that could depend on it:
  none above the program.
- `commit-service-program`:
  The background commit service itself:
  accepts requests,
  lands them in batches and answers.
  Another program that could depend on it:
  none above the program.
- `doctor-report`:
  Prints the `git cli-git doctor` report on hooks,
  settings,
  the disk and file watching.
  Another program that could depend on it:
  none above the program.
- `merge-in-isolation`:
  Completes `git merge` and `git pull` without touching staged work.
  Another program that could depend on it:
  none above the program.
- `typed-push-planning`:
  Works out what a typed `git push` would send and merges in a race that was lost.
  Another program that could depend on it:
  none above the program.
- `cli`:
  The command-line program `cli-git-native`:
  decides what each command does and runs it.
  Another program that could depend on it:
  users.
- `disposable-test-fixtures`:
  Makes disposable folders and real-Git repositories for tests,
  outside the shipped code.
  Another program that could depend on it:
  every crate's tests,
  as a dev-dependency.

### Dependency graph

Listed from the bottom:
a crate appears after every crate it uses.
It is acyclic (checked),
and each crate's `Cargo.toml` lists exactly these `path` dependencies with an exact `=` pin.
The program reaches `commit-service-program` through service mode (edge added after the review),
and
`start-the-real-git` reaches `git-lock-files` because the runner now builds and sweeps the lock ledger
(second edge added after the review;
both checked acyclic).

```text
git-environment-variables
    uses crates: none
git-index-file
    uses crates: none
text-time-and-log-line-formats
    uses crates: none
safe-file-saving
    uses crates: git-environment-variables, text-time-and-log-line-formats
read-git-files-without-git
    uses crates: git-environment-variables, git-index-file, safe-file-saving
linux-filesystem-primitives
    uses crates: none
starting-and-watching-programs
    uses crates: text-time-and-log-line-formats
git-lock-files
    uses crates: git-environment-variables, linux-filesystem-primitives, read-git-files-without-git, safe-file-saving, starting-and-watching-programs, text-time-and-log-line-formats
typed-command-reader
    uses crates: git-environment-variables, starting-and-watching-programs
start-the-real-git
    uses crates: git-environment-variables, git-index-file, git-lock-files, read-git-files-without-git, safe-file-saving, starting-and-watching-programs, text-time-and-log-line-formats, typed-command-reader
wrapper-settings
    uses crates: safe-file-saving, start-the-real-git, text-time-and-log-line-formats, typed-command-reader
commit-candidate-content
    uses crates: read-git-files-without-git, start-the-real-git, starting-and-watching-programs, text-time-and-log-line-formats, wrapper-settings
commit-service-messages
    uses crates: none
system-journal-entries
    uses crates: text-time-and-log-line-formats
commit-service-log-lines
    uses crates: commit-service-messages, start-the-real-git, starting-and-watching-programs, system-journal-entries, text-time-and-log-line-formats
crash-points-for-tests
    uses crates: starting-and-watching-programs, text-time-and-log-line-formats
durable-work-items
    uses crates: commit-service-log-lines, safe-file-saving
rules-about-git-command-lines
    uses crates: start-the-real-git, typed-command-reader
policy-engine
    uses crates: commit-candidate-content, read-git-files-without-git, rules-about-git-command-lines, start-the-real-git, starting-and-watching-programs, text-time-and-log-line-formats, typed-command-reader, wrapper-settings
standards-review-log
    uses crates: commit-service-messages, safe-file-saving, start-the-real-git, starting-and-watching-programs, text-time-and-log-line-formats
systemd-user-manager-client
    uses crates: none
commit-service-client
    uses crates: commit-candidate-content, commit-service-log-lines, commit-service-messages, crash-points-for-tests, durable-work-items, policy-engine, read-git-files-without-git, safe-file-saving, standards-review-log, start-the-real-git, starting-and-watching-programs, systemd-user-manager-client, text-time-and-log-line-formats, wrapper-settings
isolated-merge-recovery
    uses crates: commit-service-log-lines, safe-file-saving, start-the-real-git, starting-and-watching-programs, text-time-and-log-line-formats
parallel-work-within-a-budget
    uses crates: none
sha-1-and-sha-256-digests
    uses crates: none
commit-service-index-lock
    uses crates: commit-service-log-lines, commit-service-messages, crash-points-for-tests, git-lock-files, linux-filesystem-primitives, parallel-work-within-a-budget, safe-file-saving, sha-1-and-sha-256-digests, start-the-real-git, starting-and-watching-programs, typed-command-reader
three-way-text-merge
    uses crates: none
merge-conflicts-word-by-word
    uses crates: commit-service-index-lock, git-environment-variables, read-git-files-without-git, safe-file-saving, start-the-real-git, system-journal-entries, text-time-and-log-line-formats, three-way-text-merge
push-with-a-deadline
    uses crates: git-lock-files, read-git-files-without-git, safe-file-saving, start-the-real-git, starting-and-watching-programs, text-time-and-log-line-formats
upload-large-files-before-a-push
    uses crates: git-environment-variables, push-with-a-deadline, read-git-files-without-git, safe-file-saving, start-the-real-git, starting-and-watching-programs, text-time-and-log-line-formats, typed-command-reader, wrapper-settings
background-push-after-commit
    uses crates: commit-service-client, commit-service-log-lines, commit-service-messages, durable-work-items, isolated-merge-recovery, merge-conflicts-word-by-word, parallel-work-within-a-budget, push-with-a-deadline, read-git-files-without-git, safe-file-saving, start-the-real-git, starting-and-watching-programs, system-journal-entries, text-time-and-log-line-formats, upload-large-files-before-a-push, wrapper-settings
hidden-git-copy-for-preparing-commits
    uses crates: commit-candidate-content, commit-service-log-lines, git-environment-variables, merge-conflicts-word-by-word, read-git-files-without-git, safe-file-saving, start-the-real-git, system-journal-entries, text-time-and-log-line-formats, wrapper-settings
keep-copies-of-discarded-work
    uses crates: commit-service-messages, hidden-git-copy-for-preparing-commits, linux-filesystem-primitives, read-git-files-without-git, rules-about-git-command-lines, safe-file-saving, start-the-real-git, starting-and-watching-programs, text-time-and-log-line-formats, typed-command-reader, wrapper-settings
land-prepared-commits
    uses crates: commit-service-client, commit-service-index-lock, commit-service-log-lines, commit-service-messages, crash-points-for-tests, git-index-file, hidden-git-copy-for-preparing-commits, keep-copies-of-discarded-work, push-with-a-deadline, read-git-files-without-git, safe-file-saving, sha-1-and-sha-256-digests, start-the-real-git, starting-and-watching-programs, system-journal-entries, text-time-and-log-line-formats, wrapper-settings
markdown-autofix-check
    uses crates: commit-candidate-content, policy-engine, start-the-real-git, starting-and-watching-programs, wrapper-settings
convert-a-checkout-to-a-btrfs-subvolume
    uses crates: linux-filesystem-primitives, parallel-work-within-a-budget, read-git-files-without-git, safe-file-saving, sha-1-and-sha-256-digests, start-the-real-git, starting-and-watching-programs, text-time-and-log-line-formats
fix-packages-after-copying-a-folder
    uses crates: parallel-work-within-a-budget, safe-file-saving, start-the-real-git, starting-and-watching-programs, text-time-and-log-line-formats
new-worktrees-quickly
    uses crates: background-push-after-commit, commit-candidate-content, commit-service-client, commit-service-log-lines, commit-service-messages, convert-a-checkout-to-a-btrfs-subvolume, durable-work-items, fix-packages-after-copying-a-folder, linux-filesystem-primitives, parallel-work-within-a-budget, read-git-files-without-git, safe-file-saving, sha-1-and-sha-256-digests, start-the-real-git, starting-and-watching-programs, system-journal-entries, text-time-and-log-line-formats, typed-command-reader, wrapper-settings
dependent-version-bump-check
    uses crates: commit-candidate-content, policy-engine, start-the-real-git, wrapper-settings
shipped-policies
    uses crates: commit-candidate-content, dependent-version-bump-check, keep-copies-of-discarded-work, markdown-autofix-check, policy-engine, rules-about-git-command-lines, safe-file-saving, start-the-real-git, text-time-and-log-line-formats, typed-command-reader, wrapper-settings
commit-command
    uses crates: commit-candidate-content, commit-service-client, commit-service-log-lines, commit-service-messages, crash-points-for-tests, durable-work-items, git-environment-variables, git-index-file, hidden-git-copy-for-preparing-commits, keep-copies-of-discarded-work, land-prepared-commits, markdown-autofix-check, merge-conflicts-word-by-word, new-worktrees-quickly, policy-engine, read-git-files-without-git, rules-about-git-command-lines, safe-file-saving, shipped-policies, start-the-real-git, starting-and-watching-programs, system-journal-entries, text-time-and-log-line-formats, typed-command-reader, upload-large-files-before-a-push, wrapper-settings
service-after-commit-work
    uses crates: background-push-after-commit, commit-service-index-lock, commit-service-log-lines, commit-service-messages, durable-work-items, git-environment-variables, land-prepared-commits, new-worktrees-quickly, parallel-work-within-a-budget, push-with-a-deadline, read-git-files-without-git, safe-file-saving, standards-review-log, start-the-real-git, starting-and-watching-programs, system-journal-entries, upload-large-files-before-a-push, wrapper-settings
commit-service-program
    uses crates: background-push-after-commit, commit-service-index-lock, commit-service-log-lines, commit-service-messages, crash-points-for-tests, durable-work-items, git-environment-variables, land-prepared-commits, push-with-a-deadline, safe-file-saving, service-after-commit-work, start-the-real-git, starting-and-watching-programs, system-journal-entries
doctor-report
    uses crates: keep-copies-of-discarded-work, linux-filesystem-primitives, safe-file-saving, start-the-real-git, starting-and-watching-programs, text-time-and-log-line-formats, wrapper-settings
merge-in-isolation
    uses crates: background-push-after-commit, commit-service-index-lock, commit-service-log-lines, crash-points-for-tests, git-environment-variables, isolated-merge-recovery, merge-conflicts-word-by-word, policy-engine, safe-file-saving, start-the-real-git, starting-and-watching-programs, system-journal-entries, text-time-and-log-line-formats, typed-command-reader, wrapper-settings
typed-push-planning
    uses crates: background-push-after-commit, commit-candidate-content, git-environment-variables, parallel-work-within-a-budget, push-with-a-deadline, rules-about-git-command-lines, start-the-real-git, starting-and-watching-programs, text-time-and-log-line-formats, typed-command-reader, upload-large-files-before-a-push
cli
    uses crates: background-push-after-commit, commit-candidate-content, commit-command, commit-service-client, commit-service-index-lock, commit-service-program, convert-a-checkout-to-a-btrfs-subvolume, doctor-report, fix-packages-after-copying-a-folder, git-environment-variables, git-lock-files, isolated-merge-recovery, keep-copies-of-discarded-work, merge-conflicts-word-by-word, merge-in-isolation, new-worktrees-quickly, policy-engine, push-with-a-deadline, read-git-files-without-git, rules-about-git-command-lines, safe-file-saving, service-after-commit-work, shipped-policies, start-the-real-git, starting-and-watching-programs, text-time-and-log-line-formats, typed-command-reader, typed-push-planning, upload-large-files-before-a-push, wrapper-settings
disposable-test-fixtures
    uses crates: none (every other crate lists it under dev-dependencies)
```

### Crates considered and rejected

- One crate per top-level folder of the planning doc (23 groups):
  rejected,
  because a group is a layer,
  not a job.
  `basic_tools` alone holds file saving,
  JavaScript-style formats,
  parallel work,
  process facts,
  child watching,
  the journal,
  hashing and disk calls.
- A few layer crates (the owner's option B and C sketches):
  rejected by the owner's own words above.
- One crate:
  rejected by the owner ("We should not put everything in one crate.").
- `durable-syncs-and-fault-points` (`service_durable`,
  `service_fault`):
  rejected as a crate.
  `service_fault` is the service's test switch and is read by lock code below the service,
  so it is a layering fact,
  not a job.
  It becomes `crash-points-for-tests` together with `transaction_phase` (the same job:
  stop at a named step),
  and `service_durable` goes to `commit-service-index-lock`,
  its lowest user.
- `commit-transaction-records`:
  dissolved.
  `recovery_files` and `recovery_error` are file saving;
  `transaction_owner` is
  a process fact;
  `transaction_phase` is a crash point.
- `process-facts`,
  `how-long-to-wait` and `watched-child-programs`:
  merged into `starting-and-watching-programs`.
  `pid_namespace` reads `LOCK_CEILING` from the waits table and the watcher reads process facts,
  so any two of the three
  would loop,
  and no outside program wants the waits table without the watcher.
- `javascript-style-text-and-time` and `lines-printed-for-people`:
  merged into `text-time-and-log-line-formats`.
  Neither has a job of its own that a stranger would depend on.
- `hook-warnings` (two files):
  merged into `wrapper-settings`;
  both read settings and nothing else uses them.
- `built-in-policies` and `run-all-policies`:
  merged into `shipped-policies`;
  one is the list,
  the other runs it.
- `commit-service-log-lines` merged into `commit-service-messages`:
  tried twice and rejected.
  First by measurement:
  the log lines read `WaitNotes` from the index-lock crate,
  and the index-lock crate reads the
  protocol's failure code,
  so the merge made a crate loop.
  Then `WaitNotes` (plain data) moved down into the log-lines
  crate and the loop went;
  but the merged crate would give the protocol crate,
  which now has no crate dependency at all,
  a dependency on `start-the-real-git`,
  the journal and the watcher.
  It stays a crate of its own and is the weakest entry:
  its outside consumer is only a log reader.
- Weakest entries that stay,
  for the owner to veto:
  `crash-points-for-tests`,
  `commit-service-log-lines`,
  `durable-work-items`,
  `shipped-policies`,
  `markdown-autofix-check`,
  `standards-review-log`,
  `rules-about-git-command-lines`.
  Each is a unit with a use of its own inside the wrapper;
  none has a measured outside user.
- `new-worktrees-quickly` stays one crate although it is the largest:
  copying,
  the quick routes,
  the clean check,
  removal and conversion reach each other,
  and cutting it into crates made a folder loop (measured) that needs more
  code changes than the owner approved.
  It can be cut later along `is_the_worktree_clean` and `converting_a_linked_worktree`.
- `git-environment-variables`,
  `sha-1-and-sha-256-digests` and `three-way-text-merge` are one-file crates.
  They stay because other crates below the wrapper's middle (readers,
  locks,
  merge code) need them and no merge keeps
  the graph acyclic without giving a lower crate a reason to depend on a higher one.
  The SHA code duplicates what published crates do;
  replacing it is a separate choice,
  not part of this tree.

### What a crate costs

Measured on `package/catalog-specifier/core-rust`,
a small library of this repository:
besides `src/` it carries `Cargo.toml` (50 lines),
`Cargo.lock` (2755 bytes),
`mise.toml` (43 lines),
`README.md` (114 lines) and a `LICENSES/` folder with two licence texts (about 43 kB),
plus its own `target` folder.
Multiplied by the new libraries that is a few dozen of each file and about 2 MB of licence text.
The wrapper's `target` is 1.4 GB today,
and each package is its own workspace,
so dependencies such as `serde_json`
compile once per package unless the packages share one `CARGO_TARGET_DIR`.
File-enforcer's manifests globs (`package/*/*/Cargo.toml`) already cover new packages,
so the lints and workspace blocks
arrive without a new entry.

## Inside each crate

- A crate with one folder keeps its files directly in `src/`,
  listed in dependency order in the top doc of `lib.rs`.
  Its folder name from the table is only a label and appears nowhere on disk.
- A crate with several folders holds one folder per label below,
  each with a `mod.rs` that lists its files in
  dependency order,
  and a `uses/` folder of links to the folders of the same crate it uses.
  A crate root `lib.rs` holds only `mod` lines.
- No folder is nested inside another one;
  the lint then never sees a parent that uses its child.
- Folder names are plain sentences.
  The folders that exist on disk,
  with their `uses/` links:

- `text-time-and-log-line-formats`
  - `text_time_and_records_written_like_javascript/`:
    Formats that must equal JavaScript's output.
    `uses/` links:
    none.
  - `lines_the_wrapper_prints_for_the_person/`:
    Warning and debug lines,
    shell quoting for copy and run,
    and hiding passwords in URLs.
    `uses/` links:
    `text_time_and_records_written_like_javascript`.
- `read-git-files-without-git`
  - `reading_git_settings_from_files/`:
    Reads Git's settings files and proves what they say at startup.
    `uses/` links:
    none.
  - `finding_the_git_folders/`:
    Finds the Git folder and the shared Git folder of a command without starting Git.
    `uses/` links:
    `reading_git_settings_from_files`.
  - `reading_saved_commits_and_files/`:
    Reads objects,
    packs,
    trees and index entries.
    `uses/` links:
    `finding_the_git_folders`.
  - `state_shared_by_preparing_and_landing_a_commit/`:
    The repository state that preparing and landing a commit both name.
    `uses/` links:
    none.
  - `reading_branches_tags_and_markers/`:
    Reads branches,
    tags and the markers of a merge or rebase in progress.
    `uses/` links:
    `reading_saved_commits_and_files`,
    `state_shared_by_preparing_and_landing_a_commit`.
  - `settings_that_decide_how_the_index_is_read/`:
    The settings that decide what Git does to index extensions.
    `uses/` links:
    `reading_git_settings_from_files`.
- `linux-filesystem-primitives`
  - `which_disk_a_folder_is_on/`:
    Mount tables,
    the kind of disk,
    and why a disk call failed.
    `uses/` links:
    none.
  - `fast_copies_and_swaps_on_linux/`:
    Cloning a file's data and exchanging two paths.
    `uses/` links:
    `which_disk_a_folder_is_on`.
  - `file_operations_that_never_follow_a_link/`:
    File operations relative to an open folder that never follow a link.
    `uses/` links:
    none.
- `starting-and-watching-programs`
  - `how_long_the_wrapper_waits_for_each_thing/`:
    The longest waits and the push deadline setting.
    `uses/` links:
    none.
  - `starting_programs_catching_signals_and_keeping_their_output/`:
    Starts children,
    relays signals and keeps their output in files.
    `uses/` links:
    `how_long_the_wrapper_waits_for_each_thing`.
  - `which_programs_are_running_and_since_when/`:
    Process facts:
    birth identity,
    start time,
    tree,
    users of a file,
    and network connections.
    `uses/` links:
    `how_long_the_wrapper_waits_for_each_thing`.
  - `watching_a_started_program_until_it_stops_making_progress/`:
    Watches a child and ends it when it shows no sign of work.
    `uses/` links:
    `how_long_the_wrapper_waits_for_each_thing`,
    `starting_programs_catching_signals_and_keeping_their_output`,
    `which_programs_are_running_and_since_when`.
  - `recording_where_time_went/`:
    Records where one `git worktree add` spent its time.
    `uses/` links:
    `watching_a_started_program_until_it_stops_making_progress`.
  - `who_owns_a_folder/`:
    The record of which process owns a transaction folder.
    `uses/` links:
    `which_programs_are_running_and_since_when`.
- `git-lock-files`
  - `clearing_leftover_reference_locks/`:
    Judges and removes the reference lock files a killed Git left behind.
    `uses/` links:
    none.
  - `finding_who_holds_a_lock_file/`:
    Finds the programs that hold a lock file open and gathers the evidence.
    `uses/` links:
    none.
- `typed-command-reader`
  - `reading_options_the_way_git_does/`:
    Reads one command's options exactly as Git reads them.
    `uses/` links:
    none.
  - `spellings_that_switch_the_wrapper_off/`:
    The spellings of the escape hatches that several files share.
    `uses/` links:
    none.
  - `what_each_git_command_accepts/`:
    The option tables of Git commands,
    one per command,
    and the list of Git's own commands.
    `uses/` links:
    `reading_options_the_way_git_does`,
    `spellings_that_switch_the_wrapper_off`.
  - `the_words_meant_for_this_wrapper/`:
    The control words that only this wrapper understands,
    and which worktree a command targets.
    `uses/` links:
    `reading_options_the_way_git_does`,
    `spellings_that_switch_the_wrapper_off`,
    `what_each_git_command_accepts`.
- `start-the-real-git`
  - `finding_the_real_git_program/`:
    Finds the real `git` on the computer and checks its version.
    `uses/` links:
    none.
  - `starting_git_and_describing_a_run/`:
    Starts real Git with hooks off or with the caller's arguments,
    and describes a run and its answer.
    `uses/` links:
    `finding_the_real_git_program`.
  - `answering_questions_without_starting_git/`:
    Answers the wrapper's own read-only Git queries from files.
    `uses/` links:
    `starting_git_and_describing_a_run`.
  - `running_git_for_a_transaction/`:
    Runs real Git for a commit,
    with its limits and the rules for running a step again.
    `uses/` links:
    `answering_questions_without_starting_git`,
    `finding_the_real_git_program`,
    `starting_git_and_describing_a_run`.
  - `what_the_wrapper_asks_git_about_a_repository/`:
    Asks Git where the repository is and which worktree a command targets.
    `uses/` links:
    `answering_questions_without_starting_git`,
    `running_git_for_a_transaction`,
    `starting_git_and_describing_a_run`.
- `wrapper-settings`
  - `git_settings_read_once/`:
    One read of every Git setting,
    and which parts Git flushes to disk.
    `uses/` links:
    none.
  - `the_settings_file_cli_git_config_jsonc/`:
    Finds,
    reads and checks `cli-git.config.jsonc`.
    `uses/` links:
    none.
  - `hooks_a_repository_configures/`:
    Reads the hooks a repository sets and warns about hooks in the hooks folder.
    `uses/` links:
    `git_settings_read_once`,
    `the_settings_file_cli_git_config_jsonc`.
- `commit-candidate-content`
  - `reading_the_bytes_a_commit_would_save/`:
    Loads each file a commit would save,
    from the index,
    the commit before it or a prepared copy.
    `uses/` links:
    none.
  - `putting_corrected_files_into_the_private_index/`:
    Writes a corrected file into the private index a commit is prepared in.
    `uses/` links:
    `reading_the_bytes_a_commit_would_save`.
  - `scanning_those_bytes_for_forbidden_strings/`:
    Runs the repository scanner over those bytes and names its failures.
    `uses/` links:
    `putting_corrected_files_into_the_private_index`,
    `reading_the_bytes_a_commit_would_save`.
- `commit-service-messages`
  - `messages_between_the_wrapper_and_the_commit_service/`:
    The typed messages,
    their wire form and the checks on names.
    `uses/` links:
    none.
  - `naming_the_service_of_one_worktree/`:
    The name of the service instance that belongs to a worktree.
    `uses/` links:
    none.
- `rules-about-git-command-lines`
  - `rules_that_add_flags_to_a_command/`:
    Rules that add words to a command line before it runs.
    `uses/` links:
    none.
  - `rules_about_what_a_commit_may_contain/`:
    Rules about what a commit may contain and why a push scope is unknown.
    `uses/` links:
    `rules_that_add_flags_to_a_command`.
  - `rules_about_where_a_command_may_run/`:
    Rules that say a command may run only in a certain kind of worktree.
    `uses/` links:
    none.
- `commit-service-index-lock`
  - `syncing_files_off_the_waiting_path/`:
    Decides where the service's file flushes wait.
    `uses/` links:
    none.
  - `holding_the_index_lock_for_the_commit_service/`:
    Places,
    marks,
    holds and reclaims `index.lock` for the service.
    `uses/` links:
    `syncing_files_off_the_waiting_path`.
  - `waiting_for_the_index_lock_of_git/`:
    Waits for any program's hold on the lock before a command changes the checkout.
    `uses/` links:
    `holding_the_index_lock_for_the_commit_service`.
- `hidden-git-copy-for-preparing-commits`
  - `the_hidden_second_git_folder/`:
    Creates the hidden second Git folder and copies merge state into it.
    `uses/` links:
    none.
  - `replaying_changes_onto_new_commits/`:
    Re-applies prepared changes onto commits that landed in the meantime.
    `uses/` links:
    `the_hidden_second_git_folder`.
- `new-worktrees-quickly`
  - `is_the_worktree_clean/`:
    Decides without Git whether a worktree is clean enough to copy from or remove.
    `uses/` links:
    none.
  - `converting_a_linked_worktree/`:
    Converts a linked worktree into a btrfs subvolume with a journal.
    `uses/` links:
    `is_the_worktree_clean`.
  - `planning_a_worktree_add_and_what_git_made/`:
    Plans a `git worktree add` and binds it to what real Git created.
    `uses/` links:
    none.
  - `copying_ignored_files_into_a_stage/`:
    Copies the ignored state of a worktree into a private stage with a journal.
    `uses/` links:
    `planning_a_worktree_add_and_what_git_made`.
  - `making_a_worktree_quickly/`:
    The rungs that make a new worktree:
    snapshot,
    clone,
    slow route,
    trash handling.
    `uses/` links:
    `copying_ignored_files_into_a_stage`,
    `is_the_worktree_clean`,
    `planning_a_worktree_add_and_what_git_made`.
  - `copying_ignored_files_around_one_git_command/`:
    Runs the ignored-file copy around one forwarded command.
    `uses/` links:
    `copying_ignored_files_into_a_stage`,
    `making_a_worktree_quickly`.
  - `removing_a_linked_worktree/`:
    Removes a linked worktree that is a subvolume and rescues its unshown notices.
    `uses/` links:
    `is_the_worktree_clean`.
- `shipped-policies`
  - `the_built_in_policies/`: 
    `uses/` links:
    none.
  - `the_list_of_checks_every_commit_runs/`: 
    `uses/` links:
    `the_built_in_policies`.

- Every file's old path,
  new path and one-line explanation are in the companion file
  [`cli-git-source-layout-target-tree-files.md`](cli-git-source-layout-target-tree-files.md),
  one fenced tree per crate,
  files in the dependency order each top doc must list.
- File names:
  276 of the 580 existing files get a new name (the `target` table of the group map),
  the rest keep theirs because
  the name already says what the file does.
  Eight files are new (section "Changes the tree needs before files can move").
  The old folder `mod.rs` files and the old `lib.rs` are replaced by the new index files.
- Sidecar tests follow their subject:
  `<subject>_tests.rs` becomes `<new subject name>_tests.rs`,
  and a sidecar with an extra word (`candidate_store_lifecycle_tests.rs`) keeps the word after the new subject name.
  The `#[path = "..."]` line in the subject is the only place that names it.
- Jargon that no plain name replaces (index extension,
  pack,
  reference lock,
  btrfs subvolume,
  LFS,
  D-Bus) stays in the
  name where it is Git's or Linux's own word and is explained in that file's top doc,
  as MKY requires.

## Changes the tree needs before files can move

Every change is a commit that keeps the single crate building and its tests passing (see "Move order").
The loop checks above hold only with all of these applied.

### Loops between files, as they stand

Measured on `main` at `f44d921e24` with the target groups (the first number of each edge is the line of the first use):

- loop of 3:
  `child_watch`,
  `child_watch_clock_marks_and_stopping`,
  `push_progress`
  - `push/child_watch.rs:63` uses `child_watch_clock_marks_and_stopping`:
    Mark,
    Marks,
    SHORTEST_SLEEP,
    WatchFlags,
    WatchTerms,
    grace_for,
    stop_process
  - `push/child_watch.rs:45` uses `push_progress`:
    Clock,
    SAMPLE_INTERVAL
  - `push/child_watch_clock_marks_and_stopping.rs:?` (through a planned item move) uses `child_watch`:
    Patience
  - `push/child_watch_clock_marks_and_stopping.rs:17` uses `push_progress`:
    Clock
  - `push/push_progress.rs:?` (through a planned item move) uses `child_watch`:
    StopCause
- loop of 4:
  `command_options`,
  `command_options_long`,
  `command_options_short`,
  `command_options_value`
  - `command/command_options.rs:29` uses `command_options_long`:
    scan_long_option
  - `command/command_options.rs:30` uses `command_options_short`:
    scan_short_cluster
  - `command/command_options_long.rs:26` uses `command_options`:
    Occurrence,
    OptionError,
    OptionErrorKind,
    OptionSpec,
    OptionValue,
    ParseMode,
    ParsedOptions
  - `command/command_options_long.rs:29` uses `command_options_value`:
    Spelling,
    take_value
  - `command/command_options_short.rs:25` uses `command_options`:
    Occurrence,
    OptionError,
    OptionErrorKind,
    OptionSpec,
    OptionValue,
    ParseMode,
    ParsedOptions
  - `command/command_options_short.rs:28` uses `command_options_value`:
    Spelling,
    take_value
  - `command/command_options_value.rs:25` uses `command_options`:
    Arity,
    OptionError,
    OptionErrorKind,
    OptionValue
- loop of 2:
  `git_metadata`,
  `repo_read_hook`
  - `repo_read/git_metadata.rs:16` uses `repo_read_hook`:
    try_metadata
  - `repo_read/repo_read_hook.rs:14` uses `git_metadata`:
    MetadataOutput
- loop of 2:
  `real_git`,
  `real_git_resolution`
  - `invocation/real_git.rs:27` uses `real_git_resolution`:
    RecordedGit,
    ResolvedGit,
    resolve_real_git
  - `invocation/real_git_resolution.rs:36` uses `real_git`:
    RealGitNotFound,
    ResolutionInputs,
    SearchedCandidate,
    search_path_for_real_git
- loop of 2:
  `landing_location`,
  `repo_read_adapter`
  - `landing/landing_location.rs:19` uses `repo_read_adapter`:
    landing_of
  - `repo_read/repo_read_adapter.rs:13` uses `landing_location`:
    LandingLocation
- loop of 2:
  `index_lock_holder_parsers`,
  `index_lock_holders`
  - `lock/index_lock_holder_parsers.rs:19` uses `index_lock_holders`:
    Holder
  - `lock/index_lock_holders.rs:444` uses `index_lock_holder_parsers`:
    LsofProcess,
    matching_holders,
    parse_lsof_fields,
    parse_lsof_number,
    parse_restart_manager_output
- loop of 2:
  `service_index_identity`,
  `service_index_marker`
  - `landing/service_index_identity.rs:26` uses `service_index_marker`:
    marker_of
  - `landing/service_index_marker.rs:106` uses `service_index_identity`:
    BOOT_SEPARATOR,
    HolderState,
    ReaderView,
    check_procfs_view,
    current_pid_scope,
    holder_is_elsewhere,
    holder_is_gone,
    holder_is_running,
    holder_state,
    holder_state_in,
    identity_in_view,
    identity_in_view_on,
    mark_names_a_gone_holder,
    own_identity,
    pid_of_self_stat
- loop of 6:
  `service_index_attempts`,
  `service_index_claim`,
  `service_index_lock`,
  `service_index_placement`,
  `service_index_prestage`,
  `service_index_reclaim`
  - `landing/service_index_attempts.rs:27` uses `service_index_lock`:
    Announce,
    LockFailure,
    lock_path_of,
    notice_every
  - `landing/service_index_attempts.rs:29` uses `service_index_placement`:
    Platform,
    Reclaimed,
    log_placement_failure,
    place_marked,
    reclaim
  - `landing/service_index_attempts.rs:30` uses `service_index_reclaim`:
    foreign_words,
    stale_lock_words,
    unjudged_words
  - `landing/service_index_claim.rs:23` uses `service_index_placement`:
    Platform,
    place_marked
  - `landing/service_index_lock.rs:29` uses `service_index_attempts`:
    Placing,
    wait_and_place
  - `landing/service_index_lock.rs:42` uses `service_index_placement`:
    Platform,
    SYSTEM,
    log_placement_failure,
    place_marked,
    placement_failure_text,
    reclaim
  - `landing/service_index_lock.rs:35` uses `service_index_prestage`:
    Prestaged,
    take
  - `landing/service_index_lock.rs:46` uses `service_index_reclaim`:
    Reclaimed,
    foreign_words,
    remove_stale,
    stale_lock_words,
    unjudged_words
  - `landing/service_index_placement.rs:272` uses `service_index_reclaim`:
    Reclaimed,
    reclaim_if
  - `landing/service_index_prestage.rs:28` uses `service_index_lock`:
    lock_path_of
  - `landing/service_index_prestage.rs:30` uses `service_index_placement`:
    sync_file
  - `landing/service_index_reclaim.rs:23` uses `service_index_claim`:
    Claim,
    ClaimHold,
    take_claim
  - `landing/service_index_reclaim.rs:22` uses `service_index_lock`:
    lock_path_of
  - `landing/service_index_reclaim.rs:25` uses `service_index_placement`:
    Platform,
    SYSTEM
- loop of 6:
  `stale_ref_lock_found`,
  `stale_ref_lock_judge`,
  `stale_ref_lock_retire`,
  `stale_ref_lock_walk`,
  `stale_ref_locks`,
  `stopped_git_locks`
  - `lock/stale_ref_lock_found.rs:12` uses `stale_ref_lock_judge`:
    Verdict
  - `lock/stale_ref_lock_judge.rs:18` uses `stale_ref_lock_found`:
    PidFile
  - `lock/stale_ref_lock_judge.rs:21` uses `stale_ref_lock_walk`:
    pid_file_beside
  - `lock/stale_ref_lock_judge.rs:20` uses `stopped_git_locks`:
    pid_named_by
  - `lock/stale_ref_lock_retire.rs:18` uses `stale_ref_lock_found`:
    Found,
    Outcome
  - `lock/stale_ref_lock_retire.rs:19` uses `stale_ref_lock_judge`:
    Examined,
    Identity,
    Judge,
    Verdict,
    identity_of
  - `lock/stale_ref_lock_retire.rs:21` uses `stale_ref_lock_walk`:
    Candidate,
    companions_of
  - `lock/stale_ref_lock_walk.rs:13` uses `stopped_git_locks`:
    LOCK_SUFFIX,
    PID_LOCK_SUFFIX
  - `lock/stale_ref_locks.rs:25` uses `stale_ref_lock_found`:
    Found,
    Outcome,
    PidFile
  - `lock/stale_ref_locks.rs:23` uses `stale_ref_lock_judge`:
    Judge,
    Proof,
    Verdict
  - `lock/stale_ref_locks.rs:29` uses `stale_ref_lock_retire`:
    judge_and_clear
  - `lock/stale_ref_locks.rs:27` uses `stale_ref_lock_walk`:
    ref_lock_candidates
  - `lock/stopped_git_locks.rs:37` uses `stale_ref_lock_judge`:
    Judge,
    OwnEnded
  - `lock/stopped_git_locks.rs:41` uses `stale_ref_locks`:
    clear_stale_ref_locks
- loop of 4:
  `auto_push_single_flight`,
  `bounded_push`,
  `bounded_query`,
  `push_lock_clearing`
  - `push/auto_push_single_flight.rs:15` uses `bounded_query`:
    QueryOutput,
    query_window,
    run_bounded_query
  - `push/bounded_push.rs:20` uses `auto_push_single_flight`:
    PushAttempt,
    SingleFlightError
  - `push/bounded_push.rs:441` uses `push_lock_clearing`:
    run_clearing_locks
  - `push/bounded_query.rs:18` uses `auto_push_single_flight`:
    SingleFlightError
  - `push/bounded_query.rs:20` uses `bounded_push`:
    MAX_PUSH_OUTPUT,
    stop_push,
    wait_until_done
  - `lock/push_lock_clearing.rs:17` uses `auto_push_single_flight`:
    PushAttempt,
    SingleFlightError
  - `lock/push_lock_clearing.rs:18` uses `bounded_push`:
    run_bounded_push_once
- loop of 2:
  `rule_linked_worktree`,
  `rule_linked_worktree_messages`
  - `command/rule_linked_worktree.rs:48` uses `rule_linked_worktree_messages`:
    dirty_worktree_message,
    main_worktree_message,
    outside_worktree_message
  - `command/rule_linked_worktree_messages.rs:15` uses `rule_linked_worktree`:
    GuardedCommand
- loop of 5:
  `auto_push`,
  `auto_push_reconcile`,
  `push_failure_notice`,
  `push_retry`,
  `push_retry_store`
  - `push/auto_push.rs:20` uses `auto_push_reconcile`:
    RECONCILE_FAILED_PREFIX,
    RECONCILE_LINE_PREFIX,
    reconcile_divergent_push
  - `push/auto_push_reconcile.rs:31` uses `push_retry`:
    CONFLICT_PHRASE
  - `push/push_failure_notice.rs:21` uses `auto_push`:
    PUSH_FAILED_NOTE
  - `push/push_retry.rs:21` uses `push_retry_store`:
    StoredPlan,
    load,
    remove,
    save
  - `push/push_retry_store.rs:17` uses `push_failure_notice`:
    fnv_hex
- loop of 2:
  `push_stacking`,
  `stacked_merge_push`
  - `push/push_stacking.rs:35` uses `stacked_merge_push`:
    gated_merge_push
  - `push/stacked_merge_push.rs:25` uses `push_stacking`:
    STACKED_PUSH_MARKER,
    run_stacked_push
- loop of 3:
  `worktree_snapshot_source`,
  `worktree_trash`,
  `worktree_trash_dispose`
  - `worktree/worktree_snapshot_source.rs:497` uses `worktree_trash`:
    trash_location_is_ignored
  - `worktree/worktree_trash.rs:27` uses `worktree_snapshot_source`:
    SnapshotSource
  - `worktree/worktree_trash.rs:28` uses `worktree_trash_dispose`:
    Reach,
    Router,
    TRASH_REFERENCE,
    delete_trash,
    dispose_pending_trash,
    dispose_pending_trash_with,
    empty_trash
  - `worktree/worktree_trash_dispose.rs:27` uses `worktree_trash`:
    TRASH_PARENT
- loop of 3:
  `worktree_copy`,
  `worktree_ladder`,
  `worktree_ladder_run`
  - `worktree_copying/worktree_copy.rs:35` uses `worktree_ladder`:
    LadderResult,
    begin_ladder
  - `worktree/worktree_ladder.rs:32` uses `worktree_copy`:
    GitRunner,
    GitStatus,
    GitStep
  - `worktree/worktree_ladder.rs:17` uses `worktree_ladder_run`:
    Ladder,
    LadderResult,
    begin_ladder
  - `worktree/worktree_ladder_run.rs:22` uses `worktree_copy`:
    GitStatus,
    GitStep
- loop of 2:
  `conflict_side_notice`,
  `conflict_sides`
  - `conflict/conflict_side_notice.rs:15` uses `conflict_sides`:
    SavedCopy
  - `conflict/conflict_sides.rs:42` uses `conflict_side_notice`:
    clean_paths_note,
    saved_notice
- loop of 3:
  `index_reset_file`,
  `service_land_copy`,
  `service_land_follow`
  - `index/index_reset_file.rs:31` uses `service_land_follow`:
    IndexPaths,
    PrivateCopy
  - `landing/service_land_copy.rs:21` uses `service_land_follow`:
    storage_failure
  - `landing/service_land_follow.rs:269` uses `index_reset_file`:
    reset_index_file
  - `landing/service_land_follow.rs:24` uses `service_land_copy`:
    PrivateCopy

### Breaking each loop

- Option reading (`command_options`,
  `_long`,
  `_short`,
  `_value`):
  new file `command_options_types` receives
  `Occurrence`,
  `OptionError`,
  `OptionErrorKind`,
  `OptionSpec`,
  `OptionValue`,
  `ParseMode`,
  `ParsedOptions`,
  `Arity`.
- Linked-worktree rule:
  `GuardedCommand` moves from `rule_linked_worktree` to `rule_linked_worktree_messages`.
- Conflict sides:
  `SavedCopy` moves from `conflict_sides` to `conflict_side_notice`.
- Landing copy:
  `IndexPaths` moves from `service_land_follow` to `index_reset_file`;
  `index_reset_file` imports `PrivateCopy` from `service_land_copy`,
  where it is defined.
- Real Git search:
  `RealGitNotFound`,
  `ResolutionInputs`,
  `SearchedCandidate` and `search_path_for_real_git` move from
  `real_git` to `real_git_candidate`.
- Read-only Git queries:
  `MetadataOutput` moves from `git_metadata` to `repo_read_query_request_answer_and_context`.
- Index location:
  `LandingLocation` moves from `landing_location` to `repo_read_adapter`.
- Index-lock marker and identity:
  `marker_of` and `Marked` move from `service_index_marker` to `service_index_identity`.
- The six-file index-lock acquisition loop:
  new file `service_index_lock_names_and_failures` receives `Announce`,
  `LockFailure`,
  `lock_path_of`,
  `notice_every` (from `service_index_lock`) and `storage_failure`
  (from `service_land_follow`);
  `reclaim` moves from `service_index_placement` to `service_index_reclaim`,
  which already
  holds `reclaim_if`.
  Then placement uses nothing of reclaim,
  and the order
  names,
  placement,
  claim,
  reclaim,
  prestage,
  attempts,
  lock has no back edge.
- Reference locks (six files):
  new file `stale_ref_lock_names` receives `LOCK_SUFFIX`,
  `PID_LOCK_SUFFIX`,
  `pid_named_by` (from `stopped_git_locks`) and `pid_file_beside` (from `stale_ref_lock_walk`);
  `Verdict` moves from `stale_ref_lock_judge` to `stale_ref_lock_found`.
- Holder scan:
  `Holder` moves from `index_lock_holders` to `index_lock_holder_parsers`.
- Log lines and the index-lock crate:
  `WaitNotes` (plain data) moves from `service_index_found` to `service_timing`,
  so `commit-service-log-lines` does not depend on `commit-service-index-lock`.
- Review key:
  `service_review_key` (75 lines,
  uses nothing) moves from `standards-review-log` to
  `commit-service-messages`,
  so the protocol crate has no crate dependency and the index-lock crate does not pull the
  HTTPS client and `start-the-real-git` in through the protocol.
- Push runner (four files):
  new file `push_attempt_and_failure` receives `PushAttempt` and `SingleFlightError`;
  new file `bounded_push_waiting` receives `MAX_PUSH_OUTPUT`,
  `stop_push`,
  `wait_until_done` and `run_bounded_push_once`
  (all from `bounded_push`),
  which breaks the cycle between `bounded_push` and `push_lock_clearing`.
- Child watch and push progress:
  `StopCause` moves from `bounded_push` to `push_progress`;
  `Patience` is read from `push_deadline_and_patience_settings`,
  where it is defined.
- Background push (five files):
  `fnv_hex` moves from `push_failure_notice` to `auto_push_branch_key`;
  `PUSH_FAILED_NOTE` from `auto_push` to `push_failure_notice`;
  `CONFLICT_PHRASE` from `push_retry` to
  `auto_push_reconcile_basics_and_limits`.
- Push stacking and stacked merge push:
  `push_stacking` stops importing `gated_merge_push`;
  the wiring that passes it
  (today at the end of `push_stacking`) moves to its caller `entry_push_stacking_and_lfs_upload`,
  which already imports
  both.
  This goes with the first call change below.
- Snapshot trash:
  `trash_location_is_ignored` moves from `worktree_trash` to `worktree_snapshot_source`;
  `TRASH_PARENT` moves to `worktree_trash_manifest`.
- Copy and ladder:
  new file `worktree_copy_git_steps` receives `GitRunner`,
  `GitStatus` and `GitStep` from `worktree_copy`.

### The call changes, as signatures

The other agent on `refactor/git-policy-cli-fewer-closure-suppressions` is removing closure-taking functions.
So none of the changes below hands a closure;
each hands a value or an enum,
and a closure parameter that exists today
turns into an enum there too.
Drift in signatures from that branch is expected and harmless to the layout.

- Run this next (replaces the two in-process recursions `entry::plan_invocation` from `stacked_merge_push` and
  `entry::run_process` from `merge_second_pass_finish`).
  New file `what_a_command_hands_back` in `start-the-real-git`
  defines `NextCommand { arguments, environment }` and `Finished { code, stdout, stderr }`.
  - `gated_merge_push` returns `Result<NextCommand, SingleFlightError>` (the marked arguments and environment it builds
    today) and no longer calls `plan_invocation`,
    `run_planned`,
    `run_stacked_push` or `status_of_forwarded`.
  - The stacking loop in `push_stacking` becomes a step function:
    `StackingStep::{Done(i32), RunThis { command:
    NextCommand, resume: StackingState }}`,
    with the loop's state carried by value in `StackingState`.
  - `merge_second_pass_finish::run_nested` becomes `FinishStep::{Done(i32), RunThis(NextCommand)}`;
    the nesting counter
    `NESTED_FINISHES` moves to the loop.
  - Consumer:
    the top-level loop in the program crate (`entry_run_forwarded_actions`) runs the command with
    `entry::run_process`'s planning,
    feeds the status back to the step function,
    and repeats until `Done`.
    A depth guard replaces the counter.
- Current step record:
  `transaction_git` stops calling `service_busy::enter_step` and `on_landing_thread`.
  A thread-local record (step file path and detail) moves to the crate `starting-and-watching-programs`
  (the file `kinds_of_git_step` neighbour,
  new name `current_step_record`);
  the runner writes it,
  the landing code
  registers its file path there,
  and `LandingBoard` stays in `service_busy`.
  The two functions listed as removed in the
  group map are `service_busy_step_record_on_this_thread::enter_step` and `::on_landing_thread`.
- After-stop action:
  `ChildWatch::start_with_locks(waiting_for, pid, terms, locks)` and `watch_loop(... locks)` lose the
  `locks: LockLedger` parameter,
  and the line in `child_watch` that calls `LockLedger::before_start(command)` moves out.
  The watcher returns the `StopReport` it already has;
  the caller in `bounded_push` and `transaction_git` builds the
  `LockLedger` before the start and sweeps after the stop.
  The lock crate sits above the watcher,
  so the watcher cannot
  name it.
- System settings path:
  `ScopeEnvironment` in `repo_read_config_scan` gains the field `system_config: Option<PathBuf>`
  and stops calling `git_system_config::system_config_for`;
  the callers that build a `ScopeEnvironment` pass the result.
- Commands that only finish:
  `subvolume_command`,
  `doctor`,
  `worktree_convert_command` and `stacked_merge_push` return
  `Finished` and only the program crate's `entry` and `management` files build `Action`.

Two more call changes were found by the moved-item check and are new:

- `copy_file_exclusive` (moving to `private_storage`) returns `std::io::Error`;
  the copy stage wraps it in
  `WorktreeCopyError`.
  Today it returns `WorktreeCopyError`,
  a type of the worktree crate.
- `render_configuration_warning` and `render_engine_failure` (moving to `invocation_config`) call two hint functions
  of `event_hints`;
  `engine_failure_hint` and `push_hint_field` move with them.
  Compile check expected.

### Plain data types that three crates share

`CopyFacts` (with `AddPlan` and `RemovePlan`,
which are plain structs) moves to the new file `worktree_command_facts`
in `start-the-real-git`,
because `PushForward` and `CoordinatedForward`,
which the planning doc already moves down,
hold it;
leaving it in `worktree_copy_forward` would make the lock and push crates depend on the worktree crate.
The planning functions stay where they are.

### Moves of single items

The planning doc's moves,
re-keyed to the files that define the items today,
and the moves above,
from the group map
(`items`).
Each line names the item,
where it is defined today,
and its new file:

- `ChildOutcome`:
  from `starting_git_with_the_callers_arguments` in `start-the-real-git` (old file `forwarding`) to `kinds_of_git_step` in `starting-and-watching-programs`
- `outcome_of`:
  from `starting_git_with_the_callers_arguments` in `start-the-real-git` (old file `forwarding`) to `kinds_of_git_step` in `starting-and-watching-programs`
- `subcommand_of`:
  from `which_git_steps_run_again_or_end_early` in `start-the-real-git` (old file `transaction_git_step_rerun_and_end_rules`) to `kinds_of_git_step` in `starting-and-watching-programs`
- `shell_quote`:
  from `words_about_reference_locks` in `git-lock-files` (old file `stale_ref_lock_message`) to `process_numbers_in_other_containers` in `starting-and-watching-programs`
- `current_boot_id`:
  from `claim_time` in `durable-work-items` (old file `claim_time`) to `telling_one_process_from_a_reused_number` in `starting-and-watching-programs`
- `current_machine_id`:
  from `claim_time` in `durable-work-items` (old file `claim_time`) to `telling_one_process_from_a_reused_number` in `starting-and-watching-programs`
- `is_machine_id`:
  from `claim_time` in `durable-work-items` (old file `claim_time`) to `telling_one_process_from_a_reused_number` in `starting-and-watching-programs`
- `SERVICE_UNIT_NAME`:
  from `service_provision` in `commit-service-client` (old file `service_provision`) to `real_git_the_service_was_set_up_with` in `start-the-real-git`
- `runtime_unit_dir`:
  from `service_provision` in `commit-service-client` (old file `service_provision`) to `real_git_the_service_was_set_up_with` in `start-the-real-git`
- `remember_own_executable`:
  from `service_route` in `commit-service-client` (old file `service_route`) to `finding_the_real_git` in `start-the-real-git`
- `Limits`:
  from `binary_supervisor` (old file `binary_supervisor`) to `discarded_copies_settings` in `wrapper-settings`
- `append_line`:
  from `discard_audit` in `keep-copies-of-discarded-work` (old file `discard_audit`) to `debug_and_warning_lines_on_standard_error` in `text-time-and-log-line-formats`
- `effective_directory`:
  from `commit_command` in `commit-command` (old file `commit_command`) to `which_worktree_a_command_targets` in `typed-command-reader`
- `SequencerFacts`:
  from `is_a_merge_or_rebase_in_progress` in `rules-about-git-command-lines` (old file `rule_commit_sequencer`) to `answer_shapes_built_from_a_repository_layout` in `start-the-real-git`
- `SequencerState`:
  from `is_a_merge_or_rebase_in_progress` in `rules-about-git-command-lines` (old file `rule_commit_sequencer`) to `answer_shapes_built_from_a_repository_layout` in `start-the-real-git`
- `sequencer_state`:
  from `is_a_merge_or_rebase_in_progress` in `rules-about-git-command-lines` (old file `rule_commit_sequencer`) to `answer_shapes_built_from_a_repository_layout` in `start-the-real-git`
- `capture_stamp`:
  from `service_replay` in `commit-command` (old file `service_replay`) to `claim_time` in `durable-work-items`
- `LANDING_ATTEMPTS`:
  from `service_land` in `land-prepared-commits` (old file `service_land`) to `service_protocol` in `commit-service-messages`
- `Bind`:
  from `service_server_socket_binding` in `commit-service-program` (old file `service_server_socket_binding`) to `service_protocol` in `commit-service-messages`
- `work_directory`:
  from `service_work_handlers_replanning_and_acceptance` in `service-after-commit-work` (old file `service_work_handlers_replanning_and_acceptance`) to `work_item` in `durable-work-items`
- `ConversionEnvironment`:
  from `worktree_clean_content_attribute_sources` in `new-worktrees-quickly` (old file `worktree_clean_content_attribute_sources`) to `subvolume_platform` in `convert-a-checkout-to-a-btrfs-subvolume`
- `SystemAttributes`:
  from `worktree_clean_content_attribute_sources` in `new-worktrees-quickly` (old file `worktree_clean_content_attribute_sources`) to `subvolume_platform` in `convert-a-checkout-to-a-btrfs-subvolume`
- `CLAIM_PREFIX`:
  from `worktree_convert_journal` in `new-worktrees-quickly` (old file `worktree_convert_journal`) to `subvolume_preflight` in `convert-a-checkout-to-a-btrfs-subvolume`
- `enabled`:
  from `trying_in_process_answers_first` in `start-the-real-git` (old file `repo_read_hook`) to `repository_location_without_git` in `read-git-files-without-git`
- `STOP_GRACE`:
  from `pushing_until_it_ends_or_stalls` in `push-with-a-deadline` (old file `bounded_push`) to `longest_waits` in `starting-and-watching-programs`
- `positive_seconds`:
  from `how_long_a_push_may_show_nothing` in `starting-and-watching-programs` (old file `push_deadline_and_patience_settings`) to `watching_a_child_program` in `starting-and-watching-programs`
- `Patience`:
  from `how_long_a_push_may_show_nothing` in `starting-and-watching-programs` (old file `push_deadline_and_patience_settings`) to `how_long_a_push_may_show_nothing` in `starting-and-watching-programs`
- `StopCause`:
  from `pushing_until_it_ends_or_stalls` in `push-with-a-deadline` (old file `bounded_push`) to `measuring_progress_of_a_push` in `starting-and-watching-programs`
- `RESTART_COMMAND`:
  from `service_messages` in `commit-service-client` (old file `service_messages`) to `how_to_change_the_push_deadline` in `push-with-a-deadline`
- `RELEASE`:
  from `service_protocol_messages_and_name_checks` in `commit-service-messages` (old file `service_protocol_messages_and_name_checks`) to `where_worktree_add_spent_its_time` in `starting-and-watching-programs`
- `SHADOW_NO_HOOKS_NAME`:
  from `shadow_git` in `hidden-git-copy-for-preparing-commits` (old file `shadow_git`) to `warning_about_hooks_in_the_hooks_folder` in `wrapper-settings`
- `SHADOW_ROOT_NAME`:
  from `shadow_git` in `hidden-git-copy-for-preparing-commits` (old file `shadow_git`) to `warning_about_hooks_in_the_hooks_folder` in `wrapper-settings`
- `parse_size`:
  from `reading_a_cat_file_batch_reply` in `commit-candidate-content` (old file `candidate_batch`) to `discarded_copies_settings` in `wrapper-settings`
- `ENGINE_FAILURE_EXIT_CODE`:
  from `the_decision_one_command_ends_in` in `cli` (old file `action`) to `kinds_of_git_step` in `starting-and-watching-programs`
- `WorktreeIdentity`:
  from `asking_git_which_worktree_this_is` in `start-the-real-git` (old file `worktree_identity`) to `which_worktree_a_command_targets` in `typed-command-reader`
- `StopReport`:
  from `pushing_until_it_ends_or_stalls` in `push-with-a-deadline` (old file `bounded_push`) to `measuring_progress_of_a_push` in `starting-and-watching-programs`
- `EngineFailureCode`:
  from `messages_as_json_lines` in `policy-engine` (old file `diagnostics`) to `settings_of_the_worktree_a_command_runs_in` in `wrapper-settings`
- `LEGACY_CONFIG_IGNORED_CODE`:
  from `messages_as_json_lines` in `policy-engine` (old file `diagnostics`) to `settings_of_the_worktree_a_command_runs_in` in `wrapper-settings`
- `render_configuration_warning`:
  from `messages_as_json_lines` in `policy-engine` (old file `diagnostics`) to `settings_of_the_worktree_a_command_runs_in` in `wrapper-settings`
- `render_engine_failure`:
  from `messages_as_json_lines` in `policy-engine` (old file `diagnostics`) to `settings_of_the_worktree_a_command_runs_in` in `wrapper-settings`
- `start_directory`:
  from `worktree_add_plan` in `new-worktrees-quickly` (old file `worktree_add_plan`) to `clearing_locks_before_fetch_and_push` in `git-lock-files`
- `read_plain_file`:
  from `worktree_convert_journal` in `new-worktrees-quickly` (old file `worktree_convert_journal`) to `saving_private_files` in `safe-file-saving`
- `copy_file_exclusive`:
  from `worktree_copy_stage_leaf_copying` in `new-worktrees-quickly` (old file `worktree_copy_stage_leaf_copying`) to `saving_private_files` in `safe-file-saving`
- `Verdict`:
  from `service_work_handlers_replanning_and_acceptance` in `service-after-commit-work` (old file `service_work_handlers_replanning_and_acceptance`) to `work_item` in `durable-work-items`
- `PushForward`:
  from `the_decision_one_command_ends_in` in `cli` (old file `action`) to `typed_push_that_lost_a_race` in `typed-push-planning`
- `CoordinatedForward`:
  from `index_writer_forward` in `cli` (old file `index_writer_forward`) to `waiting_for_any_programs_hold_on_the_index_lock` in `commit-service-index-lock`
- `CoordinatedRun`:
  from `index_writer_forward` in `cli` (old file `index_writer_forward`) to `waiting_for_any_programs_hold_on_the_index_lock` in `commit-service-index-lock`
- `ForwardedRun`:
  from `worktree_copy_forward` in `cli` (old file `worktree_copy_forward`) to `the_git_command_a_forward_runs` in `start-the-real-git`
- `run_forwarded_git`:
  from `worktree_copy_forward` in `cli` (old file `worktree_copy_forward`) to `the_git_command_a_forward_runs` in `start-the-real-git`
- `ShippedChecks`:
  from `the_list_of_shipped_checks` in `shipped-policies` (old file `policy_checks`) to `one_complete_policy_pass` in `shipped-policies`
- `editor_would_open`:
  from `merge_isolation_git_reads` in `merge-in-isolation` (old file `merge_isolation_git_reads`) to `merge_second_pass_git_runner` in `merge-conflicts-word-by-word`
- `stdin_and_stdout_are_one_terminal`:
  from `merge_isolation_git_reads` in `merge-in-isolation` (old file `merge_isolation_git_reads`) to `merge_second_pass_git_runner` in `merge-conflicts-word-by-word`
- `Occurrence`:
  from `reading_a_command_line_option_by_option` in `typed-command-reader` (old file `command_options`) to `types_of_option_reading` in `typed-command-reader`
- `OptionError`:
  from `reading_a_command_line_option_by_option` in `typed-command-reader` (old file `command_options`) to `types_of_option_reading` in `typed-command-reader`
- `OptionErrorKind`:
  from `reading_a_command_line_option_by_option` in `typed-command-reader` (old file `command_options`) to `types_of_option_reading` in `typed-command-reader`
- `OptionSpec`:
  from `reading_a_command_line_option_by_option` in `typed-command-reader` (old file `command_options`) to `types_of_option_reading` in `typed-command-reader`
- `OptionValue`:
  from `reading_a_command_line_option_by_option` in `typed-command-reader` (old file `command_options`) to `types_of_option_reading` in `typed-command-reader`
- `ParseMode`:
  from `reading_a_command_line_option_by_option` in `typed-command-reader` (old file `command_options`) to `types_of_option_reading` in `typed-command-reader`
- `ParsedOptions`:
  from `reading_a_command_line_option_by_option` in `typed-command-reader` (old file `command_options`) to `types_of_option_reading` in `typed-command-reader`
- `Arity`:
  from `reading_a_command_line_option_by_option` in `typed-command-reader` (old file `command_options`) to `types_of_option_reading` in `typed-command-reader`
- `GuardedCommand`:
  from `rule_linked_worktree_only` in `rules-about-git-command-lines` (old file `rule_linked_worktree`) to `rejection_texts_of_the_linked_worktree_rule` in `rules-about-git-command-lines`
- `SavedCopy`:
  from `conflict_sides` in `keep-copies-of-discarded-work` (old file `conflict_sides`) to `conflict_side_notice` in `keep-copies-of-discarded-work`
- `IndexPaths`:
  from `service_land_follow` in `land-prepared-commits` (old file `service_land_follow`) to `index_reset_file` in `land-prepared-commits`
- `PrivateCopy`:
  from `service_land_copy` in `land-prepared-commits` (old file `service_land_copy`) to `service_land_copy` in `land-prepared-commits`
- `RealGitNotFound`:
  from `finding_the_real_git` in `start-the-real-git` (old file `real_git`) to `deciding_if_a_path_is_real_git` in `start-the-real-git`
- `ResolutionInputs`:
  from `finding_the_real_git` in `start-the-real-git` (old file `real_git`) to `deciding_if_a_path_is_real_git` in `start-the-real-git`
- `SearchedCandidate`:
  from `finding_the_real_git` in `start-the-real-git` (old file `real_git`) to `deciding_if_a_path_is_real_git` in `start-the-real-git`
- `search_path_for_real_git`:
  from `finding_the_real_git` in `start-the-real-git` (old file `real_git`) to `deciding_if_a_path_is_real_git` in `start-the-real-git`
- `MetadataOutput`:
  from `running_a_read_only_git_query` in `start-the-real-git` (old file `git_metadata`) to `query_request_answer_and_context` in `start-the-real-git`
- `LandingLocation`:
  from `where_the_real_index_lives` in `start-the-real-git` (old file `landing_location`) to `answer_shapes_built_from_a_repository_layout` in `start-the-real-git`
- `marker_of`:
  from `marker_inside_the_held_lock` in `commit-service-index-lock` (old file `service_index_marker`) to `who_a_lock_holder_is` in `commit-service-index-lock`
- `Marked`:
  from `marker_inside_the_held_lock` in `commit-service-index-lock` (old file `service_index_marker`) to `who_a_lock_holder_is` in `commit-service-index-lock`
- `Announce`:
  from `the_hold_on_the_index_lock` in `commit-service-index-lock` (old file `service_index_lock`) to `lock_names_and_failures` in `commit-service-index-lock`
- `LockFailure`:
  from `the_hold_on_the_index_lock` in `commit-service-index-lock` (old file `service_index_lock`) to `lock_names_and_failures` in `commit-service-index-lock`
- `lock_path_of`:
  from `the_hold_on_the_index_lock` in `commit-service-index-lock` (old file `service_index_lock`) to `lock_names_and_failures` in `commit-service-index-lock`
- `notice_every`:
  from `the_hold_on_the_index_lock` in `commit-service-index-lock` (old file `service_index_lock`) to `lock_names_and_failures` in `commit-service-index-lock`
- `storage_failure`:
  from `service_land_follow` in `land-prepared-commits` (old file `service_land_follow`) to `lock_names_and_failures` in `commit-service-index-lock`
- `reclaim`:
  from `putting_the_marked_file_in_place_as_the_lock` in `commit-service-index-lock` (old file `service_index_placement`) to `removing_a_stale_index_lock` in `commit-service-index-lock`
- `LOCK_SUFFIX`:
  from `locks_a_stopped_git_may_leave` in `git-lock-files` (old file `stopped_git_locks`) to `names_of_reference_lock_files` in `git-lock-files`
- `PID_LOCK_SUFFIX`:
  from `locks_a_stopped_git_may_leave` in `git-lock-files` (old file `stopped_git_locks`) to `names_of_reference_lock_files` in `git-lock-files`
- `pid_named_by`:
  from `locks_a_stopped_git_may_leave` in `git-lock-files` (old file `stopped_git_locks`) to `names_of_reference_lock_files` in `git-lock-files`
- `pid_file_beside`:
  from `finding_reference_lock_files` in `git-lock-files` (old file `stale_ref_lock_walk`) to `names_of_reference_lock_files` in `git-lock-files`
- `Verdict`:
  from `judging_one_reference_lock` in `git-lock-files` (old file `stale_ref_lock_judge`) to `record_of_a_judged_reference_lock` in `git-lock-files`
- `Holder`:
  from `programs_holding_a_lock_file_open` in `git-lock-files` (old file `index_lock_holders`) to `reading_the_output_of_a_holder_scan` in `git-lock-files`
- `PushAttempt`:
  from `running_one_push_at_a_time` in `push-with-a-deadline` (old file `auto_push_single_flight`) to `one_push_attempt_and_its_failure` in `push-with-a-deadline`
- `SingleFlightError`:
  from `running_one_push_at_a_time` in `push-with-a-deadline` (old file `auto_push_single_flight`) to `one_push_attempt_and_its_failure` in `push-with-a-deadline`
- `MAX_PUSH_OUTPUT`:
  from `pushing_until_it_ends_or_stalls` in `push-with-a-deadline` (old file `bounded_push`) to `waiting_for_a_push_to_end` in `push-with-a-deadline`
- `stop_push`:
  from `pushing_until_it_ends_or_stalls` in `push-with-a-deadline` (old file `bounded_push`) to `waiting_for_a_push_to_end` in `push-with-a-deadline`
- `wait_until_done`:
  from `pushing_until_it_ends_or_stalls` in `push-with-a-deadline` (old file `bounded_push`) to `waiting_for_a_push_to_end` in `push-with-a-deadline`
- `run_bounded_push_once`:
  from `pushing_until_it_ends_or_stalls` in `push-with-a-deadline` (old file `bounded_push`) to `waiting_for_a_push_to_end` in `push-with-a-deadline`
- `fnv_hex`:
  from `record_of_a_failed_backup_push` in `background-push-after-commit` (old file `push_failure_notice`) to `file_name_of_one_branch` in `background-push-after-commit`
- `PUSH_FAILED_NOTE`:
  from `pushing_a_landed_commit` in `background-push-after-commit` (old file `auto_push`) to `record_of_a_failed_backup_push` in `background-push-after-commit`
- `CONFLICT_PHRASE`:
  from `retrying_a_stopped_push` in `background-push-after-commit` (old file `push_retry`) to `words_and_limits_of_a_reconcile` in `background-push-after-commit`
- `trash_location_is_ignored`:
  from `worktree_trash` in `new-worktrees-quickly` (old file `worktree_trash`) to `worktree_snapshot_source` in `new-worktrees-quickly`
- `TRASH_PARENT`:
  from `worktree_trash` in `new-worktrees-quickly` (old file `worktree_trash`) to `worktree_trash_manifest` in `new-worktrees-quickly`
- `GitRunner`:
  from `worktree_copy` in `new-worktrees-quickly` (old file `worktree_copy`) to `worktree_copy_git_steps` in `new-worktrees-quickly`
- `GitStatus`:
  from `worktree_copy` in `new-worktrees-quickly` (old file `worktree_copy`) to `worktree_copy_git_steps` in `new-worktrees-quickly`
- `GitStep`:
  from `worktree_copy` in `new-worktrees-quickly` (old file `worktree_copy`) to `worktree_copy_git_steps` in `new-worktrees-quickly`
- `rename_noreplace`:
  from `putting_the_marked_file_in_place_as_the_lock` in `commit-service-index-lock` (old file `service_index_placement`) to `saving_private_files` in `safe-file-saving`
- `CopyFacts`:
  from `worktree_copy_forward` in `cli` (old file `worktree_copy_forward`) to `worktree_command_facts` in `start-the-real-git`
- `AddPlan`:
  from `worktree_add_plan` in `new-worktrees-quickly` (old file `worktree_add_plan`) to `worktree_command_facts` in `start-the-real-git`
- `RemovePlan`:
  from `worktree_remove_plan` in `new-worktrees-quickly` (old file `worktree_remove_plan`) to `worktree_command_facts` in `start-the-real-git`
- `engine_failure_hint`:
  from `hints_after_a_finding` in `policy-engine` (old file `event_hints`) to `settings_of_the_worktree_a_command_runs_in` in `wrapper-settings`
- `push_hint_field`:
  from `hints_after_a_finding` in `policy-engine` (old file `event_hints`) to `settings_of_the_worktree_a_command_runs_in` in `wrapper-settings`
- `WaitNotes`:
  from `what_waiting_for_the_lock_found` in `commit-service-index-lock` (old file `service_index_found`) to `service_timing` in `commit-service-log-lines`

The scan does not follow what a moved item itself uses.
This proposal's check cut every moved item out of its source
and listed its uses;
each use is at or below the new file except the ones named in the two paragraphs above.
The compiler still has the last word.

### Moves of whole files not listed above

- `worktree_add_plan_path_questions` (path and byte questions) moves to `safe-file-saving`:
  `start_directory`,
  which
  moves to the lock notice,
  uses it.
- `service_durable` and `service_fault` move down to the lock-hold crate and the crash-points crate (see rejected crates).
- `conflict_side_state`,
  `editor_would_open` and `stdin_and_stdout_are_one_terminal` as in the rescan.
- `service_review_key` moves to `commit-service-messages` (above).

### Removal of the flat re-exports

`src/native/lib.rs` re-exports 28 folders flat with `pub use folder::*;` and 2 test folders with `pub(crate) use`.
All go;
no library `lib.rs` re-exports anything (measured above:
a re-export in the root makes the root use the folder).
Every `use crate::<file>::Item;` names the folder and the file instead:
515 production files hold 3073 lines that name a sibling through `crate::` or `super::`,
and 513 test files hold
2418 more.
Across crates the path is `monochromatic_git_policy_<crate>::<folder>::<file>::Item`.
Of the 1029 items one crate uses from another,
1011 are already `pub`,
15 are `pub(crate)` and 3 are not plain
declarations;
so 15 items widen to `pub`.
In addition,
126 non-index `pub use` lines re-export a name by name;
each is
replaced by the consumers naming the defining file,
because each is a hidden use the tree cannot show.

### Tests that name a higher crate

`engine.ts` plus `testrefs.ts` list the test files whose references reach a crate the subject's crate does not use.
Thirty crate pairs,
49 test references.
For each,
the sidecar either gets a local stand-in,
or moves into the higher
crate's `tests/` folder as an integration test (which can only use that crate's public items),
or,
for `action`,
into the
program crate.
Nothing in production changes for them.

- `commit-candidate-content` tests name `commit-service-messages`:
  candidate_prediction->service_protocol
- `commit-candidate-content` tests name `policy-engine`:
  candidate_known_content->policy_content,
  candidate_known_content->policy_test_support,
  scanner_failure_code->diagnostics
- `commit-service-client` tests name `commit-service-program`:
  service_handoff->service_server
- `commit-service-index-lock` tests name `durable-work-items`:
  service_index_lock->claim_time,
  service_index_prestage->claim_time
- `commit-service-index-lock` tests name `land-prepared-commits`:
  service_index_hold->service_land_start
- `commit-service-log-lines` tests name `git-lock-files`:
  commit_timing->index_lock_evidence
- `commit-service-log-lines` tests name `keep-copies-of-discarded-work`:
  commit_timing->discard_audit
- `convert-a-checkout-to-a-btrfs-subvolume` tests name `new-worktrees-quickly`:
  subvolume_command->worktree_convert_journal,
  subvolume_index_files->worktree_clean,
  subvolume_platform->worktree_clean_content,
  subvolume_test_support->worktree_clean_content
- `doctor-report` tests name `cli`:
  doctor_filesystem->action,
  doctor_fsmonitor->action
- `git-index-file` tests name `land-prepared-commits`:
  index_full->index_reset_support_probe,
  index_untracked_io->index_reset_support_probe
- `git-index-file` tests name `sha-1-and-sha-256-digests`:
  index_full->subvolume_sha,
  index_full_write->subvolume_sha
- `keep-copies-of-discarded-work` tests name `background-push-after-commit`:
  discard_audit->push_trigger_log
- `land-prepared-commits` tests name `commit-service-program`:
  service_land->service_server
- `new-worktrees-quickly` tests name `commit-service-program`:
  worktree_trash->service_server
- `new-worktrees-quickly` tests name `service-after-commit-work`:
  worktree_removal_rescue->service_work,
  worktree_trash->service_work
- `policy-engine` tests name `cli`:
  policy_engine->action
- `policy-engine` tests name `dependent-version-bump-check`:
  event_hints->dependent_version_policy
- `policy-engine` tests name `markdown-autofix-check`:
  event_hints->policy_markdown
- `policy-engine` tests name `shipped-policies`:
  event_hints->policy_final_newline,
  event_hints->policy_forbidden_strings,
  event_hints->policy_root_context
- `read-git-files-without-git` tests name `hidden-git-copy-for-preparing-commits`:
  sequencer_marker->capture_invocation
- `read-git-files-without-git` tests name `start-the-real-git`:
  repo_read_config_typed->repo_read_query,
  repo_read_tree_cached->repo_read_query,
  sequencer_marker->recording_git,
  sequencer_marker->transaction_git
- `safe-file-saving` tests name `starting-and-watching-programs`:
  scratch_location->child_class
- `service-after-commit-work` tests name `commit-service-program`:
  service_work->service_server
- `start-the-real-git` tests name `commit-service-client`:
  real_git->service_provision,
  real_git_record->service_provision
- `start-the-real-git` tests name `land-prepared-commits`:
  transaction_git->service_busy,
  transaction_git->service_busy_step_record_on_this_thread
- `starting-and-watching-programs` tests name `typed-command-reader`:
  add_timing->command_test_support
- `system-journal-entries` tests name `typed-command-reader`:
  journald_native->command_test_support
- `typed-command-reader` tests name `keep-copies-of-discarded-work`:
  command_test_support->discard_audit
- `typed-command-reader` tests name `start-the-real-git`:
  announced_query->forwarding,
  effective_target->worktree_identity
- `wrapper-settings` tests name `keep-copies-of-discarded-work`:
  config_discarded->discarded_prune
- `wrapper-settings` tests name `land-prepared-commits`:
  config_snapshot->maintenance_decision
- `wrapper-settings` tests name `policy-engine`:
  invocation_config->repository_facts

- `test_support` (280 sidecars use it) depends on `transaction_git`,
  `worktree_copy_owner`,
  `pid_namespace` and
  `process_identity` through two functions:
  `git_context` moves to the test support of `start-the-real-git`
  and `dead_copy_owner` to the test support of `new-worktrees-quickly`.
  The rest (disposable folders,
  the `git` runner,
  the watchdog) is the crate `disposable-test-fixtures`,
  a dev-dependency
  of every crate,
  with no wrapper dependency.
- `recording_git` goes with `start-the-real-git`'s test support;
  `smart_http_support`,
  `umask_child_test_support`,
  `no_threads_child_test_support`,
  `parallel_test_support`,
  `fixture_executable` and `test_alive` go to the fixtures crate.
- The other test-only modules go to the crate of their subject (`spec/tests.txt`,
  also in the group map under
  `target_test_only_modules`).
- The sixteen `binary/*` files,
  `binary_tests.rs` and the controls that start the built program go to the program crate's
  `tests/` folder;
  `source_guard/` and its tests,
  which read the crate's own source text,
  go there too
  and read the sibling packages' `src/` folders (open point below).
- `[[test]] native_binary` (`src/native/binary/binary_tests.rs`) moves to the program crate;
  `[[test]] native_candidates` (`src/native/candidate/candidate_consumer_tests.rs`) moves to `commit-candidate-content`.

## Cargo packaging facts that bear on the tree

- The wrapper's `Cargo.toml` today sets `include` by hand (not through `file-enforcer.config.jsonc`,
  whose `cargo-base`
  entry only sets keys such as `edition`,
  `license`,
  `publish` and the lint blocks).
  Cargo ignores `exclude` when `include`
  is set (`doc/research/tests-inside-published-rust-crates.md`).
- Measured 2026-11-04 with cargo 1.101.0-nightly (31d3246e0 2026-10-27) on a scratch crate with three folders and four
  `uses/` links (`src/b/uses/a`,
  `src/c/uses/a`,
  `src/c/uses/b`),
  `cargo package --list --offline --allow-dirty`:
  - `include = ["src/**/*.rs", "/Cargo.toml", "README.md"]` lists the linked files through the links,
    including a nested copy `src/c/uses/b/uses/a/mod.rs`;
  - `include = ["src/**/*.rs", "!src/**/uses/**", "/Cargo.toml", "README.md"]` lists each file once;
  - `exclude = ["**/uses"]` without `include` lists each file once.
- After the split every package is `publish = false`,
  so nothing needs `cargo package` for crates.io,
  and an `include`
  list is not needed;
  the wrapper's one last release happens before the first `uses/` link exists
  (move order,
  step 0).
  If the owner wants the `!src/**/uses/**` pattern kept anyway for archive builds,
  `cargo package --list` is the check,
  and a `toml-keys` entry in `file-enforcer.config.jsonc` can set it.
- Exact pins:
  `doc/decision/rust-dependency-catalog.md`,
  line 57,
  says the sync writes each path dependency's `version`
  as `=<target package.version>`;
  `package/file-enforcer/cli/Cargo.toml` shows the form
  (`monochromatic-file-enforcer = { path = "../core-rust", version = "=0.1.1" }`).
  A version bump at the bottom crate therefore edits every dependent `Cargo.toml` and lockfile through that sync;
  the cascade is mechanical but touches many manifests,
  which is part of the cost of a fine split.

## Move order

Every step is one commit or one branch,
builds,
and passes the crate's tests.

0.  Step 0:
    release the wrapper's last crates.io version from the unsplit crate (the planning doc requires this before it
    depends on an unpublished crate).
    Nothing below runs earlier.
1.  Every change listed above that is not a loop break also gets its own commit before the first grouping that needs it:
    the `worktree_command_facts` move,
    the `copy_file_exclusive` error type,
    the hint-function moves,
    `rename_noreplace`,
    the removal of the named re-exports and of the test-folder root re-exports.
    Imports are rewritten wherever they
    appear (`use`,
    qualified expressions,
    type paths,
    `super::`),
    not only in `use crate::<file>` lines.
2.  Loop breaks inside the one crate:
    each bullet of "Breaking each loop" is a commit (a type moves,
    imports change).
3.  The call changes,
    one commit each,
    in this order:
    finished results,
    system settings path,
    after-stop action,
    current step record,
    run this next.
    Each keeps `mise run //package/git-policy/cli:test` green.
4.  Remove the flat re-exports:
    first rewrite every `use crate::<file>` to name the current folder
    (`crate::<old_folder>::<file>`),
    then delete the 28 `pub use` lines;
    the build proves the rewrite.
5.  Group the files into the target crates as folders of the same package:
    `src/<crate>/<folder>/<file>.rs`,
    one commit per crate,
    bottom first;
    here also the renames,
    the new index files with their top docs,
    the sidecar renames,
    the `test_support` split and the planted-control path updates (next section).
6.  Add every `uses/` link in one commit,
    then switch the two layout lint rules on for the whole package
    (enabling them crate by crate would leave the rest failing).
    They then guard acyclicity and the links;
    a use of a higher crate folder is caught only when it closes a loop,
    so the layer order itself is kept by the
    per-crate `Cargo.toml` after step 7.
    The linter's `LIST_MARKER` one-line form is used in every top doc.
7.  Lift the crate folders into packages,
    one per commit,
    in the order of the dependency graph:
    new `Cargo.toml`
    with `path` dependencies pinned `=<version>`,
    a README,
    a `mise.toml` like a sibling,
    the sidecars with it,
    the 15 `pub(crate)` items widened,
    the test references that name a higher crate fixed (see above).
    The wrapper's `Cargo.toml` loses the dependencies it no longer needs.
8.  Before the first lift,
    extract `disposable-test-fixtures` (every lifted crate uses it as a dev-dependency).
    Every generated index file declares its folders and files `pub mod` where another crate names them;
    the 15 widened items are not the only visibility work.
9.  The program crate is what is left;
    its `Cargo.toml` keeps `[[bin]]`,
    the one `[[test]]` target for the built program,
    the binstall metadata and the release profile.

Steps 5 and 6 can be done before the owner decides on the crate count,
because they only need the folder table;
the lift step is where the owner's decision on granularity bites,
and it can stop after any crate.

### How the tests and scripts follow

- Sidecars follow their subject and their `#[path]` line;
  no sidecar changes except its file name and its imports.
- The planted-control scripts (`package/git-policy/cli/bin/native-planted-controls-*.ts`),
  the fuzz drivers
  (`package/git-policy/cli.fuzz/bin/planted-*.ts`),
  the mutation tool (`package/mutation-test/cli-cargo-mutants/src/`),
  `package/git-policy/cli/mise.toml` (`--scope-prefix src/native/`,
  and the file list in the mutation task description),
  `monochromatic-lint.config.jsonc` (the `package/git-policy/cli/src/native/**/*.rs` scope near line 131),
  the e2e fixtures
  and `native-test-base.Containerfile` name source paths.
  They are updated in the same commit as the move of each file.
  Production files they name exactly,
  with their new homes:

- `src/native/candidate/candidate_batch.rs` to `package/git-policy/commit-candidate-content/src/reading_the_bytes_a_commit_would_save/reading_a_cat_file_batch_reply.rs`
- `src/native/candidate/candidate_stage.rs` to `package/git-policy/commit-candidate-content/src/reading_the_bytes_a_commit_would_save/reading_stage_records.rs`
- `src/native/command/command_add.rs` to `package/git-policy/typed-command-reader/src/what_each_git_command_accepts/command_add.rs`
- `src/native/commit/capture_invocation.rs` to `package/git-policy/hidden-git-copy-for-preparing-commits/src/the_hidden_second_git_folder/capture_invocation.rs`
- `src/native/commit/commit_args.rs` to `package/git-policy/commit-command/src/commit_args.rs`
- `src/native/commit/commit_index.rs` to `package/git-policy/commit-command/src/commit_index.rs`
- `src/native/commit/commit_preparation.rs` to `package/git-policy/commit-command/src/commit_preparation.rs`
- `src/native/commit/commit_transaction.rs` to `package/git-policy/commit-command/src/commit_transaction.rs`
- `src/native/commit/conclusion_copy.rs` to `package/git-policy/hidden-git-copy-for-preparing-commits/src/the_hidden_second_git_folder/conclusion_copy.rs`
- `src/native/commit/post_commit.rs` to `package/git-policy/commit-command/src/post_commit.rs`
- `src/native/commit/private_index_stage.rs` to `package/git-policy/commit-command/src/private_index_stage.rs`
- `src/native/config/config_file.rs` to `package/git-policy/wrapper-settings/src/the_settings_file_cli_git_config_jsonc/finding_and_reading_the_settings_file.rs`
- `src/native/config/config_loading.rs` to `package/git-policy/wrapper-settings/src/the_settings_file_cli_git_config_jsonc/deciding_how_to_load_settings.rs`
- `src/native/config/config_parse.rs` to `package/git-policy/wrapper-settings/src/the_settings_file_cli_git_config_jsonc/turning_settings_text_into_typed_settings.rs`
- `src/native/config/config_rules_file.rs` to `package/git-policy/wrapper-settings/src/the_settings_file_cli_git_config_jsonc/the_rules_file_option.rs`
- `src/native/dependent_version/dependent_version_imports.rs` to `package/git-policy/dependent-version-bump-check/src/dependent_version_imports.rs`
- `src/native/dependent_version/dependent_version_text.rs` to `package/git-policy/dependent-version-bump-check/src/dependent_version_text.rs`
- `src/native/health/external_hooks.rs` to `package/git-policy/wrapper-settings/src/hooks_a_repository_configures/hooks_set_in_git_settings.rs`
- `src/native/invocation/entry.rs` to `package/git-policy/cli/src/deciding_what_one_git_command_does.rs`
- `src/native/invocation/git_version.rs` to `package/git-policy/start-the-real-git/src/running_git_for_a_transaction/checking_the_real_git_version.rs`
- `src/native/invocation/global_arguments.rs` to `package/git-policy/typed-command-reader/src/what_each_git_command_accepts/global_arguments.rs`
- `src/native/invocation/hooks_off.rs` to `package/git-policy/start-the-real-git/src/starting_git_and_describing_a_run/starting_git_with_hooks_off.rs`
- `src/native/invocation/real_git_candidate.rs` to `package/git-policy/start-the-real-git/src/finding_the_real_git_program/deciding_if_a_path_is_real_git.rs`
- `src/native/invocation/startup_gate.rs` to `package/git-policy/cli/src/first_step_of_a_command.rs`
- `src/native/invocation/wrapped_command.rs` to `package/git-policy/cli/src/deciding_what_a_wrapped_command_does.rs`
- `src/native/invocation/wrapper_controls.rs` to `package/git-policy/typed-command-reader/src/the_words_meant_for_this_wrapper/control_words_for_the_wrapper.rs`
- `src/native/invocation/wrapper_invocation.rs` to `package/git-policy/typed-command-reader/src/the_words_meant_for_this_wrapper/removing_control_words_from_a_command.rs`
- `src/native/landing/maintenance_decision.rs` to `package/git-policy/land-prepared-commits/src/maintenance_decision.rs`
- `src/native/landing/maintenance_run.rs` to `package/git-policy/land-prepared-commits/src/maintenance_run.rs`
- `src/native/landing/service_claim_watch.rs` to `package/git-policy/service-after-commit-work/src/service_claim_watch.rs`
- `src/native/landing/service_replay.rs` to `package/git-policy/commit-command/src/service_replay.rs`
- `src/native/landing/service_work.rs` to `package/git-policy/service-after-commit-work/src/service_work.rs`
- `src/native/lock/index_lock_evidence.rs` to `package/git-policy/git-lock-files/src/finding_who_holds_a_lock_file/evidence_about_a_foreign_index_lock.rs`
- `src/native/lock/single_owner_claim.rs` to `package/git-policy/convert-a-checkout-to-a-btrfs-subvolume/src/single_owner_claim.rs`
- `src/native/markdown/markdown_linter_config.rs` to `package/git-policy/markdown-autofix-check/src/markdown_linter_config.rs`
- `src/native/markdown/markdown_linter_output.rs` to `package/git-policy/markdown-autofix-check/src/markdown_linter_output.rs`
- `src/native/outer/inner.rs` (not a production module of the scan:
  test support or a path in a test)
- `src/native/policy/event_path.rs` to `package/git-policy/policy-engine/src/repository_path_in_an_event.rs`
- `src/native/policy/policy_final_newline.rs` to `package/git-policy/shipped-policies/src/the_built_in_policies/policy_final_newline.rs`
- `src/native/policy/policy_registry.rs` to `package/git-policy/typed-command-reader/src/the_words_meant_for_this_wrapper/names_of_the_built_in_policies.rs`
- `src/native/process/process_identity.rs` to `package/git-policy/starting-and-watching-programs/src/which_programs_are_running_and_since_when/telling_one_process_from_a_reused_number.rs`
- `src/native/process/process_start.rs` to `package/git-policy/starting-and-watching-programs/src/which_programs_are_running_and_since_when/when_a_process_started.rs`
- `src/native/process/signal_relay.rs` to `package/git-policy/starting-and-watching-programs/src/starting_programs_catching_signals_and_keeping_their_output/passing_signals_to_the_child.rs`
- `src/native/push/auto_push_reconcile.rs` to `package/git-policy/background-push-after-commit/src/publishing_after_the_remote_moved.rs`
- `src/native/push/auto_push.rs` to `package/git-policy/background-push-after-commit/src/pushing_a_landed_commit.rs`
- `src/native/push/manual_push_probe.rs` to `package/git-policy/typed-push-planning/src/what_a_dry_run_push_would_send.rs`
- `src/native/push/manual_push_range.rs` to `package/git-policy/typed-push-planning/src/commits_a_push_newly_publishes.rs`
- `src/native/push/push_failure_notice.rs` to `package/git-policy/background-push-after-commit/src/record_of_a_failed_backup_push.rs`
- `src/native/push/wait_progress.rs` to `package/git-policy/starting-and-watching-programs/src/watching_a_started_program_until_it_stops_making_progress/wait_notice_clock.rs`
- `src/native/repo_read/object_abbreviation.rs` to `package/git-policy/read-git-files-without-git/src/reading_saved_commits_and_files/short_object_names.rs`
- `src/native/repo_read/repo_read_layout.rs` to `package/git-policy/read-git-files-without-git/src/finding_the_git_folders/repository_location_without_git.rs`
- `src/native/service/service_fault.rs` to `package/git-policy/crash-points-for-tests/src/fault_points_the_service_can_be_told_to_hit.rs`
- `src/native/service/service_instance.rs` to `package/git-policy/commit-service-messages/src/naming_the_service_of_one_worktree/service_instance.rs`
- `src/native/service/service_main.rs` to `package/git-policy/commit-service-program/src/the_service_as_a_mode_of_the_program.rs`
- `src/native/service/service_server.rs` to `package/git-policy/commit-service-program/src/service_server.rs`
- `src/native/shadow/replay_patch.rs` to `package/git-policy/hidden-git-copy-for-preparing-commits/src/replaying_changes_onto_new_commits/replay_patch.rs`
- `src/native/shadow/replay_reverse_apply.rs` to `package/git-policy/hidden-git-copy-for-preparing-commits/src/replaying_changes_onto_new_commits/replay_reverse_apply.rs`
- `src/native/shadow/replay_subsumption.rs` to `package/git-policy/hidden-git-copy-for-preparing-commits/src/replaying_changes_onto_new_commits/replay_subsumption.rs`
- `src/native/shadow/shadow_config.rs` to `package/git-policy/hidden-git-copy-for-preparing-commits/src/the_hidden_second_git_folder/shadow_config.rs`
- `src/native/shadow/shadow_links.rs` to `package/git-policy/hidden-git-copy-for-preparing-commits/src/the_hidden_second_git_folder/shadow_links.rs`
- `src/native/shadow/shadow_refs.rs` to `package/git-policy/hidden-git-copy-for-preparing-commits/src/the_hidden_second_git_folder/shadow_refs.rs`
- `src/native/transaction/transaction_phase.rs` to `package/git-policy/crash-points-for-tests/src/phase_markers_for_crash_tests.rs`
- `src/native/worktree_copying/worktree_copy_install_log.rs` to `package/git-policy/new-worktrees-quickly/src/copying_ignored_files_into_a_stage/worktree_copy_install_log.rs`
- `src/native/worktree_copying/worktree_copy_install.rs` to `package/git-policy/new-worktrees-quickly/src/copying_ignored_files_into_a_stage/worktree_copy_install.rs`
- `src/native/worktree_copying/worktree_copy_journal.rs` to `package/git-policy/new-worktrees-quickly/src/copying_ignored_files_into_a_stage/worktree_copy_journal.rs`
- `src/native/worktree_copying/worktree_copy_recovery.rs` to `package/git-policy/new-worktrees-quickly/src/copying_ignored_files_into_a_stage/worktree_copy_recovery.rs`
- `src/native/worktree_copying/worktree_copy_rollback.rs` to `package/git-policy/new-worktrees-quickly/src/copying_ignored_files_into_a_stage/worktree_copy_rollback.rs`
- `src/native/worktree_copying/worktree_copy.rs` to `package/git-policy/new-worktrees-quickly/src/copying_ignored_files_around_one_git_command/worktree_copy.rs`
- `src/native/worktree/worktree_convert_journal.rs` to `package/git-policy/new-worktrees-quickly/src/converting_a_linked_worktree/worktree_convert_journal.rs`

- After each planted-control path update,
  `bin/native-planted-controls-proof.ts` is the check that the mutated line still
  exists;
  a path that no longer exists must fail there,
  not pass silently.
- Documents that name `src/native` paths (`README.md`,
  `SPEC.md`,
  `DESIGN.md`,
  `doc/` entries in the package) are updated
  once at the end of step 5;
  they are prose,
  not tooling.

## Open points for the owner

Each has a recommendation;
they go into the owner's question file.

- Crate count.
  The tree has one library per job found,
  46 of them plus the fixtures crate.
  Cost is in "What a crate costs".
  Recommendation:
  accept the list as the target,
  do the in-package steps (grouping and lint) now,
  and lift crates one at a time from the
  bottom,
  stopping to ask whether to continue after the lower half;
  the weak entries listed above are the first to merge back.
- One shared `CARGO_TARGET_DIR` for all packages of the family.
  Without it each of about 48 packages compiles
  its dependencies again.
  Recommendation:
  yes,
  set in the packages' `mise.toml` environment;
  profiles still come from each
  package.
- Where `FailureCode` lives.
  It is the service's closed list of refusal reasons,
  defined in the protocol;
  the index-lock
  crate reports with it,
  so `commit-service-index-lock` depends on `commit-service-messages`,
  which depends on no crate
  (the review key moved into it).
  Recommendation:
  accept;
  the alternative is a tiny crate for the code list.
- Where `service_fault` and `service_durable` live (crash-points crate and index-lock crate).
  Recommendation:
  accept.
- Source-text controls (`source_guard` and its tests) in the program crate reading the sibling packages' `src/` folders.
  Recommendation:
  accept,
  since they police the whole family and the program is the only crate that sees all of it.
- `cli` as a package folder name does not pass MKY alone (VGW exempts package folders,
  MKY does not).
  Recommendation:
  keep `cli`,
  because the lock row `cargo:monochromatic-git-policy-cli` and the release archive name it,
  and explain it in
  the README;
  the new package names are plain.
- `PKG` says a package is complete only with tests covering every exported code path;
  the new libraries export about a
  thousand items between them (all but 15 are `pub` today).
  Recommendation:
  the sidecars move with their subjects
  and keep covering them;
  no new tests are written for the move.
- The linter README example of the file list is split over two lines and fails the rule it documents (above).
  Recommendation:
  fix the README to one line;
  it is outside this proposal.
- Two new call changes (copy error type,
  hint functions) were found by the moved-item check.
  Recommendation:
  accept both.
- The `include` list:
  drop it at the split,
  because nothing is published afterwards.
  Recommendation:
  drop.
- The exact-pin cascade touches every dependent manifest on each bump.
  Recommendation:
  rely on the catalog sync and
  say in the sync's doc that the wrapper family needs it.

## Review by the cross-vendor reviewer

The reviewer was `openai/gpt-6.1-sol` (thinking xhigh),
given the proposal,
the companion and the planning doc,
with no tools.
It read documents only and did not rerun the scanners.
Its findings,
and what changed:

- Missing crate edges (`start-the-real-git` to `git-lock-files` for the lock sweep;
  `cli` to `commit-service-program`):
  correct.
  Both edges were added to the checked graph and stay acyclic.
- The move order left changes unscheduled,
  and extraction order omitted the fixtures crate:
  correct.
  The move order now schedules each non-loop change as its own commit,
  rewrites every path form,
  extracts
  `disposable-test-fixtures` first and says that index files declare `pub mod` where another crate names a module.
- Lint rollout one folder at a time cannot be clean,
  and the layer guarantee was too strong:
  correct.
  The rollout is
  now one commit for all links,
  and the text says the rules catch loops,
  not every layer breach.
- Package names and many retained file names fail MKY alone (`cli`,
  `policy-engine`,
  `typed-command-reader`,
  `inner_git_words`,
  `service_busy`,
  `service_route`,
  `worktree_ladder` and others):
  correct and not yet changed.
  The names are one table (`target` in the group map) and the owner's veto round is the place to settle them;
  the reviewer's replacements are kept in the open points below.
- `new-worktrees-quickly` rejection unsupported by the folder graph:
  partly.
  The early three-crate cut that looped
  was the copy,
  ladder and clean-check cut,
  before `worktree_path_failure_message` moved down;
  with that file moved the
  clean-check folder is a leaf.
  The text keeps the crate whole but names it as the first candidate to cut.
- Crates with no outside consumer (`commit-command`,
  `doctor-report`,
  `merge-in-isolation`,
  `service-after-commit-work`,
  `typed-push-planning`,
  `commit-service-program`):
  accepted as a finding.
  They are orchestration;
  folding the first four
  and the push planner into `cli` is cycle-free and is recorded as an open point.
- Other findings kept as open points:
  module visibility,
  the scanner's `cfg` scope and other surfaces it may not see
  (`self::`,
  macros,
  trait impls),
  upward test cases needing concrete plans,
  `durable-work-items` and
  `starting-and-watching-programs` claiming more independence than they have,
  misleading homes for `shell_quote`,
  `append_line`,
  `RELEASE` and `ENGINE_FAILURE_EXIT_CODE`,
  the manifest migration inventory,
  tooling updated twice
  (intermediate and final paths),
  the missing fixtures tree and `main.rs` in the companion.
- Claims withdrawn:
  that single-file crates cannot merge without a cycle (`three-way-text-merge` into its only
  consumer would be cycle-free;
  it stays for reuse),
  and "zero loops" without its scope (the scanner reads `crate::`,
  `super::`,
  grouped imports and named re-exports;
  other surfaces are unchecked).

Open points added by the review:

- Fold `commit-command`,
  `doctor-report`,
  `merge-in-isolation` and `typed-push-planning` into `cli`,
  and
  `service-after-commit-work` into `commit-service-program`.
  Recommendation:
  fold them unless the owner names an outside user.
- Rename packages and files that fail MKY alone,
  using the reviewer's examples as a start.
  Recommendation:
  rename in
  step 5,
  where files move anyway.
- Split `new-worktrees-quickly` along `is_the_worktree_clean`.
  Recommendation:
  yes,
  as the first lift after the lower half.

## Next

The owner answers the open points;
then step 0 and steps 2 to 4 start,
each on its own branch.
