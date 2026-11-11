/**
 Planted guard removals in replay, capture, commit preparation, the shadow repository, signals, maintenance and pushes.

 Each entry removes one guard and names the test that must fail without it; `from` must occur exactly once in the
 current source, so a control cannot silently stop applying. See `native-planted-controls.ts` for how they run.
 */

import type { PlantedControl } from './native-planted-controls.ts';

/**
 Controls for guards in replay, capture, commit preparation, the shadow repository, signals, maintenance and pushes.
 */
export const plantedCommitControls: readonly PlantedControl[] = [
  {
    name: 'strict reverse check line match guard',
    file: 'src/native/shadow/replay_reverse_apply.rs',
    from: 'if !incomplete_last || match_end {',
    to: 'if true {',
    failing: 'shadow::replay_reverse_apply::tests::an_incomplete_preimage_line_matches_past_whitespace_when_not_anchored',
  },
  {
    name: 'patch hunk count for removals',
    file: 'src/native/shadow/replay_patch.rs',
    from: 'if marker != b\'+\' {',
    to: 'if true {',
    failing: 'shadow::replay_patch::tests::a_removal_after_an_addition_still_counts_the_old_side',
  },
  {
    name: 'subsumption text verdict per path',
    file: 'src/native/shadow/replay_subsumption.rs',
    from: 'text.contains(&entry.path)',
    to: 'false',
    failing: 'shadow::replay_subsumption::tests::the_scenario_settles_each_path_by_its_rule',
  },
  {
    name: "the all flag is removed from the commit region",
    file: "src/native/commit/commit_args.rs",
    from: "if token == \"--all\" {",
    to: "if false {",
    failing: "commit::commit_args::tests::without_all_flag_removes_only_the_all_letter",
  },
  {
    name: "index commits without staging skip the staging step",
    file: "src/native/commit/commit_index.rs",
    from: "if !selection.stage_into_index {",
    to: "if false {",
    failing: "commit::commit_index::tests::index_without_staging_matches_the_real_index",
  },
  {
    name: "a branch base is written to the shadow branch",
    file: "src/native/shadow/shadow_refs.rs",
    from: "            if let Base::Commit(oid) = base {\n                let target: Vec<u8> = end_of_symbolic_chain(refs, reference.as_bytes())?;",
    to: "            if let Base::Commit(oid) = &Base::Unborn {\n                let target: Vec<u8> = end_of_symbolic_chain(refs, reference.as_bytes())?;",
    failing: "shadow::shadow_refs::tests::files_shadow_snapshots_refs_and_points_head_at_the_branch",
  },
  {
    name: "a linked worktree keeps its info directory private",
    file: "src/native/shadow/shadow_links.rs",
    from: "if !mixed_info {",
    to: "if false {",
    failing: "shadow::shadow_links::tests::shared_entries_link_and_private_entries_stay_absent",
  },
  {
    name: "the sequencer is copied into the shadow",
    file: "src/native/commit/conclusion_copy.rs",
    from: "if recovery_step(path_present(sequencer.as_path()))? {",
    to: "if false && recovery_step(path_present(sequencer.as_path()))? {",
    failing: "commit::conclusion_copy::tests::sequencer_is_copied_recursively",
  },
  {
    name: "the repository extensions are copied into the shadow config",
    file: "src/native/shadow/shadow_config.rs",
    from: "if !format.extensions.is_empty() {",
    to: "if false {",
    failing: "shadow::shadow_config::tests::config_copies_extensions_before_the_include",
  },
  {
    name: "amend wins over the conclusion markers",
    file: "src/native/commit/capture_invocation.rs",
    from: "if amend {",
    to: "if false {",
    failing: "commit::capture_invocation::tests::conclusion_markers_decide_the_kind",
  },
  {
    name: "a failed interactive selection is reported",
    file: "src/native/commit/commit_index_selection.rs",
    from: "    if !status.success() {",
    to: "    if false {",
    failing: "commit::commit_index::tests::interactive_selection_reports_a_failed_git_add",
  },
  {
    name: "a missing shared info directory is tolerated",
    file: "src/native/shadow/shadow_links.rs",
    from: "        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}",
    to: "        Err(error) if false => {}",
    failing: "shadow::shadow_links::tests::linked_worktree_without_a_shared_info_directory_still_links",
  },
  {
    name: 'signal relay sender check',
    file: 'src/native/process/signal_relay.rs',
    from: 'if !is_process_sender(code) {',
    to: 'if false {',
    failing: 'process::signal_relay::tests::handler_marks_a_signal_only_for_a_process_sender',
  },
  {
    name: 'signal relay in the handler',
    file: 'src/native/process/signal_relay.rs',
    from: 'PENDING[slot].store(signal, std::sync::atomic::Ordering::SeqCst);',
    to: 'let _ = signal;',
    failing: 'process::signal_relay::tests::process_signal_is_relayed_and_the_child_outcome_is_reported',
  },
  {
    name: 'automatic maintenance detach retry',
    file: 'src/native/landing/maintenance_run.rs',
    from: 'if result_code != USAGE_EXIT_CODE {',
    to: 'if true {',
    failing: 'landing::maintenance_run::tests::rejected_detach_retries_without_the_flag',
  },
  {
    name: 'gc.auto positive threshold',
    file: 'src/native/landing/maintenance_decision.rs',
    from: 'Some(value) => javascript_number(value.as_str()) > 0.0,',
    to: 'Some(value) => javascript_number(value.as_str()) != 0.0,',
    failing: 'landing::maintenance_decision::tests::non_positive_gc_auto_disables',
  },
  {
    name: 'gc.autodetach fallback',
    file: 'src/native/landing/maintenance_decision.rs',
    from: 'else { config.get("gc.autodetach") };',
    to: 'else { None };',
    failing: 'landing::maintenance_decision::tests::gc_autodetach_false_runs_in_the_foreground',
  },
  {
    name: 'manual-push confirmation refuses a remote ref changed after negotiation',
    file: 'src/native/push/manual_push_probe.rs',
    from: 'RemoteBefore::Known(reported) => authoritative.as_ref() != Some(reported),',
    to: 'RemoteBefore::Known(_reported) => false,',
    failing: 'push::manual_push_probe::tests::confirmation_refuses_a_remote_ref_that_changed_after_the_report',
  },
  {
    name: 'post-commit gate blocks the backup on a blocking event',
    file: 'src/native/commit/post_commit.rs',
    from: 'let blocked = pass_exit_code(events.as_slice(), end) != 0;',
    to: 'let blocked = false;',
    failing: 'commit::post_commit::tests::a_blocked_landed_commit_ends_with_an_explicit_commit_landed_event',
  },
  {
    name: 'manual-push scope reads the final tree when no base is published',
    file: 'src/native/push/manual_push_range.rs',
    from: '    if published.is_empty() {',
    to: '    if published.is_empty() && false {',
    failing: 'push::manual_push_range::tests::an_unknown_base_scans_the_final_tree',
  },
];
