//! What:
//!  Internal native cli-git implementation,
//!  not yet the production executable.
//! Why:
//!  The Rust rewrite preserves Git's own argument,
//!  repository and transaction semantics.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Internal modules used by the forthcoming native entry point and consumer verification.
//! ```

/// The decision one invocation ends in:
///  become Git,
///  or print and exit.
pub mod action;

/// Variables added to every real-Git child:
///  lock PID injection and the forward-target marker.
pub mod child_environment;

/// The one failure type for rejected configuration content or files.
pub mod config_error;

/// Repository-root discovery and bounded reading of `cli-git.config.jsonc`.
pub mod config_file;

/// Lazy configuration loading for known inspection versus mutation/ambiguous commands.
pub mod config_loading;

/// Whole-document JSONC validation into typed settings.
pub mod config_parse;

/// Validation of the `policies` section and per-policy options.
mod config_policies;

/// The `rulesFile` option:
///  a repository-relative name that stays inside the repository.
pub mod config_rules_file;

/// Typed settings and their defaults.
pub mod config_schema;

/// One `git config --list` read, parsed, and the lookups the commit path makes against it.
pub mod config_snapshot;

/// Key-naming typed readers over parsed JSONC values.
mod config_values;

/// JSONL event rendering for wrapper-made diagnostics.
pub mod diagnostics;

/// Worktree target classification for worktree-enforcing policies.
pub mod effective_target;

/// The per-invocation decision the thin executable performs.
pub mod entry;

/// Spellings of the wrapper-only escape hatches shared by several modules.
pub mod escape_hatch;

/// Starting real Git with unchanged arguments,
///  streams and exit status.
pub mod forwarding;

/// Captured read-only real-Git queries over raw bytes.
pub mod git_metadata;

/// Git 2.56.0 global-argument boundaries over unchanged operating-system strings.
pub mod global_arguments;

/// Configuration of the repository one invocation selects.
pub mod invocation_config;

/// The `git cli-git` decision:
///  help,
///  retired commands,
///  and direct `check`/`fix`.
pub mod management;

/// The `git cli-git` argument grammar.
pub mod management_arguments;

/// The fixed registry of shipped policies.
pub mod policy_registry;

/// PATH resolution of the real Git executable.
pub mod real_git;

/// Wrapper self-exclusion for one PATH candidate.
pub mod real_git_candidate;

/// Disposable fixtures shared by unit tests;
///  never part of the release executable.
#[cfg(test)]
mod test_support;

/// Repository and worktree identity as reported by real Git.
pub mod worktree_identity;

/// Git 2.56.0 `parse-options` tokenization of one command's argument region.
pub mod command_options;

/// Long-option matching:
///  exact names,
///  unique abbreviations and `no-` negation.
mod command_options_long;

/// Final-state and position questions over a tokenized region.
pub mod command_options_query;

/// Short-option clusters and their attached or following values.
mod command_options_short;

/// The shared decision of which bytes are an option's value.
mod command_options_value;

/// Real-Git fixtures and table oracles for command tests;
///  never in the release build.
#[cfg(test)]
mod command_test_support;

/// Differential harness over `git rev-parse --parseopt`;
///  never in the release build.
#[cfg(test)]
mod command_test_parseopt;

/// Table oracle over `--git-completion-helper-all`;
///  never in the release build.
#[cfg(test)]
mod command_test_completion;

/// Facts about the arguments after `git commit`.
pub mod command_commit;

/// The complete Git 2.56.0 option table of `git commit`.
pub mod command_commit_table;

/// The shared result and builder of transforms that add tokens to a command line.
pub mod rule_argument_rewrite;

/// Pure core of the index-against-`HEAD` check.
pub mod rule_commit_index;

/// The query and reading of "does this worktree hold uncommitted state?".
pub mod rule_worktree_dirty;

/// Pure decision of the commit-only transform.
pub mod rule_commit_only;

/// User-facing diagnostics of the commit-only transform.
pub mod rule_commit_only_message;

/// Pure core of the merge,
///  cherry-pick and revert conclusion check.
pub mod rule_commit_sequencer;

