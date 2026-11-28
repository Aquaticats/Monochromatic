/**
 VM lifecycle tool definitions: list and update.
 @module
 */
import {
  defineTool,
  strictArguments,
  type ToolEntry,
} from '@monochromatic-dev/mcp-stdio/ts';

import {
  BACKEND_ARGUMENT,
  backendFromArgs,
} from './backend.ts';
import {
  errorResponse,
  textResponse,
} from './response.ts';

//region Lifecycle tools: VM listing and template updates

/**
 MCP tool: list all managed VMs and their state, on the backend resolved
 via {@link backendFromArgs}.
 */
export const listTool: ToolEntry = defineTool({
  name: 'list_vms',
  entry: {
    description:
      'Lists all managed VMs on the selected backend with their current state (running, shut off, etc.).',
    schema: strictArguments({ backend: BACKEND_ARGUMENT, },),
    handler: async function handleListVms(args,) {
      try {
        /**
         Backend resolved from the optional `backend` arg, env, or default.
         */
        const backend = await backendFromArgs(args,);
        /**
         All managed VMs queried from the selected backend.
         */
        const vms = await backend.list();
        if (vms.length
          === 0)
          return textResponse('No VMs found.',);
        /**
         One `name: state` line per VM, joined with newlines into the response body below.
         */
        const lines = vms.map(function formatVmLine(vm: Readonly<(typeof vms)[number]>,) {
          return `${vm.name}: ${vm.state}`;
        },);
        return textResponse(lines.join('\n',),);
      }
      catch (err: unknown) {
        return errorResponse({
          tag: 'list_vms',
          err,
        },);
      }
    },
  },
},);

/**
 MCP tool: refresh provider-managed images or templates, on the backend
 resolved via {@link backendFromArgs}.
 */
export const updateTool: ToolEntry = defineTool({
  name: 'update_templates',
  entry: {
    description:
      'Refreshes provider-managed images. libvirt re-downloads base images and rebuilds all templates (Windows rebuild takes 15-30 minutes); hetzner and ovh validate their credentials and report what the provider offers (nothing is built locally).',
    schema: strictArguments({ backend: BACKEND_ARGUMENT, },),
    handler: async function handleUpdateTemplates(args,) {
      try {
        /**
         Backend resolved from the optional `backend` arg, env, or default.
         */
        const backend = await backendFromArgs(args,);
        await backend.update();
        return textResponse('Templates updated successfully.',);
      }
      catch (err: unknown) {
        return errorResponse({
          tag: 'update_templates',
          err,
        },);
      }
    },
  },
},);

//endregion Lifecycle tools
