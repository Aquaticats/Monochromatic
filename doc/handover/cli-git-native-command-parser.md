# cli-git native command parser

## Purpose and how to respond

This branch ports the Git-specific command token classification of the TypeScript wrapper
(`package/git-policy/cli/src/parser/`)
and the pure decision cores of six static rules
(`package/git-policy/cli/src/rule/`)
to Rust leaf modules under `package/git-policy/cli/src/native/`.
The modules spawn no process and read no file;
every repository fact a rule needs is a typed input or a typed "needs" result.
Only Git 2.56.0 is supported,
only shipped policies are ported,
and the TypeScript executable stays active until cutover.
The accepted scope is in
[`cli-git-rust-implementation.md`](../planning/cli-git-rust-implementation.md).

Inspect the divergences from the incumbent first:
each is a place where the shipped TypeScript reads an argument differently from Git 2.56.0,
and the port follows Git with a real-Git control.
Respond by merging or cherry-picking the branch into the native wrapper work,
and by answering the questions under the "Open questions" heading.

## Branch and commits

Branch `feat/cli-git-native-command-parser`,
started from `main` at `cd54f8b64`,
oldest first:

- `3d61c74ed` tokenize command arguments as Git 2.56.0 parse-options does
- `df0755d20` port commit region facts and the commit-only decision core
- `3b7bcaea3` port push and status facts with the atomic-push and status-hints cores
- `a43282470` port add, reset, clean and stash region facts
- `6099569b5` correct Git 2.56.0 source line citations in the option tables
- `994c75662` port branch, checkout and switch branch-creation facts
- `bcaea0145` port the require-root decision core with git config scope facts
- `2421befae` refuse sequencer path output with one or two trailing bytes
- `fe851e3b4` draft of this document
- `fa8a56d2a` drop branches planted mutations showed to be equivalent and pin equals splitting
- `583e77af9` complete this document
- `ee6e3c484` pick the git config form after leading wrapper flags
- the commit that records `ee6e3c484` in this document

Two commit messages state a wrong count;
each has a corrective commit comment on GitHub.
`3d61c74ed` says 2,100 argument lists where the differential compares 2,591.
`994c75662` says 68 checkout and switch argument lists where there are 67.

## Design

One table-driven tokenizer reproduces `parse_options_step`,
`parse_short_opt`,
`parse_long_opt`,
`get_arg`
and `check_typos`
of Git 2.56.0 `parse-options.c`
over unchanged `OsString` arguments.
Each command module declares the command's complete option table,
copied from `builtin/*.c` in source order,
and reads final option states,
because Git applies options in order and the last writer of a variable wins.
An option Git does not declare is a typed refusal (`OptionError`),
because Git itself refuses it with exit 129;
the incumbent let an undeclared option consume the next token.

Wrapper-only flags,
such as `--no-enforce-only`,
are recognized by exact bytes in option position only,
reported by token position,
and removed by position with `without_tokens`,
so an equal-looking option value or path survives.
Each command module puts its own escape hatch at flag index 0
and the caller's other wrapper flags after it.

Rules that need repository state are two-phase:
a `decide_*` function reads arguments and returns either a final decision
or a `Needs*` variant;
a `resolve_*` function takes the measured fact and finishes.
This replaces the injected asynchronous checkers of the TypeScript rules.

## Module map

### Parser modules

- `src/parser/argv.ts` becomes `command_options.rs` (types and main loop),
  `command_options_short.rs`,
  `command_options_long.rs`,
  `command_options_value.rs`
  and `command_options_query.rs` (final-state questions and wrapper-flag positions).
- `src/parser/commit.ts`,
  `commit-normalise.ts`
  and `commit-flag-aliases.ts`
  become `command_commit.rs` and `command_commit_table.rs`;
  alias lists are replaced by the table.
- `src/parser/push.ts` becomes `command_push.rs`.
- `src/parser/status.ts` becomes `command_status.rs`,
  including the global `-c` and `--config-env` reading of `advice.statusHints`.
- `src/parser/add.ts` becomes `command_add.rs`.
- `src/parser/reset.ts` becomes `command_reset.rs`.
- `src/parser/clean.ts`,
  `clean-options.ts`
  and `clean-option-order.ts`
  become `command_clean.rs`.
