#!/usr/bin/env node
// Controls for the quiet-lifetime and logging regressions, run in a disposable package copy. Each case
// removes one guard, expects its named test to fail with the named text, then restores it and expects a pass:
// - each re-labelled helix-lsp record shape: without its re-labelling the clean-lifetime test sees it at ERROR;
// - re-labelling every transport ERROR record: the test of unknown records sees a real failure lowered;
// - a full log queue that waits for room: the stalled-output test sees the logging thread delayed;
// - the worker's shutdown request, its wait for servers to end, and its reaping of a killed server;
// - a server's last standard-error lines: kept and reported when it crashes, waited for when they arrive just
//   after the end, and never reported on a clean shutdown.
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

/**
 A disposable directory below the private scratch root, or a failure that names the argument.

 @param value - path the caller passed

 @param what - name of the argument, for the message

 @returns the real path of `value`

 @throws {Error} when `value` is not a directory below the private scratch root
 */
function disposable({
  value,
  what
}: {
  value: string;
  what: string;
}): string {
  const resolved = realpathSync(value);
  if ((!resolved.startsWith(realpathSync(privateRoot) + sep)) || (!statSync(resolved)
    .isDirectory())) throw new Error(`${what  } must be a disposable directory below ${  privateRoot}`);
  return resolved;
}
if (!process.env
  .usage_cache) throw new Error('Provide the disposable Cargo target-cache directory');
const cache = disposable({
  value: process.env
    .usage_cache,
  what: 'Target cache'
});
// A private Cargo home copy keeps disposable builds off the shared volume's package-cache lock.
const cargoHome = process.env
  .usage_cargo ? disposable({
    value: process.env
      .usage_cargo,
    what: 'Cargo home copy'
  }) : 'ide-cargo';
// An optional comma-separated list reruns only the named cases (their baselines always run).
const only = process.env
  .usage_only ? new Set(process.env
    .usage_only
    .split(',')) : undefined;
