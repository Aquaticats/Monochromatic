/**
 Morph Compact pi extension entry point.
 
 Replaces pi's default LLM summarization with Morph's line-deletion compression.
 Calls the Morph Compact API via the official SDK, wrapping output with
 explanatory headers so the LLM understands the verbatim-transcript format.
 
 Falls through to pi's default summarization on any error (missing key, API
 failure, split-turn compaction, etc.).
 
 When the session is too small to compact (all messages fit within
 pi's keepRecentTokens budget), cancels the compaction instead of
 falling through to pi's default summarizer, which produces empty
 "(none)" summaries in this case.
 
 For the `/morph-compact` command, compressed context is passed to the
 new pi session via a tiered IPC fallback to avoid exceeding the OS
 argument length limit:
 
 1. Argv (text ≤ 100KB): simplest, zero cleanup
 2. Temp file: one write, one read, one delete
 3. Unix domain socket: avoids data on disk, fallback for full /tmp
 4. TCP localhost: zero filesystem dependency, final fallback
 */

import type {
  ContextEvent,
  ExtensionAPI,
  ExtensionCommandContext,
  SessionBeforeCompactEvent,
} from '@earendil-works/pi-coding-agent';
import { launchTerminal, } from '@monochromatic-dev/terminal-exec-cli/ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  NO_MORPH_KEY,
  resetApiKeyCache,
  resolveMorphApiKey,
} from './api-key.ts';
import {
  handleBeforeCompact,
  resetMissingKeyWarning,
} from './compaction-handler.ts';
import { compressBranch, } from './compress-branch.ts';
import {
  handleSessionStartInject,
  launchWithLargeContext,
  MAX_COMPRESSED_ARG_BYTES,
} from './ipc-launch.ts';
import {
  filterVisibleContextMessages,
  registerVisibleContextRenderer,
} from './visible-context.ts';

//region Session state

/**
 Module-level slot for the pi extension API.
 
 Stored in a `Map` (instead of a top-level `let`) so the `session_start`
 handler can call `pi.getFlag()` and `pi.sendUserMessage()` without
 receiving the API as a parameter. The single key `'value'` distinguishes
 "not yet registered" (no entry) from "registered" (entry present).
 */
const extensionApiSlot = new Map<'value', ExtensionAPI>();

//endregion

//region Event handlers

/**
 Handle the `/morph-compact` command.
 Compresses the session context read-only via the Morph Compact API
 and launches a new terminal running pi with the compressed context.
 
 When the compressed text exceeds {@link MAX_COMPRESSED_ARG_BYTES},
 falls back through IPC tiers in order: temp file, Unix domain
 socket, TCP localhost.
 
 @param args - optional custom instructions for compression focus
 
 @param ctx - extension command context with session access and UI
 */
async function handleMorphCompactCommand({
  args,
  ctx,
}: {
  readonly args: string;
  readonly ctx: ExtensionCommandContext;
},): Promise<void> {
  /**
   Resolved Morph key gates the command early when missing.
   */
  const apiKey = await resolveMorphApiKey();
  if (apiKey === NO_MORPH_KEY) {
    ctx.ui
      .notify(
      'MORPH_API_KEY not set (env or ~/.pi/agent/mcp.json): cannot compress',
      'warning',
      );
    return;
  }

  /**
   Read-only branch view fed to the standalone compressor.
   */
  const branchEntries = ctx.sessionManager
    .getBranch();
  /**
   Snapshot of current context pressure used to choose a ratio.
   */
  const contextUsage = ctx.getContextUsage();
  /**
   Trimmed command-line instructions; empty string maps to undefined below.
   */
  const instructions = args.trim();
  /**
   Optional custom focus forwarded to compressBranch.
   */
  const customInstructions = instructions !== ''
    ? instructions
    : undefined;

  ctx.ui
    .notify(
    'Morph Compact: compressing session...',
    'info',
    );

  try {
    /**
     Compressed branch text routed through the right IPC tier below.
     */
    const compressedText = await compressBranch({
      branchEntries,
      ...((contextUsage !== undefined) ? { contextUsage, } : {}),
      apiKey,
      ...((customInstructions !== undefined) ? { customInstructions, } : {}),
    },);

    // Tier 1: argv (simplest path, zero cleanup)
    // oxlint-disable-next-line eslint-plugin-unicorn/prefer-ternary -- branching on async calls is clearer with if/else
    if (compressedText.length
      <= MAX_COMPRESSED_ARG_BYTES) {
      await launchTerminal({
        dir: ctx.cwd,
        command: [
          'pi',
          compressedText,
        ],
      },);
    }
    else {
      await launchWithLargeContext({
        cwd: ctx.cwd,
        compressedText,
      },);
    }

    ctx.ui
      .notify(
      'Morph Compact: launched new session in a separate terminal',
      'info',
      );
  }
  catch (error) {
    /**
     Best-effort diagnostic surfaced to the user via UI notify.
     */
    const message = caughtValueText(error,);
    ctx.ui
      .notify(
      `Morph Compact failed: ${message}`,
      'error',
      );
  }
}

