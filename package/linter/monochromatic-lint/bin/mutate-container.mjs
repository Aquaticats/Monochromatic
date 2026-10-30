#!/usr/bin/env node
/**
 Run cargo-mutants against an already tested, disposable container snapshot.
 Source mutation and compilation stay inside the container; only reports leave it.
 `--list <scope>` prints one scope's mutants on the host, and `--coverage` proves that the scopes together
 hold every mutant of the unscoped listing; both read the same scope table the campaigns use.
 */
import { spawnSync } from 'node:child_process';
import {
  mkdir,
  mkdtemp,
  writeFile
} from 'node:fs/promises';
import { join } from 'node:path';
import { mkdirSync as makeScratchBase } from 'node:fs';

import {
  campaignCommand,
  defaultBuildTimeoutSeconds,
  exclusionArguments,
  parseContainerLimitOptions,
  runCampaign,
} from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

// Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
// whose backing device is unknowable.
const scratchBase = join(import.meta.dirname, '..', 'node_modules', '.monochromatic', 'monochromatic-lint');
makeScratchBase(scratchBase, { recursive: true });


/** Concurrent worktrees set MONOCHROMATIC_LINT_IMAGE_TAG so one snapshot never runs another's image. */
const imageTag = process.env
  .MONOCHROMATIC_LINT_IMAGE_TAG
  ?? 'development';
/** Already tested source snapshot this campaign mutates. */
const testImage = `localhost/monochromatic-lint-test:${imageTag}`;
/** Campaign image layered over the tested snapshot. */
const mutationImage = `localhost/monochromatic-lint-mutation:${imageTag}`;

/**
 Full names of the five tests that load a real Cargo workspace through the semantic engine.
 Alone in the bounded container they took 43 to 139 seconds each, and the rest of the library suite 14 seconds
 (workspace-tests-cost-1.log), so the fast scopes skip exactly these. Each name is a whole test path that no other
 test contains, so the quick tests of the same modules still run. Record: doc/handover/unified-linter-mutation-close.md.
 */
const workspaceTests = [
  'rust_explicit_types_tests::semantic_conformance_and_source_overlay_controls',
  'rust_file_engine::tests::selected_semantics_reuses_the_manifest_session',
  'rust_inferred_constants::tests::holes_resolve_against_the_parameter_in_their_own_slot',
  'rust_workspace_tests::generated_definitions_and_build_failures_are_distinct',
  'run_directive_tests::semantic_findings_follow_the_same_directive_rules',
];

/** Per-mutant test limit, in seconds, for every scope whose own entry names none. */
const defaultTimeoutSeconds = 180;

/** One nextest `--skip` option pair, which excludes every test whose name contains the text. */
function skipArguments(name) {
  return [
    '--skip',
    name
  ];
}

/** One cargo-mutants `--file` option pair, which keeps the source files the glob matches. */
function fileArguments(glob) {
  return [
    '--file',
    glob
  ];
}

/**
 Every scope, keyed by its command-line option. `select` decides which mutants exist (it is all `--list` needs),
 `run` decides how each mutant is built and tested and comes last because it may end with libtest arguments,
 and `timeoutSeconds` is the per-mutant test limit when it differs from the default.
 cargo-mutants copies everything after its own `--` into the `cargo nextest run` command line, where nextest
 accepts several test-name filters; a second `--` hands the remaining arguments to nextest's own filter
 position, the only place it accepts `--skip` (probed: a `--skip` before it is an unexpected argument).
 */
