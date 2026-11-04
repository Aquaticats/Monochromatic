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

/// The four top-level spellings after which the wrapper announces itself,
///  and the text it prints once Git has answered.
pub mod announced_query;

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

/// The refusal of a direct command whose `--policy` selects a policy that is off.
pub mod management_selection;

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

/// A Git stand-in that logs each start and runs the real Git;
///  never part of the release executable.
#[cfg(test)]
#[cfg(unix)]
mod recording_git;

/// The Git starts of the layout questions for a user whose own file holds `core.fsmonitor = true`;
///  never part of the release executable.
#[cfg(test)]
#[cfg(unix)]
mod reader_fsmonitor_tests;

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

/// Per-case copies of a pristine fixture and worker threads for independent test cases;
///  never in the release build.
#[cfg(test)]
mod command_test_cases;

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

/// The `checkout` and `restore` command lines that take one side of a conflicted file.
pub mod command_conflict_side;

/// Conflicted paths, the decision on a command line that names them, and the saved copy of each file.
pub mod conflict_sides;

/// The linked-worktree policy's carve-out for conflicted files during a merge, rebase, cherry-pick or revert.
pub mod conflict_side_guard;

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

/// The normalization-only transaction for a correction that removed every change.
pub mod commit_no_change;

/// The finding for a staged deletion that an explicit-path commit cannot record.
pub mod commit_staged_additions;
pub mod commit_staged_deletions;

/// The `git commit` command: the transaction, the settled events, the post-commit gate and the backup.
pub mod commit_command;

/// Rule facts from the real repository with candidates read from the private index.
pub mod commit_facts;

/// The worktree completion records of selected corrections and of hook changes.
pub mod commit_worktree_records;

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

/// The record of where one wrapped commit spent its time,
/// appended beside the discard audit.
pub mod commit_timing;

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

/// The optional catalog-specifier policy over the shared rule crate:
///  every dependency in a tracked `package.json` is a catalog or workspace reference.
pub mod policy_catalog_specifiers;

/// The optional forbidden-strings policy over the linked scanner,
///  with redacted findings.
pub mod policy_forbidden_strings;

/// What one in-process linter run produced:
///  the fixed source and the findings left,
///  or why it is unusable.
pub mod markdown_linter_output;

/// The one-rule linter configuration document.
pub mod markdown_linter_config;

/// The gitignore-syntax `exclude` patterns of `markdown/autofix`,
///  matched as the linter matches them.
pub mod markdown_exclude;

/// The `markdown/autofix` policy over candidates,
///  behind its linter interface.
pub mod policy_markdown;

/// The executable's linter for that policy:
///  `monochromatic-lint` linked in-process,
///  guarded by a deadline,
///  once per candidate.
pub mod markdown_linter;

/// A job on a thread of its own with a deadline and panic containment.
pub mod markdown_linter_guard;

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

/// Where short-lived private files live:
///  the per-user scratch directory and the repository scratch directory,
///  never the system temporary directory.
pub mod scratch_location;

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

/// Line-oriented debug and warning diagnostics on standard error.
pub mod diagnostic_log;

/// The fail-closed failure of reading or recovering durable transaction state.
pub mod recovery_error;

/// The owner record of one commit-transaction directory and its liveness rule.
pub mod transaction_owner;

/// The schema-version-2 journal records of one commit transaction and their filenames.
pub mod transaction_journal;

/// UTC timestamps in the exact form of JavaScript's `toISOString`.
pub mod iso_time;

/// Running real Git for transaction,
///  recovery and landing work.
pub mod transaction_git;

/// Lexical path normalization.
pub mod lexical_path;

/// The ban on Git starting the wrapper for a hook: no entry writer, no start-name dispatch.
#[cfg(test)]
#[path = "hook_self_invocation_tests.rs"]
mod hook_self_invocation_tests;

/// What `git cli-git doctor` reports from the configuration: configured hooks and the retired `hooks` key.
pub mod doctor_config;

/// The configured external hooks, read from the snapshot, and the loud report that none runs.
pub mod external_hooks;

/// The non-blocking warning for a configured `core.hooksPath`, decided from the configuration snapshot alone.
pub mod traditional_hook_warning;

/// `git cli-git doctor`: the traditional hooks a repository holds, which are unsupported.
pub mod doctor;

