/**
 Runtime-local observation of live processes managed by `@aliou/pi-processes`.
 
 @module
 */

import type {
  ExtensionAPI,
  ToolResultEvent,
} from '@earendil-works/pi-coding-agent';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region Process identity constants

/**
 Public tool name registered by `@aliou/pi-processes`.
 
 @example
 ```ts
 console.log(PROCESS_TOOL_NAME);
 ```
 */
const PROCESS_TOOL_NAME = 'process';

/**
 Public custom-message type emitted for process lifecycle updates.
 
 @example
 ```ts
 console.log(PROCESS_UPDATE_CUSTOM_TYPE);
 ```
 */
const PROCESS_UPDATE_CUSTOM_TYPE = 'ad-process:update';

/**
 Process statuses that still represent live operating-system work.
 
 @example
 ```ts
 console.log(LIVE_PROCESS_STATUSES.running);
 ```
 */
const LIVE_PROCESS_STATUSES = {
  running: true,
  terminating: true,
  terminate_timeout: true,
} as const satisfies Readonly<Record<string, true>>;

/**
 Sentinel returned when an external snapshot does not represent live work.
 
 @example
 ```ts
 console.log(typeof PROCESS_SNAPSHOT_NOT_LIVE);
 ```
 */
const PROCESS_SNAPSHOT_NOT_LIVE: unique symbol = Symbol('process snapshot does not represent live work',);

//endregion Process identity constants

//region Monitor contracts

/**
 Immutable runtime view of live process identities.
 */
type BackgroundProcessState = {
  readonly liveProcessIds: readonly string[];
  readonly terminalProcessIds: readonly string[];
};

/**
 Goal-owned monitor boundary used by settlement policy.
 */
type BackgroundProcessMonitor = {
  /**
   Report whether observed process state contains live work.
   */
  readonly hasLiveBackgroundProcess: () => boolean;
};

/**
 Minimal message-end event accepted from Pi's extension callback.
 */
type ObservedMessageEndEvent = {
  /**
   Finalized Pi message with extension-specific fields guarded at runtime.
   */
  readonly message: unknown;
};

//endregion Monitor contracts

//region Value guards

/**
 Narrow unknown value to property record.
 
 @param value - candidate structured value
 
 @returns whether named property lookup is safe
 
 @example
 ```ts
 isPropertyRecord({ status: 'running' });
 ```
 */
function isPropertyRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return (value !== null)
    && ((typeof value) === 'object');
}

/**
 Test whether external process status still represents live work.
 
 @param status - untrusted status from process result details
 
 @returns whether status is live
 
 @example
 ```ts
 isLiveProcessStatus('running');
 ```
 */
function isLiveProcessStatus(status: unknown,): boolean {
  if ((typeof status) !== 'string')
    return false;
  return Object.hasOwn(
    LIVE_PROCESS_STATUSES,
    status,
  );
}

//endregion Value guards

//region Snapshot transitions

/**
 Read live identity from one external process snapshot.
 
 @param value - candidate `ProcessInfo` object
 
 @returns live process identity or domain sentinel for malformed or terminal snapshots
 
 @example
 ```ts
 liveProcessId({ id: 'proc_1', status: 'running' });
 ```
 */
function liveProcessId(value: unknown,): string | typeof PROCESS_SNAPSHOT_NOT_LIVE {
  if (!isPropertyRecord(value,))
    return PROCESS_SNAPSHOT_NOT_LIVE;
  if ((typeof value.id) !== 'string')
    return PROCESS_SNAPSHOT_NOT_LIVE;
  return isLiveProcessStatus(value.status,)
    ? value.id
    : PROCESS_SNAPSHOT_NOT_LIVE;
}

/**
 Add one process identity without mutating prior state.
 
 @param processIds - current unique identities
 
 @param processId - identity to retain
 
 @returns identity list containing process
 
 @example
 ```ts
 addProcessId({ processIds: [], processId: 'proc_1' });
 ```
 */
function addProcessId(
  {
    processIds,
    processId,
  }: {
    readonly processIds: readonly string[];
    readonly processId: string;
  },
): readonly string[] {
  return processIds.includes(processId,)
    ? processIds
    : [
      ...processIds,
      processId,
    ];
}

/**
 Remove one terminal process identity without mutating prior state.
 
 @param liveProcessIds - current unique identities
 
 @param processId - identity no longer live
 
 @returns identities excluding process
 
 @example
 ```ts
 removeLiveProcessId({ liveProcessIds: ['proc_1'], processId: 'proc_1' });
 ```
 */
function removeLiveProcessId(
  {
    liveProcessIds,
    processId,
  }: {
    readonly liveProcessIds: readonly string[];
    readonly processId: string;
  },
): readonly string[] {
  return liveProcessIds.filter(function isDifferentProcess(liveProcessIdValue,) {
    return liveProcessIdValue !== processId;
  },);
}

//endregion Snapshot transitions

//region Observers

