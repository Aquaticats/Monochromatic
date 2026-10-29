#!/usr/bin/env node
// Observe committed change-watch regressions failing after guard removal in a disposable package copy.
// Guards: the read-event filter, root and canonical containment, full rereads on overflow, errors, and
// lost or failed watches, unsettled-write waiting, collapse unwatching, the extra read after a new watch,
// retries and failures reported once, the watch limit as one state with backoff, the reread schedules,
// the displayed file outside the project, the quiet requirement of reads no write notification asked for,
// the folders watched for the language servers and the changes forwarded to them, rust-analyzer's
// client-side watching, the highlighting warning, and the native wiring that turns notifications into
// reads and keeps a save in progress off the screen. A case with `lib` runs a library unit test.
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
import { gpuArguments } from './gpu-arguments.ts';

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
  'ide-watch-guard-'
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
console.log(`WATCH_GUARD_ARTIFACT=${  artifact}`);

/**
 The fields every guard has: the edit that removes it and the test that must then fail.
 */
interface GuardBase {
  name: string;
  file: string;
  before: string;
  after: string;
  test: string;
  failure: string;
}

/**
 A guard whose test is a unit test of the window binary, run by nextest.
 */
interface NativeGuard extends GuardBase {
  native: true;
  lib?: undefined;
  integration?: undefined;
}

/**
 A guard whose test is a unit test of the library.
 */
interface LibraryGuard extends GuardBase {
  native?: undefined;
  lib: true;
  integration?: undefined;
}

/**
 A guard whose test is in one integration test file.
 */
interface IntegrationGuard extends GuardBase {
  native?: undefined;
  lib?: undefined;
  integration: string;
}

/**
 A guard: the edit that removes it and the one test that must then fail.
 */
