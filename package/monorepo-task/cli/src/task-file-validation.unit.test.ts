import { mkdtempDisposable, writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import { loadPackageTasks, loadRootTasks, } from '../dist/final/node/index.mjs';

//region Dynamically imported modules must provide executable task declarations

/** Malformed declarations exercise each field family accepted by the loader. */
const invalidTasks = [
  { label: 'absent task dictionary', value: null, },
  { label: 'array task dictionary', value: [], },
  { label: 'null task declaration', value: { build: null, }, },
  ...['description', 'extends', 'dir', 'usage',].map(key => ({
    label: `non-string ${key}`, value: { build: { [key]: false, }, },
  })),
  { label: 'non-list dependencies', value: { build: { depends: 'build', }, }, },
  { label: 'non-string dependency', value: { build: { depends: [false,], }, }, },
  { label: 'non-record environment', value: { build: { env: [], }, }, },
  { label: 'non-string environment value', value: { build: { env: { MODE: false, }, }, }, },
  { label: 'unsupported body', value: { build: { run: false, }, }, },
  { label: 'invalid body sequence', value: { build: { run: ['true', false,], }, }, },
  { label: 'shell without command', value: { build: { run: { shell: 'bash', }, }, }, },
  { label: 'command without shell', value: { build: { run: { run: 'true', }, }, }, },
] as const;

await describe({
  name: 'task file validation',
  children: [
    it({
      name: 'loads supported body forms and reuses root definitions only within the same repository',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'task-file-',),);
        const path = join(directory.path, 'task.ts',);
        await writeFile(path, `export default { templates: {}, tasks: {
          empty: {},
          command: { run: 'true' },
          shell: { run: { shell: 'bash', run: 'true' } },
          direct: { run: async function direct() {} },
          ordered: { depends: ['empty'], run: ['true', { shell: 'bash', run: 'true' }] },
          options: { description: 'fixture', dir: '.', usage: 'arg "file"', env: { MODE: 'test' } }
        } };\n`,);
        const first = await loadRootTasks(directory.path,);
        const repeated = await loadRootTasks(directory.path,);
        expect(first,).toBe(repeated,);
        expect(typeof first.tasks.direct?.run,).toBe('function',);
        expect(first.tasks.options?.env,).toEqual({ MODE: 'test', },);
      },
    },),
    ...invalidTasks.flatMap(({ label, value, }) => ['package', 'root', 'templates',].map(kind => it({
      name: `rejects ${label} in ${kind} definitions`,
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'task-file-invalid-',),);
        const path = join(directory.path, 'task.ts',);
        const exported = kind === 'package' ? value
          : kind === 'root' ? { templates: {}, tasks: value, } : { templates: value, tasks: {}, };
        await writeFile(path, `export default ${JSON.stringify(exported,)};\n`,);
        let caught: unknown;
        try {
          await (kind === 'package' ? loadPackageTasks(path,) : loadRootTasks(directory.path,));
        }
        catch (error) {
          caught = error;
        }
        if (!Error.isError(caught,))
          throw new Error('Expected invalid task definitions to be rejected',);
        expect(caught.name,).toBe('TaskFileError',);
        expect(caught.message,).toContain(path,);
      },
    },),)),
    ...['package', 'root',].map(kind => it({
      name: `rejects a missing default export in a ${kind} task module`,
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'task-file-default-',),);
        const path = join(directory.path, 'task.ts',);
        await writeFile(path, 'export const unrelated = {};\n',);
        let caught: unknown;
        try {
          await (kind === 'package' ? loadPackageTasks(path,) : loadRootTasks(directory.path,));
        }
        catch (error) {
          caught = error;
        }
        if (!Error.isError(caught,))
          throw new Error('Expected missing default export to be rejected',);
        expect(caught.name,).toBe('TaskFileError',);
      },
    },),),
  ],
},);

//endregion Dynamically imported modules must provide executable task declarations