- `src/parser/stash.ts` becomes `command_stash.rs` and `command_stash_table.rs`.
- The eight `src/parser/branch-create*.ts` files become
  `command_branch_create.rs` (entry),
  `command_branch_mode.rs` (the `git branch` action count),
  `command_branch_target.rs` (explicit creation and the remote-guess candidate),
  `command_branch_table.rs`
  and `command_checkout_table.rs`.
  `branch-create-strip.ts` is replaced by `without_tokens` over the reported positions.
- The escape-hatch spellings shared by several modules were in `command_escape_hatch.rs`;
  at integration they moved into `main`'s `escape_hatch.rs` (see `Integration notes`).

### Rule core modules

- `src/rule/commit-only.ts` becomes `rule_commit_only.rs` and `rule_commit_only_message.rs`.
- `src/rule/commit-index-check.ts` becomes `rule_commit_index.rs`.
- `src/rule/commit-sequencer-check.ts` becomes `rule_commit_sequencer.rs`.
- `src/rule/atomic-push.ts` becomes `rule_atomic_push.rs`,
  with the shared insertion helper in `rule_argument_rewrite.rs`.
- `src/rule/status-hints-off.ts` becomes `rule_status_hints.rs`.
- `src/rule/require-root.ts` becomes `rule_require_root.rs`,
  with `command_config.rs` and `command_config_table.rs` for the `git config` exemption.

### Test support

`command_test_support.rs` (disposable real-Git fixtures),
`command_test_parseopt.rs` (the `git rev-parse --parseopt` differential)
and `command_test_completion.rs` (the `--git-completion-helper-all` table oracle)
compile only under `cfg(test)`.

## Public API

### Tokenizer functions and types

```rust
// package/git-policy/cli/src/native/command_options.rs
pub fn parse_options(
    arguments: &[OsString],
    table: &[OptionSpec],
    mode: ParseMode,
    wrapper_flags: &[&[u8]],
) -> Result<ParsedOptions, OptionError>
```

- Types `OptionSpec`,
  `Arity`,
  `ParseMode`,
  `ParsedOptions`,
  `Occurrence`,
  `OptionValue`,
  `WrapperOccurrence`,
  `Boundary`,
  `OptionError`
  and `OptionErrorKind`;
  the builder `row` and the constants `UNREAD` and `DEFAULT_MODE`.
- `last_occurrence`,
  `is_stated`,
  `is_enabled`,
  `positional_tokens`,
  `value_bytes`,
  `has_wrapper_flag`,
  `split_wrapper_flags`,
  `without_tokens`
  and the type `WrapperFlags { escape, other }`.

### Command fact parsers

Every parser has the shape
`parse_<command>_region(region: &[OsString], wrapper_flags: &[&[u8]]) -> Result<<Command>Region, OptionError>`,
where `region` is the tokens after the subcommand word:

- `parse_commit_region` returns `CommitRegion`
  (`all`, `only: Option<bool>`, `include`, `interactive`, `patch`, `dry_run`, `amend`,
  `allow_empty`, `fixup: Option<FixupKind>`, `pathspec_from_file`, `pathspec_file_nul`,
  `pathspecs`, `wrapper`).
- `parse_push_region` returns `PushRegion` (`atomic_stated`, `dry_run`, `wrapper`).
- `parse_status_region` returns `StatusRegion` (`machine_readable`, `wrapper`);
  `has_status_hints_override(global_prefix)` reads the global options.
- `parse_add_region` returns `AddRegion` (`bulk_matches: Vec<BulkMatch>`, `resolved`, `wrapper`).
- `parse_reset_region` returns `ResetRegion` (`mode: Option<ResetMode>`, `wrapper`);
  `reset_changes_worktree` reads it.
- `parse_clean_region` returns `CleanRegion` (`dry_run`, `interactive`, `wrapper`);
  `clean_changes_worktree` reads it.
- `parse_stash_region` returns `StashRegion` (`subcommand: StashSubcommand`, `wrapper`).
- `parse_branch_creation_region(command: BranchCreationCommand, region, wrapper_flags)`
  returns `BranchCreationRegion`
  (`creates_branch`, `implicit_creation_target: Option<usize>`, `wrapper`);
  `branch_creation_command(word)` selects `Branch`, `Checkout` or `Switch`.
- `parse_config_region` returns `ConfigRegion`
  (`form: ConfigForm`, `global`, `system`, `lists`, `wrapper`);
  `config_table(form)` assembles a form's table.

