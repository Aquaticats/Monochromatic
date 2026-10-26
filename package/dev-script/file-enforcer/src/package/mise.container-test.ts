/**
 Container test runner for ensurePackage.
 Launches podman containers across the test matrix via `module-matrix`.
 
 Test matrix:
 - OS: ubuntu (apt), fedora (dnf)
 - User: root, non-root (uid 1000)
 
 @example
 ```bash
 node package/dev-script/file-enforcer/src/package/mise.container-test.ts
 ```
 */

import { matrix, } from '@monochromatic-dev/module-matrix/ts';

await matrix({
  os: [
    'container:ubuntu',
    'container:fedora',
  ],
  user: [
    'root',
    'user',
  ],
  // One container at a time: the matrix runner's podman spawn carries no
  // resource bounds, so four concurrent cells can exhaust the host (the
  // synthetic-load ban). The toolchain matrix runner bounds its cells with
  // --memory=2g --cpus=2 instead.
  concurrency: 1,
},);