/// The Git 2.56.0 option table of `git push` and its facts.
pub mod command_push;

/// The Git 2.56.0 option table of `git status`,
///  its facts and the advice-key reading.
pub mod command_status;

/// Pure decision of the atomic-push transform.
pub mod rule_atomic_push;

/// Pure decision of the status-hints transform.
pub mod rule_status_hints;

/// The Git 2.56.0 option table of `git add` and its bulk-staging facts.
pub mod command_add;

/// The Git 2.56.0 option table of `git clean` and its deletion facts.
pub mod command_clean;

/// The Git 2.56.0 option table of `git reset` and its mode.
pub mod command_reset;

/// `git stash` subcommand dispatch and wrapper-flag positions.
pub mod command_stash;

/// The commands that can overwrite or delete uncommitted state, read as Git reads them.
pub mod command_discard;

/// The Git 2.56.0 option tables of every `git stash` subcommand.
pub mod command_stash_table;

/// Branch-creation facts of `git branch`,
///  `git checkout` and `git switch`.
pub mod command_branch_create;

/// Which action a tokenized `git branch` region selects.
mod command_branch_mode;

/// The complete Git 2.56.0 option table of `git branch`.
pub mod command_branch_table;

/// Explicit creation and the remote-guess candidate of `git checkout` and `git switch`.
mod command_branch_target;

/// The complete Git 2.56.0 option tables of `git checkout` and `git switch`.
pub mod command_checkout_table;

/// The Git 2.56.0 option groups of `git config`.
pub mod command_config_table;

/// The form and file scope of a `git config` region.
pub mod command_config;

/// Pure decision of the require-root policy.
pub mod rule_require_root;

/// Whether a `git worktree` invocation creates or moves a worktree.
pub mod command_worktree;

/// Wrapper-only control spellings,
///  their meanings,
///  and removal before the subcommand.
pub mod wrapper_controls;

/// Removal of every wrapper control from one invocation,
///  by position.
pub mod wrapper_invocation;

/// Lifecycle triggers and the trigger set of every shipped policy.
pub mod policy_trigger;

/// Policy events and their JSON Lines rendering.
pub mod policy_events;

/// A pathname as events report it:
///  text,
///  plus base64 `pathBytes` when it is not UTF-8.
pub mod event_path;

/// Exact repository path bytes as the operating system takes them, and as text for messages.
pub mod path_bytes;

/// Ordered policy execution for one lifecycle point.
pub mod policy_engine;

/// Slice 6: writes one corrected file into the private commit index as a blob and an index entry.
pub mod private_index_stage;

/// Slice 6: the commit path's bounded policy convergence over the private index.
pub mod commit_convergence;

/// The commit transaction in the incumbent's order, up to the settled policies.
pub mod commit_transaction;

/// The conclusion of a settled commit transaction: preparation, landing and post-landing completion.
pub mod commit_conclusion;

/// The landing loop with replay, revalidation and the starvation reservation.
pub mod commit_landing_loop;

/// The landing lock a landing attempt takes, with the recovery every acquisition runs first.
pub mod commit_landing_lock;

/// Revalidation of a replayed tree outside both landing locks.
pub mod commit_revalidation;

/// The normalization-only transaction for a correction that removed every change.
pub mod commit_no_change;

/// The finding for a staged deletion that an explicit-path commit cannot record.
pub mod commit_staged_deletions;

/// The `git commit` command: the transaction, the settled events, the post-commit gate and the backup.
pub mod commit_command;

/// The disposable, recoverable workspace of one commit transaction.
pub mod commit_workspace;

/// Rule facts from the real repository with candidates read from the private index.
pub mod commit_facts;

/// The worktree completion records of selected corrections and of hook changes.
pub mod commit_worktree_records;

/// The replay-relevant commit options.
pub mod commit_replay_options;

/// Slice 9 skeleton: the order around a landed commit and a guarded Git child; not reachable until landing is merged.
pub mod commit_lifecycle;

/// The bounded loop that repeats a pass after corrections changed candidate content.
pub mod policy_convergence;

