/**
 Hook tests for {@link importAttributesPlugin} that need no whole build.
 
 Two branches have no other coverage: resolveId's importer-relative fallback, for
 a specifier rolldown's resolver cannot locate, and load's return for an attribute
 type no handler registers, which must leave the module to other loaders.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import type {
  LoadResult,
  Plugin,
  PluginContext,
  ResolveIdExtraOptions,
  ResolveIdResult,
  ResolvedId,
} from 'rolldown';

import {
  ATTR_QUERY_KEY,
  importAttributesPlugin,
} from '../dist/final/node/index.mjs';

/**
 The plugin's resolveId hook in its function form, taken from rolldown's own type
 so the parameter shapes stay the library's rather than this file's.
 */
type ResolveIdHook = Extract<NonNullable<Plugin['resolveId']>, (...parameters: never[]) => unknown>;

/**
 The plugin's load hook in its function form, likewise taken from rolldown's type.
 */
type LoadHook = Extract<NonNullable<Plugin['load']>, (...parameters: never[]) => unknown>;

/**
 Takes the plugin's resolveId hook in its function form.
 
 The plugin this package returns declares function hooks only, so the object form
 is a programming error rather than a case to handle.
 
 @param plugin - import attributes plugin instance
 
 @returns the hook as a callable function
 
 @throws Error when the plugin declares the hook in another form
 
 @example
 ```ts
 const resolveId = resolveIdHookOf(importAttributesPlugin());
 ```
 */
function resolveIdHookOf(plugin: Plugin,): ResolveIdHook {
  /**
   Hook value the plugin object carries.
   */
  const hook = plugin.resolveId;
  if (hook === undefined)
    throw new Error('the plugin declares no resolveId hook',);
  if (typeof hook !== 'function')
    throw new Error('the plugin declares resolveId in a form this case does not drive',);
  return hook as ResolveIdHook;
}

/**
 Takes the plugin's load hook in its function form.
 
 @param plugin - import attributes plugin instance
 
 @returns the hook as a callable function
 
 @throws Error when the plugin declares the hook in another form
 
 @example
 ```ts
 const load = loadHookOf(importAttributesPlugin());
 ```
 */
function loadHookOf(plugin: Plugin,): LoadHook {
  /**
   Hook value the plugin object carries.
   */
  const hook = plugin.load;
  if (hook === undefined)
    throw new Error('the plugin declares no load hook',);
  if (typeof hook !== 'function')
    throw new Error('the plugin declares load in a form this case does not drive',);
  return hook as LoadHook;
}

/**
 Plugin context whose resolver reports that it located nothing, which is what
 makes the plugin's importer-relative fallback run.
 
 @returns plugin context whose `resolve` answers `null`
 */
function contextResolvingNothing(): PluginContext {
  return {
    resolve: async function resolve() {
      return null;
    },
  } as unknown as PluginContext;
}

/**
 Plugin context whose resolver answers with `resolved`.
 
 @param resolved - descriptor the resolver returns
 
 @returns plugin context whose `resolve` answers with it
 */
function contextResolvingTo(resolved: ResolvedId,): PluginContext {
  return {
    resolve: async function resolve() {
      return resolved;
    },
  } as unknown as PluginContext;
}

/** Extra options the resolveId hook receives in a build, of which only `skipSelf` is rewritten. */
const RESOLVE_OPTIONS: ResolveIdExtraOptions = {
  attributes: {},
  custom: {},
  isEntry: false,
  kind: 'import-statement',
  importerRawId: undefined,
  importerAttributes: {},
};

/**
 Resolves one specifier through the plugin's hook with the given context.
 
 @param plugin - import attributes plugin instance
 
 @param context - plugin context the hook runs with
 
 @param importer - importing module, or missing for an entry
 
 @param source - specifier to resolve
 
 @returns what the hook decided
 */
async function callResolveId({
  plugin,
  context,
  importer,
  source,
}: {
  readonly plugin: ReturnType<typeof importAttributesPlugin>;
  readonly context: PluginContext;
  readonly importer?: string;
  readonly source: string;
},): Promise<ResolveIdResult> {
  /**
   Function form of the resolveId hook.
   */
  const resolveId = resolveIdHookOf(plugin,);
  if (importer === undefined) {
    return await resolveId.call(
      context,
      source,
      undefined,
      RESOLVE_OPTIONS,
    );
  }
  return await resolveId.call(
    context,
    source,
    importer,
    RESOLVE_OPTIONS,
  );
}