/**
 Apply finalized `process` tool details to monitor state.
 
 Start results add one live identity.
 List results reconcile every currently live identity.
 Other tools and malformed details leave state unchanged.
 
 @param state - current runtime process state
 
 @param event - finalized Pi tool result
 
 @returns next runtime process state
 
 @example
 ```ts
 observeProcessToolResult({ state, event });
 ```
 */
function observeProcessToolResult(
  {
    state,
    event,
  }: {
    readonly state: BackgroundProcessState;
    readonly event: ForeignBorrowed<ToolResultEvent>;
  },
): BackgroundProcessState {
  if (event.toolName !== PROCESS_TOOL_NAME)
    return state;
  /**
   Process tool details guarded before action-specific observation.
   */
  const { details, } = event;
  if (!isPropertyRecord(details,))
    return state;
  if (details.success !== true)
    return state;
  if (details.action === 'list') {
    if (!Array.isArray(details.processes,))
      return state;
    return {
      liveProcessIds: details.processes
        .map(liveProcessId,)
        .filter(function isLiveId(processId,): processId is string {
          return (typeof processId) === 'string';
        },),
      terminalProcessIds: state.terminalProcessIds,
    };
  }
  if (details.action !== 'start')
    return state;
  /**
   Live identity from successful process start result.
   */
  const processId = liveProcessId(details.process,);
  if ((typeof processId) === 'symbol')
    return state;
  /**
   Current live and terminal identities used by delayed-result guard.
   */
  const {
    liveProcessIds,
    terminalProcessIds,
  } = state;
  if (terminalProcessIds.includes(processId,))
    return state;
  return {
    liveProcessIds: addProcessId({
      processIds: liveProcessIds,
      processId,
    },),
    terminalProcessIds,
  };
}

/**
 Apply process lifecycle custom message to monitor state.
 
 @param state - current runtime process state
 
 @param event - finalized Pi message event
 
 @returns next runtime process state
 
 @example
 ```ts
 observeProcessLifecycleMessage({ state, event });
 ```
 */
function observeProcessLifecycleMessage(
  {
    state,
    event,
  }: {
    readonly state: BackgroundProcessState;
    readonly event: ForeignBorrowed<ObservedMessageEndEvent>;
  },
): BackgroundProcessState {
  if (!isPropertyRecord(event.message,))
    return state;
  /**
   Finalized message record inspected for process lifecycle provenance.
   */
  const { message, } = event;
  /**
   Provenance and lifecycle payload from finalized custom message.
   */
  const {
    customType,
    details,
    role,
  } = message;
  if (role !== 'custom')
    return state;
  if (customType !== PROCESS_UPDATE_CUSTOM_TYPE)
    return state;
  if (!isPropertyRecord(details,))
    return state;
  /**
   Process lifecycle details guarded before state update.
   */
  if ((details.kind !== 'lifecycle')
    || ((details.status !== 'exited') && (details.status !== 'killed'))
    || ((typeof details.processId) !== 'string')) {
    return state;
  }
  return {
    liveProcessIds: removeLiveProcessId({
      liveProcessIds: state.liveProcessIds,
      processId: details.processId,
    },),
    terminalProcessIds: addProcessId({
      processIds: state.terminalProcessIds,
      processId: details.processId,
    },),
  };
}

//endregion Observers

//region Registration

/**
 Register process observations and expose current liveness to goal settlement.
 
 @param pi - Pi extension API receiving passive result and message observers
 
 @returns runtime-local liveness monitor
 
 @mutates pi - registers `tool_result` and `message_end` handlers
 
 @example
 ```ts
 const monitor = registerBackgroundProcessMonitor(pi);
 ```
 */
function registerBackgroundProcessMonitor(
  pi: ForeignBorrowed<ExtensionAPI>,
): BackgroundProcessMonitor {
  /* oxlint-disable no-restricted-syntax/no-function-root-let -- Pi result and message handlers jointly maintain one runtime-local immutable process snapshot. */
  /**
   Runtime-local immutable process-state cursor shared by passive observers.
   */
  let state: BackgroundProcessState = {
    liveProcessIds: [],
    terminalProcessIds: [],
  };
  /* oxlint-enable no-restricted-syntax/no-function-root-let */

  pi.on(
    'tool_result',
    function trackProcessToolResult(event: ForeignBorrowed<ToolResultEvent>,) {
      state = observeProcessToolResult({
        state,
        event,
      },);
    },
  );
  pi.on(
    'message_end',
    function trackProcessLifecycleMessage(event: ForeignBorrowed<ObservedMessageEndEvent>,) {
      state = observeProcessLifecycleMessage({
        state,
        event,
      },);
    },
  );

  return {
    hasLiveBackgroundProcess: function hasLiveBackgroundProcess(): boolean {
      /**
       Current immutable identity list read from shared state cursor.
       */
      const { liveProcessIds, } = state;
      return liveProcessIds.length > 0;
    },
  };
}

//endregion Registration

export { registerBackgroundProcessMonitor, };
export type { BackgroundProcessMonitor, };
