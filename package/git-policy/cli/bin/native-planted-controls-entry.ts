/**
 Planted guard removals in the entry point, configuration, index locks, hooks-off starts and configured-hook reports.

 Each entry removes one guard and names the test that must fail without it; `from` must occur exactly once in the
 current source, so a control cannot silently stop applying. See `native-planted-controls.ts` for how they run.
 */

import type { PlantedControl } from './native-planted-controls.ts';

/**
 Controls for guards in the entry point, configuration, index locks, hooks-off starts and configured-hook reports.
 */
export const plantedEntryControls: readonly PlantedControl[] = [
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
    name: 'the hooks-off start switches the event hooks on',
    file: 'src/native/hooks_off.rs',
    from: '        command.arg("-c").arg(format!("hook.{event}.enabled=false"));',
    to: '        command.arg("-c").arg(format!("hook.{event}.enabled=true"));',
    failing: 'hooks_off::tests::the_option_precedes_every_argument_of_the_caller',
  },
  {
    name: 'the shadow config leaves a hook event on',
    file: 'src/native/shadow_config.rs',
    from: '        lines.push(String::from("\\tenabled = false"));',
    to: '        lines.push(String::from("\\tenabled = true"));',
    failing: 'shadow_config::tests::the_shadow_config_switches_every_hook_event_off_after_the_include',
  },
  {
    name: 'the native commit names a hooks directory that can exist',
    file: 'src/native/commit_preparation.rs',
    from: '    arguments.push(format!("core.hooksPath={NO_HOOKS_DIRECTORY}"));',
    to: '    arguments.push(String::from("core.hooksPath=.git/hooks"));',
    failing: 'commit_preparation::tests::native_commit_arguments_name_the_shadow_and_turn_traditional_hooks_off',
  },
  {
    name: 'a hook switched off by name is reported',
    file: 'src/native/external_hooks.rs',
    from: 'Lookup::Found(false)',
    to: 'Lookup::Found(true)',
    failing: 'external_hooks::tests::a_hook_switched_off_by_name_is_not_reported',
  },

];