### Rule core functions

- `decide_commit_only(arguments, wrapper_flags) -> Result<CommitOnlyDecision, OptionError>`,
  then `resolve_sequencer_state(SequencerState)`
  or `resolve_index_state(arguments, &PendingInjection, IndexVsHead)`;
  `has_commit_only_escape_hatch` answers whether the hatch was written.
- `index_query_arguments(global_prefix)` and `index_state_from_exit(Option<i32>) -> IndexVsHead`.
- `sequencer_query_arguments(global_prefix)`,
  `sequencer_head_paths(stdout) -> Result<Vec<Vec<u8>>, SequencerOutputError>`,
  `sequencer_state(SequencerFacts)` and `sequencer_state_when_query_fails()`.
- `atomic_push(arguments, wrapper_flags) -> Result<ArgumentRewrite, OptionError>`.
- `status_hints_off(arguments) -> ArgumentRewrite`
  and `has_explicit_status_hints_override(arguments)`.
- `decide_require_root(arguments, wrapper_flags) -> RequireRootDecision`
  (`Exempt(RequireRootExemption)` or `NeedsRepositoryRoot`),
  then `resolve_require_root(&RequireRootFacts) -> RequireRootVerdict`
  (`Pass` or `NotAtRoot(RequireRootViolation)`, code `NOT_AT_ROOT_CODE`).

## Ported tests

Every case of the incumbent unit tests is ported,
either with the same expectation
or as a divergence test whose expectation a real-Git control confirms:

- `src/parser/argv.unit.test.ts` (18 cases) to `command_options_tests.rs`,
  except "keeps an ordinary undeclared joined git option working",
  which needs a real table and is in `command_commit_tests.rs`:
  13 keep their expectation;
  5 cases where an undeclared option consumed the next token now expect Git's refusal.
- `src/parser/commit.unit.test.ts` (17 cases) to `command_commit_tests.rs`.
- `src/rule/commit-only.unit.test.ts` (24 cases, several parameterized)
  to `rule_commit_only_tests.rs`,
  with readings Git differs on in `rule_commit_only_divergence_tests.rs`.
- `src/rule/atomic-push.unit.test.ts` (5 cases) to `rule_atomic_push_tests.rs`.
- `src/rule/status-hints-off.unit.test.ts` (10 cases) to `rule_status_hints_tests.rs`.
- `src/rule/require-root.unit.test.ts` (9 cases) to `rule_require_root_tests.rs`:
  each repository shape on disk becomes the measured root it produces.
- `src/policy-engine/branch-worktree-check.unit.test.ts` (8 cases)
  and the escape-hatch case of `src/bin.unit.test.ts`
  to `command_branch_create_tests.rs`.
- `add.ts`,
  `push.ts`,
  `reset.ts`,
  `clean.ts`,
  `stash.ts`
  and `status.ts` have no unit test of their own;
  the tokens each recognized are kept as cases.

Rust tests per file in the owned modules (177 in all):
`command_options_tests.rs` 14,
`command_options_short_tests.rs` 6,
`command_options_long_tests.rs` 8,
`command_options_value_tests.rs` 6,
`command_options_git_tests.rs` 4,
`command_commit_tests.rs` 12,
`command_commit_git_tests.rs` 7,
`rule_commit_only_tests.rs` 11,
`rule_commit_only_divergence_tests.rs` 10,
`rule_commit_only_git_tests.rs` 3,
`rule_commit_index_tests.rs` 3,
`rule_commit_sequencer_tests.rs` 5,
`rule_argument_rewrite_tests.rs` 1,
`command_push_tests.rs` 5,
`rule_atomic_push_tests.rs` 5,
`command_status_tests.rs` 7,
`rule_status_hints_tests.rs` 5,
`command_add_tests.rs` 7,
`command_reset_tests.rs` 6,
`command_clean_tests.rs` 6,
`command_stash_tests.rs` 5,
`command_branch_create_tests.rs` 11,
`command_branch_create_target_tests.rs` 8,
`command_branch_create_git_tests.rs` 3,
`command_branch_create_guess_tests.rs` 3,
`command_config_tests.rs` 9,
`rule_require_root_tests.rs` 7.

## Divergences from the incumbent

