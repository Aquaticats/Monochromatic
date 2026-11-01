#!/usr/bin/env node
// Observe committed Language regressions failing after guard removal in a disposable package copy.
// Guards: stale-result fencing (file, revision, server process), the readiness gate in front of
// every helix-lsp call, the refusal of server-initiated workspace edits, asking again for
// hints and pull diagnostics a server left unanswered, hint windows, the diagnostics hold, and
// the start allowance with its request bound, the TypeScript lookup, and the entry that says why features are missing.
import { spawnSync } from 'node:child_process';
import {
  cpSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  statSync,
  writeFileSync
} from 'node:fs';
import { homedir } from 'node:os';
import {
  join,
  relative,
  sep
} from 'node:path';
import { containerBounds } from './container-bounds.ts';

// The bound of every container this script starts: the shared computed share of the host, or the caller's IDE_CONTAINER_* values.
const containerLimits = containerBounds();

const privateRoot = join(
  homedir(),
  'temp',
  'agent'
);
if ((statSync(privateRoot)
  .mode
  & 0o077) !== 0) throw new Error(`Guard scratch root must exclude group and other permissions: ${  privateRoot}`);
if (!process.env
  .usage_cache) throw new Error('Provide the disposable Cargo target-cache directory');
const cache = realpathSync(process.env
  .usage_cache);
if ((!cache.startsWith(realpathSync(privateRoot) + sep)) || (!statSync(cache)
  .isDirectory())) throw new Error(`Target cache must be a disposable directory below ${  privateRoot}`);
// A private Cargo home copy keeps disposable builds off the shared volume's package-cache lock.
let cargoHome = 'ide-cargo';
if (process.env
  .usage_cargo) {
  cargoHome = realpathSync(process.env
    .usage_cargo);
  if ((!cargoHome.startsWith(realpathSync(privateRoot) + sep)) || (!statSync(cargoHome)
    .isDirectory())) throw new Error(`Cargo home copy must be a disposable directory below ${  privateRoot}`);
}
const artifact = mkdtempSync(join(
  privateRoot,
  'ide-language-guard-'
));
const source = join(
  artifact,
  'package'
);
const origin = process.cwd();
cpSync(
  origin,
  source,
  {
    recursive: true,
    filter: path => ![
      'target',
      '.git',
      'node_modules'
    ].includes(relative(
      origin,
      path
    )
      .split(sep)[0] ?? '')
  }
);
mkdirSync(
  join(
    source,
    'target'
  ),
  { recursive: true }
);
console.log(`LANGUAGE_GUARD_ARTIFACT=${  artifact}`);

/**
 A guard: the edit that removes it and the test that must then fail.
 */
interface Guard {
  name: string;
  file: string;
  before: string;
  after: string;
  integration?: string;
  test: string;
  failure: string;
}

/**
 A container run recorded in `results.json` and echoed as one JSON line.
 */
interface RunResult {
  name: string;
  phase: string;
  test: string;
  status: number | null;
  accepted: boolean;
}

/**
 `text` with the one occurrence of `before` replaced by `after`.

 @param text - source of the file to mutate

 @param before - anchor that the guard exists to protect; absence means the guard moved with its code

 @param after - replacement that removes the guard

 @returns the edited text

 @throws {Error} when `before` is absent or occurs more than once, since either way the guard may be aimed at the wrong code
 */
