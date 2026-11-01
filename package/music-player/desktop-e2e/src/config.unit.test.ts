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
          only: '',
          outputRoot: '/work/app/target/e2e',
          releaseBinary: '/work/app/target/release/music-player',
          scratchRoot: '/home/tester/temp/agent/music-player-e2e',
        },);
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
      name: 'reads only the variables that are set',
      fn: async ({ expect, }) => {
        const names = ['E2E_COMPOSITOR', 'E2E_LIBRARY_GENERATOR', 'E2E_ONLY', 'E2E_OUT', 'E2E_TMP',] as const;
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