/// The one query for repository identity,
///  top level and the path below it.
pub mod repository_location;

/// Lazily fetched repository facts behind one interface,
///  with the real-Git provider.
pub mod repository_facts;

/// Pure decision of the linked-worktree-only policy.
pub mod rule_linked_worktree;

/// The rejection texts of the linked-worktree-only policy.
pub mod rule_linked_worktree_messages;

/// The append-only record of every command form that can discard uncommitted state.
pub mod discard_audit;

/// The intent-guessing hint that follows a finding, an engine failure or a notice.
pub mod event_hints;

/// Pure decision of the branch-worktree-only policy and its one-query remote guess.
pub mod rule_branch_worktree;

/// Pure decision of the add-explicit policy.
pub mod rule_add_explicit;

/// The repository content the dependent-version planner reads, and its two failure kinds.
pub mod dependent_version_content;

/// Which paths the dependent-version planner reads: manifests, configuration, source.
pub mod dependent_version_paths;

/// Strict JSON reading of a manifest's name, version and dependency names.
pub mod dependent_version_manifest;

/// Patch bumps of plain release versions and `JSON.stringify` quoting.
pub mod dependent_version_release;

/// The byte-preserving rewrite of a manifest's top-level version.
pub mod dependent_version_text;

/// Publishable package names from the generated registry configuration.
pub mod dependent_version_publishable;

/// Whether source text imports a package.
pub mod dependent_version_imports;

/// The dependency walk and the plan of dependent bumps.
pub mod dependent_version_graph;

/// Every workspace manifest at both states, and which versions were raised.
pub mod dependent_version_states;

/// Which development dependencies each dependent bundles.
pub mod dependent_version_bundled;

/// The dependent-version plan with exact manifest bytes before and after.
pub mod dependent_version_plan;

/// The decision and findings of `mono/dependent-version-bump`.
pub mod dependent_version_policy;

/// The dependent-version planner's view of a prepared candidate state.
pub mod dependent_version_lifecycle;

/// Whether a direct fix may change a tracked file it did not select.
pub mod policy_added_path;

/// The `mono/dependent-version-bump` check over a lifecycle's candidate state.
pub mod policy_dependent_version;

/// An in-memory workspace that records reads; never in the release build.
#[cfg(test)]
mod dependent_version_test_support;

/// Differential fixture reading and canonical results; never in the release build.
#[cfg(test)]
mod dependent_version_fixture_json;

/// Canonical renderings of the native planner's results; never in the release build.
#[cfg(test)]
mod dependent_version_fixture_output;

/// One differential case evaluated by the native planner; never in the release build.
#[cfg(test)]
mod dependent_version_fixture_cases;

/// The native planner against the incumbent's recorded results; never in the release build.
#[cfg(test)]
mod dependent_version_differential_tests;

/// The shipped policies as checks over rule cores and repository facts.
pub mod policy_checks;

/// What a lifecycle offers content policies,
///  and its candidates once prepared.
pub mod policy_content;

/// The built-in final-newline policy:
///  canonical final LF,
///  preserved paths,
///  findings.
pub mod policy_final_newline;

/// The optional root-context policy:
///  no top-level `CONTEXT.md` enters the index.
pub mod policy_root_context;

/// The optional forbidden-strings policy over the linked scanner,
///  with redacted findings.
pub mod policy_forbidden_strings;

/// One child program run with a time bound and output caps,
///  through files instead of pipes.
pub mod bounded_child;

/// Where the `markdown/autofix` policy finds `monochromatic-lint`:
///  absolute PATH entries only.
pub mod markdown_linter_lookup;

/// What one linter run printed:
///  the fixed source and the findings left,
///  or why it is unusable.
pub mod markdown_linter_output;

/// The one-rule linter configuration and the private temporary file it is written to.
pub mod markdown_linter_config;

/// The gitignore-syntax `exclude` patterns of `markdown/autofix`,
///  matched as the linter matches them.
pub mod markdown_exclude;

/// The `markdown/autofix` policy over candidates,
///  behind its linter interface.
pub mod policy_markdown;

