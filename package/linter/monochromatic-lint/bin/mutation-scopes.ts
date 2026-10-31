/**
 The linter's mutation scopes: which source files each campaign mutates, which tests it runs, and how long a mutant
 may take.
 */

/**
 One scope of the linter's mutation campaigns.
 */
export type MutationScope = {
  /**
   What the scope covers, for the usage listing.
   */
  readonly description: string;
  /**
   cargo-mutants options that decide which mutants exist; all `--list` needs.
   */
  readonly select: readonly string[];
  /**
   cargo-mutants options that decide how each mutant is built and tested; last because they may end with libtest
   arguments.
   */
  readonly run: readonly string[];
  /**
   Per-mutant test limit in seconds when it differs from the default.
   */
  readonly timeoutSeconds?: number;
};

/**
 Per-mutant test limit, in seconds, for every scope whose own entry names none.
 */
export const defaultTimeoutSeconds = 180;

/**
 One nextest `--skip` option pair, which excludes every test whose name contains the text.

 @param name - test name fragment
 
 @returns the option pair
 
 @example
 ```ts
 skipArguments('module::tests::slow');
 ```
 */
function skipArguments(name: string): string[] {
  return [
    '--skip',
    name,
  ];
}

/**
 One cargo-mutants `--file` option pair, which keeps the source files the glob matches.

 @param glob - file glob under the crate
 
 @returns the option pair
 
 @example
 ```ts
 fileArguments('src/markdown_*.rs');
 ```
 */
function fileArguments(glob: string): string[] {
  return [
    '--file',
    glob,
  ];
}

/**
 Full names of the five tests that load a real Cargo workspace through the semantic engine.
 Alone in the bounded container they took 43 to 139 seconds each, and the rest of the library suite 14 seconds
 (workspace-tests-cost-1.log), so the fast scopes skip exactly these. Each name is a whole test path that no other
 test contains, so the quick tests of the same modules still run. Record: doc/handover/unified-linter-mutation-close.md.
 */
const workspaceTests: readonly string[] = [
  'rust_explicit_types_tests::semantic_conformance_and_source_overlay_controls',
  'rust_file_engine::tests::selected_semantics_reuses_the_manifest_session',
  'rust_inferred_constants::tests::holes_resolve_against_the_parameter_in_their_own_slot',
  'rust_workspace_tests::generated_definitions_and_build_failures_are_distinct',
  'run_directive_tests::semantic_findings_follow_the_same_directive_rules',
];

/**
 Every scope, keyed by its command-line option. `select` decides which mutants exist (it is all `--list` needs),
 `run` decides how each mutant is built and tested and comes last because it may end with libtest arguments,
 and `timeoutSeconds` is the per-mutant test limit when it differs from the default.
 cargo-mutants copies everything after its own `--` into the `cargo nextest run` command line, where nextest
 accepts several test-name filters; a second `--` hands the remaining arguments to nextest's own filter
 position, the only place it accepts `--skip` (probed: a `--skip` before it is an unexpected argument).
 */
export const scopes: ReadonlyMap<string, MutationScope> = new Map<string, MutationScope>([
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

/**
 The whole crate against the whole suite, used when no scope option is given.
 */
export const unscoped: MutationScope = {
  description: 'every module against the whole suite',
  select: [],
  run: []
};
