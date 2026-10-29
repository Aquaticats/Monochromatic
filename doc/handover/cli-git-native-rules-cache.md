# Native worktree copy and the rules file

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

## Purpose

The forbidden-strings scanner reads its rules from `.cache/forbidden-strings.rules.txt` (the configured `rulesFile`,
relative to the worktree).
`mise run file-enforcer` generates that file, and `.gitignore` ignores it (`.gitignore:68`),
so a fresh linked worktree has none, and the native wrapper fails closed there (`policy-incomplete`, naming the path).
The user decision of 2026-10-12 is to bring the file along when the wrapper creates a worktree, from the main checkout
as well as from a linked worktree, and to copy nothing when the source has no such file.
The linked-worktree ignored-state copy (`src/native/worktree_copy*.rs`) is the mechanism, and this lane wires it into
the native executable.

## What was found

The native executable never ran the copy.
The library lifecycle (`worktree_copy::run_with_worktree_copy`) had no caller, `git worktree add` from a linked
worktree was refused (`Unported::WorktreeCopy`), and from the main checkout it was forwarded with no copy.
The incumbent TypeScript wrapper bypasses a main-checkout source too
(`observeWorktreeRepository` in `src/worktree-copy/git-observer.ts` returns not-applicable for `main-worktree`),
so copying from the main checkout is behavior the incumbent lacks.
Reproduction before the change (host Git 2.55.0, throwaway repository, script `rules-cache-repro.ts` under
`${HOME}/temp/agent`):
add from the main checkout exited 0 with no `.cache`; add from a linked worktree exited 2 with the refusal;
`git cli-git check --policy security/forbidden-strings --all` in the new worktree exited 2 with `policy-incomplete`.

## What changed

Selection (library), in `package/git-policy/cli/src/native/`:

- `worktree_copy_selection.rs`: `CopySelection` (`Ignored` or `Paths`), `read_named_ignored_roots`, `select_roots`.
  A named path is selected only when it passes the path-safety check and Git itself lists it exactly under
  `ls-files --others --ignored --exclude-standard -z -- :(top,literal)<path>`.
  Absent, tracked, non-ignored and directory names select nothing, so nothing is invented.
  The main checkout is never enumerated.
- `worktree_copy_registry.rs`: `WorktreeCopyObservation::selection` and `observe_copy_source`, which observes a main
  checkout only when the caller names paths.
- `worktree_copy_stage.rs`, `worktree_copy_transaction.rs`: take the selection.
- `worktree_copy.rs`: `Invocation::main_checkout_paths`; empty keeps the main checkout a bypass.

Wiring (executable):

- `worktree_copy_forward.rs` (new): `copy_applies` decides by repository shape (linked and bare always, the main
  checkout only for a creating command with a configured rules file, outside any repository never);
  `run_forwarded_git` runs real Git as a child, through `run_with_worktree_copy` when a copy applies, with the
  settlement lease in the child's `CLI_GIT_WORKTREE_COPY_LEASE`.
  A copy failure after a successful Git is status 2 with the copy's message, after a failed Git it keeps Git's status,
  and a lifecycle failure is status 2.
- `entry.rs` (`forward_action`): resolves the command (aliases included), asks the repository location (cached from the
  policy pass), loads the configuration only for an index writer or a main-checkout creation, and plans
  `Action::ForwardSettled` (non-writers) or a `ForwardCoordinated` that carries the copy (index writers, so the landing
  lock stays outermost as in the incumbent).
  The main checkout gets `config.policies.forbidden_strings.rules_file` as `main_checkout_paths`.
- `action.rs`, `index_writer_forward.rs`: the new action variant and `CoordinatedForward::copy`.
- Removed because the copy is no longer unported: `refusal_frontier.rs` (the worktree, alias, lease and journal
  refusals), `pending_state.rs`, and the `Unported` variants `WorktreeCopy`, `AliasResolution`,
  `WorktreeCopyRecovery` and `InheritedLease`.
  Recovery of an interrupted copy now happens in the forwarding, as in the incumbent, not as a refusal.

## Tests

- Binary tests, `src/native/binary_worktree_copy_tests.rs`, built `cli-git-native` through `git worktree add` in
  throwaway repositories, each followed by `git cli-git check --policy security/forbidden-strings --all`:
  from the main checkout (rules file copied, siblings and other ignored trees not, scanner finds its rules;
  with `--no-worktree-copy` the scanner fails closed); from a linked worktree (everything ignored is copied);
  a source without the file (nothing copied, scanner fails closed naming the path); an unconfigured main checkout
  (plain forward); an alias; Git failing (status kept); an interrupted copy recovered by the next command from a
  linked worktree and by the next creation from the main checkout; a hook that creates a worktree under the
  inherited lease (no deadlock, outer copy into both worktrees); a failed copy (status 2, message, no journal, lock
  released); `worktree move` (no summary).
