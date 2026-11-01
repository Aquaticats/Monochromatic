/**
 Planted guard removals in the entry point, configuration, index locks and hook runs.

 Each entry removes one guard and names the test that must fail without it; `from` must occur exactly once in the
 current source, so a control cannot silently stop applying. See `native-planted-controls.ts` for how they run.
 */

import type { PlantedControl } from './native-planted-controls.ts';

/**
 Controls for guards in the entry point, configuration, index locks and hook runs.
 */
export const plantedEntryControls: readonly PlantedControl[] = [
  {
    name: "post-commit disabled event runs nothing",
    file: "src/native/post_commit_hook.rs",
    from: '    if event_disabled(request.plan, "post-commit") {',
    to: '    if false && event_disabled(request.plan, "post-commit") {',
    failing: "post_commit_hook::tests::disabled_post_commit_runs_nothing",
  },
  {
    name: "post-commit runs under the hook lock",
    file: "src/native/post_commit_hook.rs",
    from: "    let lock: Option<OwnerLock> = if request.plan.skip_hook_lock {",
    to: "    let lock: Option<OwnerLock> = if true {",
    failing: "post_commit_hook::tests::hook_runs_under_the_lock_with_the_native_environment",
  },
  {
    name: "a failed copy stops post-landing before disposal",
    file: "src/native/post_landing.rs",
    from: "    .map_err(PostLandingError::Install)?;",
    to: "    .map_err(PostLandingError::Install).ok().unwrap_or_default();",
    failing: "post_landing::tests::an_install_failure_keeps_the_shadow_and_the_directory",
  },
  {
    name: "a failed shadow removal stops disposal",
    file: "src/native/landed_disposal.rs",
    from: "    remove_shadow_repository(paths.shadow)?;",
    to: "    let _ = remove_shadow_repository(paths.shadow);",
    failing: "landed_disposal::tests::a_failed_shadow_removal_keeps_the_directory_and_the_records",
  },
  {
    name: 'self-exclusion by identity and content',
    file: 'src/native/real_git_candidate.rs',
    from: 'if same_file(candidate, own_executable) || identical_content(candidate, own_executable) {',
    to: 'if false {',
    failing: 'resolution::wrapper_never_selects_itself_or_a_copy_of_itself',
  },
  {
    name: 'forward-target marker check',
    file: 'src/native/entry.rs',
    from: '&& same_file(Path::new(&target), inputs.own_executable.as_path())',
    to: '&& false',
    failing: 'resolution::different_wrapper_build_stops_instead_of_looping',
  },
  {
    name: 'dry-run commit classification',
    file: 'src/native/wrapped_command.rs',
    from: 'if region.dry_run {',
    to: 'if false {',
    failing: 'commit::a_dry_run_commits_nothing',
  },
  {
    name: 'unknown top-level key rejection',
    file: 'src/native/config_parse.rs',
    from: 'return Err(unknown_top_level_key(key.as_str()));',
    to: 'continue;',
    failing: 'config_parse::tests::unknown_and_retired_top_level_keys_are_named',
  },
  {
    name: 'legacy configuration migration diagnostic',
    file: 'src/native/config_file.rs',
    from: 'return Err(migration_required(first.as_path(), source.as_path()));',
    to: 'let _ = first;',
    failing: 'config_file::tests::legacy_configuration_alone_requires_migration',
  },
  {
    name: 'git version minimum',
    file: 'src/native/git_version.rs',
    from: 'if version < MINIMUM_GIT_VERSION {',
    to: 'if false {',
    failing: 'git_version::tests::an_older_release_is_refused',
  },
  {
    name: 'PID file proof skips the holder scan',
    file: 'src/native/index_lock_evidence.rs',
    from: 'if let PidFile::Owner(pid, PidOwner::StartedBeforeLock(_)) = &pid_file {',
    to: 'if let PidFile::Owner(pid, PidOwner::Missing) = &pid_file {',
    failing: 'index_lock_evidence::tests::gathering_skips_the_scan_only_for_a_proven_live_owner',
  },
  {
    name: 'open descriptor precedence over the PID file',
    file: 'src/native/index_lock_evidence.rs',
    from: 'if let Some(scan) = &evidence.holders',
    to: 'if let Some(scan) = &None::<HolderEvidence>',
    failing: 'index_lock_evidence::tests::classification_prefers_a_descriptor_then_the_pid_file',
  },
  {
    name: 'coordinated Git does not inherit the landing lease',
    file: 'src/native/index_writer_forward.rs',
    from: 'child_overlay.push(coordination.lease.clone());',
    to: 'let _ = coordination.lease.clone();',
    failing: 'coordination::a_nested_git_add_inside_rebase_does_not_deadlock',
  },
  {
    name: 'a proven holder is polled without the poll cap',
    file: 'src/native/index_lock_wait.rs',
    from: 'effects.sleep(backoff_ms.min(cap));',
    to: 'effects.sleep(backoff_ms);',
    failing: 'index_lock_wait::tests::a_proven_pid_file_owner_is_waited_for_beyond_the_unproven_budget',
  },
  {
    name: 'evidence gathering is not counted toward the unproven budget',
    file: 'src/native/index_lock_wait.rs',
    from: 'state.unproven_ms += effects.now_ms() - iteration_started;',
    to: 'state.unproven_ms += 0.0;',
    failing: 'index_lock_wait::tests::gathering_evidence_counts_toward_the_budget',
  },
  {
    name: 'the Git version is not checked before an index writer locks',
    file: 'src/native/index_writer_coordination.rs',
    from: 'if let Err(reason) = version(input.context) {',
    to: 'if let Err(reason) = Ok::<crate::git_version::GitRelease, String>((2, 55, 0)) {',
    failing: 'index_writer_coordination::tests::an_older_git_refuses_an_index_writer_before_the_landing_lock',
  },
  {
    name: 'a replaced real index lock is released',
    file: 'src/native/real_index_lock.rs',
    from: '        self.assert_identity()?;\n        if let Err(error) = std::fs::remove_file(&self.lock_path) {',
    to: '        if let Err(error) = std::fs::remove_file(&self.lock_path) {',
    failing: 'real_index_lock::tests::a_replaced_lock_is_never_released',
  },
  {
    name: 'safe-integer bound on process start ticks',
    file: 'src/native/process_start.rs',
    from: 'if ticks > super::json_record::MAX_SAFE_INTEGER {',
    to: 'if ticks > i64::MAX {',
    failing: 'process_start::tests::a_start_tick_beyond_the_safe_integers_is_refused',
  },
  {
    name: 'native commit holds no hook lock',
    file: 'src/native/commit_preparation.rs',
    from: '    let _hook_lock = hold_hook_lock(request.hooks, &PREPARATION_HOOK_EVENTS)',
    to: '    let _hook_lock = hold_hook_lock(request.hooks, &[])',
    failing: 'commit_preparation::tests::a_configured_hook_runs_inside_the_preparation_under_the_lock_and_the_lease',
  },
  {
    name: 'native commit does not carry the preparation lease',
    file: 'src/native/commit_preparation.rs',
    from: '        .env(lease_name, lease_value);\n    for name in NATIVE_UNSET_VARIABLES {',
    to: '        .env(lease_name, "");\n    for name in NATIVE_UNSET_VARIABLES {',
    failing: 'commit_preparation::tests::a_configured_hook_runs_inside_the_preparation_under_the_lock_and_the_lease',
  },
  {
    name: 'native commit leaves post-commit to Git',
    file: 'src/native/commit_preparation.rs',
    from: '    arguments.push(String::from("hook.post-commit.enabled=false"));',
    to: '    arguments.push(String::from("hook.post-commit.enabled=true"));',
    failing: 'commit_preparation::tests::native_commit_arguments_name_the_shadow_and_turn_the_unrun_hooks_off',
  },
  {
    name: 'pre-commit re-run ignores the hook lock',
    file: 'src/native/commit_revalidation.rs',
    from: '    let _hook_lock = hold_hook_lock(hooks, &["pre-commit"])',
    to: '    let _hook_lock = hold_hook_lock(hooks, &[])',
    failing: 'commit_revalidation::tests::a_passing_hook_sees_the_replayed_index_no_editor_the_lease_and_the_lock',
  },
  {
    name: 'hook lock skip flag ignored',
    file: 'src/native/hook_run.rs',
    from: '    if plan.skip_hook_lock || !events.iter()',
    to: '    if !events.iter()',
    failing: 'hook_run::tests::the_lock_is_held_only_when_a_run_can_find_a_hook',
  },
  {
    name: 'concurrent commits do not skip the hook lock',
    file: 'src/native/hook_dispatch_plan.rs',
    from: 'skip_hook_lock: inputs.concurrent_commits || inputs.inherited_lease_valid,',
    to: 'skip_hook_lock: inputs.inherited_lease_valid,',
    failing: 'hook_dispatch_plan::tests::plans_read_real_git_configuration',
  },
  {
    name: 'existing hooks directory reused',
    file: 'src/native/hook_plan_record.rs',
    from: '    create_private_directory(hooks_directory).map_err(|error| {',
    to: '    std::fs::create_dir_all(hooks_directory).map_err(|error| {',
    failing: 'hook_plan_record::tests::an_existing_hooks_directory_is_refused',
  },

];
