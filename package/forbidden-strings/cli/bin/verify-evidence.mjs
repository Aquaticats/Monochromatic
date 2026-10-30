#!/usr/bin/env node
/** Compare retained manifests with current compiled inputs and summarize exact terminal evidence. */
import { createHash } from 'node:crypto';
import {
  mkdir,
  readFile,
  readdir,
  stat,
  writeFile
} from 'node:fs/promises';
import { run } from './container-snapshot.mjs';
import {
  join,
  resolve
} from 'node:path';

/** Enumerate both sides of the copied source/build/test input set, including newly added current files. */
async function currentInputs({
  repository,
  fuzzing
}) {
  const scanner = 'package/forbidden-strings/cli';
  const engine = 'package/forbidden-strings/core';
  const pending = [
    'clippy.toml',
    'forbidden-strings.append.txt',
    '.github/workflows/forbidden-strings.yml',
    ...[
      'Cargo.toml',
      'Cargo.lock',
      'build.rs',
      'src',
      'tests',
      'data'
    ].map(file => `${scanner}/${file}`),
    ...[
      'Cargo.toml',
      'Cargo.lock',
      'src'
    ].map(file => `${engine}/${file}`)
  ];
  if (fuzzing) pending.push(...[
    'Cargo.toml',
    'Cargo.lock',
    'src',
    'fuzz_targets',
    'seed',
    'dictionary'
  ].map(file => `${scanner}.fuzz/${file}`));
  const inputs = new Map();
  while (pending.length > 0) {
    const path = pending.pop();
    const absolute = join(
      repository,
      path
    );
    if ((await stat(absolute)).isDirectory()) {
      for (const name of await readdir(absolute)) pending.push(`${path}/${name}`);
    } else {
      const bytes = await readFile(absolute);
      inputs.set(
        path,
        createHash('sha256')
          .update(bytes)
          .digest('hex')
      );
    }
  }
  return inputs;
}