- Unit tests: `worktree_copy_forward_tests.rs`, `entry_tests.rs` (settled-forward plans),
  `index_writer_forward_tests.rs`, and the selection, registry and lifecycle tests of the library change.
- Each behavior was shown to fail with its guard removed, then restored (guard drivers
  `${HOME}/temp/agent/rules-cache-guards.ts` and `rules-cache-guards2.ts`): selection dispatch, exact-name comparison,
  unsafe-path check, `without_prefix`, restricted observation, empty-paths bypass, opt-out, short-circuit, main-checkout
  creation condition, lease inheritance, lease to the child, copy-failure message, rules path from the configuration,
  index writer carrying the copy, coordination bypass keeping the copy, identity re-query prefix.
  The `:(top,literal)` flag is an equivalent mutant: the exact-name comparison already filters Git's listing.

## Choices open to veto

- The lifecycle runs for every forwarded command from a linked worktree or bare repository that needs a repository
  (as the incumbent does), so those commands now run Git as a child instead of replacing the process, and cost one
  directory read plus a pending-journal check.
  Commands that need no repository (`version`, `help`, `init`, `clone`, user-level `config`), the opt-out and Git's
  help and version forms skip it, which keeps the "at most the location query" guarantee and means those commands do
  not recover an interrupted copy.
- The main checkout takes the lifecycle only for a creating command whose configuration names `rulesFile`;
  other main-checkout commands keep the exact process replacement and never recover an interrupted copy.
  An interrupted main-checkout copy is recovered by the next worktree-creating command or by a linked or bare command.
- Only the configured `rulesFile` is copied; the `FORBIDDEN_STRINGS_RULES` fallback and the default
  `forbidden-strings.local.txt` are not, and the file is copied whatever the severity of the forbidden-strings policy.
- A copy that selects nothing still prints "copied 0 ignored filesystem entries ..." and takes the lock.
- Aliases are resolved for the lifecycle (`creates_or_moves_worktrees`), so an alias for `worktree add` is copied too;
  the policy pass still sees the command word as typed.
- Child runs install no signal handlers, like the incumbent's spawn and the existing coordinated forward.
- The fuzz planted control "a real commit is forwarded" targeted `refusal_frontier.rs` text that had not existed since
  the commit transaction landed; it was removed with the module (in the commit that also touched
  `doc/handover/cli-git-native-integration.md`, whose message names only the document).

## Evidence

Commits on `feat/cli-git-native-rules-cache` (based on `main` at `e1890fb471`, then merged with `main` at
`f7a4d08160`, which carries the integration branch; the merge had no conflicts):

- `254eeb701e`, `1c3248743c`, `a1e2c74fd8`, `5e04e3815e`: selection of named paths from a main-checkout source
  (library level), with its tests; `48a96a4942`: the first version of this document.
- `e0298731a9`: merge of `main` at `f7a4d08160`.
- `a531da8a4d`: the wiring (new forwarding module, `forward_action`, removal of the refusals).
- `94708ee206`, `666e1bed76`: binary and unit tests for a failed copy, the inherited lease and the coordination path.
- `c5743ee977`, `0b9b0b5c52`: documents (`cli-git-native-config.md` section "Rules file in a linked worktree",
  an integration note, and the removal of the stale fuzz planted control).
- `00208d68b9`: commands that need no repository skip the copy forwarding.
- `e7216c94a0`: a selection test; `ebc30c1b57`: the last code change, a redundant guard removed.

Gates at head `ebc30c1b57` (the last code or test change; later commits change only this document):

- Container gate, `GIT_POLICY_NATIVE_IMAGE_TAG=rules-cache`, Git 2.56.0, image
  `localhost/git-policy-native-test:rules-cache`, id `8b522aadf70d6b185442e219076adb2f68326db5f01217db61733d23c37d7285`:
  library 1653 passed, 6 ignored; binary 115 passed, 1 ignored; interop 1 passed; the gate's Clippy is clean.
  Of these, the new tests are 11 binary tests in `binary_worktree_copy_tests.rs`, and in the library
  4 in `worktree_copy_forward_tests.rs`, 5 in `entry_tests.rs`, 1 in `index_writer_forward_tests.rs`,
  and the selection, registry and lifecycle tests of the earlier commits.
