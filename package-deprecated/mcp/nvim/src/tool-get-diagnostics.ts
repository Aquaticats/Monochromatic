/**
 * Handler for the `get_diagnostics` MCP tool.
 *
 * Returns the current buffer from every running Neovim instance
 * with diagnostics from both LSP and oxlint CLI, merged and deduplicated.
 *
 * @module
 */

import type { ToolCallResult, } from '@monochromatic-dev/mcp-stdio';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts/tagged.ts';
import { dedupDiagnostics, } from './dedup.ts';
import { formatDiagnostic, } from './format.ts';
import { runOxlint, } from './lint-runner.ts';
import { rootLogger, } from './log.ts';
import {
  type Diagnostic,
  getAllDiagnostics,
  getCurrentFiles,
} from './nvim-client.ts';

import {
  buildHeader,
  formatNotes,
  modifiedCaveat,
} from './tool-helpers.ts';

//region Logging

/**
 Logger for this module, re-tagged with the enclosing function name at each
 site so a log line names the call that produced it.
 */
const getDiagnosticsLogger = tagged({
  l: rootLogger,
  tag: 'tool-get-diagnostics',
},);

//endregion Logging

/**
 * Fetches current-buffer diagnostics from all Neovim instances,
 * merges with oxlint CLI results, and formats for MCP response.
 *
 * @returns MCP tool result with formatted diagnostics text.
 *
 * @example
 * ```ts
 * const result = await handleGetDiagnostics();
 * ```
 */
export async function handleGetDiagnostics(): Promise<ToolCallResult> {
  try {
    /**
     * Current-buffer metadata from each Neovim instance; drives the header and the lint-target list.
     */
    const files = await getCurrentFiles();
    /**
     * Pre-rendered header text; placed at the top of the response so the user sees the path context first.
     */
    const header = buildHeader(files,);

    /**
     * Deduplicated absolute paths of the current buffers; multiple instances may share the same file.
     */
    const uniquePaths = [...new Set(files.map(function getPath(file,) {
      return file.path;
    },),),];

    // Query all editor diagnostics and lint in parallel, then filter to current files.
    // Using getAllDiagnostics gives us per-file grouping across all instances.
    /**
     * Concurrent fetch of editor diagnostics (all buffers) and oxlint output; filtered to current files below.
     */
    const [allEditorFiles, lintResult,] = await Promise.all([
      getAllDiagnostics(),
      runOxlint({ files: uniquePaths, },),
    ],);
    /**
     * Set form of `uniquePaths` so editor entries can be filtered to current buffers in O(1).
     */
    const currentPathSet = new Set(uniquePaths,);

    //region Merge editor and lint diagnostics per file
    /**
     * Path-keyed accumulator that holds editor+lint merge per current file before the final response.
     */
    const mergedByPath = new Map<string, Diagnostic[]>();

    for (const fileEntry of allEditorFiles) {
      if (!currentPathSet.has(fileEntry.path,))
        continue;
      /**
       * Lint diagnostics for this current file; empty when oxlint produced none, which still yields a clean merge.
       */
      const lintDiags = lintResult.diagnostics
        .get(fileEntry.path,)
        ?? [];
      mergedByPath.set(
        fileEntry.path,
        dedupDiagnostics({
          editor: fileEntry.diagnostics,
          lint: lintDiags,
        },),
      );
    }

    // Include current files that have lint-only diagnostics (not reported by LSP)
    for (const filePath of uniquePaths) {
      if (!mergedByPath.has(filePath,)) {
        /**
         * Lint-only diagnostics for this current file; LSP didn't report it, so the lint output stands on its own.
         */
        const lintDiags = lintResult.diagnostics
          .get(filePath,)
          ?? [];
        if (lintDiags.length
          > 0) {
          mergedByPath.set(
            filePath,
            lintDiags,
          );
        }
      }
    }
    //endregion Merge editor and lint diagnostics per file

    /**
     * Mutable notes list; starts from oxlint's notes and may gain an unsaved-buffer caveat below.
     */
    const allNotes = [...lintResult.notes,];
    /**
     * Unsaved-buffer caveat text; empty when every current buffer is saved, so it isn't appended in that case.
     */
    const caveat = modifiedCaveat(files,);
    if (caveat.length
      > 0)
      allNotes.push(caveat,);

    /**
     * Flattened diagnostic list used purely for the empty-vs-non-empty branching below.
     */
    const allMerged = [...mergedByPath.values(),].flat();

    if (allMerged.length
      === 0) {
      return {
        content: [{
          type: 'text',
          text: `${header}\n\nNo diagnostics in current buffers.${
            formatNotes(allNotes,)
          }`,
        },],
      };
    }

    // Single file: flat list. Multiple files: grouped by path.
    if (mergedByPath.size
      === 1) {
      /**
       * Formatted diagnostic lines for the single-file shortcut path; no indent because no section header.
       */
      const lines = allMerged.map(function formatLine(diagnostic,) {
        return formatDiagnostic({ diagnostic, },);
      },);
      return {
        content: [{
          type: 'text',
          text: `${header}\n\n${lines.join('\n',)}${formatNotes(allNotes,)}`,
        },],
      };
    }

    /**
     * Per-file text blocks; each block has the path header followed by indented diagnostic lines.
     */
    const sections = [...mergedByPath.entries(),].map(
      function formatSection([path, diagnostics,],) {
        /**
         * Diagnostic lines for this file; indented two spaces so the path header reads as the section title.
         */
        const lines = diagnostics.map(function formatLine(diagnostic,) {
          return formatDiagnostic({
            diagnostic,
            indent: '  ',
          },);
        },);
        return `${path}\n${lines.join('\n',)}`;
      },
    );

    return {
      content: [{
        type: 'text',
        text: `${header}\n\n${sections.join('\n\n',)}${formatNotes(allNotes,)}`,
      },],
    };
  }
  catch (error: unknown) {
    /**
     * Surface-friendly error text for the MCP response; preserves `Error.message` when available, otherwise stringifies.
     */
    const message = caughtValueText(error,);
    tagged({
      tag: handleGetDiagnostics.name,
      l: getDiagnosticsLogger,
    },)
      .error(`get_diagnostics failed: ${message}`,);
    return {
      content: [{
        type: 'text',
        text: `Error: ${message}`,
      },],
      isError: true,
    };
  }
}
