#!/usr/bin/env node
// Observe committed ignored-entry regressions failing after guard removal in a disposable package copy.
// Guards: the dimmed name and its 0.8 opacity, the row label that names the entry as ignored, the flag on the row
// model, nested .gitignore files, negation across files, an ignored directory's contents, the top of the work tree,
// info/exclude and the global excludes file, the entry bound, the stale-answer drop, the stamps that raise a reread
// of the shown directories, the recompile of an edited rule file, and the path outside any work tree.
// A case with `lib` runs a library unit test, `integration` an integration test, `native` a window test.
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
  'ide-ignored-guard-'
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
console.log(`IGNORED_GUARD_ARTIFACT=${  artifact}`);

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
const matcher = 'git_ignore::tests::';
const tree = 'file_tree_ignored';
const worker = 'directory_worker_ignored';
const rows = 'native::ignored_row_tests::';
const cases: Guard[] = [
  // The dimmed name: present, at 0.8, and on the name only.
  {
    name: 'dim-name',
    file: 'ui/tree.slint',
    before: 'opacity: entry.ignored ? root.ignored-opacity : 1;',
    after: 'opacity: 1;',
    native: true,
    test: `${rows  }ignored_names_are_drawn_at_eighty_percent_with_measured_contrast`,
    failure: 'is not the plain ink at 0.8 opacity over the fill'
  },
  {
    name: 'dim-opacity-value',
    file: 'ui/tree.slint',
    before: 'private property <float> ignored-opacity: 0.8;',
    after: 'private property <float> ignored-opacity: 0.6;',
    native: true,
    test: `${rows  }ignored_names_are_drawn_at_eighty_percent_with_measured_contrast`,
    failure: 'is not the plain ink at 0.8 opacity over the fill'
  },
  {
    name: 'dim-badge',
    file: 'ui/tree.slint',
    before: '                    // The arrow repeats the expanded state and the badge the shortcut in the description.\n',
    after: '                    opacity: entry.ignored ? 0.8 : 1;\n',
    native: true,
    test: `${rows  }ignored_names_are_drawn_at_eighty_percent_with_measured_contrast`,
    failure: 'of an ignored row is dimmed'
  },
  // The second channel: the accessible label, and the flag the model carries to it.
  {
    name: 'a11y-ignored-label',
    file: 'ui/tree.slint',
    before: 'accessible-label: entry.ignored ? entry.label + ", ignored by git" : entry.label;',
    after: 'accessible-label: entry.label;',
    native: true,
    test: `${rows  }ignored_rows_are_flagged_in_the_model_and_named_in_the_accessibility_tree`,
    failure: 'should be labelled'
  },
  {
    name: 'model-flag',
    file: 'src/native/navigation/present.rs',
    before: 'ignored: row.ignored,',
    after: 'ignored: false,',
    native: true,
    test: `${rows  }ignored_rows_are_flagged_in_the_model_and_named_in_the_accessibility_tree`,
    failure: equality
  },
  {
    name: 'row-flag',
    file: 'src/file_tree/rows.rs',
    before: 'ignored = answer.names.contains(&entry.name);',
    after: 'ignored = answer.names.is_empty() && false;',
    integration: tree,
    test: 'rows_are_marked_from_the_answer_of_their_own_directory',
    failure: equality
  },
  // The answer travels with its listing and is dropped with it.
  {
    name: 'stale-answer-dropped',
    file: 'src/file_tree/requests.rs',
    before: '        if !current {',
    after: '        if false && !current {',
    integration: tree,
    test: 'a_stale_reply_cannot_install_its_answer',
    failure: 'the superseded reply is reported as not applied'
  },
  {
    name: 'worker-answer',
    file: 'src/directory_worker.rs',
    before: 'Ok(entries) => matcher.evaluate(request.path(), entries),',
    after: 'Ok(_entries) => IgnoreAnswer::default(),',
    integration: worker,
    test: 'listed_rows_carry_what_the_rule_files_say',
    failure: 'file, anchored and directory patterns dim'
  },
  {
    name: 'worker-reply',
    file: 'src/directory_worker.rs',
    before: 'reply.result, reply.ignores)',
    after: 'reply.result, IgnoreAnswer::default())',
    integration: worker,
    test: 'listed_rows_carry_what_the_rule_files_say',
    failure: 'file, anchored and directory patterns dim'
  },
  // Refresh: stamps raise the flag, and an edited rule file is compiled again.
  {
    name: 'refresh-flag',
    file: 'src/file_tree/listing.rs',
    before: 'self.ignore_sources_changed = true;',
    after: 'self.ignore_sources_changed = false;',
    integration: tree,
    test: 'only_changed_rule_stamps_raise_the_reread_flag',
    failure: 'a different .gitignore stamp raises it'
  },
  {
    name: 'refresh-flag-reader',
    file: 'src/file_tree/listing.rs',
    before: 'self.ignore_sources_changed = true;',
    after: 'self.ignore_sources_changed = false;',
    integration: worker,
    test: 'a_changed_gitignore_changes_the_rows_and_raises_the_reread_flag',
    failure: 'the stamp of the root'
  },
  {
    name: 'refresh-recompile',
    file: 'src/git_ignore.rs',
    before: '&& found.stamp == stamp',
    after: '&& found.root == root',
    lib: true,
    test: `${matcher  }an_edited_rule_file_is_compiled_again_and_changes_the_stamps`,
    failure: equality
  },
  {
    name: 'refresh-recompile-reader',
    file: 'src/git_ignore.rs',
    before: '&& found.stamp == stamp',
    after: '&& found.root == root',
    integration: worker,
    test: 'a_changed_gitignore_changes_the_rows_and_raises_the_reread_flag',
    failure: 'the root listing follows the new rules at once'
  },
  {
    name: 'refresh-recompile-window',
    file: 'src/git_ignore.rs',
    before: '&& found.stamp == stamp',
    after: '&& found.root == root',
    native: true,
    test: `${rows  }a_changed_gitignore_dims_and_undims_rows_while_the_window_runs`,
    failure: 'did not reach the expected state'
  },
  // Git's rules: nested files, negation across files, ignored directories, the work tree's top, exclude files.
  {
    name: 'nested-gitignore',
    file: 'src/git_ignore.rs',
    before: 'levels.reverse();',
    after: 'levels.reverse();\n        levels.truncate(1);',
    lib: true,
    test: `${matcher  }nested_gitignore_applies_below_its_folder_and_overrides_above`,
    failure: 'the nested pattern applies'
  },
  {
    name: 'negation-across-files',
    file: 'src/git_ignore.rs',
    before: 'Match::Whitelist(_) => return false,',
    after: 'Match::Whitelist(_) => {}',
    lib: true,
    test: `${matcher  }nested_gitignore_applies_below_its_folder_and_overrides_above`,
    failure: 'the nested negation un-ignores keep.tmp'
  },
  {
    name: 'ignored-directory-contents',
    file: 'src/git_ignore.rs',
    before: 'if decide(rules.into_iter(), level, true) {',
    after: 'if false && decide(rules.into_iter(), level, true) {',
    lib: true,
    test: `${matcher  }a_file_below_an_ignored_directory_cannot_be_re_included`,
    failure: 'the negation of build/keep.txt does not help'
  },
  {
    name: 'work-tree-top',
    file: 'src/git_ignore.rs',
    before: 'if candidate == repository.top {',
    after: 'if true {',
    lib: true,
    test: `${matcher  }rules_apply_from_the_top_of_the_work_tree_and_stop_at_a_nested_repository`,
    failure: 'the project folder is below the work tree'
  },
  {
    name: 'exclude-file',
    file: 'src/git_ignore.rs',
    before: 'repository.common_dir.join("info").join("exclude")',
    after: 'repository.common_dir.join("info").join("excluded")',
    lib: true,
    test: `${matcher  }exclude_file_and_global_file_apply_below_gitignore`,
    failure: 'exclude and global rules apply'
  },
  {
    name: 'global-file',
    file: 'src/git_ignore.rs',
    before: 'let global_file = self.global(Instant::now());',
    after: 'let global_file: Option<PathBuf> = None;',
    lib: true,
    test: `${matcher  }exclude_file_and_global_file_apply_below_gitignore`,
    failure: 'exclude and global rules apply'
  },
  {
    name: 'entry-bound',
    file: 'src/git_ignore.rs',
    before: 'if entries.len() > MAX_ENTRIES {',
    after: 'if entries.len() > MAX_ENTRIES * 100 {',
    lib: true,
    test: `${matcher  }a_directory_over_the_entry_bound_dims_nothing`,
    failure: 'answer.names.is_empty()'
  },
  // A tracked file that matches a pattern: the index is not read, so it is ignored (the documented difference).
  {
    name: 'pattern-decides',
    file: 'src/git_ignore.rs',
    before: 'Match::Ignore(_) => return true,',
    after: 'Match::Ignore(_) => return false,',
    lib: true,
    test: `${matcher  }a_file_that_matches_a_pattern_is_ignored_whether_or_not_git_tracks_it`,
    failure: equality
  },
  // The watch: a rule file edited in place reports its folder, and no other file's edit does.
  {
    name: 'rule-file-notifies',
    file: 'src/change_watch/record.rs',
    before: '&& is_ignore_rule_file(path)',
    after: '&& false',
    integration: 'change_watch',
    test: 'entries::an_ignore_rule_file_edited_in_place_reports_its_folder_and_other_files_do_not',
    failure: 'an in-place edit of .gitignore was not reported'
  },
  {
    name: 'exclude-file-notifies',
    file: 'src/change_watch/record.rs',
    before: '&& named(info, "info")',
    after: '&& named(info, "infos")',
    integration: 'change_watch',
    test: 'entries::an_ignore_rule_file_edited_in_place_reports_its_folder_and_other_files_do_not',
    failure: 'an in-place edit of info/exclude was not reported'
  },
  {
    name: 'ordinary-edit-silent',
    file: 'src/change_watch/record.rs',
    before: 'if named(Some(path), ".gitignore") {',
    after: 'if true {',
    integration: 'change_watch',
    test: 'entries::an_ignore_rule_file_edited_in_place_reports_its_folder_and_other_files_do_not',
    failure: 'a content edit of an ordinary file reported its folder'
  },
  // Not a git repository: nothing is dimmed, in the matcher, the reader and the window.
  {
    name: 'no-work-tree',
    file: 'src/git_ignore/repository.rs',
    before: '\n    return None;\n}',
    after: '\n    return Some(Repository {\n        top: directory.to_path_buf(),\n        common_dir: directory.join(".git"),\n    });\n}',
    lib: true,
    test: `${matcher  }outside_a_work_tree_nothing_is_ignored`,
    failure: equality
  },
  {
    name: 'no-work-tree-reader',
    file: 'src/git_ignore/repository.rs',
    before: '\n    return None;\n}',
    after: '\n    return Some(Repository {\n        top: directory.to_path_buf(),\n        common_dir: directory.join(".git"),\n    });\n}',
    integration: worker,
    test: 'outside_a_git_work_tree_nothing_is_dimmed',
    failure: 'session.dimmed().is_empty()'
  },
  {
    name: 'no-work-tree-window',
    file: 'src/git_ignore/repository.rs',
    before: '\n    return None;\n}',
    after: '\n    return Some(Repository {\n        top: directory.to_path_buf(),\n        common_dir: directory.join(".git"),\n    });\n}',
    native: true,
    test: `${rows  }outside_a_work_tree_no_row_is_flagged_or_named_ignored`,
    failure: 'is flagged'
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
      `test(${  item.test  })`
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
      `${item.name  }-${  phase  }.log`
    ),
    output
  );
  if (result.error) throw result.error;
  const failedAsExpected = (result.status !== 0) && output.includes(item.test
    .split('::')
    .at(-1) ?? '')
    && output.includes(item.failure);
  const passedAsExpected = (result.status === 0) && output.includes(item.test
    .split('::')
    .at(-1) ?? '')
    && /1 (?:test|passed)/.test(output);
  const accepted = phase === 'removed' ? failedAsExpected : passedAsExpected;
  const line = output.split('\n')
    .find(text => (item.failure !== '') && text.includes(item.failure));
  results.push({
    name: item.name,
    phase,
    test: item.test,
    status: result.status,
    accepted,
    ...(phase === 'removed' ? { failureText: (line ?? '').trim()
      .slice(
      0,
      300
      ) } : {})
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
  if (!accepted) throw new Error(`Unexpected ${  phase
     } result for ${
     item.name
     }; inspect ${
     artifact}`);
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
console.log(`Ignored-entry guard controls passed: ${  artifact  } (${  selected.length  } of ${  cases.length  } guards)`);
