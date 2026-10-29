#!/usr/bin/env node
// Observe committed language-navigation regressions failing after guard removal in a disposable package copy.
// Guards: the window's reply checks (request identity, displayed text), dropping requests overtaken by a
// reload, closing surfaces whose text changed, storing only current snapshots, dropping a waiting target
// on a later open, and keeping outside-project files out of the history.
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
  'ide-language-navigation-guard-'
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
console.log(`LANGUAGE_NAVIGATION_GUARD_ARTIFACT=${  artifact}`);

/**
 A guard: the edit that removes it and the test that must then fail.
 */
interface Guard {
  name: string;
  file: string;
  before: string;
  after: string;
  occurrence?: number | undefined;
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
 `text` with one occurrence of `before` replaced by `after`.

 @param text - source of the file to mutate

 @param before - anchor that the guard exists to protect; absence means the guard moved with its code

 @param after - replacement that removes the guard

 @param occurrence - zero-based match to replace, for anchors that repeat

 @returns the edited text

 @throws {Error} when `before` occurs fewer than `occurrence` + 1 times
 */
function replaceOne({
  text,
  before,
  after,
  occurrence = 0
}: {
  text: string;
  before: string;
  after: string;
  occurrence?: number | undefined;
}): string {
  let offset = 0;
  let start = -1;
  for (let index = 0; index <= occurrence; index++) {
    start = text.indexOf(
      before,
      offset
    );
    if (start < 0) throw new Error(`Mutation source anchor is absent: ${  before}`);
    offset = start + before.length;
  }
  return text.slice(
    0,
    start
  ) + after
    + text.slice(start + before.length);
}
const equality = 'assertion `left == right` failed';
const cases: Guard[] = [
  // The window's own reply checks, on fabricated replies.
  {
    name: 'reply-request',
    file: 'src/native/language/guard.rs',
    before: 'if pending.number != Some(reply.request) || pending.action.kind() != reply.kind {',
    after: 'if false {',
    test: 'native::language::guard_tests::reply_to_another_request_is_not_applied',
    failure: 'a reply to an earlier request was applied'
  },
  {
    name: 'reply-text',
    file: 'src/native/language/guard.rs',
    before: 'if reply.stamp != displayed || pending.stamp != displayed {',
    after: 'if false {',
    test: 'native::language::guard_tests::reply_about_text_no_longer_displayed_is_not_applied',
    failure: 'a reply for the previous revision was applied after a reload'
  },
  // Storing and handing out snapshots only for the displayed text is guarded in inspect-annotation-guards.ts,
  // which owns the shared store. Here: the poll repaints what it stored, the popup clears the virtual rows of
  // its line, and a virtual row is over no source character.
  {
    name: 'repaint-on-accept',
    file: 'src/native/language/poll.rs',
    before: '    if repaint {\n        render(window, source);\n    }',
    after: '    if false {\n        render(window, source);\n    }',
    test: 'native::language::annotation_wiring_tests::server_hints_and_diagnostics_are_painted_by_the_poll_that_stored_them',
    failure: 'accepted hints and diagnostics were stored but never painted'
  },
  {
    name: 'popup-clears-rows',
    file: 'src/native/rows.rs',
    before: '    let top = map.block_top(line);\n    return (top, map.code_bottom(line) - top);',
    after: '    let top = map.code_top(line);\n    return (top, map.code_bottom(line) - top);',
    test: 'native::language::annotation_wiring_tests::hover_popup_above_a_line_does_not_cover_its_virtual_rows',
    failure: 'the hover covers the virtual rows of its line'
  },
  {
    name: 'rows-are-no-character',
    file: 'src/native/language/pointer.rs',
    before: '    if place.in_block {\n        return None;\n    }\n',
    after: '',
    test: 'native::language::annotation_wiring_tests::resting_and_control_click_on_virtual_rows_ask_nothing',
    failure: 'resting on a hint row asked for hover'
  },
  // Through the window and the scripted server.
  {
    name: 'cancel-overtaken',
    file: 'src/native/language/poll.rs',
    before: '        Some(pending) => pending.stamp != displayed,\n        None => false,\n    };\n    let dropped',
    after: '        Some(_pending) => false,\n        None => false,\n    };\n    let dropped',
    test: 'native::language::state_tests::hover_answer_overtaken_by_a_reload_is_not_shown',
    failure: equality
  },
  {
    name: 'pointer-under-list',
    file: 'src/native/language/outcome.rs',
    before: 'if pending.action == Action::PointerHover\n        && matches!(',
    after: 'if false\n        && matches!(',
    test: 'native::language::hover_tests::pointer_hover_answer_never_replaces_the_location_list',
    failure: 'a pointer hover answer replaced the location list'
  },
  {
    name: 'surface-text-changed',
    file: 'src/native/language/surface.rs',
    before: 'if now.stamp != opened.stamp {',
    after: 'if false {',
    test: 'native::language::hover_tests::hover_is_dismissed_when_the_file_reloads',
    failure: 'the reload did not dismiss the hover'
  },
  {
    name: 'later-open-drops-target',
    file: 'src/native/navigation/open.rs',
    before: '    source.borrow_mut().pending_jump = None;\n    if source.borrow().file_path.as_ref() == Some(&path) {',
    after: '    if source.borrow().file_path.as_ref() == Some(&path) {',
    test: 'native::language::definition_tests::a_later_open_drops_a_waiting_definition_target',
    failure: 'a dropped definition target moved the caret in another file'
  },
  {
    name: 'outside-no-history',
    file: 'src/native/navigation/open.rs',
    before: '    if outside {\n        // A file outside the project has no tree row',
    after: '    if false {\n        // A file outside the project has no tree row',
    test: 'native::language::definition_tests::outside_project_definition_opens_marked_without_tree_reveal_or_history',
    failure: 'the outside file entered the history'
  },
  // Through element handles: location rows report which one is selected.
  {
    name: 'a11y-location-selected',
    file: 'ui/references.slint',
    before: 'accessible-item-selected: selected;',
    after: 'accessible-item-selected: false;',
    test: 'native::accessible_list_tests::location_list_reports_role_name_position_and_the_selected_location',
    failure: 'does not report selected = true'
  },
  // Location rows and the list title are read once: the list and its rows carry the names.
  {
    name: 'a11y-location-text-hidden',
    file: 'ui/references.slint',
    before: '                        // The row carries its label and detail as name and description; its texts are not read again.\n                        accessible-role: none;\n',
    after: '',
    test: 'native::accessible_list_tests::location_list_reports_role_name_position_and_the_selected_location',
    failure: 'is named by 2 elements in its list'
  },
  {
    name: 'a11y-location-title-hidden',
    file: 'ui/references.slint',
    before: '            // The list carries the title as its name; the text is not read a second time.\n            accessible-role: none;\n',
    after: '',
    test: 'native::accessible_list_tests::location_list_reports_role_name_position_and_the_selected_location',
    failure: "the location list's title is read besides the list's own name"
  },
  // The production launch policy refuses rather than running unconfined; the refusal must reach the user.
  {
    name: 'launch-refused-explained',
    file: 'src/native/language/message.rs',
    before: '        ServerState::LaunchRefused { reason } => Some(format!(\n            "{server} was not started, so {wanted} is not available for this file: {reason}."\n        )),',
    after: '        ServerState::LaunchRefused { reason: _ } => None,',
    test: 'native::language::state_tests::refused_launch_is_explained_with_its_reason',
    failure: 'the refused launch was not explained'
  },
  // A resting pointer says once for each file why hover is unavailable, only when the support itself is the cause.
  {
    name: 'pointer-explains-support-problem',
    file: 'src/native/language/outcome.rs',
    before: '&& message::states_support_problem(&pending.outcomes, &language.status);',
    after: '&& false;',
    test: 'native::language::hover_tests::resting_pointer_explains_a_missing_server_once_for_each_file',
    failure: 'the resting pointer did not explain the missing server'
  },
  {
    name: 'pointer-explains-once',
    file: 'src/native/language/outcome.rs',
    before: 'language.explained = Some(pending.stamp.file);',
    after: 'language.explained = None;',
    test: 'native::language::hover_tests::resting_pointer_explains_a_missing_server_once_for_each_file',
    failure: 'the same file was explained twice'
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
 Runs one command in the bounded build container over the disposable copy.

 @param command - program and arguments that run in the container's `/work` directory

 @returns the finished child process with its captured output
 */
function podman(command: string[]) { return spawnSync(
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
// The window tests start the scripted server from beside the test binary; no mutation touches it.
const server = podman([
  'cargo',
  'build',
  '--offline',
  '--bin',
  'ide-scripted-lsp'
]);
writeFileSync(
  join(
    artifact,
    'scripted-server-build.log'
  ),
  (server.stdout ?? '') + (server.stderr ?? '')
);
if (server.error) throw server.error;
if (server.status !== 0) throw new Error(`Cannot build the scripted server; inspect ${  artifact}`);

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
  const result = podman([
    'cargo',
    'nextest',
    'run',
    '--offline',
    '--bin',
    'monochromatic-ide',
    '--filter-expr',
    `test(=${  item.test  })`
  ]);
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
      after: item.after,
      occurrence: item.occurrence
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
console.log(`Language navigation guard controls passed: ${  artifact  } (${  selected.length  } of ${  cases.length  } guards)`);
