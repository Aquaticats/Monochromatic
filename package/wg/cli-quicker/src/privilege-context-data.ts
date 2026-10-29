import { homedir, } from 'node:os';

import { PrivilegeError, } from './errors.ts';

/**
 Current caller-context schema version.
 */
export const PRIVILEGE_CONTEXT_VERSION = 1;

/**
 Maximum caller-context JSON size accepted from private file.
 */
export const MAX_PRIVILEGE_CONTEXT_BYTES = 65_536;

/**
 Environment crossing sudo through private caller-context file.
 */
export type PrivilegeEnvironment = {
  readonly HOME: string;
  readonly IPINFO_TOKEN?: string;
  readonly WG_ALLOWEDIPS_CACHE_DIRECTORY?: string;
  readonly WG_QUICKER_CALLER_PATH?: string;
  readonly WG_QUICKER_EXEMPT_COMMAND?: string;
  readonly WG_QUICKER_EXEMPT_UID?: string;
  readonly WG_QUICKER_OPENSNITCH_DAEMON_CONFIG?: string;
  readonly WG_QUICKER_OPENSNITCH_SYSTEM_FIREWALL_CONFIG?: string;
  readonly WG_QUICKER_RUNTIME_DIRECTORY?: string;
  readonly XDG_CACHE_HOME?: string;
};

/**
 Serialized caller identity and environment.
 */
export type PrivilegeContext = {
  readonly environment: PrivilegeEnvironment;
  readonly uid: number;
  readonly version: 1;
};

/**
 Allowed keys in serialized environment object.
 */
const ALLOWED_ENVIRONMENT_KEYS = new Set([
  'HOME',
  'IPINFO_TOKEN',
  'WG_ALLOWEDIPS_CACHE_DIRECTORY',
  'WG_QUICKER_CALLER_PATH',
  'WG_QUICKER_EXEMPT_COMMAND',
  'WG_QUICKER_EXEMPT_UID',
  'WG_QUICKER_OPENSNITCH_DAEMON_CONFIG',
  'WG_QUICKER_OPENSNITCH_SYSTEM_FIREWALL_CONFIG',
  'WG_QUICKER_RUNTIME_DIRECTORY',
  'XDG_CACHE_HOME',
],);

/**
 Reports non-null object suitable for property validation.
 
 @param value - Unknown JSON value.
 
 @returns Whether value is record-like.
 
 @example
 ```ts
 isRecord({ version: 1 });
 ```
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  if ((typeof value) !== 'object')
    return false;
  if (value === null)
    return false;
  return !Array.isArray(value,);
}

/**
 Reports positive safe integer UID.
 
 @param value - Unknown UID candidate.
 
 @returns Whether value is valid caller UID.
 
 @example
 ```ts
 isPositiveUid(1000);
 ```
 */
function isPositiveUid(value: unknown,): value is number {
  if ((typeof value) !== 'number')
    return false;
  if (!Number.isSafeInteger(value,))
    return false;
  return value > 0;
}

/**
 Parses JSON with domain-specific error.
 
 @param text - Bounded context text.
 
 @returns Parsed unknown JSON value.
 
 @throws {@link PrivilegeError} when JSON syntax is invalid.
 
 @example
 ```ts
 parseContextJson('{"version":1}');
 ```
 */
function parseContextJson(text: string,): unknown {
  try {
    return JSON.parse(text,);
  }
  catch (error) {
    throw new PrivilegeError(
      'Caller context is not valid JSON.',
      { cause: error, },
    );
  }
}

/**
 Reports whether environment record contains only known string entries.
 
 @param environment - Untrusted environment record.
 
 @returns Whether every key and value is allowed.
 
 @example
 ```ts
 hasValidEnvironmentEntries({ environment: { HOME: '/home/me' } });
 ```
 */
function hasValidEnvironmentEntries(
  { environment, }: { readonly environment: Readonly<Record<string, unknown>>; },
): boolean {
  return Object
    .entries(environment,)
    .every(function validEntry([key, value,],): boolean {
      if (!ALLOWED_ENVIRONMENT_KEYS.has(key,))
        return false;
      return (typeof value) === 'string';
    },);
}

/**
 Captures allowlisted caller environment before sudo resets it.
 
 @returns Caller home and configured wg-quicker values.
 
 @example
 ```ts
 capturePrivilegeEnvironment();
 ```
 */
function capturePrivilegeEnvironment(): PrivilegeEnvironment {
  /**
   Allowlisted source values read at privilege boundary.
   */
  const {
    HOME: home,
    IPINFO_TOKEN: token,
    PATH: callerPath,
    WG_ALLOWEDIPS_CACHE_DIRECTORY: allowedIpsCache,
    WG_QUICKER_EXEMPT_COMMAND: exemptCommand,
    WG_QUICKER_EXEMPT_UID: exemptUid,
    WG_QUICKER_OPENSNITCH_DAEMON_CONFIG: openSnitchDaemonConfig,
    WG_QUICKER_OPENSNITCH_SYSTEM_FIREWALL_CONFIG: openSnitchSystemFirewallConfig,
    WG_QUICKER_RUNTIME_DIRECTORY: runtimeDirectory,
    XDG_CACHE_HOME: xdgCacheHome,
  } = process.env;
  return {
    HOME: home ?? homedir(),
    ...(token === undefined ? {} : { IPINFO_TOKEN: token, }),
    ...(allowedIpsCache === undefined
      ? {}
      : { WG_ALLOWEDIPS_CACHE_DIRECTORY: allowedIpsCache, }),
    ...(callerPath === undefined
      ? {}
      : { WG_QUICKER_CALLER_PATH: callerPath, }),
    ...(exemptCommand === undefined
      ? {}
      : { WG_QUICKER_EXEMPT_COMMAND: exemptCommand, }),
    ...(exemptUid === undefined
      ? {}
      : { WG_QUICKER_EXEMPT_UID: exemptUid, }),
    ...(openSnitchDaemonConfig === undefined
      ? {}
      : { WG_QUICKER_OPENSNITCH_DAEMON_CONFIG: openSnitchDaemonConfig, }),
    ...(openSnitchSystemFirewallConfig === undefined
      ? {}
      : {
        WG_QUICKER_OPENSNITCH_SYSTEM_FIREWALL_CONFIG: openSnitchSystemFirewallConfig,
      }),
    ...(runtimeDirectory === undefined
      ? {}
      : { WG_QUICKER_RUNTIME_DIRECTORY: runtimeDirectory, }),
    ...(xdgCacheHome === undefined
      ? {}
      : { XDG_CACHE_HOME: xdgCacheHome, }),
  };
}

