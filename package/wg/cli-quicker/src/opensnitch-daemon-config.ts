import { readFile, } from 'node:fs/promises';
import { isAbsolute, } from 'node:path';

import { OpenSnitchConfigError, } from './errors.ts';

/**
 OpenSnitch release default daemon config path.
 */
const DEFAULT_DAEMON_CONFIG = '/etc/opensnitchd/default-config.json';

/**
 OpenSnitch release default system-firewall path.
 */
const DEFAULT_SYSTEM_FIREWALL_CONFIG = '/etc/opensnitchd/system-fw.json';

/**
 Environment override for custom OpenSnitch daemon config and fixtures.
 */
export const OPENSNITCH_DAEMON_CONFIG_ENVIRONMENT = 'WG_QUICKER_OPENSNITCH_DAEMON_CONFIG';

/**
 Environment override for custom OpenSnitch system-firewall config and fixtures.
 */
export const OPENSNITCH_CONFIG_ENVIRONMENT = 'WG_QUICKER_OPENSNITCH_SYSTEM_FIREWALL_CONFIG';

/**
 Sentinel indicating OpenSnitch daemon config is absent.
 */
export const OPENSNITCH_DAEMON_CONFIG_ABSENT: unique symbol = Symbol('OpenSnitch daemon config absent',);

/**
 Reports non-null JSON object.
 
 @param value - Unknown JSON value.
 
 @returns Whether value is record-like.
 
 @example
 ```ts
 isRecord({ Firewall: 'nftables' });
 ```
 */
function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  if ((typeof value) !== 'object')
    return false;
  if (value === null)
    return false;
  return !Array.isArray(value,);
}

/**
 Reports whether unknown failure carries Node filesystem code.
 
 @param error - Unknown caught value.
 
 @returns Whether value is Node error with optional code.
 
 @example
 ```ts
 isErrnoException({ code: 'ENOENT' });
 ```
 */
function isErrnoException(error: unknown,): error is NodeJS.ErrnoException {
  return Error.isError(error,);
}

/**
 Requires configured path to have process-independent meaning.
 
 @param path - Configured OpenSnitch path.
 
 @param source - Configuration source used in diagnostic.
 
 @returns Validated absolute path.
 
 @throws {@link OpenSnitchConfigError} when path is relative.
 
 @example
 ```ts
 assertAbsolutePath({ path: '/etc/opensnitchd/system-fw.json', source: 'FwOptions.ConfigPath' });
 ```
 */
function assertAbsolutePath(
  {
    path,
    source,
  }: {
    readonly path: string;
    readonly source: string;
  },
): string {
  if (!isAbsolute(path,))
    throw new OpenSnitchConfigError(`OpenSnitch ${source} must be an absolute path: ${path}`,);
  return path;
}

/**
 Resolves OpenSnitch daemon config path.
 
 @returns Explicit override or release default.
 
 @example
 ```ts
 openSnitchDaemonConfigPath();
 ```
 */
function openSnitchDaemonConfigPath(): string {
  /**
   Explicit custom daemon config path when configured.
   */
  const configured = process.env[OPENSNITCH_DAEMON_CONFIG_ENVIRONMENT];
  if ((configured === undefined) || (configured === ''))
    return DEFAULT_DAEMON_CONFIG;
  return assertAbsolutePath({
    path: configured,
    source: OPENSNITCH_DAEMON_CONFIG_ENVIRONMENT,
  },);
}

/**
 Reads daemon config or reports absent OpenSnitch installation.
 
 @param path - Effective daemon config path.
 
 @returns Raw daemon JSON or absence sentinel.
 
 @throws {@link OpenSnitchConfigError} when existing file is unreadable.
 
 @example
 ```ts
 await readDaemonConfig({ path: '/etc/opensnitchd/default-config.json' });
 ```
 */
async function readDaemonConfig(
  { path, }: { readonly path: string; },
): Promise<string | typeof OPENSNITCH_DAEMON_CONFIG_ABSENT> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    if (isErrnoException(error,) && (error.code === 'ENOENT'))
      return OPENSNITCH_DAEMON_CONFIG_ABSENT;
    throw new OpenSnitchConfigError(
      `Cannot read OpenSnitch daemon config: ${path}`,
      { cause: error, },
    );
  }
}