const scopes = new Map([
  [
    '--rust-style',
    {
      description: 'the sidecar test rule against its own tests',
      select: fileArguments('src/rust_tests_in_sidecars.rs'),
      run: [
        '--',
        'rust_tests_in_sidecars'
      ],
    }
  ],
  // This is scoped evidence, not a replacement for full-rule mutation. The baseline still runs.
  // Every test the `markdown` filter selects is a library test: the baseline at c184147f ran 136 library tests
  // and filtered out all 12 `binary` tests. `--lib` (a Cargo argument, so the build phase gets it too) stops
  // every mutant from also rebuilding the executable and the `binary` test target, which then run no test.
  [
    '--markdown',
    {
      description: 'every Markdown module against the Markdown tests',
      select: fileArguments('src/markdown_*.rs'),
      run: [
        '--cargo-arg=--lib',
        '--cargo-test-arg=markdown'
      ],
    }
  ],
  // The parent lookup's own contract control, which never walks ancestors. Before ancestor walks were bounded by
  // the node count, a constant parent made them spin and the Markdown scope could only time out on these mutants;
  // the Markdown scope now catches them too, and this scope stays as their fastest direct check.
  [
    '--markdown-parent',
    {
      description: 'the Markdown parent lookup against its contract control',
      select: [
        ...fileArguments('src/markdown_source.rs'),
        '--re',
        'MarkdownSource::parent'
      ],
      run: ['--cargo-test-arg=parents_mirror_child_edges_and_stop_at_the_root'],
    }
  ],
  // Every processor module, not only the planted guard removals of the mutation:processors task.
  [
    '--processors',
    {
      description: 'every processor module against the processor tests',
      select: fileArguments('src/processors*.rs'),
      run: ['--cargo-test-arg=processors'],
    }
  ],
  // Two test-binary filters: the resolver's own slot controls and the semantic conformance suite that reaches it.
  // The whole suite took 139 seconds here, too close to the 180 second limit.
  [
    '--inferred-constants',
    {
      description: 'generic constant-slot resolution against its slot controls and the semantic conformance suite',
      select: fileArguments('src/rust_inferred_constants.rs'),
      run: [
        '--',
        'rust_inferred_constants',
        'rust_explicit_types'
      ],
    }
  ],
  // The executable's own modules: orchestration, the binary entry point, Rust rule selection, and the input
  // expansion and fix loop it drives. Its Markdown modules (`markdown_lfs_*`, dispatch and rule settings) are under
  // the Markdown scope's glob. `main.rs` and the real streams are reached only by the `binary` test target, whose
  // test names share no substring, so this scope runs every test except the five that load Cargo workspaces;
  // those take most of the whole suite's time, which exceeded the 180 second limit (183.94 s) at 04c663cb0.
  [
    '--executable',
    {
      description: 'orchestration, the entry point, the command grammar, Rust rule selection, input expansion and the fix loop against every test except the five Cargo-workspace tests',
      select: [
        'src/run_*.rs',
        'src/main.rs',
        'src/cli_options.rs',
        'src/rust_dispatch.rs',
        'src/rust_rule_settings.rs',
        'src/file_discovery.rs',
        'src/path_inputs.rs',
        'src/fix_loop.rs',
        'src/closure_*.rs',
      ].flatMap(fileArguments),
      run: [
        '--',
        '--',
        ...workspaceTests.flatMap(skipArguments)
      ],
    }
  ],
  // Shared layers below every rule: configuration parsing, lookup, matching and merging, rule-option validation,
  // findings, the inline-directive filter, grouped edits, and the syntax-only Rust rules with their shared parse.
  // None loads a Cargo workspace.
  [
    '--core',
    {
      description: 'configuration, findings, inline directives, grouped edits and the syntax-only Rust rules against every test except the five Cargo-workspace tests',
      select: [
        'src/config_*.rs',
        'src/configuration*.rs',
        'src/diagnostic.rs',
        'src/directive_*.rs',
        'src/edits.rs',
        'src/resolved_rules.rs',
        'src/rust_rules.rs',
        'src/rust_source.rs',
      ].flatMap(fileArguments),
      run: [
        '--',
        '--',
        ...workspaceTests.flatMap(skipArguments)
      ],
    }
  ],
  // The semantic engine: workspace discovery and loading, the compiler query, the per-invocation engine, the
  // session and the explicit-type rule. Its tests are the Cargo-workspace suites, so this scope runs the whole
  // suite. Its first unmutated baseline tested in 251 seconds on a loaded host (mutation-VFaIKp), already over
  // the 180 second default, so its limit is 1,200 seconds: 4.8 times that baseline, just under cargo-mutants'
  // own automatic limit of five times the baseline. Record: doc/handover/unified-linter-mutation-close.md.
  [
    '--semantic',
    {
      description: 'the semantic engine and the explicit-type rule against the whole suite',
      select: [
        'src/rust_file_engine.rs',
        'src/rust_toolchain.rs',
        'src/rust_workspace.rs',
        'src/rust_explicit_*.rs',
        'src/rust_generic_arguments.rs',
        'src/rust_semantic_*.rs',
        'src/rust_type_diagnostic.rs',
      ].flatMap(fileArguments),
      run: [],
      timeoutSeconds: 1_200,
    }
  ],
]);

