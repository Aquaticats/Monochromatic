/**
 Library surface of the yuku-analyzer-based unused-export detector.
 
 @example
 ```ts
 import { findUnusedExports } from '@monochromatic-dev/mutation-test-cli-unused-export/ts';
 ```
 */

export {
  findUnusedExports,
  type UnusedExport,
} from './find-unused.ts';
export { workspaceResolver, } from './resolve.ts';
export {
  discoverWorkspacePackages,
  type WorkspacePackage,
} from './workspace.ts';
