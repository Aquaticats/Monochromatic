#!/usr/bin/env node
/** Run the embedding fuzzer with baked source, seeds and toolchain, retaining corpus and crash evidence. */
import {
  rm,
  writeFile
} from 'node:fs/promises';
import { join } from 'node:path';
import {
  run,
  snapshot,
  ScannerVerificationError
} from '../../cli/bin/container-snapshot.ts';

/** Bound both the fuzz campaign and every input while leaving reproducer evidence intact. */
async function main() {
  const options = process.argv
    .slice(2);
  if ((options.length > 1) || ((options.length === 1) && (options[0] !== '--clippy')))
    throw new ScannerVerificationError('Only --clippy is accepted.');
  const clippy = options[0] === '--clippy';
  const command = [
    'cargo',
    'fuzz',
    'run',
    '--fuzz-dir',
    '.',
    'fuzz_embedding',
    '--sanitizer',
    'address',
    '--target',
    'x86_64-unknown-linux-gnu',
    '--codegen-units',
    '16',
    'seed/fuzz_embedding',
    '--',
    '-max_total_time=120',
    '-max_len=20000',
    '-timeout=10',
    '-rss_limit_mb=1536',
    '-seed=94',
    '-dict=dictionary/forbidden-strings.dict',
  ];
  const effectiveCommand = clippy
    ? [
      'cargo',
      'clippy',
      '--offline',
      '--locked',
      '--all-targets',
      '--',
      '--deny',
      'warnings'
    ] : command;
  const fixture = await snapshot({
    command: effectiveCommand,
    name: clippy ? 'fuzz-clippy' : 'embedding-fuzz',
    toolchain: true,
    tool: clippy ? undefined : 'cargo-fuzz',
    fuzzing: true
  });
  let container;
  let evidencePreserved = false;
  try {
    if (clippy) {
      const result = await run({
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
      if (result.status !== 0) throw new ScannerVerificationError(`Fuzz Clippy exited ${result.status}; inspect ${fixture.evidence}.`);
      return;
    }
    container = (await run({
      command: 'podman',
      args: [
        'create',
        ...fixture.limits,
        fixture.image
      ],
      capture: true
    }))
      .stdout
      .trim();
    const result = await run({
      command: 'podman',
      args: [
        'start',
        '--attach',
        container
      ],
      allowFailure: true,
      transcript: fixture.evidence
    });
    const copied = {};
    for (const directory of [
      'seed/fuzz_embedding',
      'artifacts'
    ]) {
      const copy = await run({
        command: 'podman',
        args: [
          'cp',
          `${container}:/work/package/forbidden-strings/cli.fuzz/${directory}`,
          join(
            fixture.evidence,
            directory === 'artifacts' ? 'artifacts' : 'corpus'
          )
        ],
        capture: true,
        allowFailure: true
      });
      copied[directory] = {
        status: copy.status,
        stderr: copy.stderr
      };
    }
    evidencePreserved = (copied['seed/fuzz_embedding']
      .status
      === 0) && (copied.artifacts
        .status
        === 0);
    await writeFile(
      join(
        fixture.evidence,
        'exit.json'
      ),
      `${JSON.stringify(
        {
          status: result.status,
          signal: result.signal,
          copied
        },
        null,
        2
      )  }\n`
    );
    if ((result.status !== 0) || (!evidencePreserved))
      throw new ScannerVerificationError(`Embedding fuzzing exited ${result.status}; inspect ${fixture.evidence}.`);
  } finally {
    if (container && evidencePreserved) await run({
      command: 'podman',
      args: [
        'rm',
        '--force',
        container
      ],
      capture: true
    });
    else if (container) console.error(`Fuzz evidence retrieval failed; disposable container retained for recovery: ${container}`);
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