/// When a push needs the Git LFS upload, and the texts the parent prints around it.
pub mod lfs_pre_push;

/// What the Git LFS upload sends: the remote and the refs, read from the push and the configuration Git would read.
pub mod lfs_upload_plan;

/// The configuration the Git LFS upload plan reads, and the default remote derived from it.
pub mod lfs_upload_settings;

/// Hides the credentials inside URLs in the text relayed from `git-lfs`.
pub mod lfs_output_mask;

/// Which failed `git-lfs push` runs are tried again (server errors only), after how long, and the notice.
pub mod lfs_server_retry;

/// Whether a blob is a Git LFS pointer, decided as `git-lfs` decides it.
pub mod lfs_pointer;

/// The LFS objects a push introduces, found in process from the commits to be pushed.
#[cfg(unix)]
pub mod lfs_pointer_scan;

/// The upload plan narrowed to the objects the push introduces, or to nothing.
pub mod lfs_object_plan;

/// The parent's bounded or interactive run of the Git LFS upload.
pub mod lfs_upload;

/// The Git LFS step before a push the caller typed: pre-check, plan and upload with the caller's streams.
pub mod lfs_interactive;

/// The check that `git-lfs` can start, made before a push that is preceded by the upload.
pub mod lfs_push_precheck;

/// The auto-push's bounded `git push`, preceded by the upload, its pre-check and its explanation.
pub mod lfs_bounded_push;

/// The push of a merge that integrated a lost race, planned and run inside the running wrapper.
pub mod stacked_merge_push;

/// Every real-Git start of the wrapper carries `core.hooksPath` pointing at a directory that never exists.
pub mod hooks_off;

/// No-follow file operations recovery and landing share.
pub mod recovery_files;

/// Test-only transaction phase markers, compiled into the release executable and armed only by an environment variable.
pub mod transaction_phase;

/// Git commands run against a transaction's shadow repository.
pub mod shadow_git;

/// Native conclusion cleanup reproduced in the owning worktree.
pub mod conclusion_cleanup;

/// Replay: the unified-diff parser of subsumption (`replay_patch`).
pub mod replay_patch;

/// Replay: the exact strict reverse check of subsumption (`replay_reverse_apply`).
pub mod replay_reverse_apply;

/// Replay: the one-sided extension of subsumption over zero-context hunks (`replay_containment`).
pub mod replay_containment;

/// Replay: capture order over the landed first-parent history and the per-path decision (`replay_capture_order`).
pub mod replay_capture_order;

/// Replay: the shadow-repository Git runner that every replay step uses (`replay_shadow`).
pub mod replay_shadow;

/// Replay: shared paths of the landed history and the prepared commit, with their three entries (`replay_shared_paths`).
pub mod replay_shared_paths;

/// Replay: the text half of subsumption over index-named blobs and hunks (`replay_subsumption_text`).
pub mod replay_subsumption_text;

/// Replay: subsumption rules and the synthetic merge base (`replay_subsumption`).
pub mod replay_subsumption;

/// Replay: the merge tree, the first conflicting landed commit, and the replayed commit rebuild (`replay_merge`).
pub mod replay_merge;

/// The empty tree's object name under each known object format.
pub mod empty_tree;

/// Files-backend ref reads that answer without a Git process or decline.
pub mod ref_files;

/// Proof, without a Git process, that a sequencer marker names nothing on the files ref storage.
pub mod sequencer_marker;

/// The invocation capture:
///  repository layout, symbolic `HEAD`, conclusion kind, ref backend and base.
pub mod capture_invocation;

/// The private index and object store a commit's preparation selects, and the Git request that carries them.
pub mod private_index_request;

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

/// The entries of an empty bare repository, created in-process instead of by `git init`.
pub mod shadow_skeleton;

/// A read of the assume-unchanged and skip-worktree marks of an index file.
pub mod index_flags;

/// Loading many blobs through one `git cat-file --batch`.
pub mod blob_batch;

/// Bringing added paths' worktree copies to their landed bytes.
pub mod worktree_completion;

/// The startup step of a command: isolated merges whose process died, and what an earlier release left behind.
pub mod startup_gate;

/// Process start times for lock owners,
///  read from `/proc`,
///  `ps` or the Windows process table.
pub mod process_start;

