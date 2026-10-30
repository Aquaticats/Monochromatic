#!/usr/bin/env node
/** Independently validate exact guard failure causes on the already tested immutable guard images. */
import { createHash } from 'node:crypto';
import {
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  rm,
  stat,
  writeFile
} from 'node:fs/promises';
import {
  join,
  resolve
} from 'node:path';
import {
  run,
  ScannerVerificationError
} from './container-snapshot.ts';
import { mkdirSync as makeScratchBase } from 'node:fs';

// Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
// whose backing device is unknowable.
const scratchBase = join(import.meta.dirname, '..', 'node_modules', '.monochromatic', 'forbidden-strings');
makeScratchBase(scratchBase, { recursive: true });


/** Select only a terminal, expected control result; incomplete or previously failed campaigns do not qualify. */
async function control({
  root,
  variant
}) {
  const candidates = [];
  for (const entry of await readdir(
    root,
    { withFileTypes: true }
  )) {
    if ((!entry.isDirectory()) || (!entry.name
      .startsWith(`guard-${variant}-`))) continue;
    const directory = join(
      root,
      entry.name
    );
    try {
      const verdict = JSON.parse(await readFile(
        join(
          directory,
          'control.json'
        ),
        'utf8'
      ));
      if (!verdict.expected) continue;
      const manifest = JSON.parse(await readFile(
        join(
          directory,
          'manifest.json'
        ),
        'utf8'
      ));
      candidates.push({
        directory,
        manifest,
        time: (await stat(join(
          directory,
          'control.json'
        ))).mtimeMs
      });
    } catch (error) { console.log(`Control not eligible: ${directory}: ${error.message}`); }
  }
  candidates.sort((
    first,
    second
  ) => second.time - first.time);
  if (!candidates[0]) throw new ScannerVerificationError(`No completed ${variant} control exists.`);
  return candidates[0];
}

/** Host hooks are counted per operation; both executable cases and real parallel CLI fault paths run independently. */
async function main() {
  if (process.argv
    .length
    !== 2) throw new ScannerVerificationError('This task accepts no arguments.');
  const root = resolve(
    import.meta.dirname,
    '../target/verification'
  );
  const original = await readFile(join(
    import.meta.dirname,
    'guard-fixture.rs.txt'
  ));
  const observer = await readFile(join(
    import.meta.dirname,
    'guard-observer.rs.txt'
  ));
  const originalSha256 = createHash('sha256')
    .update(original)
    .digest('hex');
  const observerSha256 = createHash('sha256')
    .update(observer)
    .digest('hex');
  const variants = [
    'protected',
    'without-output-hook',
    'without-process-catch',
    'without-load-catch',
    'without-content-catch',
    'without-name-catch'
  ];
  for (const variant of variants) {
    const selected = await control({
      root,
      variant
    });
    const source = selected.manifest
      .sources
      .find(item => item.path === 'package/forbidden-strings/cli/tests/panic_contract.rs');
    if (source?.sha256 !== originalSha256) throw new ScannerVerificationError(`Consumer assertions differ in ${selected.directory}.`);
    const context = await mkdtemp(join(
      scratchBase,
      'scanner-guard-observer-'
    ));
    const evidence = await mkdtemp(join(
      root,
      `guard-observer-${variant}-`
    ));
    try {
      await writeFile(
        join(
          context,
          'panic_contract.rs'
        ),
        Buffer.concat([
          original,
          Buffer.from('\n'),
          observer
        ])
      );
      const command = [
        'cargo',
        'test',
        '--offline',
        '--locked',
        '--test',
        'panic_contract',
        'boundary_observer::observations',
        '--',
        '--exact',
        '--nocapture',
        '--test-threads=1'
      ];
      await writeFile(
        join(
          context,
          'Containerfile'
        ),
        [
          `FROM ${selected.manifest
            .image}`,
          'COPY panic_contract.rs /work/package/forbidden-strings/cli/tests/panic_contract.rs',
          `CMD ${JSON.stringify(command)}`,
          '',
        ].join('\n')
      );
      const imageFile = join(
        context,
        'image.id'
      );
      await run({
        command: 'podman',
        args: [
          'build',
          '--network=none',
          '--http-proxy=false',
          '--pull=never',
          '--memory=2g',
          '--cpu-period=100000',
          '--cpu-quota=200000',
          '--iidfile',
          imageFile,
          context
        ]
      });
      const image = (await readFile(
        imageFile,
        'utf8'
      )).trim();
      const {limits} = selected.manifest;
      await writeFile(
        join(
          evidence,
          'manifest.json'
        ),
        `${JSON.stringify(
          {
            variant,
            baseControl: selected.directory,
            baseImage: selected.manifest
              .image,
            image,
            originalSha256,
            observerSha256,
            command,
            limits
          },
          null,
          2
        )  }\n`
      );
      const result = await run({
        command: 'podman',
        args: [
          'run',
          '--rm',
          ...limits,
          '--env',
          'SCANNER_GUARD_HOME=/home/tester',
          '--env',
          'FORBIDDEN_STRINGS_CACHE_DIR=/home/tester/cache',
          image
        ],
        allowFailure: true,
        transcript: evidence
      });
      const stdout = await readFile(
        join(
          evidence,
          'stdout.log'
        ),
        'utf8'
      );
      const observations = {
        content: variant === 'without-content-catch' ? 'unwound' : 'engine-error',
        name: variant === 'without-name-catch' ? 'unwound' : 'engine-error',
        load: variant === 'without-load-catch' ? 'unwound' : 'error',
        'hook-calls': 7,
        'startup-exit': variant === 'without-process-catch' ? 101 : 2,
        'worker-exit': variant === 'without-process-catch' ? 101 : 2,
        'startup-payload': variant === 'without-output-hook',
        'worker-payload': variant === 'without-output-hook',
        'runtime-content-exit': variant === 'without-content-catch' ? 2 : 1,
        'runtime-name-exit': variant === 'without-name-catch' ? 2 : 1,
        'runtime-content-payload': variant === 'without-output-hook',
        'runtime-name-payload': variant === 'without-output-hook',
      };
      const matches = Object.entries(observations)
        .every(([key, value]) => stdout.includes(`observation-${key}=${value}\n`));
      const expected = (result.status === 0) && stdout.includes('1 passed')
        && matches;
      await writeFile(
        join(
          evidence,
          'control.json'
        ),
        `${JSON.stringify(
          {
            variant,
            status: result.status,
            expected,
            observations
          },
          null,
          2
        )  }\n`
      );
      if (!expected) throw new ScannerVerificationError(`Exact guard observation failed: ${variant}; inspect ${evidence}.`);
    } finally { await rm(
      context,
      {
        recursive: true,
        force: true
      }
    ); }
  }
}

await main();