function replaceOne({
  text,
  before,
  after
}: {
  text: string;
  before: string;
  after: string;
}): string {
  const start = text.indexOf(before);
  if (start === (-1)) throw new Error(`Mutation source anchor is absent: ${  before}`);
  if (text.indexOf(
    before,
    start + before.length
  ) >= 0) throw new Error(`Mutation source anchor is ambiguous: ${  before}`);
  return text.slice(
    0,
    start
  ) + after
    + text.slice(start + before.length);
}
const cases: Guard[] = [
  // Fencing, rule by rule, without a server.
  {
    name: 'fence-file',
    file: 'src/language/fence.rs',
    before: 'if displayed.file != stamp.file {',
    after: 'if false {',
    test: 'language::fence::tests::result_for_an_old_file_generation_is_dropped',
    failure: 'a result for an old file generation was accepted'
  },
  {
    name: 'fence-revision',
    file: 'src/language/fence.rs',
    before: 'if displayed.revision != stamp.revision {',
    after: 'if false {',
    test: 'language::fence::tests::result_for_an_old_revision_is_dropped',
    failure: 'a result for an old revision was accepted'
  },
  {
    name: 'fence-server',
    file: 'src/language/fence.rs',
    before: 'if known.name == answering.name && known.instance != answering.instance {',
    after: 'if false {',
    test: 'language::fence::tests::result_from_a_replaced_server_process_is_dropped',
    failure: 'a result from a replaced server process was accepted'
  },
  // The same fence through a real worker and the scripted server: a reload and a file switch overtake replies.
  {
    name: 'fence-revision-worker',
    file: 'src/language/fence.rs',
    before: 'if displayed.revision != stamp.revision {',
    after: 'if false {',
    integration: 'language',
    test: 'requests::reply_overtaken_by_a_reload_or_file_switch_is_dropped',
    failure: 'a reply for the previous revision was accepted'
  },
  {
    name: 'fence-file-worker',
    file: 'src/language/fence.rs',
    before: 'if displayed.file != stamp.file {',
    after: 'if false {',
    integration: 'language',
    test: 'requests::reply_overtaken_by_a_reload_or_file_switch_is_dropped',
    failure: 'a reply for the previous file was accepted'
  },
  {
    name: 'fence-superseded-worker',
    file: 'src/language/fence.rs',
    before: 'if displayed.revision != stamp.revision {',
    after: 'if false {',
    integration: 'language',
    test: 'requests::superseded_answer_for_reloaded_text_is_dropped_by_the_fence',
    failure: 'a superseded answer for reloaded text reached the interface'
  },
  // The readiness gate: without it helix-lsp panics on the worker thread, which the handle reports as a stopped module.
  {
    name: 'readiness-gate',
    file: 'src/language/session.rs',
    before: 'Some(client) if client.is_initialized() => Some(client),',
    after: 'Some(client) => Some(client),',
    integration: 'language',
    test: 'start::request_before_initialize_is_answered_starting_and_nothing_is_sent',
    failure: 'Language support stopped unexpectedly'
  },
  // Refusal of server-initiated edits, as a pure policy and through the scripted server's own request.
  {
    name: 'edit-refusal',
    file: 'src/language/incoming.rs',
    before: 'edit_refusal_reply(),',
    after: 'json!({ "applied": true }),',
    test: 'language::incoming::tests::workspace_edit_is_refused_with_a_normal_result',
    failure: 'a server-initiated workspace edit was not refused'
  },
  // Embedder workarounds for helix-lsp behavior recorded in doc/troubleshooting/helix-lsp-embedding-roots-and-stop.md.
  {
    name: 'root-spelling',
    file: 'src/language/root.rs',
    before: 'Ok(below) => self.helix.join(below),',
    after: 'Ok(_) => path.to_path_buf(),',
    integration: 'language',
    test: 'roots::project_reached_through_a_linked_working_directory_is_rooted_at_the_project',
    failure: 'the server was not rooted at the project reached through the linked working directory'
  },
  {
    name: 'stop-tombstone',
    file: 'src/language/attach.rs',
    before: 'worker.registry.remove_by_id(client.id());',
    after: 'worker.registry.stop(client.name());',
    integration: 'language',
    test: 'lifecycle::crash_fails_the_pending_request_and_the_next_open_restarts',
    failure: 'an exited server could not be started again'
  },
  {
    name: 'edit-refusal-worker',
    file: 'src/language/incoming.rs',
    before: 'edit_refusal_reply(),',
    after: 'json!({ "applied": true }),',
    integration: 'language',
    test: 'policy::server_requests_get_policy_replies_and_edits_change_nothing',
    failure: 'a server-initiated workspace edit was not refused'
  },
  // Asking again: a hint or pull-diagnostics request the server left unanswered for its whole timeout is not the last one.
  // The scripted server leaves exactly one request unanswered (IDE_SCRIPTED_HINT_STEPS, IDE_SCRIPTED_PULL_STEPS),
  // so the removed guard fails by the test's own wait, without any load.
  {
    name: 'hint-timeout-first',
    file: 'src/language/request/answer.rs',
    before: 'Failure::Failed(RequestFailure::Timeout) => Some(Duration::ZERO),',
    after: 'Failure::Failed(RequestFailure::Timeout) => None,',
    integration: 'language',
    test: 'again::hints_are_asked_again_when_the_first_request_times_out',
    failure: 'timed out waiting for hints for the first text after the first request timed out'
  },
  {
    name: 'hint-timeout-reload',
    file: 'src/language/request/answer.rs',
    before: 'Failure::Failed(RequestFailure::Timeout) => Some(Duration::ZERO),',
    after: 'Failure::Failed(RequestFailure::Timeout) => None,',
    integration: 'language',
    test: 'again::hints_are_asked_again_after_a_reload_when_the_request_times_out',
    failure: 'timed out waiting for hints for the reloaded text after the request timed out'
  },
  {
    name: 'pull-timeout-reload',
    file: 'src/language/request/answer.rs',
    before: '} else if matches!(error, helix_lsp::Error::Timeout(_)) {',
    after: '} else if false {',
    integration: 'language',
    test: 'again::pulled_diagnostics_are_asked_again_after_a_reload_when_the_request_times_out',
    failure: 'timed out waiting for pulled diagnostics for the reloaded text after the request timed out'
  },
  // Catching up: once every retry was superseded, the server's next message, or its next answer, makes the worker ask once more.
  {
    name: 'catch-up-notification',
    file: 'src/language/traffic.rs',
    before: 'request::catch_up(worker, index);',
    after: 'tracing::trace!(index, "catch-up removed");',
    integration: 'language',
    test: 'again::hints_are_asked_again_when_the_server_sends_a_notification_after_every_retry_was_superseded',
    failure: "timed out waiting for hints for the reloaded text after the server's notification followed the superseded retries"
  },
  {
    name: 'catch-up-answer',
    file: 'src/language/request/answer.rs',
    before: 'if answered && let Some(index) = worker.session.index_of_identity(&ticket.server) {',
    after: 'if false && let Some(index) = worker.session.index_of_identity(&ticket.server) {',
    integration: 'language',
    test: 'again::hints_are_asked_again_when_the_server_answers_another_request_after_every_retry_was_superseded',
    failure: "timed out waiting for hints for the reloaded text after the server's hover answer followed the superseded retries"
  },
  {
    name: 'catch-up-limit',
    file: 'src/language/owed.rs',
    before: 'if self.asked_again >= MAX_CATCH_UPS {',
    after: 'if false {',
    test: 'language::owed::tests::catch_up_asks_stop_at_the_limit_until_the_text_changes',
    failure: 'a server was asked again beyond the limit for one displayed text'
  },
  // A hint answer for lines the reader no longer shows is dropped; the scripted server answers the older window last.
  {
    name: 'hint-window',
    file: 'src/language/request/answer.rs',
    before: 'if hint_ask(&worker.session) != Some(ticket.ask) {',
    after: 'if false {',
    integration: 'language',
    test: 'windows::late_answer_for_an_earlier_window_does_not_replace_the_current_hints',
    failure: 'an answer for an earlier window replaced the hints of the current one'
  },
  // The diagnostics hold keeps the set pushed during it and shows it when it ends, unless a versioned set arrived first.
  {
    name: 'hold-shows-kept',
    file: 'src/language/diagnostics/store.rs',
    before: 'if let Some(kept) = hold.kept {',
    after: 'if let Some(kept) = hold.kept.filter(|_| return false) {',
    test: 'language::diagnostics::tests::set_pushed_during_the_hold_is_shown_when_the_server_answers',
    failure: 'the set pushed during the hold was dropped when the hold ended'
  },
  {
    name: 'hold-shows-kept-worker',
    file: 'src/language/diagnostics/store.rs',
    before: 'if let Some(kept) = hold.kept {',
    after: 'if let Some(kept) = hold.kept.filter(|_| return false) {',
    integration: 'language',
    test: 'requests::unversioned_set_pushed_during_the_hold_is_shown_when_the_hold_expires',
    failure: 'timed out waiting for the set pushed during the hold, shown when its fixed delay passed'
  },
  {
    name: 'hold-newer-wins',
    file: 'src/language/diagnostics/store.rs',
    before: 'hold.kept = None;',
    after: 'hold.kept = hold.kept.take();',
    test: 'language::diagnostics::tests::newer_set_replaces_the_one_kept_during_the_hold',
    failure: 'a set kept during the hold replaced a newer versioned set when the hold ended'
  },
  // Starting: helix-lsp gets three request timeouts for initialize, and the worker bounds every request by one.
  {
    name: 'start-allowance',
    file: 'src/language/config.rs',
    before: 'definition.timeout = definition.timeout.saturating_mul(START_FACTOR);',
    after: 'definition.timeout = definition.timeout.saturating_mul(1);',
    integration: 'language',
    test: 'start::initialize_answered_within_the_start_allowance_is_accepted',
    failure: 'a server that answered initialize within its start allowance was not accepted'
  },
  {
    name: 'request-bound',
    file: 'src/language/request.rs',
    before: 'return Payload::Hover(bounded(seconds, request).await);',
    after: 'return Payload::Hover(request.await);',
    integration: 'language',
    test: 'requests::request_keeps_the_request_timeout_while_the_start_allowance_is_longer',
    failure: 'a request waited longer than its request timeout'
  },
  // Finding the project's TypeScript 7 server from the displayed file's folder upward, and saying why hover and hints are missing.
  {
    name: 'typescript-document-folder',
    file: 'src/language/config/typescript.rs',
    before: '&& folder.starts_with(root)',
    after: '&& false',
    test: 'language::config::tests::typescript_is_found_for_a_project_folder_that_holds_no_install_of_its_own',
    failure: "the install below the project folder was not found from the document's folder"
  },
  {
    name: 'typescript-nearest-install',
    file: 'src/language/config/typescript.rs',
    before: 'return check_version(folder, launcher);',
    after: 'if let Ok(found) = check_version(folder, launcher) { return Ok(found); } continue;',
    test: 'language::config::tests::the_nearest_typescript_install_decides_and_an_older_one_is_not_skipped',
    failure: "the package's TypeScript 5 was skipped for the root's TypeScript 7"
  },
  {
    name: 'typescript-launcher-moved',
    file: 'src/language/config.rs',
    before: 'if found != definition.resolved {',
    after: 'if found.is_some() != definition.resolved.is_some() {',
    test: 'language::config::tests::the_registry_follows_the_launcher_to_another_install_of_version_seven',
    failure: 'the registry kept the launcher of the other install'
  },
  {
    name: 'notice-published',
    file: 'src/language/worker.rs',
    before: 'notice::with_notice(stored, document.stamp, &status, &troubles, &advisory)',
    after: 'stored',
    integration: 'language',
    test: 'notice::a_missing_program_gives_the_entry_with_its_remedy',
    failure: 'the entry for a missing program'
  },
  {
    name: 'notice-waits-for-start',
    file: 'src/language/notice.rs',
    before: 'if starting {',
    after: 'if false {',
    test: 'language::notice::tests::a_server_that_is_still_starting_gives_no_entry',
    failure: 'an entry was shown while the server was still starting'
  },
  {
    name: 'hint-failure-kept',
    file: 'src/language/request/answer/trouble.rs',
    before: '*slot = failure;',
    after: '*slot = None;',
    integration: 'language',
    test: 'notice::a_failed_hint_request_is_named_and_a_later_answer_clears_it',
    failure: 'the entry for a failed hint request'
  },
  // A failed start states what the server wrote to standard error, and handing the lines over leaves the log record whole.
  {
    name: 'cause-hand-over-copies',
    file: 'src/language/attach/report.rs',
    before: 'then.after_end(stderr_tail::lines(&record.server));',
    after: 'then.after_end(stderr_tail::take(&record.server));',
    test: 'language::attach::report::tests::the_lines_are_handed_over_complete_and_the_record_still_reports_them',
    failure: 'handing the lines over drained them from the log record'
  },
  {
    name: 'cause-reaches-reason',
    file: 'src/language/attach.rs',
    before: 'record.state = ServerState::FailedToStart { reason: updated };',
    after: 'tracing::trace!(%updated, "cause removed");',
    integration: 'language',
    test: 'quiet::a_server_that_ends_during_its_start_is_logged_with_its_stderr_lines',
    failure: 'the cause in the reason'
  },
  // A scratch directory is emptied in place: a sandbox binds it by path while another launch of the same project prepares it.
  {
    name: 'scratch-kept-in-place',
    file: 'src/language/launch/state.rs',
    before: '        empty_in_place(directory)?;',
    after: '        if directory.exists() { let _ = std::fs::remove_dir_all(directory); }',
    test: 'language::launch::tests::preparation_empties_a_scratch_directory_without_replacing_it',
    failure: 'the scratch directory was removed'
  },
  // Each process has a scratch directory of its own: a second window on the project must not empty the first one's.
  {
    name: 'scratch-per-process',
    file: 'src/language/launch/scratch.rs',
    before: '        return format!("{PREFIX}{}-{nanoseconds:x}", std::process::id());',
    after: '        return "tmp".to_string();',
    test: 'language::launch::scratch::tests::a_second_process_does_not_empty_the_scratch_directory_of_a_running_first_one',
    failure: 'two processes were given one scratch directory'
  },
  // The lock a process holds for its life is what keeps another process from removing its directory as an ended one's.
  {
    name: 'scratch-lock-held',
    file: 'src/language/launch/scratch.rs',
    before: '    held.push((directory.to_path_buf(), file));',
    after: '    drop(file);',
    test: 'language::launch::scratch::tests::the_scratch_directory_of_an_ended_process_is_removed_by_the_next_preparation',
    failure: "a running process's directory was removed as if its owner had ended"
  },
  // The directory of a process that ended (or was killed) is removed by the next preparation.
  {
    name: 'scratch-ended-removed',
    file: 'src/language/launch/state.rs',
    before: '        reap_ended(directory);',
    after: '        let _ = directory;',
    test: 'language::launch::scratch::tests::the_scratch_directory_of_an_ended_process_is_removed_by_the_next_preparation',
    failure: 'the directory of an ended process stayed'
  },
  // A sandbox that stopped before the lookup ran is not blamed on the project's toolchain.
  {
    name: 'sandbox-stop-named',
    file: 'src/language/toolchain/state.rs',
    before: '        if failed.stopped_by_sandbox() {',
    after: '        if false {',
    test: 'language::toolchain::state::tests::a_sandbox_that_stopped_before_the_lookup_is_not_blamed_on_the_toolchain',
    failure: 'the notice blamed the toolchain for a sandbox that stopped'
  },
];
// An optional comma-separated list reruns only the named guards.
const only = process.env
  .usage_only ? new Set(process.env
    .usage_only
    .split(',')) : undefined;