Line numbers refer to the Git 2.56.0 source at commit
`a018953688f1b10bddf91bff8747068f5f4746a4`.

### Tokenizer readings

- An undeclared option is refused,
  never allowed to consume a following token
  (`parse-options.c:1164-1165`, `1224-1233`).
- Letters that spell a long option after one dash (`-all`, `-amend`) are refused
  (`check_typos`, `parse-options.c:622-640`).
- Under `PARSE_OPT_KEEP_UNKNOWN_OPT` abbreviations are disabled (`parse-options.c:502-503`).
- `--end-of-options` ends option parsing like `--`;
  a lone `-` is positional.

### Commit and commit-only

- Every letter of a cluster is read, so `-qa` and `-qam msg` contain `-a`;
  after an optional-value letter the rest is its value, so `-ua` is not `-a`
  (`parse_short_opt`, `parse-options.c:426-461`).
- A separated token after `--untracked-files` or `-S` is a pathspec,
  because those options take an attached value only (`commit.c:1730-1739`, `1766-1775`).
- The last of `--all`/`--no-all`,
  `--only`/`--no-only`,
  `--dry-run`/`--no-dry-run`
  and the status formats wins (`commit.c:1752-1761`).
- `--fixup=reword:<commit>` turns `--only` on inside Git and refuses `-o` with paths,
  so the rule forwards it unchanged;
  `--fixup=amend:<commit>` may be pathless;
  an unknown suboption is left to Git (`commit.c:1296-1307`, `1378-1411`).
- `git commit --porcelain` takes no value, unlike `git status` (`commit.c:1757-1758`).
- The escape hatch is removed by position only;
  the incumbent removed every equal token, including a message.

### Push and atomic-push

- `--at` and `--no-at` are `--atomic` and its negation;
  either is the caller's choice and suppresses injection.
- `-nf` is a dry run;
  in `-o -n`, `--repo -n` and `--exec -n` the `-n` is a value,
  so the push is real (`push.c:707-743`).
- `--atomic` as an option value or after `--` is not a choice.

### Status and status-hints

- The last of `--short`, `--porcelain` and `--long` decides the format;
  `--sh` is ambiguous in `git status` (`--short`, `--show-stash`).
- `--config-env=advice.statusHints=VAR` is an explicit override like `-c`.
  The key ends at the first `=` for `-c` and at the last `=` for `--config-env`
  (`config.c:495`, `511`).

### Add

- `-vA`, `-Av`, `--al` and `--no-ignore-removal` are bulk staging;
  `-A --no-all` and `-A --ignore-removal` are not,
  because `--ignore-removal` writes the same variable (`add.c:252-259`).
- `--resolved` is reported as its own fact.

### Reset, clean and stash

- The five reset modes write one variable, so the last decides (`reset.c:350-382`).
- `git clean -i -n` deletes nothing;
  the incumbent treated every interactive clean as deleting
  (`clean.c:191`, `223`, `267`, `1050-1078`).
- Stash dispatch follows `cmd_stash` (`stash.c:2465-2479`):
  `create` takes every token as message text,
  and `git stash -push` is refused as a single-dash spelling of a long word
  (`parse-options.c:622-640`).

### Sequencer check

- The three head-file paths are read from one `git rev-parse --git-path` output
  whose directory may contain a newline,
  so the reader checks every byte instead of splitting lines.

### Branch, checkout and switch

- Display options (`-v`, `--color`, `--format`, `--sort`, `-i`, `--abbrev`, `--column`)
  do not select listing, so `git branch -v topic` creates `topic`
  (`branch.c:1083-1097`).
- `-a` or `-r` with a name,
  the `--set-upstream` tracking mode,
  `--dry-run` without `--delete-merged`,
  and `--recurse-submodules` with an action
  make Git refuse, so nothing is created (`branch.c:1099-1107`, `1294-1299`).
- `git checkout --no-track origin/topic` and `git switch --no-track origin/topic` create `topic`
  (`checkout.c:1986-1997`).
- `git checkout topic --` and `git switch -- topic` still guess a remote branch;
  `git checkout -- topic` restores a path
  (`checkout.c:1473-1477`, `1487-1492`, `1519-1522`).
- A pathspec file does not stop the guess (`checkout.c:2058-2101`);
  `--overlay` and `--no-overlay` do (`checkout.c:1686-1714`).