- `mise run //package/git-policy/cli:native:clippy`: zero findings.
- End-to-end, `GIT_POLICY_NATIVE_IMAGE_TAG=rules-cache`, seed 1, `--git 2.55.0`, at `00208d68b9`
  (the later commits change a test, a redundant guard and documents): 38 scenario runs, 10 baseline pass,
  26 concurrency pass, 2 concurrency skip (the two `replay-unavailable-*` scenarios, which prove degradation only
  before Git 2.40.0); finished in 252 s.
- Host Git 2.55.0: `native:test:binary:host -- worktree_copy` passes (11 of 11).
  Five other binary tests fail on the host only for host reasons (they assert Git 2.56.0's version text, or need the
  gate image's linter).
- `native:clippy:windows` is not run: the host has no `x86_64-pc-windows-gnu` target (`E0463`).

## Mutation

Container scoped runs through the slot limiter, `--timeout 600`, `NATIVE_MUTATION_SKIP_PLANTED=1` after the first
campaign (the planted-control phase ran once, all 66 controls noticed, at `5e04e3815e`).

- `worktree_copy_selection.rs` (earlier campaign, 56 mutants): the three survivors that deleted `without_prefix` were
  killed afterwards by two tests, each shown to fail with the field deleted.
- `worktree_copy_forward.rs` and `index_writer_forward.rs`, with the selection file again (68 mutants):
  no survivor in the new files; 56 caught, 8 unviable, 4 missed, all in `worktree_copy_selection.rs`.
  That run used the stale scratch script `bin/mutate-native-container-noplanted.mjs`, whose skip arguments differ from
  `mutate-native-container.mjs`; its survivors were rechecked with the regular script
  (the by-function run of `worktree_copy_selection.rs`, 37 mutants).
- `worktree_copy_selection.rs` by function, regular script (37 mutants): 34 caught, 3 missed.
  The misses at lines 68 and 69 (`||` to `&&` inside the unsafe-path test) are equivalent:
  an empty path still has an empty component, and a path that starts with `/` is still absolute on Unix,
  so no input changes the outcome.
  The miss at line 189 (`is_excluded` always false) was killed afterwards by
  `special_entries_under_an_excluded_root_are_not_selected`, shown to fail with the function replaced.
- `entry.rs`, `forward_action`, `configuration_needed` and `finish_forwarded` (16 mutants): 14 caught, 1 unviable,
  1 missed: the `if creates_or_moves` guard on the main-checkout paths, equivalent because the copy decision already
  requires a creating command; the guard was removed in `ebc30c1b57`.
- Not rerun, and not changed in this work: the four inherited survivors in `worktree_copy_registry.rs` (lines 350 and
  407, two mutants) and `worktree_copy.rs` (line 257), untriaged, as listed in
  `doc/handover/cli-git-native-worktree-copy.md`.
- Mutation after the last two commits is not rerun; the tests that kill the changed mutants pass in the gate.

## Trailing gates

- Windows checks (the target is missing on this host) and fuzz smoke are not run.
- Rerun commands:

```sh
# doc/handover/cli-git-native-rules-cache.md: container gate and Clippy
GIT_POLICY_NATIVE_IMAGE_TAG=rules-cache node ~/temp/agent/gate-slot.ts mise run //package/git-policy/cli:native:test:container
node ~/temp/agent/gate-slot.ts mise run //package/git-policy/cli:native:clippy
# doc/handover/cli-git-native-rules-cache.md: binary tests on the host
mise run //package/git-policy/cli:native:test:binary:host -- worktree_copy
# doc/handover/cli-git-native-rules-cache.md: scoped mutation without the planted-control phase
NATIVE_MUTATION_SKIP_PLANTED=1 GIT_POLICY_NATIVE_IMAGE_TAG=rules-cache node ~/temp/agent/gate-slot.ts mise run //package/git-policy/cli:native:mutation:scoped -- --file src/native/worktree_copy_forward.rs --timeout 600
# doc/handover/cli-git-native-rules-cache.md: end-to-end suite, as in doc/handover/cli-git-native-integration.md
GIT_POLICY_NATIVE_IMAGE_TAG=rules-cache node ~/temp/agent/gate-slot.ts mise run //package/git-policy/cli:test:e2e:concurrent:native --seed 1 --git 2.55.0
```
