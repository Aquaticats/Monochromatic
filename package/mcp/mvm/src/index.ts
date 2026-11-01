#!/usr/bin/env node
/**
 MCP server entry point for mvm; exposes VM operations as MCP tools.
 @module
 */
import {
  createMcpServer,
  serve,
} from '@monochromatic-dev/mcp-stdio/ts';

import {
  execTool,
  runTool,
} from './tools-exec.ts';
import {
  createTool,
  destroyTool,
} from './tools-lifecycle-mutate.ts';
import {
  listTool,
  updateTool,
} from './tools-lifecycle.ts';
import {
  pullTool,
  pushTool,
} from './tools-transfer.ts';



//region Server setup: create and serve the MCP server

/**
 MCP server instance exposing mvm operations as tools, built via
 {@link createMcpServer}.
 */
const server = createMcpServer({
  config: {
    name: 'mvm',
    version: '0.1.0',
    title: 'mvm VM manager',
    instructions:
      'Manages throwaway virtual machines for running commands that could crash or exhaust the host. '
        + 'Nothing records which backend a VM was created on, so every follow-up call must repeat the '
        + 'same `backend` argument used at create time, and `list_vms` only ever shows one backend at a time. '
        + 'Prefer `run_in_vm` for one-shot commands: it creates, runs, and destroys in a single call, '
        + 'leaving no VM to track. Creating or updating Windows images on the libvirt backend takes '
        + '15 to 30 minutes.',
  },
  tools: [
    listTool,
    createTool,
    destroyTool,
    execTool,
    runTool,
    updateTool,
    pushTool,
    pullTool,
  ],
},);

await serve({ server, },);

//endregion Server setup
