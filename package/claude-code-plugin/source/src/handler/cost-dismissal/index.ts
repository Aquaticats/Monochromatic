/**
 Claude Code Stop hook that shadow-records size dismissals in thinking.

 Scanning is local and the record carries hashes and indexes only,
 so no reasoning text reaches the log or any external provider.
 The hook never blocks a stop.

 @module
 */

import type {
  PostToolUseInput,
  PostToolUseOutput,
  PreToolUseInput,
  PreToolUseOutput,
  StopInput,
  StopOutput,
} from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import type {
  ReadonlyDeep,
} from 'type-fest';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  findSizeDismissal,
  type CostDismissalHit,
} from './detect.ts';
import {
  appendShadowRecord,
  shadowLogPath,
  type CostDismissalRecord,
} from './shadow-record.ts';
import { readTextUnits, } from './transcript-read.ts';

/**
 Module logger for cost-dismissal shadow records.
 */
const logger = tagged({ tag: 'cost-dismissal', },);

/**
 Environment reads the handler depends on,
 injectable so tests never touch the real state directory.
 */
type CostDismissalDependencies = {
  /**
   Environment to read configuration from.
   */
  readonly env?: ReadonlyDeep<NodeJS.ProcessEnv>;
};

/**
 Events the handler serves.

 Each one sees something the others miss:
 `PreToolUse` names the action a dismissal led to before it runs,
 `PostToolUse` proves the action ran,
 and `Stop` is the only event that sees turn-ending thinking
 with no tool call after it.
 */
type CostDismissalInput = PreToolUseInput | PostToolUseInput | StopInput;

/**
 Output of the cost-dismissal handler.

 The hook only records,
 so it sets no `permissionDecision` and lets every tool call through.
 */
type CostDismissalOutput = PreToolUseOutput | PostToolUseOutput | StopOutput;

/**
 Builds the hook handler that shadow-records size dismissals in thinking.

 Serves `PreToolUse`,
 `PostToolUse`,
 and `Stop` from one implementation,
 with each observation tagged by the event that produced it.
 Scanning is local and the record carries hashes and indexes only,
 so no reasoning text reaches the log or any external provider.
 The handler never throws and never denies:
 a shadow record that cannot be written must not fail the tool call.

 @param dependencies - Ambient inputs, defaulting to `process.env`

 @returns Handler for one hook event

 @example
 ```ts
 const handler = createCostDismissalHandler();
 await handler({ hook_event_name: 'PreToolUse', tool_name: 'Edit', tool_input: {}, tool_use_id: 't', session_id: 's', transcript_path: '/t', cwd: '/c', permission_mode: 'default' });
 ```
 */
function createCostDismissalHandler(
  dependencies: Readonly<CostDismissalDependencies> = {},
): (event: ReadonlyDeep<CostDismissalInput>,) => Promise<CostDismissalOutput> {
  /**
   Resolved ambient inputs.
   */
  const {
    env = process.env,
  } = dependencies;
  /**
   Factory logger so every handler line names this hook.
   */
  const rl = tagged({
    tag: createCostDismissalHandler.name,
    l: logger,
  },);

  return async function costDismissalHandler(event: ReadonlyDeep<CostDismissalInput>,): Promise<CostDismissalOutput> {
    /**
     Handler-scoped logger.
     */
    const l = tagged({
      tag: costDismissalHandler.name,
      l: rl,
    },);
    try {
      /**
       Transcript of the session, absent on events that carry none.
       */
      const transcriptPath = event.transcript_path ?? '';
      /**
       Tail scan of the session transcript, empty when the event carries none.
       */
      const scan = (transcriptPath === '')
        ? {
          units: [],
          rows: 0,
        }
        : await readTextUnits(transcriptPath,);
      /**
       Hits of the whole tail, in block order.
       */
      const hits: CostDismissalHit[] = [];
      for (const unit of scan.units) {
        hits.push(...findSizeDismissal(unit,),);
      }
      /**
       Assistant text units of the tail scan.
       */
      const { units, } = scan;
      /**
       Number of assistant text units scanned.
       */
      const scannedUnits = units.length;
      /**
       Number of hits found in the tail.
       */
      const foundHits = hits.length;
      /**
       Time the record is written.
       */
      const now = new Date()
        .toISOString();
      /**
       Record of this event's observation,
       with tool attribution on the tool events only.
       */
      const record: CostDismissalRecord = (event.hook_event_name === 'Stop')
        ? {
          sessionId: event.session_id,
          time: now,
          transcriptPath,
          hookEvent: event.hook_event_name,
          textBlocks: scannedUnits,
          hits,
        }
        : {
          sessionId: event.session_id,
          time: now,
          transcriptPath,
          hookEvent: event.hook_event_name,
          toolName: event.tool_name,
          toolUseId: event.tool_use_id,
          textBlocks: scannedUnits,
          hits,
        };
      await appendShadowRecord({
        record,
        logPath: shadowLogPath(env,),
      },);
      l.info(`cost-dismissal shadow recorded ${String(foundHits,)} hits of ${String(scannedUnits,)} text units`,);
    } catch (error) {
      l.warn(`cost-dismissal shadow skipped: ${String(error,)}`,);
    }
    return {};
  };
}

/**
 Parses raw stdin as a {@link CostDismissalInput}.

 Input is trusted; it comes from Claude Code's hook dispatch system.

 @param raw - JSON-encoded event payload received on stdin

 @returns the parsed hook event

 @example
 ```ts
 costDismissalParser('{"hook_event_name":"PreToolUse","tool_name":"Edit"}');
 ```
 */
function costDismissalParser(raw: string,): CostDismissalInput {
  /* oxlint-disable-next-line typescript/no-unsafe-type-assertion -- trusted input from Claude Code hook system */
  return JSON.parse(raw,) as CostDismissalInput;
}

/**
 Serializes the cost-dismissal output for stdout.

 No trailing newline; matches Claude Code's wire convention.

 @param output - {@link CostDismissalOutput} handler result to serialize

 @returns JSON-encoded payload for Claude Code's hook reader

 @mutates output - `JSON.stringify` may invoke `toJSON`, getters, or proxy traps.

 @example
 ```ts
 costDismissalWriter({});
 // => '{}'
 ```
 */
function costDismissalWriter(output: CostDismissalOutput,): string {
  return JSON.stringify(output,);
}

/**
 Default handler instance, wired to `process.env`.
 */
const costDismissalHandler: (event: ReadonlyDeep<CostDismissalInput>,) => Promise<CostDismissalOutput> =
  createCostDismissalHandler();

export type {
  CostDismissalDependencies,
  CostDismissalInput,
  CostDismissalOutput,
};

export {
  createCostDismissalHandler,
  costDismissalHandler,
  costDismissalParser,
  costDismissalWriter,
};

export {
  findSizeDismissal,
} from './detect.ts';

export type {
  CostDismissalHit,
} from './detect.ts';

export type {
  CostDismissalRecord,
} from './shadow-record.ts';