/**
 Calls the plugin's load hook for one ID.
 
 @param plugin - import attributes plugin instance
 
 @param id - module ID to load
 
 @returns what the hook decided
 */
async function callLoad({
  plugin,
  id,
}: {
  readonly plugin: ReturnType<typeof importAttributesPlugin>;
  readonly id: string;
},): Promise<LoadResult> {
  /**
   Function form of the load hook.
   */
  const load = loadHookOf(plugin,);
  return await load.call(
    contextResolvingNothing(),
    id,
    {
      rawId: id,
      attributes: {},
    },
  );
}

await describe({
  name: importAttributesPlugin.name,
  children: [
    //region resolveId relative fallback

    it({
      name: 'tags a relative specifier the resolver could not locate against the importer directory',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'import-attributes-hooks-',),);
        /**
         Importer naming a tagged specifier whose file the resolver reports as missing.
         */
        const importerPath = join(directory.path, 'entry.ts',);
        /**
         Specifier the transform would have tagged before the resolver ran.
         */
        const source = `./schema.sql?${ATTR_QUERY_KEY}=text`;

        const resolved = await callResolveId({
          plugin: importAttributesPlugin(),
          context: contextResolvingNothing(),
          importer: importerPath,
          source,
        },);

        expect(resolved,).toEqual({
          id: `${join(directory.path, 'schema.sql',)}?${ATTR_QUERY_KEY}=text`,
          external: false,
        },);
      },
    },),
    it({
      name: 'reports nothing when a bare tagged specifier does not resolve',
      fn: async () => {
        const resolved = await callResolveId({
          plugin: importAttributesPlugin(),
          context: contextResolvingNothing(),
          importer: '/project/entry.ts',
          source: `@scope/pkg?${ATTR_QUERY_KEY}=text`,
        },);

        expect(resolved,).toBeNull();
      },
    },),
    it({
      name: 'reports nothing when an entry tags a specifier the resolver could not locate',
      fn: async () => {
        const resolved = await callResolveId({
          plugin: importAttributesPlugin(),
          context: contextResolvingNothing(),
          source: `./schema.sql?${ATTR_QUERY_KEY}=text`,
        },);

        expect(resolved,).toBeNull();
      },
    },),
    it({
      name: 'reports the resolver result unchanged when it answers',
      fn: async () => {
        /**
         Descriptor a located file resolves to; the hook reads only its id.
         */
        const located: ResolvedId = {
          id: '/project/schema.sql',
          external: false,
          attributes: {},
          moduleSideEffects: true,
          meta: {},
        };
        const resolved = await callResolveId({
          plugin: importAttributesPlugin(),
          context: contextResolvingTo(located,),
          importer: '/project/entry.ts',
          source: `./schema.sql?${ATTR_QUERY_KEY}=text`,
        },);

        expect(resolved,).toEqual({
          id: `/project/schema.sql?${ATTR_QUERY_KEY}=text`,
          external: false,
        },);
      },
    },),
    it({
      name: 'reports nothing for an untagged specifier with no importer to scan',
      fn: async () => {
        const resolved = await callResolveId({
          plugin: importAttributesPlugin(),
          context: contextResolvingNothing(),
          source: './schema.sql',
        },);

        expect(resolved,).toBeNull();
      },
    },),

    //endregion resolveId relative fallback

    //region load

    it({
      name: 'leaves an ID carrying no attribute marker to other loaders',
      fn: async () => {
        expect(await callLoad({ plugin: importAttributesPlugin(), id: '/project/schema.sql', },),)
          .toBeNull();
      },
    },),
    it({
      name: 'leaves a marked ID whose type no handler registers to other loaders',
      fn: async () => {
        expect(await callLoad({
          plugin: importAttributesPlugin(),
          id: `/project/schema.sql?${ATTR_QUERY_KEY}=unknown`,
        },),).toBeNull();
      },
    },),

    //endregion load
  ],
},);
