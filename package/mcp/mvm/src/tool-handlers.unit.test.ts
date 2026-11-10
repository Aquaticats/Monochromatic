/**
 Cases for every mvm tool handler, driven against a recorded backend.

 The handlers are reachable in production only through the bin entry, whose import starts
 a stdio server loop, so these cases import the built module that re-exports them. Each
 handler is called the way `mcp-stdio` calls it: with arguments its schema already
 accepted. That is the only way to observe the operations a handler chooses and the
 argument objects it builds without provisioning a real VM.

 The backend the handlers resolve through the CLI's registry is replaced for the whole
 file, so every case runs against {@link fakeBackend} and nothing reaches KVM or Hetzner.
 Cases therefore run one at a time: they share that one replacement, and its spies and
 its list belong to whichever case is running.
 */

import type {
  Backend,
  ExecResult,
  VmInfo,
} from '@monochromatic-dev/mvm-cli/ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import type { ToolCallResult, ToolEntry, } from '@monochromatic-dev/mcp-stdio/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  mkdtemp,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  BACKENDS,
  createTool,
  destroyTool,
  execTool,
  listTool,
  pullTool,
  pushTool,
  runTool,
  updateTool,
} from '../dist/final/node/exports-for-built-artifact-tests.mjs';

//region Fake backend: answers with fixed values and records what it was asked

/**
 Backend kind the replacement answers to.
 Hetzner is registered for every platform, so these cases never depend on the host
 supporting the libvirt backend.
 */
const FAKE_BACKEND_KIND = 'hetzner';

/**
 Result the fake reports for an executed command, formatted by the handlers under test.
 */
const EXEC_RESULT: ExecResult = {
  stdout: 'Linux dev-01',
  stderr: '',
  exitCode: 0,
};

/**
 Bytes the fake hands back for a pull, written to the host by the handler.
 */
const PULLED_BYTES = Buffer.from('guest file contents',);

/**
 VMs the fake reports from `list`.
 A listing case that needs a different shape stubs this operation for its own run, so no
 case has to change what another one sees.
 */
const LISTED_VMS: readonly VmInfo[] = [
  {
    name: 'dev-01',
    state: 'running',
  },
  {
    name: 'win-01',
    state: 'shut off',
  },
];

/**
 Backend whose operations resolve with fixed values and touch nothing.
 A case spies on the operations it expects and asserts the arguments the handler built.
 
 @returns Backend recording nothing on its own
 
 @example
 ```ts
 const create = ctx.sinon.spy(fakeBackend, 'create');
 ```
 */
function inertBackend(): Backend {
  return {
    create: async function create() {},
    clone: async function clone() {},
    destroy: async function destroy() {},
    destroyAll: async function destroyAll() {},
    list: async function list() {
      return LISTED_VMS;
    },
    exec: async function exec() {
      return EXEC_RESULT;
    },
    run: async function run() {
      return EXEC_RESULT;
    },
    pushFile: async function pushFile() {
      return '';
    },
    pullFile: async function pullFile() {
      return PULLED_BYTES;
    },
    update: async function update() {},
    shell: async function shell() {},
  };
}

/**
 Backend every handler in this file resolves.
 */
const fakeBackend = inertBackend();

/**
 Registry entry as it was before this file replaced it, restored once the cases finish.
 */
const REAL_HETZNER_ENTRY = BACKENDS[FAKE_BACKEND_KIND];

/**
 Points the handlers at {@link fakeBackend} for as long as the returned value lives.
 
 @returns Disposable that puts the entry the CLI shipped back, including after a failed
   case, so the real loader is never left replaced
 
 @example
 ```ts
 using registryEntry = fakeBackendInstalled();
 ```
 */
function fakeBackendInstalled(): Disposable {
  BACKENDS[FAKE_BACKEND_KIND] = {
    meta: REAL_HETZNER_ENTRY.meta,
    load: async function loadFakeBackend() {
      return fakeBackend;
    },
  };
  return {
    [Symbol.dispose]() {
      BACKENDS[FAKE_BACKEND_KIND] = REAL_HETZNER_ENTRY;
    },
  };
}

/**
 Message the first content item of a handler response carries, which is all a client reads.
 
 @param response - Result a tool handler answered with
 
 @returns Text of its single content item
 
 @throws When a handler answered with anything but text
 
 @example
 ```ts
 expect(firstText(await listTool.handler({}))).toBe('No VMs found.');
 ```
 */
function firstText(response: ToolCallResult,): string {
  const [first] = response.content;
  if ((first === undefined) || (first.type !== 'text'))
    throw new Error(`expected a single text content item, received "${String(first?.type,)}"`,);
  return first.text;
}

//endregion Fake backend

//region Handler calls that carry no usable argument

/**
 One handler call carrying no argument at all, per tool that extracts a required string.
 */