/**
 Parses daemon config with integration-specific diagnostic.
 
 @param text - Raw daemon JSON.
 
 @param path - Effective daemon config path.
 
 @returns Parsed daemon object.
 
 @throws {@link OpenSnitchConfigError} when JSON or root shape is invalid.
 
 @example
 ```ts
 parseDaemonConfig({ text: '{"Firewall":"nftables"}', path: '/tmp/default-config.json' });
 ```
 */
function parseDaemonConfig(
  {
    text,
    path,
  }: {
    readonly text: string;
    readonly path: string;
  },
): Readonly<Record<string, unknown>> {
  /**
   Parsed untrusted daemon config.
   */
  const parsed = (function parseUnknown(): unknown {
    try {
      return JSON.parse(text,);
    }
    catch (error) {
      throw new OpenSnitchConfigError(
        `OpenSnitch daemon config is not valid JSON: ${path}`,
        { cause: error, },
      );
    }
  })();
  if (!isRecord(parsed,))
    throw new OpenSnitchConfigError(`OpenSnitch daemon config has invalid root: ${path}`,);
  return parsed;
}

/**
 Resolves effective system-firewall path from explicit override or daemon config.
 
 Installation requires nftables when `requireNftables` is true.
 Removal skips backend check so owned rules can be cleaned after config changes.
 
 @param requireNftables - Whether unsupported backend must reject startup.
 
 @returns Effective system-firewall path,
 or absence sentinel when startup finds no OpenSnitch daemon config.
 
 @throws {@link OpenSnitchConfigError} when existing daemon config is malformed or unsupported.
 
 @example
 ```ts
 await resolveOpenSnitchSystemFirewallPath({ requireNftables: true });
 ```
 */
export async function resolveOpenSnitchSystemFirewallPath(
  { requireNftables, }: { readonly requireNftables: boolean; },
): Promise<string | typeof OPENSNITCH_DAEMON_CONFIG_ABSENT> {
  /**
   Explicit system-firewall path taking precedence over daemon field.
   */
  const configuredSystemPath = process.env[OPENSNITCH_CONFIG_ENVIRONMENT];
  if ((!requireNftables)
    && (configuredSystemPath !== undefined)
    && (configuredSystemPath !== '')) {
    return assertAbsolutePath({
      path: configuredSystemPath,
      source: OPENSNITCH_CONFIG_ENVIRONMENT,
    },);
  }
  /**
   Effective daemon config path and optional content.
   */
  const daemonPath = openSnitchDaemonConfigPath();
  /**
   Daemon config text or absence sentinel.
   */
  const daemonText = await readDaemonConfig({ path: daemonPath, },);
  if ((typeof daemonText) === 'symbol') {
    if (daemonText !== OPENSNITCH_DAEMON_CONFIG_ABSENT)
      throw new OpenSnitchConfigError('Unexpected OpenSnitch daemon-config result.',);
    if (requireNftables)
      return OPENSNITCH_DAEMON_CONFIG_ABSENT;
    if ((configuredSystemPath === undefined) || (configuredSystemPath === ''))
      return DEFAULT_SYSTEM_FIREWALL_CONFIG;
    return assertAbsolutePath({
      path: configuredSystemPath,
      source: OPENSNITCH_CONFIG_ENVIRONMENT,
    },);
  }
  /**
   Parsed daemon config supplying backend and default system path.
   */
  const daemonConfig = parseDaemonConfig({
    text: daemonText,
    path: daemonPath,
  },);
  if (requireNftables && (daemonConfig.Firewall !== 'nftables')) {
    throw new OpenSnitchConfigError(
      `wg-quicker OpenSnitch auto-configuration requires Firewall = nftables in ${daemonPath}.`,
    );
  }
  if ((configuredSystemPath !== undefined) && (configuredSystemPath !== '')) {
    return assertAbsolutePath({
      path: configuredSystemPath,
      source: OPENSNITCH_CONFIG_ENVIRONMENT,
    },);
  }
  /**
   OpenSnitch firewall options carrying watched system config path.
   */
  const options = daemonConfig.FwOptions;
  if (isRecord(options,)
    && ((typeof options.ConfigPath) === 'string')
    && (options.ConfigPath !== '')) {
    return assertAbsolutePath({
      path: options.ConfigPath,
      source: 'FwOptions.ConfigPath',
    },);
  }
  return DEFAULT_SYSTEM_FIREWALL_CONFIG;
}