/// The Git version check run once before work that depends on Git's behavior.
pub mod git_version;

/// The private per-user cache of the version text, keyed by the Git executable's identity.
pub mod git_version_cache;

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

/// The landing location a forwarded index writer coordinates with:
///  the real index and the registry.
pub mod landing_location;

/// Running a coordinated forwarded index writer:
///  the landing lock held while real Git runs as a child.
pub mod index_writer_forward;

/// The `git merge` command lines the wrapper completes in isolation, and those it leaves to native Git.
pub mod merge_options;

/// The durable intent record of an isolated merge.
pub mod merge_isolation_journal;

/// The raw differences of an isolated merge and the worktree state of each changed path.
pub mod merge_isolation_changes;

/// Recovery of an isolated merge whose process died.
pub mod merge_isolation_recovery;

/// The commit message of an isolated merge.
pub mod merge_isolation_message;

/// The refusal findings of an isolated merge.
pub mod merge_isolation_report;

/// A true merge completed in isolation while the index holds staged changes.
pub mod merge_isolation;

/// Automatic maintenance decision: config record parsing and the enable and detach rules.
pub mod maintenance_decision;

/// Running automatic maintenance after a landing: the config read, then `git maintenance run --auto`.
pub mod maintenance_run;

/// Catching SIGINT, SIGTERM, SIGHUP and SIGQUIT while a Git child runs, and relaying only process-sent signals.
pub mod signal_relay;

/// The per-branch file name of the push failure notice: a reversible, flat, case-safe key.
pub mod auto_push_branch_key;

/// How long a command waits for the commit service's hold on the index, and the variable that sets it.
pub mod wait_limit;

/// The shared clock of every wait notice: the grace, the repeat interval and a queued commit's place in line.
pub mod wait_progress;

/// A watcher for waits that block on a child process: notices at the cadence and the child stopped at the bound.
pub mod child_watch;

/// The classes of Git step a commit starts for itself, with the bound and the notice grace of each.
pub mod child_class;

/// Read-only Git queries of the auto-push, each with a deadline and separate output streams.
pub mod bounded_query;

/// The auto-push's real `git push` with a deadline and its low-speed abort.
pub mod bounded_push;

/// The processes of this host that use a directory or a file.
pub mod process_use;

/// The processes below one process, read before it is stopped and signalled only while they are still the ones found.
pub mod process_tree;

/// The real Git of a push, its query runner and the failure types of a push.
pub mod auto_push_single_flight;

/// Where a plain `git push` of one named branch goes, from the configuration and the local refs.
pub mod auto_push_destination;

/// The push a merge, pull, cherry-pick, revert or am hands to the service when it leaves the branch ahead of its upstream.
#[cfg(unix)]
pub mod auto_push_trigger;

/// The record of each decision of the push trigger: handed over, or declined with its reason.
pub mod push_trigger_log;

/// Publishing a landed commit that a moved remote branch rejected: the merge is made from objects alone and pushed.
pub mod auto_push_reconcile;

/// The paths of a dirty worktree and of a fast-forward, and their overlap.
pub mod auto_push_dirty_paths;

/// Post-commit auto-push of a landed commit: the push argv, the surfaced output and the failure reporting.
pub mod auto_push;

/// The post-commit gate over a landed commit: its events, its blocking and the backup it allows.
pub mod post_commit;

/// Authoritative manual-push updates discovered by a private dry-run push and confirmed with ls-remote.
pub mod manual_push_porcelain;

/// Why a manual push's scope could not be established, as the cause its hint is chosen by.
pub mod manual_push_cause;

/// The updates a manual `git push` would send, learned from Git's own dry-run report and confirmed against the remote.
pub mod manual_push_probe;

/// Published-set and range computation for manual-push scans: which pushed commits a remote does not have.
pub mod manual_push_range;

/// An explicit push that lost a race: run as a child, merge in what landed on the remote and publish the merge.
pub mod push_stacking;

/// Starting a child again when the operating system refuses it for lack of processes, within a bounded wait.
pub mod start_retry;

/// The manual-push candidate scope of one push: the commits and trees the destination lacks.
pub mod manual_push_lifecycle;
/// The one failure type of the linked-worktree ignored-state copy and its diagnostics.
pub mod worktree_copy_error;

