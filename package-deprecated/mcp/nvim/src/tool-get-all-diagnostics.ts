/**
 * Handler for the `get_all_diagnostics` MCP tool.
 *
 * Returns diagnostics from both LSP and oxlint CLI across all buffers
 * in every running Neovim instance, merged and deduplicated.
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
  type FileDiagnostics,
  getAllDiagnostics,
} from './nvim-client.ts';

import { formatNotes, } from './tool-helpers.ts';

//region Logging

/**
 Logger for this module, re-tagged with the enclosing function name at each
 site so a log line names the call that produced it.
 */
const getAllDiagnosticsLogger = tagged({
  l: rootLogger,
  tag: 'tool-get-all-diagnostics',
},);

//endregion Logging

/**
 * Fetches all-buffer diagnostics from every Neovim instance,
 * merges with oxlint CLI results, and formats for MCP response.
 *
 * @returns MCP tool result with formatted diagnostics text grouped by file.
 *
 * @example
 * ```ts
 * const result = await handleGetAllDiagnostics();
 * ```
 */
export async function handleGetAllDiagnostics(): Promise<ToolCallResult> {
  try {
    /**
     * Per-file LSP diagnostics from every Neovim instance, already merged across instances.
     */
    const editorFiles = await getAllDiagnostics();
    /**
     * Paths fed to oxlint; matches the set of files the editors currently have open.
     */
    const allPaths = editorFiles.map(function extractPath(fileEntry,) {
      return fileEntry.path;
    },);

    /**
     * CLI lint output keyed by absolute path, plus any notes (e.g. unsaved-buffer caveat).
     */
    const lintResult = await runOxlint({ files: allPaths, },);

    //region Merge editor and lint diagnostics per file
    /**
     * Path-keyed accumulator that holds editor+lint merge per file before the final result list.
     */
    const mergedByPath = new Map<string, Diagnostic[]>();

    for (const fileEntry of editorFiles) {
      /**
       * Lint diagnostics for this path; empty when oxlint produced none, which still yields a clean editor-only merge.
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

    // Include files that only appear in lint output (not open in editor)
    for (const [lintPath, lintDiags,] of lintResult.diagnostics) {
      if (!mergedByPath.has(lintPath,)) {
        mergedByPath.set(
          lintPath,
          lintDiags,
        );
      }
    }
    //endregion Merge editor and lint diagnostics per file

    /**
     * Final list shape consumed by the MCP response; one entry per file with merged diagnostics.
     */
    const result: FileDiagnostics[] = [...mergedByPath.entries(),].map(
      function toFileDiagnostics([path, diagnostics,],) {
        return {
          path,
          diagnostics,
        };
      },
    );

    if (result.length
      === 0) {
      return {
        content: [{
          type: 'text',
          text: `No diagnostics in any buffer.${formatNotes(lintResult.notes,)}`,
        },],
      };
    }

    /**
     * Per-file text blocks; each block has the path header followed by indented diagnostic lines.
     */
    const sections = result.map(function formatSection(fileEntry,) {
      /**
       * Diagnostic lines for this file; indented two spaces so the path header reads as the section title.
       */
      const lines = fileEntry.diagnostics
        .map(function formatLine(diagnostic,) {
        return formatDiagnostic({
          diagnostic,
          indent: '  ',
        },);
      },);
      return `${fileEntry.path}\n${lines.join('\n',)}`;
    },);

    return {
      content: [{
        type: 'text',
        text: `${sections.join('\n\n',)}${formatNotes(lintResult.notes,)}`,
      },],
    };
  }
  catch (error: unknown) {
    /**
     * Surface-friendly error text for the MCP response; preserves `Error.message` when available, otherwise stringifies.
     */
    const message = caughtValueText(error,);
    tagged({
      tag: handleGetAllDiagnostics.name,
      l: getAllDiagnosticsLogger,
    },)
      .error(`get_all_diagnostics failed: ${message}`,);
    return {
      content: [{
        type: 'text',
        text: `Error: ${message}`,
      },],
      isError: true,
    };
  }
}
