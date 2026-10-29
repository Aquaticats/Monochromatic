#!/usr/bin/env node
/** Verify scanner binary and embedding consumers, or all-feature Clippy, in a disposable source snapshot. */
import {
  rm,
  writeFile
} from 'node:fs/promises';
import { join } from 'node:path';
import {
  run,
  snapshot,
  ScannerVerificationError
} from './container-snapshot.mjs';

/** Preserve exit evidence on both successful and failed verification. */
async function main() {
  const options = process.argv
    .slice(2);
  if ((options.length > 1) || ((options.length === 1) && (options[0] !== '--clippy')
    && (options[0] !== '--release')))
    throw new ScannerVerificationError('Only --clippy or --release is accepted.');
  const clippy = options[0] === '--clippy';
  const command = clippy
    ? [
      'cargo',
      'clippy',
      '--offline',
      '--locked',
      '--all-targets',
      '--all-features',
      '--',
      '--deny',
      'warnings'
    ]
    : [
      'cargo',
      'test',
      '--offline',
      '--locked',
      '--all-targets',
      '--all-features',
      '--',
      '--test-threads=1'
    ];
  if (options[0] === '--release') command.splice(
    2,
    0,
    '--release'
  );
  const fixture = await snapshot({
    command,
    name: clippy ? 'clippy' : options[0] === '--release' ? 'release-test' : 'test',
    toolchain: clippy
  });
  try {
    const result = run({
      command: 'podman',
      args: [
        'run',
        '--rm',
        ...fixture.limits,
        fixture.image
      ],
      allowFailure: true,
      transcript: fixture.evidence
    });
    await writeFile(
      join(
        fixture.evidence,
        'exit.json'
      ),
      `${JSON.stringify({
        status: result.status,
        signal: result.signal
      })  }\n`
    );
    if (result.status !== 0) throw new ScannerVerificationError(`Verification exited ${result.status}; inspect ${fixture.evidence}.`);
  } finally {
    await rm(
      fixture.context,
      {
        recursive: true,
        force: true
      }
    );
  }
}

await main();