/// Which linked worktrees a worktree-creating Git command registered, before and after it ran.
pub mod worktree_copy_registry;

/// The ignored source paths a linked-worktree copy selects, and the path validation every selection passes.
pub mod worktree_copy_selection;

/// What a copy from a main checkout leaves out: caches, optionally run records, and other checkouts.
pub mod worktree_copy_prune;

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

/// The system configuration file a real Git executable reads, learned once per executable identity.
pub mod git_system_config;

/// In-process repository reads: the plain-text configuration reader that gives up on any spelling it cannot prove.
pub mod repo_read_config_text;

/// In-process repository reads: proof that no configuration scope sets `core.bare` or `core.worktree`.
pub mod repo_read_config_scan;

/// In-process repository reads: proof that every configuration value Git types at startup is one Git accepts silently.
pub mod repo_read_config_typed;

/// In-process repository reads: the location, Git paths and object format found from the file system.
pub mod repo_read_layout;

/// In-process repository reads: the layout turned into the shapes the wrapper's existing queries answer with.
pub mod repo_read_adapter;

/// In-process repository reads: object kinds, names and the zlib step shared by loose objects and packs.
pub mod repo_read_object;

/// In-process repository reads: one pack, its index lookup and delta resolution.
#[cfg(unix)]
pub mod repo_read_pack;

/// In-process repository reads: the object database of a repository, its packs and alternates.
#[cfg(unix)]
pub mod repo_read_odb;

/// The abbreviated object name Git prints (`core.abbrev`, the automatic length, uniqueness), computed without starting Git.
#[cfg(unix)]
pub mod object_abbreviation;

/// In-process repository reads: trees, flattened trees and commit fields.
#[cfg(unix)]
pub mod repo_read_tree;

/// In-process repository reads: the entries of an index file, versions 2 to 4.
pub mod repo_read_index;

/// In-process repository reads: the difference between two sorted entry lists and the output spellings.
#[cfg(unix)]
pub mod repo_read_diff;

/// In-process repository reads: the files ref storage, one ref by name, a symbolic chain and the list of all refs.
#[cfg(unix)]
pub mod repo_read_refs;

/// In-process repository reads: the merged configuration listing with includes and environment entries.
pub mod repo_read_config_list;

/// In-process repository reads: answers to the exact read-only command lines the wrapper issues.
#[cfg(unix)]
pub mod repo_read_query;

/// In-process repository reads: answers for the reference and configuration command lines.
#[cfg(unix)]
pub mod repo_read_query_refs;

/// In-process repository reads: the two runner entry points that try the answers before starting Git.
pub mod repo_read_hook;

/// The standards-review shadow run: JavaScript text behaviors the log depends on (UTF-16 lengths, number text).
pub mod standards_review_text;

/// The standards-review shadow run: a file's changed old-side ranges read from the patch.
pub mod standards_review_patch;

/// The standards-review shadow run: which commits get an entry, and the capped model input.
pub mod standards_review_input;

/// The standards-review shadow run: the Decisions API request body and the answer reading.
pub mod standards_review_request;

/// The standards-review shadow run: log entry rendering and appending.
pub mod standards_review_entry;

/// The standards-review shadow run: the in-process HTTPS call to the decisions gateway, with a total deadline.
pub mod standards_review_client;

/// The standards-review shadow run: the two real-Git reads (patch and file texts) of a landed commit.
pub mod standards_review_source;

/// The standards-review shadow run, end to end, and its bounded runner. Nothing calls it yet.
pub mod standards_review_run;

/// A loopback HTTP server for the standards-review tests; never in the release build.
#[cfg(test)]
mod standards_review_mock;

/// The durable work-item record for asynchronous work (review, push, maintenance) left for a resident consumer.
pub mod work_item;
pub mod work_item_recovery;

/// The clocks a claim's lease is judged by.
pub mod claim_time;

/// The repositories the service looks at again when a claim another instance holds may have run out.
pub mod service_retry;

/// Messages between the wrapper and the resident commit service.
pub mod service_protocol;

/// What the user is told when the commit service cannot land a commit.
pub mod service_messages;

/// The commit service's journal and its restart decisions.
pub mod service_journal;

/// Test-only fault points of the commit service.
pub mod service_fault;

/// The commit service's serialized step.
pub mod service_land;