/**
 Captures caller context for serialization.
 
 @returns Caller UID and allowlisted environment.
 
 @throws {@link PrivilegeError} when runtime lacks non-root UID.
 
 @example
 ```ts
 capturePrivilegeContext();
 ```
 */
export function capturePrivilegeContext(): PrivilegeContext {
  /**
   Effective UID captured before privilege transition.
   */
  const uid = process.getuid?.();
  if ((uid === undefined) || (uid === 0))
    throw new PrivilegeError('Caller context requires non-root Linux UID.',);
  return {
    environment: capturePrivilegeEnvironment(),
    uid,
    version: PRIVILEGE_CONTEXT_VERSION,
  };
}

/**
 Parses exact caller-context JSON shape.
 
 @param text - Bounded private-file contents.
 
 @returns Validated caller context.
 
 @throws {@link PrivilegeError} when JSON or shape is invalid.
 
 @example
 ```ts
 parsePrivilegeContext({ text: '{"version":1,"uid":1000,"environment":{"HOME":"/home/me"}}' });
 ```
 */
export function parsePrivilegeContext(
  { text, }: { readonly text: string; },
): PrivilegeContext {
  /**
   Parsed untrusted JSON value.
   */
  const value = parseContextJson(text,);
  if (!isRecord(value,))
    throw new PrivilegeError('Caller context has invalid shape.',);
  /**
   Outer schema fields narrowed independently.
   */
  const {
    environment,
    uid,
    version,
  } = value;
  if (version !== PRIVILEGE_CONTEXT_VERSION)
    throw new PrivilegeError('Caller context has invalid version.',);
  if (!isPositiveUid(uid,))
    throw new PrivilegeError('Caller context has invalid UID.',);
  if (!isRecord(environment,))
    throw new PrivilegeError('Caller context environment is invalid.',);
  if (!hasValidEnvironmentEntries({ environment, },))
    throw new PrivilegeError('Caller context environment is invalid.',);
  /**
   Allowlisted environment fields reconstructed without unsafe assertion.
   */
  const {
    HOME: home,
    IPINFO_TOKEN: token,
    WG_ALLOWEDIPS_CACHE_DIRECTORY: allowedIpsCache,
    WG_QUICKER_CALLER_PATH: callerPath,
    WG_QUICKER_EXEMPT_COMMAND: exemptCommand,
    WG_QUICKER_EXEMPT_UID: exemptUid,
    WG_QUICKER_OPENSNITCH_DAEMON_CONFIG: openSnitchDaemonConfig,
    WG_QUICKER_OPENSNITCH_SYSTEM_FIREWALL_CONFIG: openSnitchSystemFirewallConfig,
    WG_QUICKER_RUNTIME_DIRECTORY: runtimeDirectory,
    XDG_CACHE_HOME: xdgCacheHome,
  } = environment;
  if (((typeof home) !== 'string') || (home === ''))
    throw new PrivilegeError('Caller context HOME is invalid.',);
  return {
    environment: {
      HOME: home,
      ...((typeof token) === 'string' ? { IPINFO_TOKEN: token, } : {}),
      ...((typeof allowedIpsCache) === 'string'
        ? { WG_ALLOWEDIPS_CACHE_DIRECTORY: allowedIpsCache, }
        : {}),
      ...((typeof callerPath) === 'string'
        ? { WG_QUICKER_CALLER_PATH: callerPath, }
        : {}),
      ...((typeof exemptCommand) === 'string'
        ? { WG_QUICKER_EXEMPT_COMMAND: exemptCommand, }
        : {}),
      ...((typeof exemptUid) === 'string'
        ? { WG_QUICKER_EXEMPT_UID: exemptUid, }
        : {}),
      ...((typeof openSnitchDaemonConfig) === 'string'
        ? { WG_QUICKER_OPENSNITCH_DAEMON_CONFIG: openSnitchDaemonConfig, }
        : {}),
      ...((typeof openSnitchSystemFirewallConfig) === 'string'
        ? {
          WG_QUICKER_OPENSNITCH_SYSTEM_FIREWALL_CONFIG: openSnitchSystemFirewallConfig,
        }
        : {}),
      ...((typeof runtimeDirectory) === 'string'
        ? { WG_QUICKER_RUNTIME_DIRECTORY: runtimeDirectory, }
        : {}),
      ...((typeof xdgCacheHome) === 'string' ? { XDG_CACHE_HOME: xdgCacheHome, } : {}),
    },
    uid,
    version: PRIVILEGE_CONTEXT_VERSION,
  };
}