/// The executable's linter for that policy:
///  `monochromatic-lint` on PATH,
///  bounded,
///  once per candidate.
pub mod markdown_linter;

/// The executable's panic hook:
///  where an internal error happened,
///  never its message.
pub mod panic_notice;

/// Installing a direct fix's corrections into the worktree,
///  all or none.
pub mod direct_fix_install;

/// The direct fix:
///  converge corrections in memory,
///  then install them.
pub mod direct_fix;

/// The fixed argument transforms of a forwarded command,
///  in order.
pub mod policy_transforms;

/// One complete pass:
///  built-in policies,
///  fixed transforms,
///  optional policies.
pub mod policy_pass;

/// The names of the commands built into Git 2.56.0.
pub mod git_builtins;

/// The work of the installed cli-git that is not ported,
///  and the refusal notice.
pub mod unported;

/// The lifecycle of one wrapped Git command,
///  from refusals to forwarding.
pub mod wrapped_command;

/// A scripted repository-facts provider for unit tests;
///  never in the release build.
#[cfg(test)]
mod policy_test_support;

/// The one failure type of the candidate layer and its closed list of causes.
pub mod candidate_error;

/// Validated Git object names and the file modes a candidate can carry.
pub mod candidate_object;

/// Pure framing of one `git cat-file --batch` reply.
pub mod candidate_batch;

/// Pure parsing of raw NUL-delimited changed-path records.
pub mod candidate_record;

/// The long-lived object reader that answers every blob read of an invocation.
pub mod candidate_reader;

/// Immutable candidate versions:
///  identity,
///  pathname,
///  mode,
///  change and object per path.
pub mod candidate_version;

/// Per-invocation listing,
///  lazy bytes and invalidation of candidate versions.
pub mod candidate_store;

/// Pure parsing of `git ls-files --stage` records and the delta between two index states.
pub mod candidate_stage;

/// The files of a commit's tree with their modes and objects.
pub mod candidate_tree;

/// A private,
///  timestamp-preserving copy of the real index,
///  removed when dropped.
pub mod candidate_private_index;

/// Which candidate paths have their `text` Git attribute unset,
///  read from the state the candidates were listed from.
pub mod candidate_attribute;

/// What `git add` would stage,
///  and the worktree files a direct command selects.
pub mod candidate_prediction;

/// Which candidates hold content that a parent commit's tree already holds,
///  found by exact object name.
pub mod candidate_known_content;

/// The linked forbidden-strings scanner:
///  one load per invocation,
///  typed redacted findings.
pub mod scanner_adapter;

/// Rules-file precedence and which candidates the scanner is given.
pub mod scanner_selection;

/// One scan pass over a candidate version's exact bytes.
pub mod scanner_run;

/// Which engine failure code each candidate and scanner failure carries.
pub mod scanner_failure_code;

/// Process isolation and rule fixtures for scanner controls;
///  never in the release build.
#[cfg(test)]
mod scanner_test_support;

/// Private,
///  crash-safe file primitives shared by every durable record.
pub mod private_storage;

/// Compact JSON records read and written with the incumbent's `JSON` semantics.
pub mod json_record;

/// Random identifiers in the layout of the incumbent's `randomUUID`.
pub mod random_id;

/// Text operations with the exact semantics of JavaScript's built-ins.
pub mod js_text;

/// The PID namespace an owner record names, and the rule for owners of another one.
pub mod pid_namespace;

/// Process-birth identities in the incumbent's exact spelling.
pub mod process_identity;

/// The owner record inside every owner-lock directory.
pub mod owner_lock_record;

/// Rename-published owner locks with dead-owner retirement and an unbounded wait.
pub mod owner_lock;

/// Line-oriented debug and warning diagnostics on standard error.
pub mod diagnostic_log;

/// The fail-closed failure of reading or recovering durable transaction state.
pub mod recovery_error;

/// The owner record of one commit-transaction directory and its liveness rule.
pub mod transaction_owner;

/// The per-worktree registry of commit-transaction directories.
pub mod transaction_registry;

/// The schema-version-2 journal records of one commit transaction and their filenames.
pub mod transaction_journal;

