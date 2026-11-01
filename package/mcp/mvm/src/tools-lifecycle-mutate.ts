/**
 VM mutation tool definitions: create and destroy.
 @module
 */
import {
  defineTool,
  strictArguments,
  type ToolEntry,
} from '@monochromatic-dev/mcp-stdio/ts';

import * as v from 'valibot';

import {
  BACKEND_ARGUMENT,
  backendFromArgs,
} from './backend.ts';
import {
  optionalString,
  requiredString,
} from './tool-arguments.ts';
import {
  errorResponse,
  invalidArgumentsResponse,
  textResponse,
} from './response.ts';
import { requiredStringArgument, } from './required-string-argument.ts';

//region Mutation tools: VM creation and destruction

/**
 MCP tool: create a new VM, optionally cloned from an existing one, on the
 backend resolved via {@link backendFromArgs}.
 */
export const createTool: ToolEntry = defineTool({
  name: 'create_vm',
  entry: {
    description:
      'Creates and starts a new VM on the selected backend. libvirt supports ubuntu (default), fedora, alpine, windows, or a custom template name; hetzner supports ubuntu/debian/fedora/rocky/centos/alma or a literal Hetzner image slug. Windows libvirt VMs take 15-30 minutes on first creation. When `from` is provided, clones from that existing VM instead. `server_type` and `location` apply to the hetzner backend only.',
    schema: strictArguments({
      name: requiredString(
        'VM name (alphanumeric, hyphens, underscores; hetzner additionally forbids underscores)',
      ),
      from: optionalString('Clone from this existing VM instead of creating fresh',),
      image: optionalString(
        'Image to use (backend-specific): e.g. ubuntu (default), fedora, alpine, windows, or a custom/literal image name',
      ),
      server_type: optionalString(
        'Hetzner server type (e.g. cx23); defaults to the cheapest non-deprecated type; ignored by the libvirt backend',
      ),
      location: optionalString(
        'Hetzner location or comma-separated fallback series (e.g. fsn1,nbg1); ignored by the libvirt backend',
      ),
      backend: BACKEND_ARGUMENT,
    },),
    handler: async function handleCreateVm(args,) {
      /**
       New VM name validated as string so the backend receives a stable type regardless of MCP client encoding.
       */
      const name = requiredStringArgument(args.name,);
      /**
       Optional source VM to clone from; `undefined` selects the create-fresh path below.
       */
      const from = ((typeof args.from) === 'string') ? args.from : undefined;
      /**
       Optional image template; absence falls back to the backend default.
       */
      const image = ((typeof args.image) === 'string') ? args.image : undefined;
      /**
       Optional Hetzner server type; ignored by the libvirt backend.
       */
      const serverType = ((typeof args.server_type) === 'string') ? args.server_type : undefined;
      /**
       Optional Hetzner location series; ignored by the libvirt backend.
       */
      const location = ((typeof args.location) === 'string') ? args.location : undefined;
      try {
        /**
         Backend resolved from the optional `backend` arg, env, or default.
         */
        const backend = await backendFromArgs(args,);
        await (from !== undefined
          ? backend.clone({
            destination: name,
            source: from,
          },)
          : backend.create({
            name,
            ...(image !== undefined ? { image, } : {}),
            ...(serverType !== undefined ? { serverType, } : {}),
            ...(location !== undefined ? { location, } : {}),
          },));
        /**
         Trailing fragment appended to the success message to disclose clone provenance when applicable.
         */
        const suffix = from !== undefined ? ` (cloned from ${from})` : '';
        return textResponse(
          `VM ${name} created${suffix} and started. Use exec_in_vm to run commands inside it.`,
        );
      }
      catch (err: unknown) {
        return errorResponse({
          tag: 'create_vm',
          err,
        },);
      }
    },
  },
},);

/**
 MCP tool: destroy VMs by name or all at once, on the backend resolved via
 {@link backendFromArgs}.
 */
export const destroyTool: ToolEntry = defineTool({
  name: 'destroy_vm',
  entry: {
    description:
      'Force-stops and deletes a VM by name, or all managed VMs when `all` is true. Provide exactly one of `name` or `all`. Operates on the selected backend only.',
    // Exactly-one-target is expressed here rather than left to the handler, so a client can
    // reject an ambiguous call before it reaches this server. Each branch is strict, which is
    // what makes the union exclusive: `{ name, all: true }` carries a property neither branch
    // admits and so matches neither. The message is the one a caller sees on refusal, so it
    // states the rule rather than reporting a union mismatch.
    schema: v.union(
      [
        strictArguments({
          name: requiredString('VM name to destroy (mutually exclusive with all)',),
          backend: BACKEND_ARGUMENT,
        },),
        strictArguments({
          all: v.pipe(
            v.literal(true,),
            v.description('Destroy every managed VM (mutually exclusive with name)',),
          ),
          backend: BACKEND_ARGUMENT,
        },),
      ],
      'Provide either `name` or `all: true`, not both, and not neither.',
    ),
    handler: async function handleDestroyVm(args,) {
      /**
       Optional single-VM target; mutually exclusive with `all` and validated below.
       */
      const name = ((typeof args.name) === 'string') ? args.name : undefined;
      /**
       Optional destroy-everything flag; mutually exclusive with `name` and validated below.
       */
      const all = ((typeof args.all) === 'boolean') ? args.all : undefined;

      // Refuse ambiguous targets before resolving a backend, so a rejected call never
      // reaches one. Answering `all: true` first would destroy every VM for a caller who
      // named the single VM they meant, which is the opposite of what they asked for and
      // is not recoverable once the backend has run.
      if ((name !== undefined) && (all === true)) {
        return invalidArgumentsResponse({
          tag: 'destroy_vm',
          text:
            `Provide either \`name\` or \`all: true\`, not both. Received name "${name}" `
              + `alongside all: true, and destroying every VM is not what naming one asks for. `
              + `Send only \`name\` to destroy that VM, or only \`all: true\` to destroy every VM.`,
        },);
      }
      if ((name === undefined) && (all !== true)) {
        return invalidArgumentsResponse({
          tag: 'destroy_vm',
          text: 'Provide either `name` to destroy one VM, or `all: true` to destroy every VM.',
        },);
      }

      try {
        /**
         Backend resolved from the optional `backend` arg, env, or default.
         */
        const backend = await backendFromArgs(args,);
        if (all === true) {
          await backend.destroyAll();
          return textResponse('All VMs destroyed.',);
        }
        if (name === undefined)
          throw new Error('destroy_vm reached backend dispatch without a target',);
        await backend.destroy({ name, },);
        return textResponse(`VM ${name} destroyed.`,);
      }
      catch (err: unknown) {
        return errorResponse({
          tag: 'destroy_vm',
          err,
        },);
      }
    },
  },
},);

//endregion Mutation tools
