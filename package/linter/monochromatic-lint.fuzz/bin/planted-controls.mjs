#!/usr/bin/env node
/**
 Positive controls for the fuzz invariants.
 Plants one defect at a time in a copy of the linter and requires the generator controls,
 which call the same invariant functions as the fuzz targets, to fail.
 The worktree is only read; every edit and build happens in this package's ignored `target/planted` tree.
 */
import { spawnSync } from 'node:child_process';
import {
  cp,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile
} from 'node:fs/promises';
import {
  join,
  resolve
} from 'node:path';

/** Preserve the failed verification boundary rather than masking it with a cleanup error. */
class PlantedControlError extends Error {}

/** Repository-relative inputs the helper library needs to build. */
const inputs = [
  'package/linter/monochromatic-lint/Cargo.toml',
  'package/linter/monochromatic-lint/Cargo.lock',
  'package/linter/monochromatic-lint/README.md',
  'package/linter/monochromatic-lint/src',
  'package/linter/monochromatic-lint/fixtures',
  'package/linter/monochromatic-lint.fuzz/Cargo.toml',
  'package/linter/monochromatic-lint.fuzz/Cargo.lock',
  'package/linter/monochromatic-lint.fuzz/src',
  'package/linter/monochromatic-lint.fuzz/fuzz_targets',
  'package/jsonc-edit/core-rust/Cargo.toml',
  'package/jsonc-edit/core-rust/Cargo.lock',
  'package/jsonc-edit/core-rust/src',
  'package/jsonc-edit/core-rust/fixtures',
  'package/jsonc-edit/core-rust.fuzz/Cargo.toml',
  'package/jsonc-edit/core-rust.fuzz/Cargo.lock',
  'package/jsonc-edit/core-rust.fuzz/src',
  'package/jsonc-edit/core-rust.fuzz/fuzz_targets',
];

/**
 Defects with observable behavior, one per fuzz target.
 Each `from` must occur exactly once so a control cannot silently stop applying.
 */
const plants = [
  {
    name: 'configuration: the warn severity is rejected',
    file: 'package/linter/monochromatic-lint/src/configuration_rules.rs',
    edits: [{
      from: 'if severity != "off" && severity != "warn" && severity != "error" {',
      to: 'if severity != "off" && severity != "error" {',
    }],
  },
  {
    name: 'merge_values: a later array replaces an earlier one instead of concatenating',
    file: 'package/linter/monochromatic-lint/src/config_merge.rs',
    edits: [{
      from: 'merged.extend_from_slice(elements);',
      to: 'merged = elements.to_vec();'
    }],
  },
  {
    name: 'rust_style: a file named like a sidecar is still reported',
    file: 'package/linter/monochromatic-lint/src/rust_tests_in_sidecars.rs',
    edits: [{
      from: 'stem == "tests" || stem.ends_with("_tests")',
      to: 'stem == "tests"'
    }],
  },
  {
    name: 'rust_explicit_types: an initialized binding needs no annotation',
    file: 'package/linter/monochromatic-lint/src/rust_explicit_declarations.rs',
    edits: [{
      from: 'if statement.ty().is_some() {',
      to: 'if statement.ty().is_some() || statement.initializer().is_some() {',
    }],
  },
  {
    name: 'markdown: a trailing colon is not heading punctuation',
    file: 'package/linter/monochromatic-lint/src/markdown_punctuation.rs',
    edits: [{
      from: "if character != '.' && character != ':' {",
      to: "if character != '.' {"
    }],
  },
  {
    name: 'orchestration: an unlabeled rustdoc fence is not a doc test',
    file: 'package/linter/monochromatic-lint/src/processors_fences.rs',
    edits: [{
      from: '            && !(rustdoc && language.is_empty())\n',
      to: ''
    }],
  },
];