/// The exact encoding of every journal record.
pub mod transaction_journal_encode;

/// Strict parsers of every journal record.
pub mod transaction_journal_parse;

/// UTC timestamps in the exact form of JavaScript's `toISOString`.
pub mod iso_time;

/// The filesystem identity recorded with every real `index.lock` a transaction creates.
pub mod filesystem_identity;

/// Running real Git for transaction,
///  recovery and landing work.
pub mod transaction_git;

/// The preparation and landing leases nested wrapper invocations inherit.
pub mod transaction_lease;

/// The dispatch plan of one commit transaction, its JSON spelling and the config reads it makes.
pub mod hook_dispatch_plan;

/// The hooks directory of one transaction: its plan and one entry per preparation event.
pub mod hook_dispatch_entries;

/// The hook dispatcher entered by a transaction's hook entry name, under the hook lock.
pub mod hook_dispatch;

/// Whether a commit event can have a user hook, so an event without one needs no dispatcher entry.
pub mod hook_presence;

/// No-follow file operations recovery and landing share.
pub mod recovery_files;

/// Durable evidence a dead transaction left:
///  attempt records,
///  owned locks and Git's PID file.
pub mod recovery_evidence;

/// The reflog search that proves which commit an interrupted transaction landed.
pub mod recovery_reflog;

/// Deciding whether a landing landed,
///  and installing its recorded post-index.
pub mod recovery_completion;

/// The `.keep` files protecting a landing's migrated pack.
pub mod pack_keep;

/// Test-only transaction phase markers, compiled into the release executable and armed only by an environment variable.
pub mod transaction_phase;

/// Landing target comparison, nonce reflog message and compare-and-swap.
pub mod landing_support;

/// Migration of a prepared commit's shadow objects into a kept pack of the real store.
pub mod landing_objects;

/// Post-index a landing installs, computed against the real index at landing.
pub mod landing_index;

/// The check that a landing changed no staged entry outside the paths it committed.
pub mod landing_invariant;

/// One landing attempt inside the landing critical section, and its outcome.
pub mod landing_attempt;

/// The `concurrent-commit` finding for a landing that landed nothing.
pub mod landing_findings;

/// Git commands run against a transaction's shadow repository.
pub mod shadow_git;

/// Native conclusion cleanup reproduced in the owning worktree.
pub mod conclusion_cleanup;

/// The per-worktree capture store:
///  location,
///  sequence and identity.
pub mod capture_store;

/// `captured.json`,
///  landed-capture records,
///  their recording and pruning.
pub mod capture_records;

/// Replay: the unified-diff parser of subsumption (`replay_patch`).
pub mod replay_patch;

/// Replay: the exact strict reverse check of subsumption (`replay_reverse_apply`).
pub mod replay_reverse_apply;

/// Replay: the one-sided extension of subsumption over zero-context hunks (`replay_containment`).
pub mod replay_containment;

/// Replay: raw commit objects as bytes, the unsigned rewrite and signed-rebuild inputs (`replay_object`).
pub mod replay_object;

/// Replay: capture order over the landed first-parent history and the per-path decision (`replay_capture_order`).
pub mod replay_capture_order;

/// Replay: the starvation reservation of the next landing slot (`replay_reservation`).
pub mod replay_reservation;

/// Replay: the shadow-repository Git runner that every replay step uses (`replay_shadow`).
pub mod replay_shadow;

/// Replay: shared paths of the landed history and the prepared commit, with their three entries (`replay_shared_paths`).
pub mod replay_shared_paths;

/// Replay: the text half of subsumption over index-named blobs and hunks (`replay_subsumption_text`).
pub mod replay_subsumption_text;

/// Replay: subsumption rules and the synthetic merge base (`replay_subsumption`).
pub mod replay_subsumption;

/// Replay: capture-order history and the per-path decisions from the transaction's and landed records (`replay_capture_history`).
pub mod replay_capture_history;

/// Replay: the merge tree, the first conflicting landed commit, and the replayed commit rebuild (`replay_merge`).
pub mod replay_merge;