/// What the commit service's landing does once the branch has moved: index entries and conclusion state.
pub mod service_land_follow;

/// The commit service's hold on Git's index lock across a landing.
pub mod service_index_lock;

/// The attempts and the wait of an acquisition of the index lock.
pub mod service_index_attempts;

/// The marker the service writes into the index lock and the questions asked of a marker.
pub mod service_index_marker;

/// The marker of the next batch's index lock, made and flushed while the service is idle.
pub mod service_index_prestage;

/// What an acquisition of the index lock found when it had to wait, for the service's log.
pub mod service_index_found;

/// Who a marker's holder is: the identity a process writes, and the answers a reader gives (gone, running, unknown).
pub mod service_index_identity;

/// The service's hold on a placed index lock: the check that it is still the holder's, the release and the removal.
pub mod service_index_hold;

/// Whether the index lock is still the file the holder placed, and the record of a lock that could not be settled.
pub mod service_index_ownership;

/// The staging files of the index lock.
pub mod service_index_staging;

/// How the service's marked file becomes the index lock, and the cleanup of staging files.
pub mod service_index_placement;

/// The inotify watch the wait for the index lock blocks on.
pub mod service_index_watch;

/// The preparation of a commit that the service lands, for every command line.
pub mod service_prepare;

/// The `discardedCopies` section of `cli-git.config.jsonc`.
pub mod config_discarded;

/// The bound on the folder of saved copies of conflicted files: age, size, audit and doctor report.
pub mod discarded_prune;

/// The folder of saved copies opened once without following a link, with every read and removal addressed through descriptors.
pub mod discarded_store;

/// The native journal protocol and the standard error fallback the service's entries go through.
pub mod journald_native;

/// The lines the commit service writes about its life and about the time each request took.
pub mod service_timing;

/// The line the commit service writes when a signal ends it.
pub mod service_signal;

/// The commit service's loop.
pub mod service_server;

/// The wrapper's side of a commit through the commit service.
pub mod service_client;

/// Which commit command lines the commit service covers, and the message they stand for.
pub mod service_commit_args;

/// The one summary line a landed commit prints, with the abbreviated name Git would print.
pub mod landed_summary;

/// The wait of a wrapped index writer for the commit service's own hold on the index.
pub mod service_index_wait;

/// A forwarded index writer started again when the commit service took the index before its Git did.
pub mod service_hold_retry;

/// The commit command's path through the commit service.
pub mod service_commit;

/// The service's memory of when the bytes of the commits it landed were captured, per worktree.
pub mod service_captures;

/// The retry of a commit the service sent back: the captured change merged onto the branch, never prepared again from the worktree.
pub mod service_replay;

/// Which way a commit goes: through the service, set up first, or the earlier path.
pub mod service_route;

/// The wrapper's own provisioning of the commit service's socket activation.
pub mod service_provision;

/// A minimal D-Bus client for the user session bus, for the provisioning's two calls.
pub mod systemd_dbus;

/// The message and identities of a service commit, made as native `git commit` makes them.
pub mod service_message;

/// What an amend takes from the commit it replaces: parents, message and author.
pub mod service_amend;

/// The commits real Git builds for the service: an editor, a signature, and what the service does not embed.
pub mod service_native;

/// The renewal that keeps the claims of a shared work directory from looking abandoned while their holder works.
pub mod service_claim_renewal;

/// The service's consumer of after-commit push and upkeep work.
pub mod service_work;

/// The wrapper's side of after-commit work: records and the wake.
pub mod service_handoff;

/// Which commit service instance belongs to a worktree: its name, socket and unit instances.
pub mod service_instance;

/// Whether a worktree's head still names the branch a commit was prepared on.
pub mod service_head;

/// How a commit headed for the service notices that its prepared content was removed before it landed.
pub mod service_pruned;

/// The record of a failed backup push and the rule for when a wrapper command shows it.
pub mod push_failure_notice;

/// What becomes of the unshown push notices and unfinished pushes of a linked worktree that Git removes.
pub mod worktree_removal_rescue;

/// The plan for one `git worktree add` that the cheap-worktree ladder serves: its destination and insertion point.
#[cfg(unix)]
pub mod worktree_add_plan;

