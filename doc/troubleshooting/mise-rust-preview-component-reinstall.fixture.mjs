#!/usr/bin/env node
import { appendFileSync } from 'node:fs';
import { basename } from 'node:path';

const executable = basename(process.argv[1] ?? '');
const args = process.argv
  .slice(2);
const logPath = process.env
  .FAKE_RUST_LOG;
if (logPath) appendFileSync(
  logPath,
  `${executable} ${args.join(' ')}\n`
);

if (executable === 'rustc') {
  process.stdout
    .write('rustc 1.100.0-nightly (fixture 2026-09-11)\n');
} else if ((args[0] === 'show') && (args[1] === 'profile')) {
  process.stdout
    .write('default\n');
} else if ((args[0] === 'component') && (args[1] === 'list')) {
  process.stdout
    .write(`${[
    'clippy-x86_64-unknown-linux-gnu',
    'llvm-tools-x86_64-unknown-linux-gnu',
    'rust-docs-x86_64-unknown-linux-gnu',
    'rust-src',
    'rustfmt-x86_64-unknown-linux-gnu',
  ].join('\n')  }\n`);
} else if ((args[0] === 'target') && (args[1] === 'list')) {
  process.stdout
    .write('x86_64-unknown-linux-gnu\n');
} else if ((args[0] === 'toolchain') && (args[1] === 'install')) {
  process.stderr
    .write('info: fixture toolchain unchanged\n');
} else {
  process.stderr
    .write(`unsupported fixture invocation: ${executable} ${args.join(' ')}\n`);
  process.exitCode = 2;
}