/**
 Handle the `session_start` event.
 
 Reads compressed context from whichever IPC channel is active
 (checked in priority order: file → Unix socket → TCP) and
 injects it as a user message. Then resets per-session state.
 
 Takes no parameters: the pi API is held in the module-level
 {@link extensionApiSlot}, so the event payload itself is unused
 by this handler.
 */
async function handleSessionStart(): Promise<void> {
  /**
   Late-bound pi API captured at extension init; absent before first registration.
   */
  const api = extensionApiSlot.get('value',);
  if (api !== undefined)
    await handleSessionStartInject(api,);

  resetMissingKeyWarning();
  resetApiKeyCache();
}

//endregion

//region Extension entry point

/**
 Morph Compact pi extension entry point.
 Replaces pi's default LLM summarization with Morph's line-deletion compression.
 
 @example
 ```typescript
 // Auto-discovered from ~/.pi/agent/extensions/morph-compact/index.ts
 // or loaded via pi install / pi -e
 // Requires MORPH_API_KEY env var or entry in ~/.pi/agent/mcp.json
 ```
 
 @param pi - the pi extension API
 */
export default function morphCompact(
  pi: ForeignBorrowed<ExtensionAPI>,
): void {
  extensionApiSlot.set(
    'value',
    pi,
  );

  pi.registerFlag(
    'morph-compact-file',
    {
      description: 'Temp file path for receiving compressed context (internal)',
      type: 'string',
    },
  );
  pi.registerFlag(
    'morph-compact-socket',
    {
      description: 'Unix socket path for receiving compressed context (internal)',
      type: 'string',
    },
  );
  pi.registerFlag(
    'morph-compact-tcp',
    {
      description: 'TCP address (host:port) for receiving compressed context (internal)',
      type: 'string',
    },
  );

  registerVisibleContextRenderer({ pi, },);

  pi.on(
    'context',
    function hideVisibleContextMarkerFromAgent(event: ForeignBorrowed<ContextEvent>,) {
      return {
        messages: filterVisibleContextMessages({
          messages: event.messages,
        },),
      };
    },
  );

  pi.on(
    'session_before_compact',
    /**
     Bridges host compaction event into Morph workflow.
     
     @param event - Host-owned compaction event.
     
     @param ctx - Active extension context.
     
     @returns Morph compaction, cancellation, or fallthrough signal.
     
     @mutates event - Compaction can retain `event.signal` through a dependent `AbortSignal.any` relation.
     */
    async function bridgeBeforeCompact(
      event: ForeignBorrowed<SessionBeforeCompactEvent>,
      ctx,
    ) {
      /**
       Morph outcome mapped to pi's before-compact result contract.
       */
      const outcome = await handleBeforeCompact({
        event,
        ctx,
      },);
      if (outcome.kind
        === 'cancel')
        return { cancel: true, };
      if (outcome.kind
        === 'compaction')
        return { compaction: outcome.result, };
      // Fall through to pi's default compaction without clobbering other
      // extensions' results (the runner preserves a prior result on undefined).
      return undefined;
    },
  );

  pi.registerCommand(
    'morph-compact',
    {
      description: 'Compress session context and launch a new pi terminal with it',
      handler: function bridgeMorphCompactCommand(
        args,
        ctx,
      ) {
        return handleMorphCompactCommand({
          args,
          ctx,
        },);
      },
    },
  );

  pi.on(
    'session_start',
    handleSessionStart,
  );
}

//endregion