const MISSING_ARGUMENT_CASES: readonly { readonly tool: ToolEntry; }[] = [
  { tool: execTool, },
  { tool: runTool, },
  { tool: createTool, },
  { tool: pushTool, },
  { tool: pullTool, },
];

/**
 One handler call whose named argument carries a number where the tool requires a string,
 one case per extraction site in the five tools that validate their strings at runtime.
 */
const SPOILED_ARGUMENT_CASES: readonly {
  readonly tool: ToolEntry;
  readonly argument: string;
  readonly args: Readonly<Record<string, unknown>>;
}[] = [
  {
    tool: execTool,
    argument: 'name',
    args: {
      name: 1,
      command: 'uname -a',
    },
  },
  {
    tool: execTool,
    argument: 'command',
    args: {
      name: 'dev-01',
      command: 1,
    },
  },
  {
    tool: runTool,
    argument: 'command',
    args: { command: 1, },
  },
  {
    tool: createTool,
    argument: 'name',
    args: { name: 1, },
  },
  {
    tool: pushTool,
    argument: 'name',
    args: {
      name: 1,
      hostPath: 'artifact.txt',
      guestPath: '/root/artifact.txt',
    },
  },
  {
    tool: pushTool,
    argument: 'hostPath',
    args: {
      name: 'dev-01',
      hostPath: 1,
      guestPath: '/root/artifact.txt',
    },
  },
  {
    tool: pushTool,
    argument: 'guestPath',
    args: {
      name: 'dev-01',
      hostPath: 'artifact.txt',
      guestPath: 1,
    },
  },
  {
    tool: pullTool,
    argument: 'name',
    args: {
      name: 1,
      guestPath: '/etc/hostname',
      hostPath: 'pulled-hostname.txt',
    },
  },
  {
    tool: pullTool,
    argument: 'guestPath',
    args: {
      name: 'dev-01',
      guestPath: 1,
      hostPath: 'pulled-hostname.txt',
    },
  },
  {
    tool: pullTool,
    argument: 'hostPath',
    args: {
      name: 'dev-01',
      guestPath: '/etc/hostname',
      hostPath: 1,
    },
  },
];

//endregion Handler calls that carry no usable argument

using registryEntry = fakeBackendInstalled();