/** The whole crate against the whole suite, used when no scope option is given. */
const unscoped = {
  description: 'every module against the whole suite',
  select: [],
  run: []
};

/** Verification failures keep the failing operation visible. */
class VerificationError extends Error {}

/** List one scope's mutant names on the host, exactly as a campaign of that scope would select them. */
function listMutants(scope) {
  const args = [
    'mutants',
    '--list',
    '--no-config',
    ...exclusionArguments([]),
    ...scope.select
  ];
  const result = spawnSync(
    'cargo',
    args,
    {
      cwd: process.cwd(),
      encoding: 'utf8',
      maxBuffer: 64 * 1_024
        * 1_024
    }
  );
  if (result.error)
    throw new VerificationError(
      'cargo mutants --list could not start.',
      { cause: result.error }
    );
  if (result.status !== 0)
    throw new VerificationError(`cargo mutants --list failed (status ${result.status}): ${result.stderr}`);
  return result.stdout
    .split('\n')
    .filter(line => line.length > 0);
}

/**
 Compare the union of every scope's listing with the unscoped listing, name by name, and keep every listing.
 A mutant outside every scope is never tried by any campaign, so any difference fails.
 */
async function coverage() {
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
    'coverage-'
  ));
  const all = listMutants(unscoped);
  await writeFile(
    join(
      evidence,
      'unscoped.txt'
    ),
    `${all.join('\n')  }\n`
  );
  const union = new Set();
  const counts = {};
  for (const [option, scope] of scopes) {
    const names = listMutants(scope);
    counts[option] = names.length;
    await writeFile(
      join(
        evidence,
        `${option.slice(2)}.txt`
      ),
      `${names.join('\n')  }\n`
    );
    for (const name of names)
      union.add(name);
  }
  const unscopedSet = new Set(all);
  const outside = all.filter(name => !union.has(name));
  const unknown = [...union].filter(name => !unscopedSet.has(name));
  const summary = {
    unscoped: all.length,
    unscopedDistinct: unscopedSet.size,
    union: union.size,
    perScope: counts,
    outside,
    unknown
  };
  await writeFile(
    join(
      evidence,
      'summary.json'
    ),
    `${JSON.stringify(
      summary,
      null,
      2
    )  }\n`
  );
  console.log(JSON.stringify(
    summary,
    null,
    2
  ));
  console.log(`Coverage evidence: ${evidence}`);
  if ((outside.length > 0) || (unknown.length > 0)
    || (unscopedSet.size !== all.length))
    throw new VerificationError(`The scopes do not hold exactly the unscoped mutants: ${outside.length} outside every scope, ${unknown.length} not in the unscoped listing; inspect ${evidence}.`);
}