- Final states decide:
  `--no-guess --guess`,
  `--detach --no-detach`
  and `--orphan x --no-orphan` guess again.

### Require-root and git config

- `git config` is exempt only when `--global`, `--system` or listing is in option position;
  `git config user.name --global` stores the text `--global` in the repository
  (`config.c:1100-1101`, `1155-1156`, `1403-1405`).
- `git config list`, the 2.56.0 spelling of `--list`, is exempt (`config.c:1633-1660`).
- When Git runs no subcommand (`git --version status`, an unknown global option),
  nothing needs a root.

## Not ported

- Process and filesystem work,
  by the delegation's leaf-module boundary:
  running the index and sequencer queries,
  testing head-file presence,
  the remote-branch probe of `branch-worktree-remote-guess.ts`,
  `findRoot` in `require-root.ts`,
  and resolving the real Git binary.
  The cores expose the query arguments and parse the outputs.
- The effective directory of `parse-global-options.ts`.
  `global_arguments.rs` deliberately does not reproduce `-C` chaining,
  so the engine measures it.
- Policy-engine adapters,
  finding serialization,
  logging,
  and `policy-engine/controls.ts` (wrapper control parsing),
  which are outside the rule cores.
- Branch-worktree rejection messages (`branch-worktree-messages.ts`);
  the parser reports facts only.

## Verification

### Gate

`GIT_POLICY_NATIVE_IMAGE_TAG=command-parser mise run //package/git-policy/cli:native:test:container`
at `ee6e3c484`:
186 tests passed,
0 failed;
Clippy passed (`{"tests":true,"clippy":true}`).
Image `f157e220fd2bd272df9f09f5a12f19e3befe1cdbb2c089fe9c5ae2fd42db7edb`,
built from the audited Git 2.56.0 base
`6ec87f6d290a2f59bda5b3ffd4197058fe0749d02b4978c877e8edf6dc38802a`,
run with `--network=none`, `--memory=2g`, `--cpus=2` and `--pids-limit=128`.
Of the 186 tests, 177 are in this branch's modules;
the other 9 are the existing `global_arguments_tests.rs` (5) and `config_loading_tests.rs` (4).
Cargo prints one manifest warning on every run,
`unused dependency monochromatic-jsonc-edit`,
which predates this branch and comes from `Cargo.toml`, which this branch does not change.

### Oracles against the real binary

- Tokenizer differential against `git rev-parse --parseopt --stuck-long`:
  2,591 argument lists
  (2,156 in the default mode,
  210 each with `PARSE_OPT_KEEP_DASHDASH` and `PARSE_OPT_STOP_AT_NON_OPTION`,
  and 15 curated sequences).
- Table oracle against `git <command> --git-completion-helper-all`:
  29 tables
  (commit, status, push, add, reset, clean,
  12 stash subcommands,
  branch, checkout, switch,
  and 8 `git config` forms).
- `git branch` differential:
  558 argument lists run against a fixture repository,
  comparing "a branch appeared" with `creates_branch`;
  188 created a branch and 22 were refused.
- `git checkout` and `git switch`:
  67 argument lists compared the same way against a fixture
  whose only remote has `topic`.
- Behavior controls in disposable repositories for every divergence that changes an outcome,
  such as `-qam` committing tracked changes,
  `git clean -i -n` deleting nothing,
  and `git config user.name --global` storing a value.

### Planted mutations

The scratch harness exports one commit with `git archive`,
applies one textual change to one guard,
and runs the package's container runner on the copy;
a mutation counts as killed only when a test fails,
never when only compilation or Clippy fails.

- Tokenizer, at `3d61c74ed`:
  31 mutations;
  30 killed by tests.
  The survivor removed the lone `-h` special case,
  which is equivalent for tables without an `h` letter;
  `a_lone_h_is_help_even_when_declared` in `command_options_short_tests.rs` now kills it.
- Ported families, at `bcaea0145`:
  83 mutations over the commit, commit-only, sequencer, index, push, atomic-push,
  status, status-hints, rewrite, add, reset, clean, stash, branch, checkout, switch,
  config and require-root guards;
  78 killed by tests, 5 survived.