/** Run the generator controls in the copied sidecar; report status and the failing control names. */
function controls({
  root,
  targetDirectory
}) {
  const run = spawnSync(
    'cargo',
    [
      'nextest',
      'run',
      '--lib',
      '--offline',
      '--locked'
    ],
    {
      cwd: join(
        root,
        'package/linter/monochromatic-lint.fuzz'
      ),
      encoding: 'utf8',
      env: {
        ...process.env,
        CARGO_TARGET_DIR: targetDirectory,
        CARGO_BUILD_JOBS: '2'
      },
      maxBuffer: 64 * 1_024
        * 1_024,
    }
  );
  if (run.error)
    throw new PlantedControlError(
      'cargo could not start.',
      { cause: run.error }
    );
  const output = `${run.stdout}\n${run.stderr}`;
  return {
    status: run.status,
    // nextest prints `FAIL [ 0.012s] ( 3/38) <binary id> <test name>`, or `FAIL + LEAK [` when the failed test left a
    // child process behind, and repeats each failure in its closing recap; the test name ends the line.
    failed: [...new Set([...output.matchAll(/^\s*FAIL\b[^[]*\[.*\s(\S+)$/gmu)].map(match => match[1]))],
    passed: [...output.matchAll(/^\s*PASS\b[^[]*\[/gmu)].length,
    output,
  };
}

/** Apply one plant's edits to its file text, refusing an edit that does not match exactly once. */
function planted({
  plant,
  original
}) {
  let text = original;
  for (const edit of plant.edits) {
    const occurrences = text.split(edit.from)
      .length
      - 1;
    if (occurrences !== 1)
      throw new PlantedControlError(`Planted control "${plant.name}" matches ${occurrences} times in ${plant.file}; it must match exactly once.`);
    text = text.replace(
      edit.from,
      edit.to
    );
  }
  return text;
}

/** Copy the inputs, prove the unplanted controls pass, then require every plant to fail a control. */
async function main() {
  const repository = resolve(
    import.meta.dirname,
    '../../../..'
  );
  // A fixed copy location keeps Cargo's dependency builds reusable between runs of this task.
  const work = join(
    process.cwd(),
    'target',
    'planted'
  );
  const root = join(
    work,
    'tree'
  );
  const targetDirectory = join(
    work,
    'target'
  );
  await rm(
    root,
    {
      recursive: true,
      force: true
    }
  );
  const evidenceRoot = join(
    process.cwd(),
    'target',
    'verification'
  );
  await mkdir(
    evidenceRoot,
    { recursive: true }
  );
  const evidence = await mkdtemp(join(
    evidenceRoot,
    'planted-'
  ));
  try {
    for (const input of inputs)
      await cp(
        join(
          repository,
          input
        ),
        join(
          root,
          input
        ),
        { recursive: true }
      );
    const baseline = controls({
      root,
      targetDirectory
    });
    await writeFile(
      join(
        evidence,
        'baseline.log'
      ),
      baseline.output
    );
    if ((baseline.status !== 0) || (baseline.passed === 0))
      throw new PlantedControlError(`Unplanted generator controls did not pass; inspect ${evidence}.`);
    console.log(`Unplanted controls: ${baseline.passed} passed`);
    const results = [];
    for (const plant of plants) {
      const path = join(
        root,
        plant.file
      );
      const original = await readFile(
        path,
        'utf8'
      );
      await writeFile(
        path,
        planted({
          plant,
          original
        })
      );
      const run = controls({
        root,
        targetDirectory
      });
      await writeFile(
        path,
        original
      );
      // A compile error is not an observation: the plant must build and a named control must fail.
      const noticed = (run.status !== 0) && (run.failed
        .length
        > 0);
      results.push({
        name: plant.name,
        file: plant.file,
        edits: plant.edits,
        status: run.status,
        failed: run.failed,
        noticed
      });
      await writeFile(
        join(
          evidence,
          `planted-${results.length - 1}.log`
        ),
        run.output
      );
      console.log(`Planted defect "${plant.name}": ${noticed ? `noticed by ${run.failed
        .join(', ')}` : 'NOT noticed'}`);
    }
    await writeFile(
      join(
        evidence,
        'results.json'
      ),
      `${JSON.stringify(
        {
          baselinePassed: baseline.passed,
          results
        },
        null,
        2
      )  }\n`
    );
    console.log(`Planted-control evidence: ${evidence}`);
    const unnoticed = results.filter(result => !result.noticed);
    if (unnoticed.length > 0)
      throw new PlantedControlError(`Planted defects were not noticed: ${unnoticed.map(result => result.name)
        .join('; ')}.`);
  } finally {
    await rm(
      root,
      {
        recursive: true,
        force: true
      }
    );
  }
}

await main();