/// The empty subvolume made at the destination of a `git worktree add` before Git runs, and its cleanup.
#[cfg(target_os = "linux")]
pub mod worktree_subvolume;

/// Whether a worktree is clean enough to copy from or to remove without asking Git, proven in process.
#[cfg(target_os = "linux")]
pub mod worktree_clean;
pub mod worktree_attributes;
pub mod worktree_clean_content;

/// The plan of a `git worktree remove` that names one directory and no option.
#[cfg(unix)]
pub mod worktree_remove_plan;

/// The removal of a clean subvolume worktree by one ioctl, and the plan of the `git worktree remove` it serves.
#[cfg(target_os = "linux")]
pub mod worktree_remove;

/// Fixtures shared by the controls of the cheap-worktree ladder.
#[cfg(all(test, target_os = "linux"))]
mod worktree_ladder_test_support;

/// The snapshot source: whether the creating worktree can be snapshotted for a new worktree, and what that needs.
#[cfg(target_os = "linux")]
pub mod worktree_snapshot_source;

/// The snapshot rung's mechanics: snapshot to a stage name, make it a worktree, swap it into place.
#[cfg(target_os = "linux")]
pub mod worktree_snapshot;

/// The binding of a `git worktree add` to the directories Git made for it, its record, and the check that the binding holds.
#[cfg(target_os = "linux")]
pub mod worktree_add_binding;

/// The clone rung: tracked files cloned into the new worktree, the source's index installed with fresh stat fields.
#[cfg(target_os = "linux")]
pub mod worktree_clone;

/// What both filling rungs read from Git's `--no-checkout` result, and the one step that applies the tree difference.
#[cfg(target_os = "linux")]
pub mod worktree_ladder_target;

/// The ladder's choice of rung before real Git runs.
#[cfg(target_os = "linux")]
pub mod worktree_ladder_run;

/// One entry from the wrapper a person ran to the journal, sent without waiting.
#[cfg(target_os = "linux")]
pub mod client_journal;

/// The plain sentence that says why an add was not made as a snapshot, and where it goes.
#[cfg(target_os = "linux")]
pub mod worktree_slow_route;

/// The ladder's steps after real Git returned: the snapshot rung and the fall back to Git's checkout.
#[cfg(target_os = "linux")]
pub mod worktree_ladder_finish;

/// The cheap-worktree ladder around one `git worktree add`: what runs before real Git and after it.
#[cfg(unix)]
pub mod worktree_ladder;

/// The words that say how to change the push deadline, for the terminal and for the resident commit service.
pub mod push_deadline_advice;

/// The mechanical ban: no release code starts the wrapper, passes it management words or writes shell text.
#[cfg(test)]
#[path = "no_self_invocation_tests.rs"]
mod no_self_invocation_tests;

/// The structural check that the review's network client is reachable only from the service run; never in the release build.
#[cfg(test)]
mod standards_review_boundary_tests;

/// The standards-review shadow run: appending an entry to the shared log without a lock, once per commit, with repair.
pub mod standards_review_append;

/// The service's handler for a review work item, bounded by a call deadline and a total budget.
pub mod service_review;

/// The standards-review credential: redacted when printed, read from the wrapper's environment, sent in the work message.
pub mod service_review_key;

/// The typed failure of one Linux filesystem primitive:
///  the call, the path and the error number.
#[cfg(target_os = "linux")]
pub mod fs_primitive_error;

/// Which filesystem holds a path (btrfs, overlayfs, other) and whether a directory is a btrfs subvolume root.
#[cfg(target_os = "linux")]
pub mod fs_kind;

/// The mount of a path read from `/proc/self/mountinfo`, with its options (`user_subvol_rm_allowed`).
#[cfg(target_os = "linux")]
pub mod fs_mount_options;

/// The btrfs ioctls that create, snapshot and destroy a subvolume, called in process.
#[cfg(target_os = "linux")]
pub mod fs_btrfs_ioctl;

/// `FICLONE` of one file into another and the atomic `RENAME_EXCHANGE` of two paths.
#[cfg(target_os = "linux")]
pub mod fs_clone_exchange;

/// Directory-relative file operations that never follow a link: the held-handle primitives of the clone rung and of the
/// snapshot's stub removal.
#[cfg(target_os = "linux")]
pub mod fs_beneath;