/// Replay: the replayed index, the shadow HEAD move, and the pre-commit re-run decision of revalidation (`replay_revalidation`).
pub mod replay_revalidation;

/// Replay: one replay of a prepared commit onto a moved target, the integrator's replay result (`replay_commit`).
pub mod replay_commit;
/// The invocation capture:
///  repository layout, symbolic `HEAD`, conclusion kind, ref backend and base.
pub mod capture_invocation;

/// The capture under capture order:
///  the capture lock, the store identity, the sequence and `captured.json`.
pub mod capture_order;

/// The shadow repository `config` file and the copied repository format.
pub mod shadow_config;

/// The shadow repository links to the real shared entries.
pub mod shadow_links;

/// The shadow ref store: snapshot, private target, symbolic refs and `HEAD`.
pub mod shadow_refs;

/// The shadow conclusion copies at preparation.
pub mod conclusion_copy;

/// Argument transforms of native preparation: normalisation, global split, `-a` removal and private arguments.
pub mod commit_args;

/// The private commit index: copies, selection modes, pathspec files and listings.
pub mod commit_index;

/// Native `git commit` in the shadow, the prepared commit read back and `prepared.json`.
pub mod commit_preparation;

/// Creation of the shadow repository of one transaction.
pub mod shadow_create;

/// Loading many blobs through one `git cat-file --batch`.
pub mod blob_batch;

/// Bringing added paths' worktree copies to their landed bytes.
pub mod worktree_completion;

/// Owner inspection of registry entries and the oldest-first order.
pub mod recovery_inspect;

/// Recovery of one dead-owner transaction.
pub mod recovery_landing;

/// Disposal of a landed transaction: shadow removal, directory removal, then landed-capture pruning.
pub mod landed_disposal;

/// The post-commit run after a landing: author environment, hook lock, and the hook runner with inherited streams.
pub mod post_commit_hook;

/// Post-landing completion of one landed commit: worktree copies, disposal, maintenance, then `post-commit`.
pub mod post_landing;

/// Recovery across every transaction directory of one worktree.
pub mod recovery_scan;

/// Startup recovery of interrupted commit transactions.
pub mod commit_recovery;

/// Startup recovery as a step of a wrapped or direct command.
pub mod transaction_gate;

/// Process start times for lock owners,
///  read from `/proc`,
///  `ps` or the Windows process table.
pub mod process_start;

/// The Git version check run once before work that depends on Git's behavior.
pub mod git_version;

/// Processes holding a lock file open,
///  found by device and inode.
pub mod index_lock_holders;

/// Parsers of `lsof -F pcDi` records and Restart Manager lines,
///  matched by device and inode.
pub mod index_lock_holder_parsers;

/// Evidence about a foreign `index.lock` and the verdict it supports.
pub mod index_lock_evidence;

/// The Git command a forwarded invocation runs,
///  after ordinary alias expansion.
pub mod forwarded_command;

/// Index-writer classification and the commands that coordinate with a landing.
pub mod index_writer_commands;

/// Waiting for a foreign index.lock under the classification rules and Git's backoff schedule.
pub mod index_lock_wait;

/// The host side effects of the index-lock wait:
///  /proc reads,
///  sleeps,
///  randomness and standard error.
pub mod index_lock_wait_host;

/// The landing location a forwarded index writer coordinates with:
///  the real index and the registry.
pub mod landing_location;

/// Coordination of a forwarded index writer:
///  the landing lock,
///  the wait for a foreign lock and the lease.
pub mod index_writer_coordination;

/// Running a coordinated forwarded index writer:
///  the landing lock held while real Git runs as a child.
pub mod index_writer_forward;

/// The real index.lock a landing creates,
///  identifies and releases only while it is the same file.
pub mod real_index_lock;
/// Automatic maintenance decision: config record parsing and the enable and detach rules.
pub mod maintenance_decision;

/// Running automatic maintenance after a landing: the config read, then `git maintenance run --auto`.
pub mod maintenance_run;

/// Catching SIGINT, SIGTERM, SIGHUP and SIGQUIT while a Git child runs, and relaying only process-sent signals.
pub mod signal_relay;

