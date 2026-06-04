import { createRequire, } from 'node:module';

import type { TsdownPluginOption, } from 'tsdown';

/**
 * Minimal shape needed from `rolldown/experimental` for unbundled DTS generation.
 *
 * @example
 * ```ts
 * const module: RolldownExperimentalModule = await loadRolldownExperimental();
 * module.isolatedDeclarationPlugin();
 * ```
 */
type RolldownExperimentalModule = {
  readonly isolatedDeclarationPlugin: (config?: { readonly stripInternal?: boolean; },) => TsdownPluginOption;
};

/**
 * Tests whether a runtime value can expose named properties safely.
 *
 * @param value - Enables safe property lookup after dynamic import validation.
 *
 * @returns Whether property lookups are safe for subsequent guards.
 *
 * @example
 * ```ts
 * isRecord({ isolatedDeclarationPlugin: () => undefined, }); // true
 * isRecord(null); // false
 * ```
 */
function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  if ((typeof value) !== 'object') {
    return false;
  }
  return value !== null;
}

/**
 * Validates the dynamically resolved rolldown experimental module.
 *
 * @param value - Confirms module namespace came from tsdown's dependency graph.
 *
 * @returns Narrows `value` to the rolldown experimental module shape.
 *
 * @throws When `rolldown/experimental` does not expose `isolatedDeclarationPlugin`.
 *
 * @example
 * ```ts
 * const value: unknown = { isolatedDeclarationPlugin: () => undefined, };
 * assertRolldownExperimentalModule(value);
 * value.isolatedDeclarationPlugin();
 * ```
 */
function assertRolldownExperimentalModule(
  value: unknown,
): asserts value is RolldownExperimentalModule {
  if (!isRecord(value)) {
    throw new Error('rolldown/experimental does not expose isolatedDeclarationPlugin.');
  }
  if ((typeof value.isolatedDeclarationPlugin) !== 'function') {
    throw new Error('rolldown/experimental does not expose isolatedDeclarationPlugin.');
  }
}

/**
 * Loads rolldown experimental exports through tsdown's own dependency graph.
 *
 * @returns Validated rolldown experimental module namespace.
 *
 * @throws When tsdown cannot resolve rolldown's experimental module.
 *
 * @example
 * ```ts
 * const module = await loadRolldownExperimental();
 * module.isolatedDeclarationPlugin();
 * ```
 */
async function loadRolldownExperimental(): Promise<RolldownExperimentalModule> {
  /**
   * Resolved file URL for tsdown's package metadata.
   */
  const tsdownPackageUrl = import.meta.resolve('tsdown/package.json');

  /**
   * CommonJS resolver scoped to tsdown's dependency graph.
   */
  const requireFromTsdown = createRequire(tsdownPackageUrl,);

  /**
   * Module path for tsdown's own rolldown experimental entry.
   */
  const rolldownExperimentalPath = requireFromTsdown
    .resolve('rolldown/experimental');

  /**
   * Unchecked module namespace loaded from tsdown's rolldown copy.
   */
  const module: unknown = await import(rolldownExperimentalPath,);
  assertRolldownExperimentalModule(module,);
  return module;
}

/**
 * Rolldown experimental exports resolved from tsdown's own dependency graph.
 *
 * Builtin rolldown plugins are matched with `instanceof` inside rolldown, so loading
 * a second direct rolldown copy makes the plugin object invisible to tsdown's bundled rolldown.
 * Resolving through `tsdown/package.json` keeps both sides on the same class identity.
 */
const rolldownExperimentalModule = await loadRolldownExperimental();

/**
 * Creates rolldown's builtin isolated declaration plugin for tsdown builds.
 *
 * @returns Plugin instance from tsdown's rolldown copy, preserving builtin plugin identity.
 *
 * @example
 * ```ts
 * import { defineConfig, } from 'tsdown';
 * import { unbundledDtsPlugin, } from './unbundled-dts-plugin.ts';
 *
 * export default defineConfig({ dts: false, plugins: [unbundledDtsPlugin(),], },);
 * ```
 */
export function unbundledDtsPlugin(): TsdownPluginOption {
  return rolldownExperimentalModule.isolatedDeclarationPlugin();
}
