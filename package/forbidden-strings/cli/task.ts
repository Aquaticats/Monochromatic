/**
 Package tasks for `@monochromatic-dev/forbidden-strings-cli`, replacing the
 `mise.toml` tasks CI ran through `mise run` with node-native definitions for
 `@monochromatic-dev/monorepo-task-cli`.
 */

import type { PackageTaskFile, } from '../../monorepo-task/cli/src/types.ts';

export default {
  'verify:evidence': {
    description: 'Reconcile retained source manifests with current compiled inputs without hiding mismatches',
    run: 'node bin/verify-evidence.mjs',
  },
  'verify:markdown': {
    description: 'Render scanner READMEs and native-verification evidence through the installed CommonMark HTML-tree pipeline',
    run: 'node bin/render-docs.mjs',
  },
  'dependencies:fetch': {
    description: 'Fetch the existing locked dependency archives before offline container verification',
    run: 'cargo fetch --locked',
  },
  'lint:clippy:container': {
    description: 'Run all-target/all-feature Clippy with warnings denied in the bounded offline scanner snapshot',
    run: 'node bin/test-container.ts --clippy',
  },
  'test:release:container': {
    description: 'Verify the actual release scanner and all-feature embedding consumers inside the bounded snapshot',
    run: 'node bin/test-container.ts --release',
  },
  'test:guards:observe:container': {
    description: 'Verify exact failure causes, per-operation hook ownership, independent worker cases, and real parallel CLI faults',
    run: 'node bin/observe-guards-container.mjs',
  },
  'test:guards:container': {
    description: 'Prove real executable and public-loader panic guards with passing and disabled disposable variants',
    run: 'node bin/guard-container.mjs',
  },
  'test:mutation:startup:container': {
    description: 'Re-test startup filter mutations with the full all-feature baseline and complete retained outcomes',
    run: 'node bin/mutate-container.ts --startup',
  },
  'test:mutation:pathname:container': {
    description: 'Mutate only pathname normalization, prefix counting and component masking with the full all-feature baseline and complete retained outcomes',
    run: 'node bin/mutate-container.ts --pathname',
  },
  'test:mutation:container': {
    description: 'Mutation-test embedding owners in a bounded mount-free snapshot, retaining every outcome',
    run: 'node bin/mutate-container.ts',
  },
  'test:container': {
    description: 'Run binary and embedded scanner tests unprivileged with Git 2.56.0 in a bounded mount-free container',
    run: 'node bin/test-container.ts',
  },
  'build': {
    description: 'Build the forbidden-strings binary in release mode',
    run: 'cargo build --release',
  },
  'build:debug': {
    description: 'Build the forbidden-strings binary in debug mode',
    run: 'cargo build',
  },
  'generate:rules': {
    description: 'Regenerate the embedded data/builtin-rules.txt baseline from the vendored betterleaks config',
    run: [
      'node src/mise.port-betterleaks.ts',
      'cargo run --release --manifest-path ../../forbidden-strings/core.bench/Cargo.toml --bin dialectport',
    ],
  },
  'lint': {
    description: 'Check the scanner without building it',
    run: 'cargo check',
  },
  'lint:fuzzing': {
    description: 'Check the fuzzing feature set',
    run: 'cargo check --features fuzzing',
  },
  'lint:clippy': {
    description: 'Clippy with warnings denied in release mode',
    run: 'cargo clippy --release --all-targets -- --deny warnings',
  },
  'lint:clippy:windows': {
    description: 'Clippy for the Windows GNU target with warnings denied',
    run: 'cargo clippy --locked --all-targets --all-features --target x86_64-pc-windows-gnu -- --deny warnings',
  },
  'format:clippy': {
    description: 'Apply Clippy fixes',
    run: 'cargo clippy --fix --allow-dirty --allow-staged',
  },
  'package': {
    description: 'Package the crate for publication',
    run: 'cargo package',
  },
  'publish': {
    description: 'Publish the crate',
    run: 'cargo publish',
  },
  'test': {
    description: 'Run the scanner test suite in release mode',
    run: 'cargo nextest run --release',
  },
  'scan:all': {
    description: 'Scan the repository with the built scanner',
    depends: ['build',],
    run: 'cargo run --release -- scan .',
  },
  'lock:members': {
    description: 'Refresh the workspace lockfile offline',
    run: 'cargo update --offline --workspace',
  },
} satisfies PackageTaskFile;