/** Build and run the tool over an immutable input image, then retain its complete report. */
async function campaign({
  scope,
  shard,
  examine,
  limits
}) {
  // cargo-mutants keeps a mutant that matches any `--re`, so a second one would widen a scope that has its own.
  if ((examine !== undefined)
    && scope.select
      .includes('--re'))
    throw new VerificationError('This scope already selects mutants by name; --re would widen it.');
  const timeoutSeconds = scope.timeoutSeconds ?? defaultTimeoutSeconds;
  await runCampaign({
    testImage,
    mutationImage,
    testImageHint: '',
    scratchBase,
    contextPrefix: 'monochromatic-lint-mutation-',
    evidenceRoot: join(
      process.cwd(),
      'target',
      'verification'
    ),
    evidencePrefix: 'mutation-',
    limits,
    containerReport: '/work/mutation-report',
    command: campaignCommand({
      outputDirectory: '/work/mutation-report',
      timeoutSeconds,
      buildTimeoutSeconds: defaultBuildTimeoutSeconds,
      additionalExclusions: [],
      tail: [
        ...(shard === undefined ? [] : [
          '--shard',
          shard
        ]),
        ...scope.select,
        ...(examine === undefined ? [] : [
          '--re',
          examine
        ]),
        ...scope.run,
      ],
    }),
    manifest: ({ baseImage, campaignImage, toolSha256, container, command }) => ({
      baseImage,
      campaignImage,
      shard: shard ?? null,
      examine: examine ?? null,
      toolSha256,
      container,
      command,
      limits: {
        memory: limits.memory,
        cpus: limits.cpus,
        pids: limits.pidsLimit,
        buildTimeoutSeconds: defaultBuildTimeoutSeconds,
        testTimeoutSeconds: timeoutSeconds
      },
    }),
  });
}

/** Resolve one optional scope option to its table entry, or to the unscoped campaign when none is given. */
function scopeFor(option) {
  if (option === undefined)
    return unscoped;
  const scope = scopes.get(option);
  if (scope === undefined)
    throw new VerificationError(`Unknown scope ${option}; expected one of ${[...scopes.keys()].join(', ')}.`);
  return scope;
}

/**
 Dispatch: a campaign by default, optionally one `--shard k/n` of it or only the mutants whose names match
 `--re <regex>`, `--list [scope]` for one listing,
 and `--coverage` for the scope-union proof.
 */
async function main() {
  // `--memory`, `--cpus` and `--pids-limit` override the 2g, 2 CPU and 128 process bounds of the build and the container.
  const {
    limits,
    rest: options
  } = parseContainerLimitOptions({
    options: process.argv
      .slice(2),
    acceptPidsLimit: true,
    leadingOnly: false,
    valuelessFlags: [
      ...scopes.keys(),
      '--list',
      '--coverage'
    ]
  });
  if (options[0] === '--coverage') {
    if (options.length !== 1)
      throw new VerificationError('--coverage takes no other option.');
    await coverage();
    return;
  }
  if (options[0] === '--list') {
    if (options.length > 2)
      throw new VerificationError('--list takes at most one scope option.');
    console.log(listMutants(scopeFor(options[1]))
      .join('\n'));
    return;
  }
  const shardAt = options.indexOf('--shard');
  let shard;
  if (shardAt !== (-1)) {
    shard = options[shardAt + 1];
    const parts = /^(\d+)\/(\d+)$/u.exec(shard ?? '');
    if ((parts === null) || (Number(parts[1]) >= Number(parts[2])))
      throw new VerificationError(`--shard needs k/n with k from 0 to n - 1, not ${shard}.`);
    options.splice(
      shardAt,
      2
    );
  }
  const examineAt = options.indexOf('--re');
  let examine;
  if (examineAt !== (-1)) {
    examine = options[examineAt + 1];
    if ((examine === undefined) || (examine.length === 0))
      throw new VerificationError('--re needs a mutant-name regex.');
    options.splice(
      examineAt,
      2
    );
  }
  if (options.length > 1)
    throw new VerificationError(`Only one of ${[...scopes.keys()].join(', ')} is accepted.`);
  await campaign({
    scope: scopeFor(options[0]),
    shard,
    examine,
    limits
  });
}

await main();