/** Report every mismatch rather than treating a Git HEAD as source-snapshot proof. */
async function main() {
  const repository = resolve(
    import.meta.dirname,
    '../../../..'
  );
  const roots = [
    resolve(
      import.meta.dirname,
      '../target/verification'
    ),
    resolve(
      import.meta.dirname,
      '../../cli.fuzz/target/verification'
    )
  ];
  const records = [];
  for (const root of roots) {
    for (const entry of await readdir(
      root,
      { withFileTypes: true }
    )) {
      if ((!entry.isDirectory()) || (entry.name === 'docs')) continue;
      const directory = join(
        root,
        entry.name
      );
      const manifestPath = join(
        directory,
        'manifest.json'
      );
      let manifest;
      try { manifest = JSON.parse(await readFile(
        manifestPath,
        'utf8'
      )); }
      catch (error) { console.log(`No readable manifest in ${directory}: ${error.message}`);
        continue; }
      if (!manifest.sources) continue;
      const differences = [];
      const inputs = await currentInputs({
        repository,
        fuzzing: manifest.sources
          .some(source => source.path
            .startsWith('package/forbidden-strings/cli.fuzz/'))
      });
      const testedPaths = new Set();
      for (const source of manifest.sources) {
        // Task scripts/docs do not enter Cargo's artifact. Guard-injected source intentionally differs and remains separately labeled.
        if (source.path
          .includes('/bin/')
          || source.path
            .endsWith('/mise.toml')) continue;
        testedPaths.add(source.path);
        const current = inputs.get(source.path);
        if (source.sha256 !== current) differences.push({
          path: source.path,
          tested: source.sha256,
          current
        });
      }
      for (const [path, current] of inputs) {
        if (!testedPaths.has(path)) differences.push({
          path,
          current,
          addedAfterSnapshot: true
        });
      }
      const endings = {};
      for (const file of [
        'exit.json',
        'control.json',
        'mutants.out/outcomes.json'
      ]) {
        try { endings[file] = JSON.parse(await readFile(
          join(
            directory,
            file
          ),
          'utf8'
        )); }
        catch (error) { endings[file] = { unavailable: error.message }; }
      }
      const timestamp = (await stat(manifestPath)).mtime
        .toISOString();
      records.push({
        directory,
        timestamp,
        image: manifest.image,
        sourceSha256: manifest.sourceSha256,
        command: manifest.command,
        sources: manifest.sources,
        differences,
        endings
      });
      console.log(`${entry.name}: ${differences.length} compiled-input differences; ${endings['exit.json']
        .status
        ?? endings['control.json']
          .status
        ?? 'no terminal status'}`);
    }
  }
  const guarded = records.filter(record => (record.endings['control.json']
    .variant
    === 'protected')
    && record.endings['control.json']
      .expected)
    .sort((
      first,
      second
    ) => second.timestamp
      .localeCompare(first.timestamp))[0];
  const guardComparisons = [];
  const mainSources = new Map();
  const extraction = join(
    roots[0],
    'guard-source-comparison'
  );
  await mkdir(
    extraction,
    { recursive: true }
  );
  async function nativeMain(record) {
    if (mainSources.has(record.image)) return mainSources.get(record.image);
    const container = run({
      command: 'podman',
      args: [
        'create',
        '--network=none',
        '--memory=2g',
        '--cpus=2',
        '--pids-limit=128',
        record.image
      ],
      capture: true
    })
      .stdout
      .trim();
    const destination = join(
      extraction,
      `${container}.main.rs`
    );
    try {
      run({
        command: 'podman',
        args: [
          'cp',
          `${container}:/work/package/forbidden-strings/cli/src/main.rs`,
          destination
        ],
        capture: true
      });
      const source = await readFile(
        destination,
        'utf8'
      );
      mainSources.set(
        record.image,
        source
      );
      return source;
    } finally { run({
      command: 'podman',
      args: [
        'rm',
        container
      ],
      capture: true
    }); }
  }
  const testRegistration = '\n/// Process-isolated filter controls do not alter production startup or the embedding host\'s environment.\n#[cfg(test)]\n#[path = "main_logging_tests.rs"]\nmod logging_tests;\n';
  if (guarded) {
    for (const record of records) {
      if ((!record.directory
        .includes('/guard-without-')) || (!record.endings['control.json']
          .expected)) continue;
      const first = new Map(guarded.sources
        .filter(source => (!source.path
          .includes('/bin/')) && (!source.path
            .endsWith('/mise.toml')))
        .map(source => [
          source.path,
          source.sha256
        ]));
      const second = new Map(record.sources
        .filter(source => (!source.path
          .includes('/bin/')) && (!source.path
            .endsWith('/mise.toml')))
        .map(source => [
          source.path,
          source.sha256
        ]));
      const paths = new Set([
        ...first.keys(),
        ...second.keys()
      ]);
      const differences = [...paths].filter(path => first.get(path) !== second.get(path));
      let mainDifferenceOnlyTestRegistration;
      if (differences.includes('package/forbidden-strings/cli/src/main.rs') && (record.endings['control.json']
        .variant
        !== 'without-output-hook')) {
        const original = await nativeMain(guarded);
        const disabled = await nativeMain(record);
        mainDifferenceOnlyTestRegistration = original.replace(
          testRegistration,
          ''
        ) === disabled.replace(
          testRegistration,
          ''
        );
        if (!mainDifferenceOnlyTestRegistration) throw new Error(`Unexpected main implementation difference in ${record.directory}.`);
      }
      const comparison = {
        protected: guarded.directory,
        disabled: record.directory,
        differences,
        mainDifferenceOnlyTestRegistration
      };
      guardComparisons.push(comparison);
      console.log(`Guard input comparison ${record.directory}: ${differences.join(', ')}`);
    }
  }
  await writeFile(
    join(
      roots[0],
      'reconciliation.json'
    ),
    `${JSON.stringify(
      records,
      null,
      2
    )  }\n`
  );
  await writeFile(
    join(
      roots[0],
      'guard-input-comparisons.json'
    ),
    `${JSON.stringify(
      guardComparisons,
      null,
      2
    )  }\n`
  );
}

await main();
