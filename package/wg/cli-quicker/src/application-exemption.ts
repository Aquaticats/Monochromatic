import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { resolveApplicationExemptionCommand, } from './application-exemption-command.ts';
import { BypassRouteError, } from './errors.ts';
import {
  run,
} from './runner.ts';
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
 Explicit target UID environment variable for non-sudo service execution.
 */
const TARGET_UID_ENV = 'WG_QUICKER_EXEMPT_UID';

/**
 Environment values needed to select application user.
 */
export type ApplicationUidEnvironment = {
  /**
   Explicit override for service or capability-based execution.
   */
  readonly WG_QUICKER_EXEMPT_UID?: string;

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
 Interpreter, and Qure.
 
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
   Explicit service override or original sudo user.
   */
  const configured = environment.WG_QUICKER_EXEMPT_UID ?? environment.SUDO_UID;
  /**
   Candidate text, using non-root effective UID only when no environment value exists.
   */
  const candidate = configured ?? ((currentUid > 0) ? String(currentUid,) : undefined);
  if (candidate === undefined) {
    throw new BypassRouteError(
      `Application exemptions require SUDO_UID or ${TARGET_UID_ENV} when wg-quicker runs as root.`,
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
 Starts detached Rust watcher after bypass routing exists.
 
 @param interfaceName - Stable watcher key and tunnel owner.
 
 @param mark - Socket mark routed through bypass table.
 
 @example
 ```ts
 await startApplicationExemptions({ interfaceName: 'wg0', mark: 100 });
 ```
 */
export async function startApplicationExemptions(
  {
    interfaceName,
    mark,
  }: {
    readonly interfaceName: string;
    readonly mark: number;
  },
): Promise<void> {
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
   Explicit service execution target when configured.
   */
  const explicitUid = process
    .env
    .WG_QUICKER_EXEMPT_UID;
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
    environment: {
      ...(explicitUid === undefined
        ? {}
        : { WG_QUICKER_EXEMPT_UID: explicitUid, }),
      ...(sudoUid === undefined
        ? {}
        : { SUDO_UID: sudoUid, }),
    },
    currentUid,
  },);
  fl.debug(`starting application watcher for ${interfaceName}, uid=${String(uid,)}`,);
  /**
   Exact companion path validated before command execution.
   */
  const command = await resolveApplicationExemptionCommand();
  await run({
    command,
    args: applicationWatchStartArgs({
      interfaceName,
      mark,
      uid,
    },),
  },);
}

/**
 Stops watcher when current config or persisted bypass state proves exemptions were enabled.
 
 @param interfaceName - Stable watcher key and tunnel owner.
 
 @param configured - Whether current config still carries `ExemptMark`.
 
 @example
 ```ts
 await stopApplicationExemptions({ interfaceName: 'wg0', configured: true });
 ```
 */
export async function stopApplicationExemptions(
  {
    interfaceName,
    configured,
  }: {
    readonly interfaceName: string;
    readonly configured: boolean;
  },
): Promise<void> {
  /**
   Persisted bypass state detects changed config whose `ExemptMark` was removed.
   */
  const state = configured
    ? BYPASS_STATE_ABSENT
    : await readBypassState({ interfaceName, },);
  if ((!configured) && (state === BYPASS_STATE_ABSENT))
    return;
  /**
   Exact companion path validated before command execution.
   */
  const command = await resolveApplicationExemptionCommand();
  await run({
    command,
    args: applicationWatchStopArgs({ interfaceName, },),
  },);
}