- The 5 survivors:
  - Dropping the length-remainder check of `sequencer_head_paths` survived
    because no case had one or two trailing bytes;
    `2421befae` adds both.
  - Swapping first and last `=` in `equals_index` survived
    because no case had a second `=`;
    `fa8a56d2a` adds `-c advice.statusHints=a=b` and `--config-env=advice.statusHints=x=VAR`
    with real-Git controls (`config.c:495`, `511`).
  - Weakening the `-no-` typo check and the `-h` check before an assumed `git stash push`
    survived because the assumed-push pass refuses those tokens with the same kind and index;
    `fa8a56d2a` removes both checks as equivalent and adds the three-letter typo boundary.
  - Dropping the default-listing term of the `git branch` action count survived
    because with no name nothing is created either way;
    `fa8a56d2a` removes the term as equivalent.
- Follow-up, at `fa8a56d2a`:
  the two remaining survivor guards and three new mutations of the stash typo boundary;
  all 5 killed by tests.
- Leading wrapper flags of `git config`, at `ee6e3c484`:
  disabling the skip, and picking the form from the first token again;
  both killed by tests.

## Integration notes

- Edit outside the owned files, in `global_arguments.rs`:

  ```diff
  --- a/package/git-policy/cli/src/native/global_arguments.rs
  +++ b/package/git-policy/cli/src/native/global_arguments.rs
  @@ -37 +37 @@
  -const VALUE_OPTIONS: &[&[u8]] = &[
  +pub(crate) const VALUE_OPTIONS: &[&[u8]] = &[
  ```

  `command_status.rs` reads it to skip the values of global options.
  `main` still has the private form, so a merge must keep the `pub(crate)` line.
- Resolved at integration (main session, 2026-10-05):
  `command_escape_hatch.rs` repeated `WORKTREE_ENFORCEMENT_ESCAPE_HATCH`;
  the file is deleted,
  `BRANCH_WORKTREE_ESCAPE_HATCH` moved into `escape_hatch.rs`,
  and the command modules import both from there.
  `main`'s `strip_escape_hatch` removes the hatch by spelling with a value-option list;
  the command modules here report hatch positions,
  and `without_tokens` removes exactly those.
- Resolved at integration:
  `command_test_support.rs` now re-exports `REAL_GIT` and `remove` from `test_support.rs`,
  which were identical.
  Its `fixture`, `git_output`, `git` and `repository` stay separate on purpose:
  the `native-command-` directory prefix keeps command fixtures apart from wrapper fixtures in one test process,
  and its runners also set `GIT_EDITOR=true` and close standard input.
- A parser `Err(OptionError)` means Git 2.56.0 itself refuses the command.
  The recommended engine handling is to forward the command unchanged,
  so Git prints its own error and exits 129.
- Pass every wrapper-only spelling the engine recognizes
  (the `--no-enforce-<policy>` flags and `--cli-git-keep-going`)
  as `wrapper_flags` to every parser;
  otherwise Git's tables refuse them as unknown options.
  `parse_stash_region` and `parse_config_region` skip leading wrapper flags
  before choosing a subcommand or form,
  because the wrapper removes them before Git runs.
- Remove wrapper controls from the global prefix,
  the tokens before the subcommand,
  before calling any decision built on `global_layout`.
  `global_layout` reports an unknown global option as `InvalidOption`,
  and `decide_commit_only`, `atomic_push`, `status_hints_off` and `decide_require_root`
  then leave the command alone,
  so `git --cli-git-keep-going status` from a subdirectory would pass require-root.
- `resolve_require_root` needs the effective directory and the repository root.
  `main`'s `resolve_worktree_identity` already asks Git with the forwarded global prefix,
  which is one candidate source for the root.
- The branch-creation facts read option names and argument counts only.
  Where Git later refuses for an option value, configuration or repository state
  (`--track=bogus`, `-b a -B b`, `--orphan x --track y`),
  the fact still says "creates";
  the policy then rejects a command Git would have refused anyway.

## Open questions

- Should an `OptionError` forward the command unchanged,
  or should the engine report a finding first?
  Forwarding matches what Git does;
  a finding would add wrapper output to a command that fails anyway.
- Which root does require-root compare with:
  the incumbent's nearest valid Git marker,
  or the worktree top level Git reports?
  They differ for `--git-dir`, `--work-tree` and `GIT_DIR`.
- Should the branch-worktree policy keep rejecting commands Git refuses for a value
  (the over-report in "Integration notes"),
  or should it ask Git to validate first?