const selected = only ? cases.filter(item => only.has(item.name)) : cases;
if (only && (selected.length !== only.size)) throw new Error(`Unknown guard name in: ${
   process.env
     .usage_only}`);
const results: RunResult[] = [];
const baselined = new Set();

/**
 Runs one guard test in the disposable copy, records the result, and stops the script when it is not the expected one.

 @param item - guard whose test runs

 @param phase - baseline (unmodified), removed (guard removed, must fail), or restored (must pass again)

 @throws {Error} when the run is not the expected result
 */
function run({
  item,
  phase
}: {
  item: Guard;
  phase: string;
}): void {
  const command = [
    'cargo',
    'test',
    '--offline',
    '--no-default-features',
    ...(item.integration ? [
      '--test',
      item.integration
    ] : ['--lib']),
    item.test,
    '--',
    '--exact',
    '--nocapture'
  ];
  const result = spawnSync(
    'podman',
    [
      'run',
      '--rm',
      '--network=none',
      ...containerLimits.limitArguments,
      '--security-opt',
      'label=disable',
      '--volume',
      `${source  }:/work`,
      '--volume',
      `${cache  }:/work/target`,
      '--volume',
      `${cargoHome  }:/cargo`,
      '--workdir',
      '/work',
      ...containerLimits.environmentArguments,
      '--env',
      'HELIX_RUNTIME=/work/target/debug/runtime',
      'localhost/monochromatic/ide',
      ...command,
    ],
    {
      encoding: 'utf8',
      maxBuffer: 64 * 1_024
        * 1_024
    }
  );
  const output = (result.stdout ?? '') + (result.stderr ?? '');
  writeFileSync(
    join(
      artifact,
      `${item.name  }-${
         phase
         }.log`
    ),
    output
  );
  if (result.error) throw result.error;
  const failedAsExpected = (result.status !== 0) && output.includes(item.test)
    && output.includes(item.failure);
  const passedAsExpected = (result.status === 0) && output.includes(item.test)
    && output.includes('test result: ok. 1 passed');
  const accepted = phase === 'removed' ? failedAsExpected : passedAsExpected;
  results.push({
    name: item.name,
    phase,
    test: item.test,
    status: result.status,
    accepted
  });
  writeFileSync(
    join(
      artifact,
      'results.json'
    ),
    JSON.stringify(
      results,
      null,
      2
    )
  );
  if (!accepted) throw new Error(`Unexpected ${  phase  } result for ${  item.name  }; inspect ${  artifact}`);
  console.log(
    JSON.stringify(results.at(-1)),
  );
}
for (const item of selected) {
  // A restored run of an earlier case is already this test's unmodified baseline.
  const key = `${item.integration ?? 'lib'  }:${  item.test}`;
  if (!baselined.has(key)) {
    run({
      item,
      phase: 'baseline'
    });
    baselined.add(key);
  }
  const path = join(
    source,
    item.file
  );
  const original = readFileSync(
    path,
    'utf8'
  );
  try {
    writeFileSync(
      path,
      replaceOne({
        text: original,
        before: item.before,
        after: item.after
      })
    );
    run({
      item,
      phase: 'removed'
    });
  } finally { writeFileSync(
    path,
    original
  ); }
  run({
    item,
    phase: 'restored'
  });
}
console.log(`Language guard controls passed: ${  artifact  } (${  selected.length  } of ${  cases.length  } guards)`);
