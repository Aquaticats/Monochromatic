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
    name: 'self-exclusion by identity and content',
    file: 'src/native/invocation/real_git_candidate.rs',
    from: 'if same_file(candidate, own_executable) || identical_content(candidate, own_executable) {',
    to: 'if false {',
    failing: 'resolution::wrapper_never_selects_itself_or_a_copy_of_itself',
  },
  {
    name: 'forward-target marker check',
    file: 'src/native/invocation/entry.rs',
    from: '&& same_file(Path::new(&target), inputs.own_executable.as_path())',
    to: '&& false',
    failing: 'invocation::entry::stop_tests::forward_target_naming_this_executable_stops',
  },
  {
    name: 'dry-run commit classification',
    file: 'src/native/invocation/wrapped_command.rs',
    from: 'if region.dry_run {',
    to: 'if false {',
    failing: 'commit::a_dry_run_commits_nothing',
  },
  {
    name: 'unknown top-level key rejection',
    file: 'src/native/config/config_parse.rs',
    from: 'return Err(unknown_top_level_key(key.as_str()));',
    to: 'continue;',
    failing: 'config::config_parse::tests::unknown_and_retired_top_level_keys_are_named',
  },
  {
    name: 'legacy configuration migration diagnostic',
    file: 'src/native/config/config_file.rs',
    from: 'return Err(migration_required(first.as_path(), source.as_path()));',
    to: 'let _ = first;',
    failing: 'config::config_file::tests::legacy_configuration_alone_requires_migration',
  },
  {
    name: 'git version minimum',
    file: 'src/native/invocation/git_version.rs',
    from: 'if version < MINIMUM_GIT_VERSION {',
    to: 'if false {',
    failing: 'invocation::git_version::tests::an_older_release_is_refused',
  },
  {
    name: 'PID file proof skips the holder scan',
    file: 'src/native/lock/index_lock_evidence.rs',
    from: 'if let PidFile::Owner(pid, PidOwner::StartedBeforeLock(_)) = &pid_file {',
    to: 'if let PidFile::Owner(pid, PidOwner::Missing) = &pid_file {',
    failing: 'lock::index_lock_evidence::tests::gathering_skips_the_scan_only_for_a_proven_live_owner',
  },
  {
    name: 'open descriptor precedence over the PID file',
    file: 'src/native/lock/index_lock_evidence.rs',
    from: 'if let Some(scan) = &evidence.holders',
    to: 'if let Some(scan) = &None::<HolderEvidence>',
    failing: 'lock::index_lock_evidence::tests::classification_prefers_a_descriptor_then_the_pid_file',
  },
  {
    name: 'safe-integer bound on process start ticks',
    file: 'src/native/process/process_start.rs',
    from: 'if ticks > crate::json_record::MAX_SAFE_INTEGER {',
    to: 'if ticks > i64::MAX {',
    failing: 'process::process_start::tests::a_start_tick_beyond_the_safe_integers_is_refused',
  },
  {
    name: 'the hooks-off start switches the event hooks on',
    file: 'src/native/invocation/hooks_off.rs',
    from: '        result.push(OsString::from(format!("hook.{event}.enabled=false")));',
    to: '        result.push(OsString::from(format!("hook.{event}.enabled=true")));',
    failing: 'invocation::hooks_off::tests::the_options_follow_the_global_options_of_the_caller',
  },
  {
    name: 'the shadow config leaves a hook event on',
    file: 'src/native/shadow/shadow_config.rs',
    from: String.raw`        lines.push(String::from("\tenabled = false"));`,
    to: String.raw`        lines.push(String::from("\tenabled = true"));`,
    failing: 'shadow::shadow_config::tests::the_shadow_config_switches_every_hook_event_off_after_the_include',
  },
  {
    name: 'the native commit names a hooks directory that can exist',
    file: 'src/native/commit/commit_preparation.rs',
    from: '    arguments.push(format!("core.hooksPath={NO_HOOKS_DIRECTORY}"));',
    to: '    arguments.push(String::from("core.hooksPath=.git/hooks"));',
    failing: 'commit::commit_preparation::tests::native_commit_arguments_name_the_shadow_and_turn_traditional_hooks_off',
  },
  {
    name: 'a hook switched off by name is reported',
    file: 'src/native/health/external_hooks.rs',
    from: 'Lookup::Found(false)',
    to: 'Lookup::Found(true)',
    failing: 'health::external_hooks::tests::a_hook_switched_off_by_name_is_not_reported',
  },

];
