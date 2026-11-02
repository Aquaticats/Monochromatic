import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { resolveApplicationExemptionCommand, } from './application-exemption-command.ts';
import { sanitizeText, } from './escape-text.ts';
import {
  BypassRouteError,
  WatcherMissingError,
} from './errors.ts';
import {
  run,
} from './runner.ts';
import {
  POLICY_NOT_IN_FORCE_MARKER,
  watcherMissingAtStopReport,
  watcherMissingAbortMessage,
} from './update-policy-notice.ts';
import {
  BYPASS_STATE_ABSENT,
  readBypassState,
} from './tunnel-bypass-state.ts';

/**
 Module logger for application cgroup watcher lifecycle.
 */
const l = tagged({ tag: 'application-exemption', },);

/**
 Maximum Linux UID representable by watcher command UAPI.
 */
const MAX_UID = 4_294_967_295;

/**
 Environment values needed to select application user.
 */
export type ApplicationUidEnvironment = {
  /**
   Original caller identity exported by sudo.
   */
  readonly SUDO_UID?: string;
};

/**
 Reports ASCII decimal text without regular expressions.
 
 @param value - Candidate UID text.
 
 @returns Whether every character is decimal digit.
 
 @example
 ```ts
 isDecimalUid('1000'); // true
 ```
 */
function isDecimalUid(value: string,): boolean {
  if (value === '')
    return false;
  for (const character of value) {
    if ((character < '0') || (character > '9'))
      return false;
  }
  return true;
}

/**
 Resolves user whose `app.slice` contains Ghostty, Steam, Helium, Pale Moon, Firefox Nightly, ChatGPT,
 Interpreter, Qure, and OdyTTY.
 
 Explicit override wins over sudo identity. A capability-based non-root caller
 can use its own UID. Direct root execution must specify an override rather
 than accidentally watching root's application slice.
 
 @param environment - Explicit and sudo UID variables.
 
 @param currentUid - Effective UID of invoking process.
 
 @returns Validated unsigned Linux UID.
 
 @throws {@link BypassRouteError} when no unambiguous application user exists.
 
 @example
 ```ts
 resolveApplicationUid({ environment: { SUDO_UID: '1000' }, currentUid: 0 });
 ```
 */
export function resolveApplicationUid(
  {
    environment,
    currentUid,
  }: {
    readonly environment: ApplicationUidEnvironment;
    readonly currentUid: number;
  },
): number {
  /**
   Original sudo user,
   the only environment value that selects the watched user.
   */
  const configured = environment.SUDO_UID;
  /**
   Candidate text, using non-root effective UID only when no environment value exists.
   */
  const candidate = configured ?? ((currentUid > 0) ? String(currentUid,) : undefined);
  if (candidate === undefined) {
    throw new BypassRouteError(
      'Application exemptions require SUDO_UID when wg-quicker runs as root.',
    );
  }
  if (!isDecimalUid(candidate,))
    throw new BypassRouteError(`Application exemption UID is not decimal: ${candidate}`,);
  /**
   Numeric UID after syntax validation.
   */
  const uid = Number(candidate,);
  if ((!Number
    .isSafeInteger(uid,))
    || (uid < 0)
    || (uid > MAX_UID))
    throw new BypassRouteError(`Application exemption UID is outside unsigned 32-bit range: ${candidate}`,);
  return uid;
}

/**
 Builds exact Rust watcher start argument contract.
 
 @param interfaceName - Stable watcher key.
 
 @param mark - Socket mark routed through bypass table.
 
 @param uid - Desktop session owner.
 
 @returns Fresh command argument list.
 
 @example
 ```ts
 applicationWatchStartArgs({ interfaceName: 'wg0', mark: 100, uid: 1000 });
 ```
 */
export function applicationWatchStartArgs(
  {
    interfaceName,
    mark,
    uid,
  }: {
    readonly interfaceName: string;
    readonly mark: number;
    readonly uid: number;
  },
): readonly string[] {
  return [
    'watch-start',
    interfaceName,
    String(mark,),
    String(uid,),
  ];
}

/**
 Builds exact idempotent watcher stop argument contract.
 
 @param interfaceName - Stable watcher key.
 
 @returns Fresh command argument list.
 
 @example
 ```ts
 applicationWatchStopArgs({ interfaceName: 'wg0' });
 ```
 */
export function applicationWatchStopArgs(
  { interfaceName, }: { readonly interfaceName: string; },
): readonly string[] {
  return [
    'watch-stop',
    interfaceName,
  ];
}

/**
 Picks the loud reports out of the watcher's startup output,
 so a policy that failed to start reaches the output of `up`.

 @param stderr - Standard error of the watcher start command.

 @returns The lines that name a policy not in force,
 escaped, because they embed names a user chose and reach a terminal.

 @example
 ```ts
 startupFailureLines({ stderr: 'POLICY NOT IN FORCE: process watch: x\nother' });
 ```
 */
export function startupFailureLines({ stderr, }: { readonly stderr: string; },): readonly string[] {
  return stderr.split('\n',)
    .filter(function isLoud(line,): boolean {
      return line.includes(POLICY_NOT_IN_FORCE_MARKER,);
    },)
    .map(function escapeLine(line,): string {
      return sanitizeText({ text: line, },);
    },);
}

/**
 What starting the watcher led to.
 */
export type StartOutcome =
  | { readonly ok: true; }
    | {
      readonly ok: false;
      readonly message: string;
      readonly searched: readonly string[]
    };

/**
 What stopping the watcher led to.
 */
export type ExemptionOutcome =
  | { readonly ok: true; }
    | {
      readonly ok: false;
      readonly report: string
    };

