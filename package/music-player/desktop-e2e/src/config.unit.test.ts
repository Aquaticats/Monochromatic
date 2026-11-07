/**
 Tests for the run configuration: defaults derived from an injected home and app directory, and the overrides.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  loadRunConfig,
  overridesFromEnvironment,
} from './config.ts';
import { DEFAULT_UNIT_LIMITS, } from './session-launch.ts';

await describe({
  name: loadRunConfig.name,
  children: [
    it({
      name: 'derives every default from the app directory and the home directory',
      fn: async ({ expect, }) => {
        const config = loadRunConfig({
          appDir: '/work/app',
          home: '/home/tester',
          overrides: {},
          defaultCompositorPath: '/mise/installs/nested/bin/nested',
        },);
        expect(config,).toEqual({
          appDir: '/work/app',
          compositorPath: '/mise/installs/nested/bin/nested',
          generatorPath: '/home/tester/temp/agent/agent-scale/make-scale-library.ts',
          gpu: '',
          only: '',
          outputRoot: '/work/app/target/e2e',
          releaseBinary: '/work/app/target/release/music-player',
          scratchRoot: '/home/tester/temp/agent/music-player-e2e',
          unitLimits: DEFAULT_UNIT_LIMITS,
          visible: false,
        },);
      },
    },),
    it({
      name: 'asks for a visible window only when the override says so, and for a named device only when given one',
      fn: async ({ expect, }) => {
        const base = {
          appDir: '/work/app',
          home: '/home/tester',
          defaultCompositorPath: '/c',
        } as const;
        expect(loadRunConfig({ ...base, overrides: {}, },).visible,).toBe(false,);
        expect(loadRunConfig({ ...base, overrides: { visible: false, }, },).visible,).toBe(false,);
        expect(loadRunConfig({ ...base, overrides: { visible: true, }, },).visible,).toBe(true,);
        expect(loadRunConfig({ ...base, overrides: { gpu: 'software', }, },).gpu,).toBe('software',);
      },
    },),
    it({
      name: 'lets the overrides replace the generator, filter, output and scratch paths',
      fn: async ({ expect, }) => {
        const config = loadRunConfig({
          appDir: '/work/app',
          home: '/home/tester',
          overrides: { generatorPath: '/g.ts', only: 'seek', outputRoot: '/out', scratchRoot: '/tmp-x', },
          defaultCompositorPath: '/mise/installs/nested/bin/nested',
        },);
        expect(config.generatorPath,).toBe('/g.ts',);
        expect(config.only,).toBe('seek',);
        expect(config.outputRoot,).toBe('/out',);
        expect(config.scratchRoot,).toBe('/tmp-x',);
        expect(config.releaseBinary,).toBe('/work/app/target/release/music-player',);
      },
    },),
    it({
      name: 'lets the overrides replace the memory, CPU and task limits of a session unit',
      fn: async ({ expect, }) => {
        const config = loadRunConfig({
          appDir: '/work/app',
          home: '/home/tester',
          overrides: { unitMemoryMax: '3G', unitCpuQuota: '700%', unitTasksMax: '77', },
          defaultCompositorPath: '/mise/installs/nested/bin/nested',
        },);
        expect(config.unitLimits,).toEqual({ memoryMax: '3G', cpuQuota: '700%', tasksMax: '77', },);
      },
    },),
    it({
      name: 'gives every session unit a memory, CPU and task limit when no override is set',
      fn: async ({ expect, }) => {
        const config = loadRunConfig({
          appDir: '/work/app',
          home: '/home/tester',
          overrides: {},
          defaultCompositorPath: '/mise/installs/nested/bin/nested',
        },);
        expect(Object.keys(config.unitLimits,).toSorted(),).toEqual(['cpuQuota', 'memoryMax', 'tasksMax',],);
      },
    },),
    it({
      name: 'lets the compositor path be replaced over the installed one, for a build that is not installed yet',
      fn: async ({ expect, }) => {
        const config = loadRunConfig({
          appDir: '/work/app',
          home: '/home/tester',
          overrides: { compositorPath: '/work/nws/target/release/monochromatic-nested-wayland-session', },
          defaultCompositorPath: '/mise/installs/nested/bin/nested',
        },);
        expect(config.compositorPath,).toBe('/work/nws/target/release/monochromatic-nested-wayland-session',);
      },
    },),
  ],
},);

await describe({
  name: overridesFromEnvironment.name,
  concurrency: 1,
  children: [
    it({
      name: 'reads the unit limits from E2E_UNIT_MEMORY_MAX, E2E_UNIT_CPU_QUOTA and E2E_UNIT_TASKS_MAX',
      fn: async ({ expect, }) => {
        const names = ['E2E_UNIT_MEMORY_MAX', 'E2E_UNIT_CPU_QUOTA', 'E2E_UNIT_TASKS_MAX',] as const;
        const saved = names.map((name,) => [name, process.env[name],] as const);
        process.env.E2E_UNIT_MEMORY_MAX = '3G';
        process.env.E2E_UNIT_CPU_QUOTA = '700%';
        process.env.E2E_UNIT_TASKS_MAX = '77';
        const read = overridesFromEnvironment();
        for (const [name, value,] of saved)
          if (value === undefined)
            Reflect.deleteProperty(process.env, name,);
          else
            process.env[name] = value;
        expect(read.unitMemoryMax,).toBe('3G',);
        expect(read.unitCpuQuota,).toBe('700%',);
        expect(read.unitTasksMax,).toBe('77',);
      },
    },),
    it({
      name: 'asks for a visible window only when E2E_VISIBLE is exactly 1, so 0, empty and words never open one',
      fn: async ({ expect, }) => {
        const saved = process.env.E2E_VISIBLE;
        const readWith = (value: string,): boolean => {
          process.env.E2E_VISIBLE = value;
          return overridesFromEnvironment().visible === true;
        };
        const answers = {
          one: readWith('1',),
          zero: readWith('0',),
          empty: readWith('',),
          word: readWith('true',),
          spaced: readWith(' 1',),
        };
        Reflect.deleteProperty(process.env, 'E2E_VISIBLE',);
        const unset = overridesFromEnvironment().visible === true;
        if (saved !== undefined)
          process.env.E2E_VISIBLE = saved;
        expect(answers.one,).toBe(true,);
        expect(answers.zero,).toBe(false,);
        expect(answers.empty,).toBe(false,);
        expect(answers.word,).toBe(false,);
        expect(answers.spaced,).toBe(false,);
        expect(unset,).toBe(false,);
      },
    },),
    it({
      name: 'reads the render device a person names from E2E_GPU',
      fn: async ({ expect, }) => {
        const saved = process.env.E2E_GPU;
        process.env.E2E_GPU = '/dev/dri/renderD128';
        const read = overridesFromEnvironment().gpu;
        if (saved === undefined)
          Reflect.deleteProperty(process.env, 'E2E_GPU',);
        else
          process.env.E2E_GPU = saved;
        expect(read,).toBe('/dev/dri/renderD128',);
      },
    },),
    it({
      name: 'reads only the variables that are set',
      fn: async ({ expect, }) => {
        const names = [
          'E2E_COMPOSITOR',
          'E2E_LIBRARY_GENERATOR',
          'E2E_ONLY',
          'E2E_OUT',
          'E2E_TMP',
          'E2E_UNIT_CPU_QUOTA',
          'E2E_UNIT_MEMORY_MAX',
          'E2E_UNIT_TASKS_MAX',
          'E2E_VISIBLE',
          'E2E_GPU',
        ] as const;
        const saved = names.map((name,) => [name, process.env[name],] as const);
        for (const name of names)
          Reflect.deleteProperty(process.env, name,);
        expect(overridesFromEnvironment(),).toEqual({},);
        process.env.E2E_ONLY = 'seek';
        process.env.E2E_OUT = '/out';
        expect(overridesFromEnvironment(),).toEqual({ only: 'seek', outputRoot: '/out', },);
        process.env.E2E_LIBRARY_GENERATOR = '/g.ts';
        process.env.E2E_TMP = '/t';
        process.env.E2E_COMPOSITOR = '/c/nested';
        expect(overridesFromEnvironment(),).toEqual({
          compositorPath: '/c/nested',
          generatorPath: '/g.ts',
          only: 'seek',
          outputRoot: '/out',
          scratchRoot: '/t',
        },);
        for (const [name, value,] of saved)
          if (value === undefined)
            Reflect.deleteProperty(process.env, name,);
          else
            process.env[name] = value;
      },
    },),
  ],
},);
