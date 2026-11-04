/**
 Planted guard removals in worktree copy and the post-commit gate.

 Each entry removes one guard and names the test that must fail without it; `from` must occur exactly once in the
 current source, so a control cannot silently stop applying. See `native-planted-controls.ts` for how they run.
 */

import type { PlantedControl } from './native-planted-controls.ts';

/**
 Controls for guards in worktree copy and the post-commit gate.
 */
export const plantedLandingControls: readonly PlantedControl[] = [
  {
    name: 'worktree copy recovery leaves a live owner\'s journal',
    file: 'src/native/worktree_copying/worktree_copy_recovery.rs',
    from: '        if !journal_owner_is_dead(&journal) {',
    to: '        if false && !journal_owner_is_dead(&journal) {',
    failing: 'worktree_copying::worktree_copy_recovery::tests::pending_check_leaves_a_live_journal_and_recovers_a_dead_one',
  },
  {
    name: 'worktree copy preflight refuses differing destination entries',
    file: 'src/native/worktree_copying/worktree_copy_install.rs',
    from: 'if !entry_matches(stage_root, destination_root, entry)? {',
    to: 'if false {',
    failing: 'worktree_copying::worktree_copy_install::tests::preflight_refuses_differing_entries_without_overwriting',
  },
  {
    name: 'worktree copy rollback content proof',
    file: 'src/native/worktree_copying/worktree_copy_rollback.rs',
    from: '            !entry_matches(stage_root, destination_root, entry)?',
    to: '            false',
    failing: 'worktree_copying::worktree_copy_rollback::tests::edited_file_is_retained_by_path',
  },
  {
    name: 'worktree copy journal private-mode refusal',
    file: 'src/native/worktree_copying/worktree_copy_journal.rs',
    from: '|| (metadata.mode() & NON_PRIVATE_MODE_BITS) != 0',
    to: '|| false',
    failing: 'worktree_copying::worktree_copy_journal::tests::unsafe_journal_mode_fails_closed_and_is_kept',
  },
  {
    name: 'worktree copy install log unfinished-append truncation',
    file: 'src/native/worktree_copying/worktree_copy_install_log.rs',
    from: 'if complete_length < total_length {',
    to: 'if false {',
    failing: 'worktree_copying::worktree_copy_install_log::tests::unfinished_trailing_append_is_truncated',
  },
  {
    name: 'worktree copy settlement exit status',
    file: 'src/native/worktree_copying/worktree_copy.rs',
    from: '(true, true) => return 2,',
    to: '(true, true) => return 0,',
    failing: 'worktree_copying::worktree_copy::tests::exit_status_follows_the_settlement_table',
  },
  {
    name: 'worktree copy installation batches',
    file: 'src/native/worktree_copying/worktree_copy_install.rs',
    from: 'entries.chunks(INSTALL_BATCH_ENTRIES)',
    to: 'entries.chunks(entries.len().max(1))',
    failing: 'worktree_copying::worktree_copy_install::tests::records_grow_by_batch_not_by_entry',
  },
  {
    name: 'post-commit stage ignores the forbidden-strings rules variable',
    file: 'src/native/commit/post_commit.rs',
    from: '    checks.scanner_settings.rules_variable = environment_value(request.environment, RULES_VARIABLE);',
    to: '    checks.scanner_settings.rules_variable = None;',
    failing: 'commit::post_commit::tests::the_rules_variable_reaches_the_post_commit_scanner',
  },
  {
    name: 'post-commit stage ignores the configured forbidden-strings options',
    file: 'src/native/commit/post_commit.rs',
    from: '    checks.scanner_settings.options = request.config.forbidden_strings.clone();',
    to: '    checks.scanner_settings.options = PolicyConfig::defaults().forbidden_strings;',
    failing: 'commit::post_commit::tests::the_configured_rules_file_reaches_the_post_commit_scanner',
  },
  {
    name: 'post-commit stage runs the Markdown policy with no rule selected',
    file: 'src/native/commit/post_commit.rs',
    from: '    checks.markdown.options = request.config.markdown_autofix.clone();',
    to: '    checks.markdown.options = request.config.markdown_autofix.clone();\n    checks.markdown.options.rules.clear();',
    failing: 'commit::post_commit::tests::a_landed_markdown_file_is_checked_in_process',
  },
  {
    name: 'post-commit stage ignores the configured Markdown options',
    file: 'src/native/commit/post_commit.rs',
    from: '    checks.markdown.options = request.config.markdown_autofix.clone();',
    to: '    checks.markdown.options = PolicyConfig::defaults().markdown_autofix;',
    failing: 'commit::post_commit::tests::the_markdown_options_reach_the_post_commit_policy',
  },
  {
    name: 'manual-push probe Git children drop the invocation environment',
    file: 'src/native/invocation/wrapped_command.rs',
    from: 'environment: environment_with_overlay(environment, context.overlay.as_slice()),',
    to: 'environment: environment_with_overlay(&environment[..0], context.overlay.as_slice()),',
    failing: 'push::push_environment::a_manual_push_probes_and_pushes_with_the_invocation_environment',
  },
  {
    name: 'an event path keeps the exact bytes of a name that is not UTF-8',
    file: 'src/native/policy/event_path.rs',
    from: 'exact: Some(bytes.to_vec()),',
    to: 'exact: None,',
    failing: 'policy::event_path::tests::a_name_that_is_not_utf8_keeps_its_bytes',
  },
  {
    name: 'the candidate pathspec of a name that is not UTF-8 keeps its exact bytes',
    file: 'src/native/commit/commit_transaction.rs',
    from: '                return os_from_path_bytes(pathspec.as_slice());',
    to: '                return OsString::from(String::from_utf8_lossy(pathspec.as_slice()).into_owned());',
    failing: 'commit::commit_transaction::tests::the_candidate_request_keeps_the_bytes_of_a_name_that_is_not_utf8',
  },
  {
    name: 'a correction of a name that is not UTF-8 is staged under its exact bytes',
    file: 'src/native/commit/private_index_stage.rs',
    from: '    request.arguments.push(os_from_path_bytes(path));',
    to: '    request.arguments.push(std::ffi::OsString::from(String::from_utf8_lossy(path).into_owned()));',
    failing: 'policy::event_path_commit::a_commit_correction_of_a_name_that_is_not_utf8_reports_the_exact_bytes',
  },
];
