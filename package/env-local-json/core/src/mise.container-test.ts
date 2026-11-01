/**
 Container matrix runner for the encrypted env store reader.
 
 Runs `env-local-json.unit.matrix.test.ts` in Ubuntu as root and as a
 non-root user through `@monochromatic-dev/test-harness-core-matrix`. The test file
 installs sops and age from the mounted repo config, so the container
 proves the same toolchain the developer uses.
 
 The file list is explicit: default discovery walks the package tree, and
 the package's workspace symlinks make that walk needlessly expensive.
 
 @example
 ```bash
 node src/mise.container-test.ts
 ```
 */

import { join, } from 'node:path';

import { matrix, } from '@monochromatic-dev/test-harness-core-matrix/ts';

await matrix({
  files: [
    join(
      import.meta.dirname,
      'env-local-json.unit.matrix.test.ts',
    ),
  ],
  os: ['container:ubuntu',],
  user: [
    'root',
    'user',
  ],
  runtime: ['node',],
},);
