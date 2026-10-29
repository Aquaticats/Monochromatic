#!/usr/bin/env node
/** Mutate the embedding owners inside a mount-free source snapshot; preserve every outcome and diff. */
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

/** Each optional scope narrows only the mutated files; every scope runs the same baseline and complete consumer suite. */
const SCOPES = {
  '--startup': {
    name: 'startup-mutation',
    files: ['main.rs']
  },
  // Pathname normalization, native prefix counting and component masking, mutated without the other embedding owners.
  '--pathname': {
    name: 'pathname-mutation',
    files: [
      'path_scan.rs',
      'path_name_bytes.rs'
    ]
  },
};

/** Print the mutants a campaign with the same arguments would test; `stdout.log` then holds one mutant name per line. */
async function listMutants(fixture) {
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
        signal: result.signal,
        listed: true
      })  }\n`
    );
    if (result.status !== 0) throw new ScannerVerificationError(`Mutant listing exited ${result.status}; inspect ${fixture.evidence}.`);
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

/** Scoped mutations retain the baseline, all policy branches and the complete scanner consumer suite. */
async function main() {
  const options = process.argv
    .slice(2);
  // `--list` combines with any scope: it appends cargo-mutants' own `--list` to the unchanged campaign arguments.
  const listing = options.filter(option => option === '--list');
  const scopes = options.filter(option => option !== '--list');
  if ((listing.length > 1) || (scopes.length > 1)
    || ((scopes.length === 1) && (!Object.hasOwn(
      SCOPES,
      scopes[0]
    ))))
    throw new ScannerVerificationError(`Accepted: at most one of ${Object.keys(SCOPES)
      .join(', ')}, optionally with --list.`);
  const scope = scopes.length === 1 ? SCOPES[scopes[0]] : undefined;
  const list = listing.length === 1;
  const command = [
    'cargo',
    'mutants',
    '--in-place',
    '--all-features',
    '--baseline',
    'run',
    '--no-config',
    '--no-shuffle',
    '--colors=never',
    '--build-timeout',
    '300',
    '--timeout',
    '120',
    '--output',
    '/work/mutation-report',
    '--cargo-arg=--offline',
    '--cargo-arg=--locked',
    '--cargo-test-arg=--',
    '--cargo-test-arg=--test-threads=1',
    '--cargo-test-arg=--skip=rule::frx::compile_tests::builtin_ported_all_compile',
    '--cargo-test-arg=--skip=rule::frx::compile_tests::append_ported_compiles_end_to_end',
    '--exclude-re',
    String.raw`replace \+= with \*=`,
    '--exclude-re',
    'replace -= with /=',
  ];
  const files = scope ? scope.files : [
    'scanner.rs',
    'load_request.rs',
    'scan_finding.rs',
    'frx_scan.rs',
    'path_scan.rs',
    'path_name_bytes.rs',
    'frx_load.rs',
    'process_boundary.rs',
    'main.rs',
    'runtime_cache/mod.rs',
    'runtime_cache/warning.rs',
  ];
  for (const file of files) command.push(
    '--file',
    `src/${file}`
  );
  if (list) command.push('--list');
  const name = scope ? scope.name : 'mutation';
  const fixture = await snapshot({
    command,
    name: list ? `${name}-list` : name,
    tool: 'cargo-mutants'
  });
  if (list) {
    await listMutants(fixture);
    return;
  }
  let container;
  let reportPreserved = false;
  try {
    container = run({
      command: 'podman',
      args: [
        'create',
        ...fixture.limits,
        fixture.image
      ],
      capture: true
    })
      .stdout
      .trim();
    const result = run({
      command: 'podman',
      args: [
        'start',
        '--attach',
        container
      ],
      allowFailure: true,
      transcript: fixture.evidence
    });
    const report = run({
      command: 'podman',
      args: [
        'cp',
        `${container}:/work/mutation-report/.`,
        fixture.evidence
      ],
      capture: true,
      allowFailure: true
    });
    reportPreserved = report.status === 0;
    await writeFile(
      join(
        fixture.evidence,
        'exit.json'
      ),
      `${JSON.stringify(
        {
          status: result.status,
          signal: result.signal,
          reportCopied: report.status === 0,
          reportCopyError: report.stderr
        },
        null,
        2
      )  }\n`
    );
    if ((result.status !== 0) || (report.status !== 0))
      throw new ScannerVerificationError(`Mutation exited ${result.status}; report-copy status ${report.status}; inspect ${fixture.evidence}.`);
  } finally {
    if (container && reportPreserved) run({
      command: 'podman',
      args: [
        'rm',
        '--force',
        container
      ],
      capture: true
    });
    else if (container) console.error(`Report retrieval failed; disposable container retained for recovery: ${container}`);
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
