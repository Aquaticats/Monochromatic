/**
 The symbols this package's built-artifact tests reach, re-exported from one module.

 The package's other entry is the `mvm-mcp` bin, and importing it starts a stdio server
 loop that never returns, so no test can reach a handler or a response builder through it.
 This second entry exists so tests import those symbols from built output rather than from
 source, which `test-import/require-eventual-artifact` requires of every package with a
 build.

 Nothing in this package imports it, and no MCP client needs it: these exports are not
 part of the package's API, and they exist only so the built artifact under test is the
 one tests load.

 @module
 */

//region Tool entries: the handlers tests drive with a recorded backend

export {
  execTool,
  runTool,
} from './tools-exec.ts';
export {
  createTool,
  destroyTool,
} from './tools-lifecycle-mutate.ts';
export {
  listTool,
  updateTool,
} from './tools-lifecycle.ts';
export {
  pullTool,
  pushTool,
} from './tools-transfer.ts';

//endregion Tool entries

//region Backend registry: the one the handlers in this bundle resolve through

/*
 Re-exported from the dependency rather than restated, so this bundle and the module it
 loads here share a single registry object: whatever a test installs here is what the
 bundled handlers call, and nothing can reach a real backend through a second copy.
 */
export { BACKENDS, } from '@monochromatic-dev/mvm-cli/ts';

//endregion Backend registry

//region Response builders: the shapes and sections every tool answers through

export {
  errorResponse,
  formatExecResult,
  invalidArgumentsResponse,
  textResponse,
} from './response.ts';

//endregion Response builders
