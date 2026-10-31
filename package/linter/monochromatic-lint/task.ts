/**
 Package tasks for `@monochromatic-dev/linter-monochromatic-lint`, replacing the
 `mise.toml` tasks CI ran through `mise run` with node-native definitions for
 `@monochromatic-dev/monorepo-task-cli`.
 */

import type { PackageTaskFile, } from '../../monorepo-task/cli/src/types.ts';

export default {
  'build': {
    description: 'Build the native unified-linter implementation',
    run: 'cargo build --release',
  },
  'build:debug': {
    description: 'Build the native unified-linter implementation for iteration',
    run: 'cargo build',
  },
  'lint': {
    description: 'Check the Rust implementation without code generation',
    run: 'cargo check',
  },
  'lint:types': {
    description: 'Check Rust types and deny Clippy findings',
    run: [
      'cargo check',
      'cargo clippy --all-targets -- -D warnings',
    ],
  },
  'format:rust': {
    description: 'Format the native unified-linter source',
    run: 'cargo fmt',
  },
  'lint:rustfmt': {
    description: 'Check native source formatting',
    run: 'cargo fmt --check',
  },
  'test': {
    description: 'Run native unit and conformance tests',
    run: 'cargo nextest run --all-targets',
  },
  'test:host': {
    description: 'Run the host-only real run of the closure check over package/wg/cli-quicker-exempt',
    run: 'cargo nextest run --test closure_binary --run-ignored only',
  },
  'dependencies:fetch': {
    description: 'Fetch the locked dependency archives before offline container verification',
    run: 'cargo fetch --locked',
  },
  'lock': {
    description: 'Regenerate the owning Cargo lockfile without fetching dependencies',
    run: 'cargo generate-lockfile --offline',
  },
  'lint:container': {
    description: 'Clippy inside the bounded offline snapshot',
    depends: ['test:container',],
    run: 'node bin/clippy-container.ts',
  },
  'lint:rust-style': {
    description: 'Clippy the Rust-style fixture inside the bounded snapshot',
    depends: ['test:rust-style',],
    run: 'node bin/clippy-container.ts',
  },
  'mutation': {
    description: 'Mutation-test the unified linter in the bounded snapshot',
    depends: ['test:container',],
    run: 'node bin/mutate-container.ts',
  },
  'test:container': {
    description: 'Run the unified-linter tests in the bounded offline snapshot',
    run: 'node bin/test-container.ts',
  },
  'test:rust-style': {
    description: 'Run the Rust-style fixture tests in the bounded snapshot',
    run: 'node bin/test-container.ts --rust-style',
  },
  'test:markdown': {
    description: 'Run the Markdown fixture tests in the bounded snapshot',
    run: 'node bin/test-container.ts --markdown',
  },
} satisfies PackageTaskFile;
