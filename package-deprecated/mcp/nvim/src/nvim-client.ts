/**
 * Public API for querying Neovim diagnostics and file metadata.
 *
 * Connects to all discoverable Neovim instances and exposes
 * functions to retrieve diagnostics and current file info.
 *
 * @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts/tagged.ts';
import { rootLogger, } from './log.ts';
import { getAllClients, } from './nvim-connection.ts';
import {
  LUA_GET_CURRENT_BUF_DIAGNOSTICS,
  LUA_GET_CURRENT_FILE,
  mapRawDiagnostic,
} from './nvim-lua.ts';
import {
  type CurrentFile,
  type Diagnostic,
  normalizeMessage,
  SEVERITY_MAP,
} from './nvim-types.ts';

export { getAllDiagnostics, } from './nvim-client-all-diagnostics.ts';

export {
  normalizeMessage,
  SEVERITY_MAP,
};

export type {
  CurrentFile,
  Diagnostic,
  FileDiagnostics,
} from './nvim-types.ts';

//region Logging

/**
 Logger for this module, re-tagged with the enclosing function name at each
 site so a log line names the call that produced it.
 */
const nvimClientLogger = tagged({
  l: rootLogger,
  tag: 'nvim-client',
},);

//endregion Logging

//region Public API: query diagnostics and file info across all Neovim instances

/**
 * Returns diagnostics for the current buffer from each Neovim instance.
 * Each instance contributes its own current buffer's diagnostics independently.
 *
 * @returns Array of diagnostics from all instances' current buffers, flattened.
 *
 * @example
 * ```ts
 * const diags = await getDiagnostics();
 * ```
 */
export async function getDiagnostics(): Promise<Diagnostic[]> {
  /**
   * Every discovered Neovim instance; queried concurrently to keep latency tied to the slowest, not the sum.
   */
  const nvimClients = getAllClients();

  /**
   * Per-instance diagnostic arrays; flattened before return so callers see one combined list.
   */
  const results = await Promise.all(
    nvimClients.map(async function queryCurrentBuffer(nvim,) {
      try {
        /* oxlint-disable typescript-eslint/no-unsafe-type-assertion -- Neovim executeLua returns msgpack data matching our Lua query */
        /**
         * Raw msgpack records from the Lua bridge; mapped to typed diagnostics below.
         */
        const raw = (await nvim
          .executeLua(
            LUA_GET_CURRENT_BUF_DIAGNOSTICS,
            [],
          )) as Record<string, unknown>[];
        /* oxlint-enable typescript-eslint/no-unsafe-type-assertion */
        return raw
          .map(function mapDiag(d,) {
            return mapRawDiagnostic(d,);
          },);
      }
      catch (error: unknown) {
        tagged({
          tag: queryCurrentBuffer.name,
          l: nvimClientLogger,
        },)
          .error(
            `Failed to query instance for current buffer diagnostics: ${
              caughtValueText(error,)
            }`,
          );
        return [];
      }
    },),
  );

  return results.flat();
}

/**
 * Returns current file metadata from each Neovim instance.
 * Each instance has its own current buffer, so this returns one entry per instance.
 *
 * @returns Array of current file metadata, one per connected Neovim instance.
 *
 * @example
 * ```ts
 * const files = await getCurrentFiles();
 * // => [{ path: "/src/a.ts", ... }, { path: "/src/b.ts", ... }]
 * ```
 */
export async function getCurrentFiles(): Promise<CurrentFile[]> {
  /**
   * Every discovered Neovim instance; queried concurrently so latency tracks the slowest, not the sum.
   */
  const nvimClients = getAllClients();

  /**
   * Per-instance current-file records or `null` when the instance failed; nulls are filtered out below.
   */
  const results = await Promise.all(
    nvimClients.map(async function queryCurrentFile(nvim,) {
      try {
        /* oxlint-disable typescript-eslint/no-unsafe-type-assertion -- Neovim executeLua returns msgpack data matching our Lua query */
        /**
         * Raw record from the Lua bridge; field types are coerced individually below.
         */
        const result = (await nvim.executeLua(
          LUA_GET_CURRENT_FILE,
          [],
        )) as Record<
          string,
          unknown
        >;
        /* oxlint-enable typescript-eslint/no-unsafe-type-assertion */
        return {
          path: ((typeof result.path) === 'string') ? result.path : '',
          filetype: ((typeof result.filetype) === 'string') ? result.filetype : '',
          modified: ((typeof result.modified) === 'boolean') ? result.modified : false,
        };
      }
      catch (error: unknown) {
        tagged({
          tag: queryCurrentFile.name,
          l: nvimClientLogger,
        },)
          .error(`Failed to query instance for current file: ${caughtValueText(error,)}`,);
        return null;
      }
    },),
  );

  return results.filter(function isNotNull(result,): result is CurrentFile {
    return result !== null;
  },);
}

//endregion Public API
