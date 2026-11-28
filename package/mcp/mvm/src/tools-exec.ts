/**
 Command execution tool definitions: exec and run.
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
  optionalPositiveInteger,
  optionalString,
  requiredString,
} from './tool-arguments.ts';
import {
  errorResponse,
  formatExecResult,
  textResponse,
} from './response.ts';
import { requiredStringArgument, } from './required-string-argument.ts';

//region Execution tools: run commands inside VMs

/**
 MCP tool: execute a command inside a named running VM, resolved via
 {@link backendFromArgs}.
 */
export const execTool: ToolEntry = defineTool({
  name: 'exec_in_vm',
  entry: {
    description:
      'Runs a shell command inside a named VM and returns stdout, stderr, and exit code. libvirt uses the QEMU guest agent (bash for Linux, PowerShell for Windows); the cloud backends (hetzner, ovh) run it in the remote login shell over SSH.',
    schema: strictArguments({
      name: requiredString('VM name to execute in',),
      command: requiredString(
        'Shell command to run inside the VM (libvirt: bash/PowerShell via guest agent; hetzner and ovh: remote login shell)',
      ),
      backend: BACKEND_ARGUMENT,
    },),
    handler: async function handleExecInVm(args,) {
      /**
       Target VM name validated as string so the backend receives a stable type regardless of MCP client encoding.
       */
      const name = requiredStringArgument(args.name,);
      /**
       Shell command validated as string for the same reason as `name`.
       */
      const command = requiredStringArgument(args.command,);
      try {
        /**
         Backend resolved from the optional `backend` arg, env, or default.
         */
        const backend = await backendFromArgs(args,);
        /**
         Execution result holding stdout, stderr, and exit code; formatted into the response below.
         */
        const result = await backend.exec({
          command,
          name,
        },);
        return textResponse(formatExecResult(result,),);
      }
      catch (err: unknown) {
        return errorResponse({
          tag: 'exec_in_vm',
          err,
        },);
      }
    },
  },
},);

/**
 MCP tool: create an ephemeral VM, run a command, then destroy it, on the
 backend resolved via {@link backendFromArgs}.
 */
export const runTool: ToolEntry = defineTool({
  name: 'run_in_vm',
  entry: {
    description:
      'Creates an ephemeral VM, runs a shell command inside it, then destroys the VM. Returns stdout, stderr, and exit code. Optionally clones from an existing VM instead of creating fresh; a clone keeps the source size. `vcpus` and `memory` ask for floors on a fresh VM: libvirt gives exactly those, a cloud backend picks the smallest size with at least both. Runs on the selected backend.',
    schema: strictArguments({
      command: requiredString(
        'Shell command to run inside the VM (libvirt: bash/PowerShell via guest agent; hetzner and ovh: remote login shell)',
      ),
      from: optionalString(
        'Clone from this existing VM name instead of creating fresh. Use list_vms to see available names (e.g. "win-01", not "windows").',
      ),
      vcpus: optionalPositiveInteger(
        'Fewest virtual CPUs a fresh machine may have',
      ),
      memory: optionalPositiveInteger(
        'Least memory a fresh machine may have, in gibibytes',
      ),
      backend: BACKEND_ARGUMENT,
    },),
    handler: async function handleRunInVm(args,) {
      /**
       Shell command validated as string so the ephemeral VM receives a stable type regardless of MCP client encoding.
       */
      const command = requiredStringArgument(args.command,);
      /**
       Optional source VM to clone from; absence selects the create-fresh path inside the backend run.
       */
      const from = ((typeof args.from) === 'string') ? args.from : undefined;
      /**
       Optional floor on virtual CPUs, or `undefined` when none was asked.
       */
      const vcpus = ((typeof args.vcpus) === 'number') ? args.vcpus : undefined;
      /**
       Optional floor on memory, in gibibytes.
       */
      const memoryGiB = ((typeof args.memory) === 'number') ? args.memory : undefined;
      try {
        /**
         Backend resolved from the optional `backend` arg, env, or default.
         */
        const backend = await backendFromArgs(args,);
        /**
         Execution result holding stdout, stderr, and exit code; formatted into the response below.
         */
        const result = await backend.run({
          command,
          ...(from !== undefined ? { from, } : {}),
          ...(vcpus !== undefined ? { vcpus, } : {}),
          ...(memoryGiB !== undefined ? { memoryGiB, } : {}),
        },);
        return textResponse(formatExecResult(result,),);
      }
      catch (err: unknown) {
        return errorResponse({
          tag: 'run_in_vm',
          err,
        },);
      }
    },
  },
},);

//endregion Execution tools