/**
 Starts detached Rust watcher after bypass routing exists.
 
 @param interfaceName - Stable watcher key and tunnel owner.
 
 @param mark - Socket mark routed through bypass table.
 
 @param resolveCommand - Finder of the Rust companion,
 replaceable in tests.
 
 @param runCommand - Command runner,
 replaceable in tests.
 
 @returns Success,
 or the message saying the companion was not found,
 with every location searched.
 A missing companion does not throw here:
 the caller decides, and `startExemptionsOrAbort` stops the start.
 
 @example
 ```ts
 await startApplicationExemptions({ interfaceName: 'wg0', mark: 100 });
 ```
 */
export async function startApplicationExemptions(
  {
    interfaceName,
    mark,
    resolveCommand = resolveApplicationExemptionCommand,
    runCommand = run,
  }: {
    readonly interfaceName: string;
    readonly mark: number;
    readonly resolveCommand?: typeof resolveApplicationExemptionCommand;
    readonly runCommand?: typeof run;
  },
): Promise<StartOutcome> {
  /**
   Function-scoped lifecycle logger.
   */
  const fl = tagged({
    tag: startApplicationExemptions.name,
    l,
  },);
  /**
   Current effective UID, defaulting to root semantics when API is unavailable.
   */
  const currentUid = process.geteuid?.() ?? 0;
  /**
   Original caller exported by sudo when present.
   */
  const sudoUid = process
    .env
    .SUDO_UID;
  /**
   Target desktop-session UID.
   */
  const uid = resolveApplicationUid({
    environment: (sudoUid === undefined
      ? {}
      : { SUDO_UID: sudoUid, }),
    currentUid,
  },);
  fl.debug(`starting application watcher for ${interfaceName}, uid=${String(uid,)}`,);
  /**
   Companion found in a root-controlled location,
   or every location searched.
   */
  const command = await resolveCommand();
  if (!command.found) {
    /**
     Message for the error that stops the start.
     */
    const message = watcherMissingAbortMessage({ searched: command.searched, },);
    fl.error(message,);
    return {
      ok: false,
      message,
      searched: command.searched,
    };
  }
  /**
   Output of the start command,
   whose standard error carries the watcher's startup reports.
   */
  const started = await runCommand({
    command: command.path,
    args: applicationWatchStartArgs({
      interfaceName,
      mark,
      uid,
    },),
  },);
  for (const line of startupFailureLines({ stderr: started.stderr, },))
    fl.error(line,);
  return { ok: true, };
}

/**
 Starts the watcher and stops the start when the program cannot be found.

 @param interfaceName - Stable watcher key and tunnel owner.

 @param mark - Socket mark routed through bypass table.

 @param start - Starter of the watcher,
 replaceable in tests.

 @throws {@link WatcherMissingError} when the program is in no root-controlled location,
 so the caller rolls the start back.

 @example
 ```ts
 await startExemptionsOrAbort({ interfaceName: 'wg0', mark: 100 });
 ```
 */
export async function startExemptionsOrAbort(
  {
    interfaceName,
    mark,
    start = startApplicationExemptions,
  }: {
    readonly interfaceName: string;
    readonly mark: number;
    readonly start?: typeof startApplicationExemptions;
  },
): Promise<void> {
  /**
   What the start led to.
   */
  const outcome = await start({
    interfaceName,
    mark,
  },);
  if (!outcome.ok)
    throw new WatcherMissingError({
      message: outcome.message,
      searched: outcome.searched,
    },);
}

/**
 Stops watcher when current config or persisted bypass state proves exemptions were enabled.
 
 @param interfaceName - Stable watcher key and tunnel owner.
 
 @param configured - Whether current config still carries `ExemptMark`.
 
 @param resolveCommand - Finder of the Rust companion,
 replaceable in tests.
 
 @param runCommand - Command runner,
 replaceable in tests.
 
 @returns Success or nothing to stop,
 or the report saying why the watcher could not be stopped.
 A missing companion does not throw;
 the caller decides what it means.
 `down` treats it as a failed step,
 because a watcher started earlier may still run,
 and the rollback of a failed `up` does not.
 
 @example
 ```ts
 await stopApplicationExemptions({ interfaceName: 'wg0', configured: true });
 ```
 */
export async function stopApplicationExemptions(
  {
    interfaceName,
    configured,
    resolveCommand = resolveApplicationExemptionCommand,
    runCommand = run,
  }: {
    readonly interfaceName: string;
    readonly configured: boolean;
    readonly resolveCommand?: typeof resolveApplicationExemptionCommand;
    readonly runCommand?: typeof run;
  },
): Promise<ExemptionOutcome> {
  /**
   Function-scoped lifecycle logger.
   */
  const fl = tagged({
    tag: stopApplicationExemptions.name,
    l,
  },);
  /**
   Persisted bypass state detects changed config whose `ExemptMark` was removed.
   */
  const state = configured
    ? BYPASS_STATE_ABSENT
    : await readBypassState({ interfaceName, },);
  if ((!configured) && (state === BYPASS_STATE_ABSENT))
    return { ok: true, };
  /**
   Companion found in a root-controlled location,
   or every location searched.
   */
  const command = await resolveCommand();
  if (!command.found) {
    /**
     Report that a watcher started earlier may still run.
     */
    const report = watcherMissingAtStopReport({ searched: command.searched, },);
    fl.error(report,);
    return {
      ok: false,
      report,
    };
  }
  await runCommand({
    command: command.path,
    args: applicationWatchStopArgs({ interfaceName, },),
  },);
  return { ok: true, };
}