type Guard = NativeGuard | LibraryGuard | IntegrationGuard;

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
const equality = 'assertion `left == right` failed';
const watch = 'change_watch';
const policy = 'refresh_policy';
const intervals = 'refresh_intervals';
const quiet = 'quiet_read';
const language = 'language';
const cases: Guard[] = [
  // Event classification on notify's thread.
  {
    name: 'read-events-ignored',
    file: 'src/change_watch/record.rs',
    before: 'EventKind::Access(_) => {\n            return Reaction::Ignore;',
    after: 'EventKind::Access(_) => {\n            return Reaction::Entries(SourceChange::Settled);',
    integration: watch,
    test: 'source::the_ides_own_reads_report_nothing',
    failure: 'reads reported changes'
  },
  {
    name: 'open-write-unsettled',
    file: 'src/change_watch/record.rs',
    before: 'EventKind::Modify(ModifyKind::Data(_)) => {\n            return Reaction::Content(SourceChange::Unsettled);',
    after: 'EventKind::Modify(ModifyKind::Data(_)) => {\n            return Reaction::Content(SourceChange::Settled);',
    integration: watch,
    test: 'source::displayed_file_changes_are_settled_only_when_finished',
    failure: 'a truncating write still open looked finished'
  },
  {
    name: 'outside-root-ignored',
    file: 'src/change_watch/record.rs',
    before: 'if !path.starts_with(&guard.root) {',
    after: 'if false {',
    integration: watch,
    test: 'boundary::events_naming_paths_outside_the_root_are_ignored',
    failure: 'an event outside the root reached the displayed file'
  },
  {
    name: 'overflow-rereads',
    file: 'src/change_watch/record.rs',
    before: 'rereading everything shown"\n        );\n        lock(shared).pending.everything = true;',
    after: 'rereading everything shown"\n        );',
    integration: watch,
    test: 'recovery::queue_overflow_requests_a_full_reread',
    failure: "an overflowed queue's full reread was not reported"
  },
  {
    name: 'error-rereads',
    file: 'src/change_watch/record.rs',
    before: '    guard.pending.everything = true;\n    let mut lost = false;\n    for path in &error.paths {',
    after: '    let mut lost = false;\n    for path in &error.paths {',
    integration: watch,
    test: 'recovery::a_notification_error_requests_a_full_reread',
    failure: "a notification error's full reread was not reported"
  },
  {
    name: 'moved-watch-dropped',
    file: 'src/change_watch/record.rs',
    before: '                guard.stale.insert(path.clone());\n                guard.pending.everything = true;',
    after: '                guard.pending.everything = true;',
    integration: watch,
    test: 'recovery::a_renamed_watched_folder_is_not_followed_to_its_new_name',
    failure: "a renamed folder's full reread and lost watch was not reported"
  },
  // The watch thread: containment, kernel unwatching, extra reads, failures, and retries.
  {
    name: 'root-containment',
    file: 'src/change_watch/watch_ops.rs',
    before: 'let resolved = match workspace.resolve(path) {',
    after: 'let resolved = match Ok::<_, anyhow::Error>(path.to_path_buf()) {',
    integration: watch,
    test: 'boundary::folders_outside_the_root_are_never_watched_or_reported',
    failure: 'refused watches falling back to a full reread was not reported'
  },
  {
    name: 'canonical-alias',
    file: 'src/change_watch/watch_ops.rs',
    before: 'if resolved != path {',
    after: 'if false {',
    integration: watch,
    test: 'boundary::folders_outside_the_root_are_never_watched_or_reported',
    failure: 'a symbolic-link alias inside the root was watched'
  },
  {
    name: 'moved-kernel-unwatch',
    file: 'src/change_watch/reconcile.rs',
    before: '        if watches.active.remove(path) {\n            kernel.remove(path);',
    after: '        if watches.active.remove(path) {',
    integration: watch,
    test: 'recovery::a_renamed_folder_with_an_unwatched_parent_loses_its_kernel_watch',
    failure: 'the moved folder kept a kernel watch that reports under its old name'
  },
  {
    name: 'collapse-unwatch',
    file: 'src/change_watch/reconcile.rs',
    before: '        watches.active.remove(path);\n        kernel.remove(path);',
    after: '        watches.active.remove(path);',
    integration: watch,
    test: 'entries::collapsing_a_folder_removes_its_kernel_watch',
    failure: 'the collapsed folder kept its kernel watch'
  },
  {
    name: 'new-watch-reread',
    file: 'src/change_watch/watch_thread.rs',
    before: 'published.pending.directories.extend(outcome.established);',
    after: '',
    integration: watch,
    test: 'entries::a_new_watch_reports_its_folder_and_the_displayed_file_once',
    failure: "the root's new watch was not reported"
  },
  {
    name: 'new-watch-source',
    file: 'src/change_watch/watch_thread.rs',
    before: '    if (outcome.switched || reached)',
    after: '    if (outcome.switched)',
    integration: watch,
    test: 'entries::a_displayed_file_is_reported_when_its_retried_folder_watch_starts',
    failure: "the displayed file after its folder's retried watch was not reported"
  },
  {
    name: 'switched-file-source',
    file: 'src/change_watch/watch_thread.rs',
    before: '    if (outcome.switched || reached)',
    after: '    if (false || reached)',
    integration: watch,
    test: 'entries::a_newly_displayed_file_in_a_watched_folder_is_reported_once',
    failure: 'the newly displayed file was not reported'
  },
  {
    name: 'folder-permission-reported',
    file: 'src/change_watch/record.rs',
    before: '            guard.pending.directories.insert(path.clone());',
    after: '',
    integration: watch,
    test: 'entries::a_permission_change_on_a_watched_folder_reports_it',
    failure: 'a permission change was not reported'
  },
  {
    name: 'failed-watch-rereads',
    file: 'src/change_watch/reconcile.rs',
    before: 'everything: pass.lost || entered,',
    after: 'everything: entered,',
    integration: watch,
    test: 'recovery::a_failed_watch_requests_a_full_reread_and_is_retried_on_request',
    failure: "a failed watch's full reread was not reported"
  },
  {
    name: 'retry-failed',
    file: 'src/change_watch/reconcile.rs',
    before: 'if !allowed.failed && watches.failed.contains_key(&path) {',
    after: 'if watches.failed.contains_key(&path) {',
    integration: watch,
    test: 'recovery::a_failed_watch_requests_a_full_reread_and_is_retried_on_request',
    failure: 'the retried watch and its extra read was not reported'
  },
  {
    name: 'failure-reported-once',
    file: 'src/change_watch/reconcile.rs',
    before: 'if watches.failed.get(&path) == Some(&message) {',
    after: 'if false {',
    integration: watch,
    test: 'recovery::a_watch_failing_the_same_way_is_reported_once_until_it_changes',
    failure: 'an unchanged failure was reported again'
  },
  // The watch limit as one state: library tests with a fake kernel and chosen times.
  {
    name: 'limit-stops-adding',
    file: 'src/change_watch/reconcile.rs',
    before: '        if pass.limit_hit {\n            watches.limited.insert(path);\n            continue;\n        }',
    after: '',
    lib: true,
    test: 'change_watch::reconcile::tests::the_watch_limit_is_one_state_with_backoff',
    failure: 'adds continued after the limit answered'
  },
  {
    name: 'limit-backoff',
    file: 'src/change_watch/reconcile.rs',
    before: '        Some(backoff) => backoff.may_retry(now),',
    after: '        Some(_backoff) => true,',
    lib: true,
    test: 'change_watch::reconcile::tests::the_watch_limit_is_one_state_with_backoff',
    failure: 'a sweep retried before the backoff allowed it'
  },
  {
    name: 'limit-entered-once',
    file: 'src/change_watch/reconcile.rs',
    before: 'if let Some(backoff) = &mut watches.limit {',
    after: 'if let Some(backoff) = None::<&mut LimitBackoff> {',
    lib: true,
    test: 'change_watch::reconcile::tests::the_watch_limit_is_one_state_with_backoff',
    failure: 'a retry that hit the limit again reread everything'
  },
  {
    name: 'limit-left',
    file: 'src/change_watch/reconcile.rs',
    before: 'if watches.limit.is_some() && watches.limited.is_empty() {',
    after: 'if false {',
    lib: true,
    test: 'change_watch::reconcile::tests::freed_watches_end_the_limit_state',
    failure: 'the limit state outlived free watches'
  },
  {
    name: 'limit-file-first',
    file: 'src/change_watch/reconcile.rs',
    before: '    if let Some(directory) = parent {\n        order.push(directory.to_path_buf());\n    }',
    after: '',
    lib: true,
    test: 'change_watch::reconcile::tests::the_displayed_files_folder_is_watched_first',
    failure: "the displayed file's folder was not tried first"
  },
  {
    name: 'limit-user-retry',
    file: 'src/change_watch/reconcile.rs',
    before: '        limited: shown_changed || sweep_retry || request.user_retry,',
    after: '        limited: shown_changed || sweep_retry,',
    lib: true,
    test: 'change_watch::reconcile::tests::a_user_retry_skips_the_backoff',
    failure: 'a scroll did not retry at once'
  },
  {
    name: 'limit-backoff-cap',
    file: 'src/change_watch/limit.rs',
    before: '.saturating_mul(2).min(LONGEST_LIMIT_RETRY);',
    after: '.saturating_mul(2);',
    lib: true,
    test: 'change_watch::reconcile::tests::the_backoff_is_capped',
    failure: 'the wait grew past its cap'
  },
  // A displayed file outside the project is neither watched nor polled like a failed watch.
  {
    name: 'outside-folder-not-asked',
    file: 'src/change_watch.rs',
    before: 'if parent.starts_with(&self.root) {',
    after: 'if true {',
    integration: watch,
    test: 'boundary::a_displayed_file_outside_the_root_is_not_watched_and_not_a_failure',
    failure: "the outside file's folder was treated as a failed watch"
  },
  {
    name: 'outside-file-sweep',
    file: 'src/refresh_policy/source.rs',
    before: 'if self.watched || self.outside_project {',
    after: 'if self.watched {',
    integration: intervals,
    test: 'a_file_outside_the_project_is_reread_on_the_sweep',
    failure: 'a file outside the project was polled like a failed watch'
  },
  // Reads no write notification asked for accept only a file that has been quiet.
  {
    name: 'quiet-read-recent',
    file: 'src/file_reload.rs',
    before: 'Ok(age) => age >= quiet,',
    after: 'Ok(_age) => true,',
    integration: quiet,
    test: 'a_quiet_read_drops_a_file_written_within_the_quiet_period',
    failure: 'a file written just now was accepted'
  },
  {
    name: 'quiet-read-ahead',
    file: 'src/file_reload.rs',
    before: '            false\n        }\n    };',
    after: '            true\n        }\n    };',
    integration: quiet,
    test: 'a_quiet_read_drops_a_file_written_within_the_quiet_period',
    failure: 'ahead of this clock was accepted'
  },
  {
    name: 'reread-classified',
    file: 'src/change_watch/watch_thread.rs',
    before: 'published.pending.source = Some(SourceChange::Reread);',
    after: 'published.pending.source = Some(SourceChange::Settled);',
    integration: watch,
    test: 'entries::a_newly_displayed_file_in_a_watched_folder_is_reported_once',
    failure: 'the newly displayed file was reported as a finished write'
  },
  {
    name: 'reread-needs-quiet',
    file: 'src/refresh_policy/source.rs',
    before: 'return self.pending_since.is_some() && self.notified;',
    after: 'return self.pending_since.is_some();',
    integration: quiet,
    test: 'a_reread_with_no_write_behind_it_requires_quiet',
    failure: 'a reread with no write behind it skipped the quiet check'
  },
  {
    name: 'reread-keeps-wait',
    file: 'src/refresh_policy/source.rs',
    before: '            SourceChange::Reread => {\n                if first {',
    after: '            SourceChange::Reread => {\n                self.unsettled_at = None;\n                if first {',
    integration: quiet,
    test: 'a_reread_with_no_write_behind_it_requires_quiet',
    failure: 'a reread ended the wait for an unfinished write'
  },
  // Folders watched for the language servers: shared with the tree, yielding to it, their own limit state.
  {
    name: 'server-shares-tree-watch',
    file: 'src/change_watch/server_watch.rs',
    before: '        if self.servers.active.contains(path) {\n            self.tree_held.insert(path.to_path_buf());',
    after: '        if false {\n            self.tree_held.insert(path.to_path_buf());',
    lib: true,
    test: 'change_watch::server_watch::tests::a_folder_the_servers_watch_is_shared_with_the_tree',
    failure: 'a second kernel watch was added'
  },
  {
    name: 'server-keeps-shared-watch',
    file: 'src/change_watch/server_watch.rs',
    before: '        if self.servers.active.contains(path) {\n            return;\n        }',
    after: '',
    lib: true,
    test: 'change_watch::server_watch::tests::a_folder_the_servers_watch_is_shared_with_the_tree',
    failure: "collapsing in the tree removed the servers' watch"
  },
  {
    name: 'tree-takes-server-watch',
    file: 'src/change_watch/server_watch.rs',
    before: 'if !self.tree_held.contains(held) {',
    after: 'if false {',
    lib: true,
    test: 'change_watch::server_watch::tests::the_tree_comes_first_under_the_limit',
    failure: 'the tree reached the limit although a server watch could be given up'
  },
  {
    name: 'servers-wait-for-tree',
    file: 'src/change_watch/server_watch.rs',
    before: '        if request.tree_limited || limit_hit {',
    after: '        if limit_hit {',
    lib: true,
    test: 'change_watch::server_watch::tests::the_servers_wait_while_the_tree_waits',
    failure: 'the servers added a watch while the tree waited'
  },
  {
    name: 'server-limit-backoff',
    file: 'src/change_watch/server_watch.rs',
    before: '    let retry_limited = request.desired_changed || backoff_retry;',
    after: '    let retry_limited = true;',
    lib: true,
    test: 'change_watch::server_watch::tests::the_tree_comes_first_under_the_limit',
    failure: 'the servers retried before the backoff allowed it'
  },
  {
    name: 'servers-yield-backs-off',
    file: 'src/change_watch/server_watch.rs',
    before: '    update_limit(servers, backoff_retry && (limit_hit || yielded), now);',
    after: '    update_limit(servers, backoff_retry && limit_hit, now);',
    lib: true,
    test: 'change_watch::server_watch::tests::the_servers_wait_while_the_tree_waits',
    failure: 'so the watch thread would wake every 50 ms'
  },
  {
    name: 'server-limit-left',
    file: 'src/change_watch/server_watch.rs',
    before: '    if servers.limited.is_empty() {\n        if servers.limit.take().is_some() {',
    after: '    if false {\n        if servers.limit.take().is_some() {',
    lib: true,
    test: 'change_watch::server_watch::tests::the_tree_comes_first_under_the_limit',
    failure: "the servers' limit state outlived the free watch"
  },
  // The folder scan: the search's ignore rules, pruned folders, empty folders, and new folders by their parent.
  {
    name: 'scan-ignore-rules',
    file: 'src/change_watch/server_scan.rs',
    before: '        .args(RIPGREP_SETTINGS)\n        .args(["--files", "--null", "--"])',
    after: '        .args(RIPGREP_SETTINGS)\n        .arg("--no-ignore")\n        .args(["--files", "--null", "--"])',
    lib: true,
    test: 'change_watch::server_scan::tests::source_folders_follow_the_search_ignore_rules',
    failure: equality
  },
  {
    name: 'scan-pruned',
    file: 'src/change_watch/server_scan.rs',
    before: '    let Ok(relative) = file.strip_prefix(base) else {\n        return true;\n    };',
    after: '    let Ok(relative) = file.strip_prefix(base) else {\n        return true;\n    };\n    if relative.as_os_str().len() < usize::MAX {\n        return false;\n    }',
    lib: true,
    test: 'change_watch::server_scan::tests::pruned_folders_are_left_out_without_an_ignore_file',
    failure: equality
  },
  {
    name: 'scan-empty-provisional',
    file: 'src/change_watch/server_scan.rs',
    before: '    result.provisional = empty_children(&result.directories);',
    after: '    result.provisional = BTreeSet::new();',
    lib: true,
    test: 'change_watch::server_scan::tests::source_folders_follow_the_search_ignore_rules',
    failure: 'the empty folder was not watched provisionally'
  },
  {
    name: 'scan-new-folder-parent',
    file: 'src/change_watch/server_scan.rs',
    before: '            wanted.insert(parent.to_path_buf());',
    after: '            wanted.insert(candidate.clone());',
    lib: true,
    test: 'change_watch::server_scan::tests::new_folders_are_classified_by_their_parent',
    failure: equality
  },
  // Registered watchers and gathered changes, without a server.
  {
    name: 'merge-final-state',
    file: 'src/language/watched_files.rs',
    before: '        (_, ServerChangeKind::Deleted) => {\n            return ServerChangeKind::Deleted;',
    after: '        (_, ServerChangeKind::Deleted) => {\n            return ServerChangeKind::Changed;',
    lib: true,
    test: 'language::watched_files::tests::the_merged_kind_matches_the_final_state',
    failure: equality
  },
  {
    name: 'glob-segment',
    file: 'src/language/watched_files.rs',
    before: 'GlobBuilder::new(pattern).literal_separator(true).build();',
    after: 'GlobBuilder::new(pattern).literal_separator(false).build();',
    lib: true,
    test: 'language::watched_files::tests::glob_patterns_follow_the_protocol',
    failure: 'relative patterns matched the wrong files'
  },
  {
    name: 'relative-pattern-base',
    file: 'src/language/watched_files.rs',
    before: '                Err(_) => false,\n            };\n        }\n        None => {',
    after: '                Err(_) => true,\n            };\n        }\n        None => {',
    lib: true,
    test: 'language::watched_files::tests::glob_patterns_follow_the_protocol',
    failure: 'relative patterns matched the wrong files'
  },
  {
    name: 'watcher-kinds',
    file: 'src/language/watched_files.rs',
    before: '    if !watcher.kinds.contains(wanted_kind(kind)) {',
    after: '    if false {',
    lib: true,
    test: 'language::watched_files::tests::kinds_limit_what_a_watcher_hears',
    failure: equality
  },
  {
    name: 'no-watchers-nothing',
    file: 'src/language/watched_files.rs',
    before: '        if self.registrations.is_empty() {\n            return false;\n        }',
    after: '',
    lib: true,
    test: 'language::watched_files::tests::only_running_servers_with_watchers_hear_anything',
    failure: 'a change was gathered with no watcher registered'
  },
  {
    name: 'stopped-server-forgotten',
    file: 'src/language/watched_files.rs',
    before: '            if !host.live(*server) {',
    after: '            if false {',
    lib: true,
    test: 'language::watched_files::tests::only_running_servers_with_watchers_hear_anything',
    failure: equality
  },
  // Forwarding through the worker and the change watcher, with the scripted server.
  {
    name: 'forward-registrations',
    file: 'src/language/traffic.rs',
    before: '                worker.watched.register(server, identifier, options);',
    after: '',
    integration: language,
    test: 'watched::changes_reach_the_server_by_glob_and_kind',
    failure: 'folders were watched for the servers'
  },
  {
    name: 'forward-pulls-diagnostics',
    file: 'src/language/forward.rs',
    before: '        if opened {\n            request::pull_diagnostics(worker, index, 0);\n        }',
    after: '',
    integration: language,
    test: 'watched::changes_reach_the_server_by_glob_and_kind',
    failure: 'a diagnostics pull after the file changes'
  },
  {
    name: 'forward-holds-provisional',
    file: 'src/change_watch/record.rs',
    before: '        && guard.server.provisional.contains(folder)\n    {',
    after: '        && guard.server.provisional.contains(folder)\n        && false\n    {',
    integration: language,
    test: 'watched::new_folders_are_followed_unless_ignored',
    failure: "a new ignored folder's file was sent"
  },
  {
    name: 'forward-scan-creations',
    file: 'src/change_watch/server_wake.rs',
    before: '    if found.initial {\n        return;\n    }',
    after: '    if found.initial || true {\n        return;\n    }',
    integration: language,
    test: 'watched::new_folders_are_followed_unless_ignored',
    failure: "the new folders' files"
  },
  // rust-analyzer registers watchers with the IDE instead of watching the project itself.
  {
    name: 'rust-analyzer-client-watching',
    file: 'src/language/config.rs',
    before: '    rust_analyzer::watch_through_client(&mut configuration);',
    after: '',
    lib: true,
    test: 'language::config::rust_analyzer::tests::rust_analyzer_watches_through_the_client',
    failure: equality
  },
  // Highlighting that cannot start is a warning only for a file some language applies to.
  {
    name: 'highlight-plain-text',
    file: 'src/reload_read.rs',
    before: '"highlighting cannot start, but no language applies to this file; it stays plain text");\n            return Ok(None);',
    after: '"highlighting cannot start, but no language applies to this file; it stays plain text");\n            return Err(initialization_failure(error, path));',
    lib: true,
    test: 'reload_read::tests::plain_text_is_not_a_failure_when_highlighting_cannot_start',
    failure: 'a plain text file was reported as a highlighting failure'
  },
  {
    name: 'highlight-language-kept',
    file: 'src/reload_read.rs',
    before: '        Ok(true) => {\n            return Err(initialization_failure(error, path));',
    after: '        Ok(true) => {\n            return Ok(None);',
    lib: true,
    test: 'reload_read::tests::a_language_file_keeps_the_failure_when_highlighting_cannot_start',
    failure: 'a Rust file lost its highlighting without a failure'
  },
  // Reread schedules.
  {
    name: 'settled-now',
    file: 'src/refresh_policy/source.rs',
    before: '                None => {\n                    return true;',
    after: '                None => {\n                    return false;',
    integration: policy,
    test: 'unfinished_writes_wait_for_quiet_within_a_limit',
    failure: 'a finished write was not read at once'
  },
  {
    name: 'write-quiet',
    file: 'src/refresh_policy/source.rs',
    before: '>= WRITE_QUIET',
    after: '>= WRITE_WAIT_LIMIT',
    integration: policy,
    test: 'unfinished_writes_wait_for_quiet_within_a_limit',
    failure: 'a quiet unfinished write was not read'
  },
  {
    name: 'write-wait-limit',
    file: 'src/refresh_policy/source.rs',
    before: '\n                        || now.saturating_duration_since(since) >= WRITE_WAIT_LIMIT',
    after: '',
    integration: policy,
    test: 'unfinished_writes_wait_for_quiet_within_a_limit',
    failure: 'a continuously written file waited past the limit'
  },
  {
    name: 'timer-waits-for-write',
    file: 'src/refresh_policy/source.rs',
    before: '>= WRITE_WAIT_LIMIT;',
    after: '>= WRITE_WAIT_LIMIT\n                        || now.saturating_duration_since(last) >= SAFETY_SWEEP;',
    integration: intervals,
    test: 'timers_do_not_read_a_file_whose_write_is_unfinished',
    failure: 'the sweep read a file 10 ms into an unfinished write'
  },
  {
    name: 'watched-source-timer',
    file: 'src/refresh_policy/source.rs',
    before: '            SAFETY_SWEEP\n        } else {',
    after: '            UNWATCHED_SOURCE_POLL\n        } else {',
    integration: policy,
    test: 'source_timers_depend_on_whether_its_directory_is_watched',
    failure: 'a watched file was polled'
  },
  {
    name: 'notified-first',
    file: 'src/refresh_policy/directories.rs',
    before: '        if let Some(path) = first_ready(&mut self.changed, shown, &self.started, now) {\n            return Some(path);\n        }',
    after: '',
    integration: policy,
    test: 'notified_directories_are_read_first_in_visible_order',
    failure: equality
  },
  {
    name: 'only-unwatched-polled',
    file: 'src/refresh_policy/directories.rs',
    before: 'if !watched.contains(path) {',
    after: 'if true {',
    integration: policy,
    test: 'notified_directories_are_read_first_in_visible_order',
    failure: 'watched directories were polled'
  },
  {
    name: 'unwatched-interval',
    file: 'src/refresh_policy/directories.rs',
    before: 'Some(at) => now.saturating_duration_since(at) >= UNWATCHED_DIRECTORY_POLL,',
    after: 'Some(_at) => true,',
    integration: policy,
    test: 'unwatched_directories_keep_the_old_round_robin',
    failure: equality
  },
  {
    name: 'sweep-interval',
    file: 'src/refresh_policy/directories.rs',
    before: 'if now.saturating_duration_since(last) < SAFETY_SWEEP {',
    after: 'if true {',
    integration: policy,
    test: 'the_safety_sweep_rereads_everything_after_notifications',
    failure: 'the safety sweep did not start'
  },
  {
    name: 'sweep-finishes-first',
    file: 'src/refresh_policy/directories.rs',
    before: '        if !self.sweep.is_empty() {\n            return false;\n        }',
    after: '',
    integration: intervals,
    test: 'sweep_rereads_every_folder_when_one_pass_outlasts_the_interval',
    failure: 'while sweeps restarted from the top'
  },
  {
    name: 'source-reread-gap',
    file: 'src/refresh_policy/source.rs',
    before: '            if !rested {\n                return false;\n            }',
    after: '',
    integration: policy,
    test: 'notified_rereads_keep_a_gap_per_item',
    failure: 'a notified source reread did not wait for the gap'
  },
  {
    name: 'highlight-reread-gap',
    file: 'src/refresh_policy/source.rs',
    before: 'if highlight_missing && rested {',
    after: 'if highlight_missing {',
    integration: policy,
    test: 'notified_rereads_keep_a_gap_per_item',
    failure: 'missing highlighting was requested again inside the gap'
  },
  {
    name: 'folder-reread-gap',
    file: 'src/refresh_policy/directories.rs',
    before: 'if rested && pending.remove(path) {',
    after: 'if pending.remove(path) {',
    integration: policy,
    test: 'notified_rereads_keep_a_gap_per_item',
    failure: 'a notified folder reread did not wait for the gap'
  },
  // Native wiring: notifications become due reads for the shipped tree and source.
  // The first change of the tree test can meet a 1 s sweep by chance, so any later step may be the one that fails.
  {
    name: 'native-tree-notified',
    file: 'src/native/navigation/watch.rs',
    before: 'navigation.directories.changed(directory);',
    after: '',
    native: true,
    test: 'native_tree_follows_changes_in_one_of_several_expanded_folders',
    failure: 'did not appear within'
  },
  {
    name: 'native-shown-watched',
    file: 'src/native/navigation/present.rs',
    before: 'watch::show(source, navigation);',
    after: '',
    native: true,
    test: 'native_tree_follows_changes_in_one_of_several_expanded_folders',
    failure: 'did not appear within'
  },
  {
    name: 'native-source-notified',
    file: 'src/native/navigation/watch.rs',
    before: 'current.refresh.changed(change, now);',
    after: '',
    native: true,
    test: 'native_source_follows_atomic_replace_and_delete_then_recreate',
    failure: 'the caret replacement did not appear within'
  },
  {
    name: 'native-source-mode',
    file: 'src/native/navigation/watch.rs',
    before: 'current.refresh.set_watched(watched);',
    after: '',
    native: true,
    test: 'native_source_follows_atomic_replace_and_delete_then_recreate',
    failure: 'native navigation did not reach the expected state'
  },
  // A read no write notification asked for, here the first read of a file outside the project, waits for quiet.
  {
    name: 'native-timer-read-quiet',
    file: 'src/native/reload.rs',
    before: '            Some(WRITE_QUIET)\n        };',
    after: '            None\n        };',
    native: true,
    test: 'reads_no_notification_asked_for_do_not_show_a_save_in_progress',
    failure: 'a save in progress was shown'
  },
  // The application's language tick hands the worker's file-change queue to the change watcher.
  {
    name: 'native-forward-relay',
    file: 'src/native/language/poll.rs',
    before: '    relay_watching(language, navigation);\n',
    after: '',
    native: true,
    test: 'an_external_change_reaches_the_server_through_the_application',
    failure: 'the external change did not reach the server'
  },
  // A save in progress stays off the screen: unfinished writes and new files wait for the writer.
  {
    name: 'native-save-in-progress',
    file: 'src/change_watch/record.rs',
    before: 'EventKind::Modify(ModifyKind::Data(_)) => {\n            return Reaction::Content(SourceChange::Unsettled);',
    after: 'EventKind::Modify(ModifyKind::Data(_)) => {\n            return Reaction::Content(SourceChange::Settled);',
    native: true,
    test: 'native_source_does_not_show_an_in_place_save_before_it_finishes',
    failure: 'a save in progress was shown'
  },
  {
    name: 'native-rewrite-in-progress',
    file: 'src/change_watch/record.rs',
    before: 'EventKind::Create(_) => {\n            return Reaction::Entries(SourceChange::Unsettled);',
    after: 'EventKind::Create(_) => {\n            return Reaction::Entries(SourceChange::Settled);',
    native: true,
    test: 'native_source_does_not_show_a_deleted_file_before_its_rewrite_finishes',
    failure: 'a rewrite in progress was shown'
  },
];
// An optional comma-separated list reruns only the named guards, for example after adding one.
const only = process.env
  .usage_only ? new Set(process.env
    .usage_only
    .split(',')) : undefined;