const artifact = mkdtempSync(join(
  privateRoot,
  'ide-language-lifecycle-guard-'
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
console.log(`LANGUAGE_LIFECYCLE_GUARD_ARTIFACT=${  artifact}`);

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
const strict = 'quiet::clean_lifetime_logs_no_error_level_record';
const unknown = 'quiet::unknown_helix_error_records_keep_their_level';
const killed = 'lifecycle::server_that_ignores_exit_is_killed_and_reaped_before_the_drop_returns';
const stalled = 'logging::background::tests::a_blocked_output_never_delays_the_logging_thread';
const crashed = 'quiet::a_crashed_server_is_logged_with_its_last_stderr_lines';
const settled = 'language::attach::report::tests::an_end_waits_for_lines_that_arrive_after_it';

/**
 One test run recorded in `results.json` and echoed as one JSON line.
 */
interface RunResult {
  name: string;
  test: string;
  expect: 'pass' | string[];
  status: number | null;
  accepted: boolean;
}

/**
 A guard: the edit that removes it and the test that must then fail with every one of the listed texts.
 */
interface Guard {
  name: string;
  test: string;
  file: string;
  before: string;
  after: string;
  failure: string[];
}

const results: RunResult[] = [];

/**
 Runs one test in the bounded container and compares the outcome with `expect`.
 Tests of the library's own modules run with `--lib`; the others are in the `language` integration test.

 @param name - label of the run, used for its log file

 @param test - full test path

 @param expect - 'pass', or the texts a failure must contain

 @throws {Error} when the outcome is not the expected one
 */
function run({
  name,
  test,
  expect
}: {
  name: string;
  test: string;
  expect: 'pass' | string[];
}): void {
  const target = test.startsWith('logging::') || test.startsWith('language::') ? ['--lib'] : [
    '--test',
    'language'
  ];
  const command = [
    'cargo',
    'test',
    '--offline',
    '--no-default-features',
    ...target,
    test,
    '--',
    '--exact',
    '--nocapture',
    '--include-ignored'
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
      `${name  }.log`
    ),
    output
  );
  if (result.error) throw result.error;
  const accepted = expect === 'pass'
    ? (result.status === 0) && output.includes(test)
      && output.includes('test result: ok. 1 passed')
    : (result.status !== 0) && output.includes(test)
      && expect.every(text => output.includes(text));
  results.push({
    name,
    test,
    expect,
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
  if (!accepted) throw new Error(`Unexpected result for ${  name
     }; inspect ${
     artifact}`);
  console.log(
    JSON.stringify(results.at(-1)),
  );
}

/**
 Applies one text replacement to a file for the duration of `body`.

 @param path - file to edit

 @param before - anchor to replace

 @param after - replacement

 @param body - work to run while the edit is in place

 @throws {Error} when the anchor is absent or ambiguous
 */
function mutated({
  path,
  before,
  after,
  body
}: {
  path: string;
  before: string;
  after: string;
  body: () => void;
}): void {
  const original = readFileSync(
    path,
    'utf8'
  );
  try {
    writeFileSync(
      path,
      replaceOne({
        text: original,
        before,
        after
      })
    );
    body();
  } finally { writeFileSync(
    path,
    original
  ); }
}

const relabel = 'src/logging/relabel.rs';
const cases: Guard[] = [
  // A server's standard-error line, as TypeScript 7's server writes `context canceled`, reaches ERROR again.
  {
    name: 'relabel-server-stderr-line',
    test: strict,
    file: relabel,
    before: '        return Some(Matched {\n            shape: Shape::ServerStderrLine,\n            server: name,\n            line,\n        });\n',
    after: '        let _mutant_drops = line;\n        return None;\n',
    failure: [
      'a clean lifetime logged at ERROR',
      String.raw`scripted-ls err <- \"context canceled`
    ]
  },
  // The end of a server's standard error, at every server exit, reaches ERROR again.
  {
    name: 'relabel-end-of-server-stderr',
    test: strict,
    file: relabel,
    before: '        return Some(Matched {\n            shape: Shape::EndOfServerStderr,\n            server: name,\n            line: "",\n        });\n',
    after: '        return None;\n',
    failure: [
      'a clean lifetime logged at ERROR',
      'scripted-ls err: <- StreamClosed'
    ]
  },
  // A `-32801` answer that the worker asks again for reaches ERROR again.
  {
    name: 'relabel-moot-answer',
    test: strict,
    file: relabel,
    before: '            return Some(Matched {\n                shape: Shape::MootAnswer,\n                server: name,\n                line: "",\n            });\n',
    after: '            return None;\n',
    failure: [
      'a clean lifetime logged at ERROR',
      'ServerError(-32801): content modified'
    ]
  },
  // Every transport ERROR record is lowered, so a real request failure no longer shows as an error.
  {
    name: 'relabel-every-transport-error',
    test: unknown,
    file: relabel,
    before: '    if level != log::Level::Error || target != TRANSPORT_TARGET {\n        return None;\n    }',
    after: '    if level == log::Level::Error {\n        return Some(Matched { shape: Shape::MootAnswer, server: "", line: "" });\n    }',
    failure: [
      'no ERROR record ending with',
      'InternalError: scripted initialize failure'
    ]
  },
  // A full log queue waits for room instead of dropping, so a stalled output stalls the logging thread.
  {
    name: 'log-queue-waits-when-full',
    test: stalled,
    file: 'src/logging/background.rs',
    before: '            self.shared.queued.fetch_sub(size, Ordering::SeqCst);\n            self.shared.lose(Loss {\n                records: 1,\n                bytes: size as u64,\n            });\n            return;\n',
    after: '            self.shared.queued.fetch_sub(size, Ordering::SeqCst);\n            while self.shared.queued.load(Ordering::SeqCst).saturating_add(size) > self.shared.budget {\n                thread::sleep(Duration::from_millis(1));\n            }\n            self.shared.queued.fetch_add(size, Ordering::SeqCst);\n',
    failure: ['logging waited for the stalled output']
  },
  // The worker drops its servers without asking them to shut down.
  {
    name: 'no-shutdown-request',
    test: strict,
    file: 'src/language/worker.rs',
    before: 'client.force_shutdown();',
    after: '',
    failure: ['the server was not asked to shut down']
  },
  // The worker asks, then drops the registry without waiting for the processes to end.
  {
    name: 'no-wait-for-exit',
    test: strict,
    file: 'src/language/worker.rs',
    before: 'while running > 0 {',
    after: 'while false {',
    failure: ['the server was not asked to shut down']
  },
  // The worker thread ends right after its runtime, without reaping the server it had to kill. The child is usually
  // a zombie by the time the test looks; on a busy host it can still be running (state R), killed but not yet ended.
  // The log bridge no longer keeps a server's standard-error lines, so the crash record has none.
  {
    name: 'stderr-tail-not-kept',
    test: crashed,
    file: 'src/logging/stderr_tail.rs',
    before: '    tail.lines.push_back(line);\n',
    after: '    let _mutant_drops = line;\n',
    failure: [
      'the ended-server record lacks the server\'s last lines',
      'was_ready=true\n'
    ]
  },
  // The crash record is written at once instead of waiting for the end of the server's standard error.
  {
    name: 'stderr-tail-no-settle',
    test: settled,
    file: 'src/language/attach/report.rs',
    before: 'while !stderr_tail::is_closed(&record.server) && started.elapsed() < SETTLE {',
    after: 'while false && started.elapsed() < SETTLE {',
    failure: ['the record did not wait for the last line']
  },
  // A clean shutdown reports each server's kept lines as if it had crashed.
  {
    name: 'stderr-tail-on-clean-shutdown',
    test: strict,
    file: 'src/language/worker.rs',
    before: '            crate::logging::stderr_tail::forget(client.name());\n',
    after: '            tracing::warn!(server = client.name(), stderr_tail = ?crate::logging::stderr_tail::take(client.name()), "language server process ended");\n',
    failure: [
      'a clean shutdown logged the server\'s last lines',
      'scripted server starting'
    ]
  },
  {
    name: 'no-reap-after-kill',
    test: killed,
    file: 'src/language/worker.rs',
    before: 'reap::finish(REAP_GRACE);',
    after: '',
    failure: [
      'a child process was left when the drop returned',
      'ide-scripted-ls'
    ]
  },
];
for (const name of only ?? []) if (!cases.some(item => item.name === name)) throw new Error(`Unknown case: ${  name}`);
const selected = cases.filter(item => (!only) || only.has(item.name));
// Unmodified sources: every test a selected case relies on passes.
for (const test of new Set(selected.map(item => item.test))) run({
  name: `baseline-${  test.split('::')
    .at(-1)}`,
  test,
  expect: 'pass'
});
for (const item of selected) {
  mutated({
    path: join(
      source,
      item.file
    ),
    before: item.before,
    after: item.after,
    body: () =>{  run({
      name: `${item.name  }-removed`,
      test: item.test,
      expect: item.failure
    }); }
  });
  run({
    name: `${item.name  }-restored`,
    test: item.test,
    expect: 'pass'
  });
}
console.log(`Language lifecycle and logging guard controls passed: ${  artifact  } (${  results.length  } runs)`);