await describe({
  name: 'mvm tool handlers',
  concurrency: 1,
  children: [
    //region Seam: the handlers resolve through the registry entry this file installs

    it({
      name: 'resolves its backend through the replacement registry entry',
      fn: async () => {
        const installed = BACKENDS[FAKE_BACKEND_KIND];

        // The handlers run from a bundle carrying its own copy of the CLI registry, so the
        // replacement only holds when it is installed on that same object. This case is the
        // positive control: without it, a case could reach a real backend unnoticed.
        expect(installed,).toBeDefined();
        expect(await installed?.load(),).toBe(fakeBackend,);
      },
    },),

    //endregion Seam

    //region Argument refusal: a client that bypassed the advertised schema

    ...MISSING_ARGUMENT_CASES.map(function mapMissingArgument(missingCase,) {
      return it({
        name: `${missingCase.tool.name} refuses a call carrying no arguments at all`,
        fn: async () => {
          let caught: unknown;
          try {
            await missingCase.tool.handler({},);
          }
          catch (error: unknown) {
            caught = error;
          }

          // The server validates arguments before dispatch, so this branch is reachable
          // only by a caller that skipped the schema. It must still stop here rather than
          // hand `undefined` to a backend.
          expect(caught,).toBeInstanceOf(TypeError,);
          expect(caughtValueText(caught,),).toBe(
            'Required MCP tool argument must be a string.',
          );
        },
      },);
    },),

    ...SPOILED_ARGUMENT_CASES.map(function mapSpoiledArgument(spoiledCase,) {
      return it({
        name: `${spoiledCase.tool.name} refuses a call whose ${spoiledCase.argument} is not a string`,
        fn: async () => {
          let caught: unknown;
          try {
            await spoiledCase.tool.handler(spoiledCase.args,);
          }
          catch (error: unknown) {
            caught = error;
          }

          expect(caught,).toBeInstanceOf(TypeError,);
          expect(caughtValueText(caught,),).toBe(
            'Required MCP tool argument must be a string.',
          );
        },
      },);
    },),

    //endregion Argument refusal

    //region create_vm: create fresh or clone, and what each path forwards

    describe({
      name: createTool.name,
      children: [
        it({
          name: 'creates with only the name when no optional argument is given',
          fn: async ctx => {
            const create = ctx.sinon.spy(fakeBackend, 'create',);
            const clone = ctx.sinon.spy(fakeBackend, 'clone',);

            const response = await createTool.handler({
              name: 'dev-01',
              backend: FAKE_BACKEND_KIND,
            },);

            expect(create.callCount,).toBe(1,);
            // Absent optionals must not be forwarded as explicit `undefined`: a backend
            // that distinguishes "passed" from "absent" would take the wrong default.
            expect(create.firstCall.args,).toEqual([{ name: 'dev-01', },],);
            expect(clone.callCount,).toBe(0,);
            expect(firstText(response,),).toBe(
              'VM dev-01 created and started. Use exec_in_vm to run commands inside it.',
            );
          },
        },),
        it({
          name: 'forwards image, server_type and location under the backend argument names',
          fn: async ctx => {
            const create = ctx.sinon.spy(fakeBackend, 'create',);

            await createTool.handler({
              name: 'web-01',
              image: 'fedora',
              server_type: 'cx23',
              location: 'fsn1,nbg1',
              backend: FAKE_BACKEND_KIND,
            },);

            // The tool's snake_case argument names map to the backend's camelCase ones.
            expect(create.callCount,).toBe(1,);
            expect(create.firstCall.args,).toEqual([{
              name: 'web-01',
              image: 'fedora',
              serverType: 'cx23',
              location: 'fsn1,nbg1',
            },],);
          },
        },),
        it({
          name: 'clones from the named source instead of creating',
          fn: async ctx => {
            const create = ctx.sinon.spy(fakeBackend, 'create',);
            const clone = ctx.sinon.spy(fakeBackend, 'clone',);

            const response = await createTool.handler({
              name: 'web-02',
              from: 'web-01',
              backend: FAKE_BACKEND_KIND,
            },);

            expect(clone.callCount,).toBe(1,);
            expect(clone.firstCall.args,).toEqual([{
              destination: 'web-02',
              source: 'web-01',
            },],);
            expect(create.callCount,).toBe(0,);
            expect(firstText(response,),).toBe(
              'VM web-02 created (cloned from web-01) and started. Use exec_in_vm to run commands inside it.',
            );
          },
        },),
        it({
          name: 'drops image and server_type when it clones instead of creating',
          fn: async ctx => {
            const create = ctx.sinon.spy(fakeBackend, 'create',);
            const clone = ctx.sinon.spy(fakeBackend, 'clone',);

            await createTool.handler({
              name: 'web-03',
              from: 'web-01',
              image: 'fedora',
              server_type: 'cx23',
              backend: FAKE_BACKEND_KIND,
            },);

            // A clone carries a source VM, so neither image nor server type can apply;
            // forwarding them would promise a choice the backend cannot honour.
            expect(clone.callCount,).toBe(1,);
            expect(clone.firstCall.args,).toEqual([{
              destination: 'web-03',
              source: 'web-01',
            },],);
            expect(create.callCount,).toBe(0,);
          },
        },),
      ],
    },),

    //endregion create_vm

    //region destroy_vm: one VM or every VM

    describe({
      name: destroyTool.name,
      children: [
        it({
          name: 'destroys the named VM alone',
          fn: async ctx => {
            const destroy = ctx.sinon.spy(fakeBackend, 'destroy',);
            const destroyAll = ctx.sinon.spy(fakeBackend, 'destroyAll',);

            const response = await destroyTool.handler({
              name: 'dev-01',
              backend: FAKE_BACKEND_KIND,
            },);

            expect(destroy.callCount,).toBe(1,);
            expect(destroy.firstCall.args,).toEqual([{ name: 'dev-01', },],);
            expect(destroyAll.callCount,).toBe(0,);
            expect(firstText(response,),).toBe('VM dev-01 destroyed.',);
          },
        },),
        it({
          name: 'destroys every VM when all is true',
          fn: async ctx => {
            const destroy = ctx.sinon.spy(fakeBackend, 'destroy',);
            const destroyAll = ctx.sinon.spy(fakeBackend, 'destroyAll',);

            const response = await destroyTool.handler({
              all: true,
              backend: FAKE_BACKEND_KIND,
            },);

            expect(destroyAll.callCount,).toBe(1,);
            expect(destroy.callCount,).toBe(0,);
            expect(firstText(response,),).toBe('All VMs destroyed.',);
          },
        },),
      ],
    },),

    //endregion destroy_vm

    //region exec_in_vm and run_in_vm: commands inside VMs

    describe({
      name: execTool.name,
      children: [
        it({
          name: 'runs the command in the named VM and formats the result',
          fn: async ctx => {
            const exec = ctx.sinon.spy(fakeBackend, 'exec',);

            const response = await execTool.handler({
              name: 'dev-01',
              command: 'uname -a',
              backend: FAKE_BACKEND_KIND,
            },);

            expect(exec.callCount,).toBe(1,);
            expect(exec.firstCall.args,).toEqual([{
              command: 'uname -a',
              name: 'dev-01',
            },],);
            expect(firstText(response,),).toBe('stdout:\nLinux dev-01\n\nexit code: 0',);            },
        },),
      ],
    },),

    describe({
      name: runTool.name,
      children: [
        it({
          name: 'forwards from when the caller named a source VM',
          fn: async ctx => {
            const run = ctx.sinon.spy(fakeBackend, 'run',);

            await runTool.handler({
              command: 'uname -a',
              from: 'base-01',
              backend: FAKE_BACKEND_KIND,
            },);

            expect(run.callCount,).toBe(1,);
            expect(run.firstCall.args,).toEqual([{
              command: 'uname -a',
              from: 'base-01',
            },],);
          },
        },),
        it({
          name: 'omits from when the caller named no source VM',
          fn: async ctx => {
            const run = ctx.sinon.spy(fakeBackend, 'run',);

            await runTool.handler({
              command: 'uname -a',
              backend: FAKE_BACKEND_KIND,
            },);

            expect(run.callCount,).toBe(1,);
            const [arguments_,] = run.firstCall.args;
            expect(arguments_,).toEqual({ command: 'uname -a', },);
            // The create-fresh path is selected by the absence of `from`, not by an
            // explicit `undefined`, so the key must not be there at all.
            expect(Object.hasOwn(arguments_, 'from',),).toBe(false,);
          },
        },),
      ],
    },),

    //endregion exec_in_vm and run_in_vm

    //region list_vms and update_templates: the two tools that take no required argument

    describe({
      name: listTool.name,
      children: [
        it({
          name: 'reports that no VMs were found when the backend lists none',
          fn: async ctx => {
            // The empty branch is this case's own state rather than the fake's default.
            ctx.sinon.stub(fakeBackend, 'list',).resolves([],);

            const response = await listTool.handler({ backend: FAKE_BACKEND_KIND, },);

            expect(firstText(response,),).toBe('No VMs found.',);
          },
        },),
        it({
          name: 'reports one name and state line per listed VM',
          fn: async () => {
            const response = await listTool.handler({ backend: FAKE_BACKEND_KIND, },);

            expect(firstText(response,),).toBe('dev-01: running\nwin-01: shut off',);
          },
        },),
      ],
    },),

    describe({
      name: updateTool.name,
      children: [
        it({
          name: 'reports success after the backend refreshed its images',
          fn: async ctx => {
            const update = ctx.sinon.spy(fakeBackend, 'update',);

            const response = await updateTool.handler({ backend: FAKE_BACKEND_KIND, },);

            expect(update.callCount,).toBe(1,);
            expect(firstText(response,),).toBe('Templates updated successfully.',);
          },
        },),
      ],
    },),

    //endregion list_vms and update_templates

    //region push_to_vm and pull_from_vm: file transfer

    describe({
      name: pushTool.name,
      children: [
        it({
          name: 'pushes the named host file to the named guest path',
          fn: async ctx => {
            const pushFile = ctx.sinon.spy(fakeBackend, 'pushFile',);

            const response = await pushTool.handler({
              name: 'dev-01',
              hostPath: 'artifact.txt',
              guestPath: '/root/artifact.txt',
              backend: FAKE_BACKEND_KIND,
            },);

            expect(pushFile.callCount,).toBe(1,);
            expect(pushFile.firstCall.args,).toEqual([{
              name: 'dev-01',
              hostPath: 'artifact.txt',
              guestPath: '/root/artifact.txt',
            },],);
            expect(firstText(response,),).toBe(
              'Pushed artifact.txt -> /root/artifact.txt in VM dev-01',
            );
          },
        },),
      ],
    },),

    describe({
      name: pullTool.name,
      children: [
        it({
          name: 'writes the pulled bytes to the host path and reports their count',
          fn: async () => {
            /**
             Disposable directory this case writes the pulled file into.
             */
            const directory = await mkdtemp(join(scratchRoot(), 'mcp-mvm-pull-',),);
            /**
             Host path the handler is asked to write, inside the disposable directory.
             */
            const hostPath = join(directory, 'pulled.txt',);

            const response = await pullTool.handler({
              name: 'dev-01',
              guestPath: '/etc/hostname',
              hostPath,
              backend: FAKE_BACKEND_KIND,
            },);

            // The handler reads the guest path itself and only hands the backend the pair
            // it needs, so the host path never reaches the backend call.
            expect(await readFile(hostPath, 'utf8',),).toBe(PULLED_BYTES.toString('utf8',),);
            expect(firstText(response,),).toBe(
              `Pulled /etc/hostname -> ${hostPath} from VM dev-01 (${
                String(PULLED_BYTES.length,)
              } bytes)`,
            );
          },
        },),
      ],
    },),

    //endregion push_to_vm and pull_from_vm
  ],
},);