/// The btrfs fixture root of the primitive and migration controls, and the loud report where there is none.
#[cfg(all(test, target_os = "linux"))]
mod fs_btrfs_test_support;

/// SHA-1 and SHA-256, for the checksum of a rewritten Git index.
#[cfg(target_os = "linux")]
pub mod subvolume_sha;

/// The stat rewrite of a Git index file's bytes.
#[cfg(target_os = "linux")]
pub mod subvolume_index;

/// The linked worktrees of a main checkout and the index stat rewrite applied to files.
#[cfg(target_os = "linux")]
pub mod subvolume_index_files;

/// The failure type of the subvolume migration and its simulated-kill hook.
#[cfg(target_os = "linux")]
pub mod subvolume_error;

/// The operating-system operations of the migration behind one trait, and the Linux implementation.
#[cfg(target_os = "linux")]
pub mod subvolume_platform;

/// Copying owner, permissions, times and extended attributes onto a copy.
#[cfg(target_os = "linux")]
pub mod subvolume_sync_attrs;

/// One idempotent pass that makes a directory tree match another, in process.
#[cfg(target_os = "linux")]
pub mod subvolume_sync;

/// The index locks the migration holds, marked with a token that names the migration.
#[cfg(target_os = "linux")]
pub mod subvolume_lock;

/// An open directory that every operation inside it is addressed through, so a replaced path cannot redirect them.
#[cfg(target_os = "linux")]
pub mod subvolume_anchor;

/// Removal of a directory tree whose directories may lack owner write permission.
#[cfg(target_os = "linux")]
pub mod subvolume_remove;

/// The migration journal beside the checkout.
#[cfg(target_os = "linux")]
pub mod subvolume_journal;

/// The paths of a migration and the refusals that stop it before anything is created.
#[cfg(target_os = "linux")]
pub mod subvolume_preflight;

/// The state a migration command carries between its steps.
#[cfg(target_os = "linux")]
pub mod subvolume_run;

/// Converting a main checkout into a btrfs subvolume, resumably.
#[cfg(target_os = "linux")]
pub mod subvolume_migrate;

/// Undoing a migration and deleting the old tree.
#[cfg(target_os = "linux")]
pub mod subvolume_rollback;

/// The grammar of `git cli-git migrate-to-subvolume`.
pub mod subvolume_arguments;

/// `git cli-git migrate-to-subvolume`: root discovery, modes and exit statuses.
#[cfg(target_os = "linux")]
pub mod subvolume_command;

/// The stand-in platform and fixture checkouts of the subvolume migration controls.
#[cfg(all(test, target_os = "linux"))]
mod subvolume_test_support;

/// The migration controls that need the real btrfs ioctls, `FICLONE` and exchange.
#[cfg(all(test, target_os = "linux"))]
mod subvolume_btrfs_tests;

/// Quoting of bytes printed inside a shell command for a person to run.
#[cfg(target_os = "linux")]
pub mod shell_quote;

/// The filesystem section of `git cli-git doctor`: btrfs subvolume, overlayfs, the removal mount option and its commands.
#[cfg(target_os = "linux")]
pub mod doctor_filesystem;

/// Converting a linked worktree that is an ordinary directory into a btrfs subvolume at the same path, resumably.
#[cfg(target_os = "linux")]
pub mod worktree_convert;
pub mod worktree_convert_journal;
pub mod worktree_convert_remove;

/// The grammar of `git cli-git convert-worktree-to-subvolume`.
pub mod worktree_convert_arguments;

/// `git cli-git convert-worktree-to-subvolume`: the worktrees to cover and the report.
#[cfg(target_os = "linux")]
pub mod worktree_convert_command;

/// Setting aside what a snapshot of the main checkout leaves out, and having the commit service empty it.
#[cfg(target_os = "linux")]
pub mod worktree_trash;

/// Disposal of a snapshot's trash: the work item, the foreground fallback and the service's handler.
#[cfg(target_os = "linux")]
pub mod worktree_trash_dispose;

/// What a snapshot's leave-out made and moved, with identities, as the text a work item carries.
#[cfg(target_os = "linux")]
pub mod worktree_trash_manifest;

/// Removal of a directory tree beneath a held parent handle, never following a link.
#[cfg(target_os = "linux")]
pub mod worktree_trash_remove;