/// Exact removal of retired hk hook registrations from one explicit Git configuration scope.
pub mod maintenance_hk_config;

/// The per-branch push coordination file name: a reversible, flat, case-safe key.
pub mod auto_push_branch_key;

/// The per-branch `last-pushed` record: validation, tolerant reads and atomic publication.
pub mod auto_push_record;

/// Waiting for a lock a live process holds: a slow-wait notice naming the holder and a configurable backstop.
pub mod live_lock;

/// Read-only Git queries of the auto-push, each with a deadline and separate output streams.
pub mod bounded_query;

/// The auto-push's real `git push` with a deadline, its low-speed abort and the lock wait limit derived from it.
pub mod bounded_push;

/// Single-flight push coordination per branch: owner lock, record joins and the push itself.
pub mod auto_push_single_flight;

/// Publishing a landed commit that a moved remote branch rejected: the merge is made from objects alone and pushed.
pub mod auto_push_reconcile;

/// The paths of a dirty worktree and of a fast-forward, and their overlap.
pub mod auto_push_dirty_paths;

/// Post-commit auto-push of a landed commit: the push argv, the surfaced output and the failure reporting.
pub mod auto_push;

/// The post-commit gate over a landed commit: its events, its blocking and the backup it allows.
pub mod post_commit;

/// Authoritative manual-push updates discovered by a private dry-run push and confirmed with ls-remote.
pub mod manual_push_probe;

/// Published-set and range computation for manual-push scans: which pushed commits a remote does not have.
pub mod manual_push_range;

/// An explicit push that lost a race: run as a child, merge in what landed on the remote and publish the merge.
pub mod push_stacking;

/// The manual-push candidate scope of one push: the commits and trees the destination lacks.
pub mod manual_push_lifecycle;
/// The one failure type of the linked-worktree ignored-state copy and its diagnostics.
pub mod worktree_copy_error;

/// Which linked worktrees a worktree-creating Git command registered, before and after it ran.
pub mod worktree_copy_registry;

/// The ignored source paths a linked-worktree copy selects, and the path validation every selection passes.
pub mod worktree_copy_selection;

/// The deterministic parent-first manifest of a copy, and the mode application to a staged copy.
pub mod worktree_copy_manifest;

/// Exact comparison of manifest entries against the filesystem, and the source stability check after staging.
pub mod worktree_copy_compare;

/// The private, validated stage of one source's ignored state beside a destination worktree.
pub mod worktree_copy_stage;

/// The exclusive creation of one destination entry from the stage, and the identity captured after it.
#[cfg(unix)]
pub mod worktree_copy_install_entry;

/// Installation of a validated stage into a new worktree, with preflight, batches and rollback on failure.
#[cfg(unix)]
pub mod worktree_copy_install;

/// Removal of the paths a failed installation created, only where their identity and content still prove them cli-git's own.
#[cfg(unix)]
pub mod worktree_copy_rollback;

/// The durable journal of one copy: exact encoding, strict validation, private storage and removal.
#[cfg(unix)]
pub mod worktree_copy_journal;

/// The durable claim on a stage container's name, and the removal of stages an interrupted copy left behind.
#[cfg(unix)]
pub mod worktree_copy_stage_claim;

/// The append-only install log of one stage: per-batch intents and creations, with resumption.
#[cfg(unix)]
pub mod worktree_copy_install_log;

/// One journaled installation to an end, resumption from the log, and synchronization of a created worktree.
#[cfg(unix)]
pub mod worktree_copy_transaction;

/// The settlement lock that serializes copy installation and recovery, its owner record, and inherited leases.
#[cfg(unix)]
pub mod worktree_copy_lock;

/// Recovery of interrupted copies: registration checks, discards with notices, resumption, and the pending check.
#[cfg(unix)]
pub mod worktree_copy_recovery;

/// The linked-worktree ignored-state copy around one forwarded Git invocation: the integrator's entry point.
#[cfg(unix)]
pub mod worktree_copy;

/// Which forwarded invocations the copy applies to, and the child run that carries its settlement lease.
pub mod worktree_copy_forward;