const selected = only ? cases.filter(item => only.has(item.name)) : cases;
if (only && (selected.length !== only.size)) throw new Error(`Unknown guard name in: ${
   process.env
     .usage_only}`);
// Every anchor is checked before the first build, so one that no longer matches the source fails at once.
for (const item of cases) replaceOne({
  text: readFileSync(
    join(
      source,
      item.file
    ),
    'utf8'
  ),
  before: item.before,
  after: item.after
});
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
  // What each kind of case runs: a native window test, a library unit test, or an integration test.
  const command = item.native
    ? [
      'cargo',
      'nextest',
      'run',
      '--offline',
      '--bin',
      'monochromatic-ide',
      '--filter-expr',
      `test(${  item.test
         })`
    ]
    : item.lib
      ? [
        'cargo',
        'test',
        '--offline',
        '--no-default-features',
        '--lib',
        item.test,
        '--',
        '--nocapture'
      ]
      : [
        'cargo',
        'test',
        '--offline',
        '--no-default-features',
        '--test',
        item.integration,
        item.test,
        '--',
        '--nocapture'
      ];
  function start() { return spawnSync(
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
      'SLINT_EMIT_DEBUG_INFO=1',
      ...gpuArguments,
      '--env',
      'HELIX_RUNTIME=/work/target/debug/runtime',
      'localhost/monochromatic/ide',
      ...command,
    ],
    {
      encoding: 'utf8',
      maxBuffer: 16 * 1_024
        * 1_024
    }
  ); }
  let result = start();
  // Concurrent podman sessions can briefly lock podman's own storage database before the container exists;
  // that is not a test outcome, so the identical command is started again.
  for (let attempt = 1; (attempt <= 3) && (result.status === 125)
    && (result.stderr ?? '').includes('database is locked'); attempt++) {
    console.log(`podman storage was locked; starting ${  item.name
       } ${
       phase
       } again (${
       attempt
       })`);
    result = start();
  }
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
    && /1 (?:test|passed)/.test(output);
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
  if (!baselined.has(item.test)) {
    run({
      item,
      phase: 'baseline'
    });
    baselined.add(item.test);
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
    const changed = replaceOne({
      text: original,
      before: item.before,
      after: item.after
    });
    if (changed === original) throw new Error(`Mutation did not change ${  item.file}`);
    writeFileSync(
      path,
      changed
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
console.log(`Watch guard controls passed: ${  artifact  } (${  selected.length  } of ${  cases.length  } guards)`);
