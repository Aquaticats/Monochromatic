/**
 * Retrieves and merges diagnostics across all buffers from all Neovim instances.
 *
 * Groups diagnostics by file path, deduplicating when multiple instances
 * have the same file open.
 *
 * @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts/tagged.ts';
import { uniqueDiagnostics, } from './dedup.ts';
import { rootLogger, } from './log.ts';
import { getAllClients, } from './nvim-connection.ts';
import {
  LUA_GET_ALL_DIAGNOSTICS,
  mapRawDiagnostic,
} from './nvim-lua.ts';
import type {
  Diagnostic,
  FileDiagnostics,
} from './nvim-types.ts';

//region Logging

/**
 Logger for this module, re-tagged with the enclosing function name at each
 site so a log line names the call that produced it.
 */
const allDiagnosticsLogger = tagged({
  l: rootLogger,
  tag: 'nvim-client-all-diagnostics',
},);

//endregion Logging

/**
 * Returns diagnostics across all buffers from all Neovim instances, grouped by file path.
 * When multiple instances have the same file open, their diagnostics are merged under one entry.
 *
 * @returns Array of file-grouped diagnostics from all instances.
 *
 * @example
 * ```ts
 * const files = await getAllDiagnostics();
 * ```
 */
export async function getAllDiagnostics(): Promise<FileDiagnostics[]> {
  /**
   * Every discovered Neovim instance; queried concurrently so total latency tracks the slowest.
   */
  const nvimClients = getAllClients();

  /**
   * Per-instance file-grouped diagnostic lists; merged into a single path-keyed map below.
   */
  const instanceResults = await Promise.all(
    nvimClients.map(async function queryAllBuffers(nvim,) {
      try {
        /* oxlint-disable typescript-eslint/no-unsafe-type-assertion -- Neovim executeLua returns msgpack data matching our Lua query */
        /**
         * Raw msgpack file entries from the Lua bridge; mapped to typed FileDiagnostics below.
         */
        const raw = (await nvim.executeLua(
          LUA_GET_ALL_DIAGNOSTICS,
          [],
        )) as Record<
          string,
          unknown
        >[];
        /* oxlint-enable typescript-eslint/no-unsafe-type-assertion */
        return raw.map(function mapFileEntry(file,) {
          /**
           * Buffer path from the Lua bridge; coerced to empty string when missing so the Map key is always a string.
           */
          const filePath = ((typeof file.path) === 'string') ? file.path : '';
          /* oxlint-disable typescript-eslint/no-unsafe-type-assertion -- Neovim msgpack array narrowed via Array.isArray */
          /**
           * Raw diagnostic records for this file; empty when the Lua bridge returned a non-array (defensive).
           */
          const fileDiags = Array.isArray(file.diagnostics,)
            ? file.diagnostics as Record<string, unknown>[]
            : [];
          /* oxlint-enable typescript-eslint/no-unsafe-type-assertion */
          return {
            path: filePath,
            diagnostics: fileDiags.map(function mapDiag(d,) {
              return mapRawDiagnostic(d,);
            },),
          };
        },);
      }
      catch (error: unknown) {
        tagged({
          tag: queryAllBuffers.name,
          l: allDiagnosticsLogger,
        },)
          .error(`Failed to query instance for all diagnostics: ${caughtValueText(error,)}`,);
        return [];
      }
    },),
  );

  //region Merge diagnostics from all instances by file path
  /**
   * Path-keyed accumulator that gathers every instance's diagnostics for each file before final dedup.
   */
  const byPath = new Map<string, Diagnostic[]>();

  for (const instanceFiles of instanceResults) {
    for (const fileEntry of instanceFiles) {
      /**
       * Diagnostics already collected for this path from earlier instances; extended in place when present.
       */
      const existing = byPath.get(fileEntry.path,);
      if (existing !== undefined)
        existing.push(...fileEntry.diagnostics,);
      else {
        byPath.set(
          fileEntry.path,
          [...fileEntry.diagnostics,],
        );
      }
    }
  }
  //endregion Merge diagnostics from all instances by file path

  return [...byPath.entries(),].map(function toFileDiagnostics([path, diagnostics,],) {
    return {
      path,
      diagnostics: uniqueDiagnostics(diagnostics,),
    };
  },);
}
