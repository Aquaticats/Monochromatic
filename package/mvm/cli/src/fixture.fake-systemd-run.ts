/**
 Program text and file names of the stand-in `systemd-run` this package's tests
 put on `PATH`, and the names it shares with the fake `virsh`.

 The stand-in starts nothing. It records its arguments and leaves a marker in
 the fake virsh's state directory, which stands for the session daemon's unit
 being up; the fake `virsh` answers once that marker exists. A second start of
 the same unit fails the way systemd fails it, so a test can watch two
 processes race for the one unit name.

 A test that needs a start refused while the unit name is still loaded, and a
 later start accepted, writes the outcomes it wants into the state directory;
 the stand-in consumes them in order.

 @module
 */

/**
 Name of the file in the fake virsh's state directory that stands for the started unit.
 */
export const SESSION_DAEMON_UNIT_MARKER = 'session-daemon-unit.json';

/**
 Name of the file in the fake virsh's state directory that records every stand-in invocation.
 */
export const SYSTEMD_RUN_CALLS_FILE = 'systemd-run-calls.jsonl';

/**
 Name of the file in the fake virsh's state directory that scripts the next starts.
 */
export const SYSTEMD_RUN_OUTCOMES_FILE = 'systemd-run-outcomes.json';

/**
 One scripted answer of the stand-in.

 - `accepted`: the start succeeds, even when the marker already exists.
 - `refused`: systemd's "already loaded" answer, and the marker is left as it is.
 - `exclusive`: the default; the start succeeds unless the marker already exists.
 */
export type SystemdRunOutcome = 'accepted' | 'exclusive' | 'refused';

/**
 Milliseconds the stand-in waits before it claims the unit name, so that two
 starts launched together overlap the way two real starts do; a real
 `systemd-run` returns in about a hundred milliseconds.
 */
const CLAIM_DELAY_MS = 1_000;

/**
 Variable that shortens that wait, so a test can watch several retries inside a
 deadline shorter than the default wait.
 */
export const CLAIM_DELAY_ENV = 'MVM_FAKE_SYSTEMD_RUN_DELAY_MS';

/**
 JavaScript body of the stand-in, for `writeExecutable` in the CLI harness.

 @returns Program text, without the leading interpreter line

 @example
 ```ts
 await writeExecutable({ body: systemdRunBody(), name: 'systemd-run', sandbox });
 ```
 */
export function systemdRunBody(): string {
  return [
    'const { appendFileSync, readFileSync, writeFileSync } = require(\'node:fs\');',
    'const { join } = require(\'node:path\');',
    'const args = process.argv.slice(2);',
    // A real `systemd-run --version` prints its version and exits without starting or contacting anything.
    'if (args.includes(\'--version\')) {',
    String.raw`  process.stdout.write('systemd 262 (262.2-1)\n');`,
    '  process.exit(0);',
    '}',
    'const directory = process.env.MVM_FAKE_VIRSH_DIR;',
    'if (directory === undefined) { throw new Error("MVM_FAKE_VIRSH_DIR is not set; the stand-in systemd-run needs its state directory"); }',
    `const claimDelayMs = Number(process.env[${JSON.stringify(CLAIM_DELAY_ENV,)}] ?? ${String(CLAIM_DELAY_MS,)});`,
    `const outcomesFile = join(directory, ${
      JSON.stringify(SYSTEMD_RUN_OUTCOMES_FILE,)
    });`,
    `const markerFile = join(directory, ${
      JSON.stringify(SESSION_DAEMON_UNIT_MARKER,)
    });`,
    'const unitArgument = args.find((argument) => argument.startsWith(\'--unit=\'));',
    'const unit = unitArgument === undefined ? \'\' : unitArgument.slice(\'--unit=\'.length);',
    'let outcome = \'exclusive\';',
    'try {',
    '  const remaining = JSON.parse(readFileSync(outcomesFile, \'utf8\'));',
    '  if (Array.isArray(remaining) && remaining.length > 0) {',
    '    outcome = remaining.shift();',
    '    writeFileSync(outcomesFile, JSON.stringify(remaining));',
    '  }',
    '}',
    'catch (error) {',
    '  if (error.code !== \'ENOENT\') { throw error; }',
    '}',
    // A blocking wait, not a busy loop: the stand-in adds no host load while it waits.
    'Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, claimDelayMs);',
    'let accepted = false;',
    'if (outcome === \'accepted\') {',
    '  writeFileSync(markerFile, JSON.stringify({ args, unit }));',
    '  accepted = true;',
    '}',
    'else if (outcome !== \'refused\') {',
    '  try {',
    '    writeFileSync(markerFile, JSON.stringify({ args, unit }), { flag: \'wx\' });',
    '    accepted = true;',
    '  }',
    '  catch (error) {',
    '    if (error.code !== \'EEXIST\') { throw error; }',
    '  }',
    '}',
    `appendFileSync(join(directory, ${
      JSON.stringify(SYSTEMD_RUN_CALLS_FILE,)
    }), JSON.stringify({ accepted, args }) + '\\n');`,
    'if (accepted) {',
    String.raw`  process.stdout.write('Running as unit: ' + unit + '.service\n');`,
    '}',
    'else {',
    "  process.stderr.write('Failed to start transient service unit: Unit ' + unit",
    String.raw`    + '.service was already loaded or has a fragment file.\n');`,
    '  process.exitCode = 1;',
    '}',
  ].join('\n',);
}
