/**
 Program text and file names of the stand-in `systemd-run` this package's tests
 put on `PATH`, and the names it shares with the fake `virsh`.

 The stand-in starts nothing. It records its arguments and leaves a marker in
 the fake virsh's state directory, which stands for the session daemon's unit
 being up; the fake `virsh` answers once that marker exists. A second start of
 the same unit fails the way systemd fails it, so a test can watch two
 processes race for the one unit name.

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
 Milliseconds the stand-in waits before it claims the unit name, so that two
 starts launched together overlap the way two real starts do; a real
 `systemd-run` returns in about a hundred milliseconds.
 */
const CLAIM_DELAY_MS = 1_000;

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
    'const { appendFileSync, writeFileSync } = require(\'node:fs\');',
    'const { join } = require(\'node:path\');',
    'const directory = process.env.MVM_FAKE_VIRSH_DIR;',
    'if (directory === undefined) { throw new Error("MVM_FAKE_VIRSH_DIR is not set; the stand-in systemd-run needs its state directory"); }',
    `const claimDelayMs = ${String(CLAIM_DELAY_MS,)};`,
    'const args = process.argv.slice(2);',
    'const unitArgument = args.find((argument) => argument.startsWith(\'--unit=\'));',
    'const unit = unitArgument === undefined ? \'\' : unitArgument.slice(\'--unit=\'.length);',
    // A blocking wait, not a busy loop: the stand-in adds no host load while it waits.
    'Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, claimDelayMs);',
    'let accepted = true;',
    'try {',
    `  writeFileSync(join(directory, ${
      JSON.stringify(SESSION_DAEMON_UNIT_MARKER,)
    }), JSON.stringify({ args, unit }), { flag: 'wx' });`,
    '}',
    'catch (error) {',
    '  if (error.code !== \'EEXIST\') { throw error; }',
    '  accepted = false;',
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
