import { readFile, } from 'node:fs/promises';

import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { expandAllowedFromFiles, } from './config-expand.ts';
import { parseConfigText, } from './config-parse.ts';
import type { WireguardConfig, } from './config-types.ts';
import { ConfigError, } from './errors.ts';

/**
 Extension stripped from config basename to derive interface name.
 */
const CONF_EXTENSION = '.conf';

/**
 Directory matching wg-quick bare-name resolution.
 */
const CONFIG_DIR = '/etc/wireguard';

/**
 Module logger for config loading boundary.
 */
const l = tagged({ tag: 'config-load', },);

/**
 Reads config text while translating filesystem failure.
 
 @param path - Resolved config path.
 
 @returns Complete UTF-8 config text.
 
 @throws {@link ConfigError} when path cannot be read.
 
 @example
 ```ts
 await readConfigText({ path: '/etc/wireguard/wg0.conf' });
 ```
 */
async function readConfigText(
  { path, }: { readonly path: string; },
): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error: unknown) {
    l.error(`failed to read ${path}: ${String(error,)}`,);
    throw new ConfigError(
      `Config file does not exist or is unreadable: ${path}`,
      { cause: error, },
    );
  }
}

/**
 Resolves config argument and optionally expands peer source directives.
 
 Bare name resolves under `/etc/wireguard/<name>.conf`.
 Down callers disable expansion so teardown never performs DNS or ASN work.
 
 @param arg - Interface name or explicit config path.
 
 @param expandAllowedIps - Whether peer source files must be generated for `up`.
 
 @returns Parsed config,
 optionally with generated peer lines.
 
 @example
 ```ts
 await loadConfig({ arg: 'wg0', expandAllowedIps: true });
 ```
 */
export async function loadConfig(
  {
    arg,
    expandAllowedIps,
  }: {
    readonly arg: string;
    readonly expandAllowedIps: boolean;
  },
): Promise<WireguardConfig> {
  /**
   Whether argument already names concrete path.
   */
  const isPath = arg.endsWith(CONF_EXTENSION,) || arg.includes('/',);
  /**
   Absolute or caller-relative config path.
   */
  const path = isPath ? arg : `${CONFIG_DIR}/${arg}${CONF_EXTENSION}`;
  /**
   Config basename used to derive interface name.
   */
  const base = path.slice(path.lastIndexOf('/',) + 1,);
  /**
   Interface name without optional `.conf` suffix.
   */
  const interfaceName = base.endsWith(CONF_EXTENSION,)
    ? base.slice(
      0,
      -CONF_EXTENSION.length,
    )
    : base;
  l.debug(`loading ${path} for ${expandAllowedIps ? 'up' : 'down'}`,);
  /**
   Parsed config before optional address-set generation.
   */
  const parsed = parseConfigText({
    interfaceName,
    text: await readConfigText({ path, },),
  },);
  if ((!expandAllowedIps) || (parsed.allowedFromFiles
    .length
    === 0)) {
    l.debug('returning config without source-file expansion',);
    return parsed;
  }
  l.debug(`expanding ${String(parsed.allowedFromFiles
    .length,)} peer source directive(s)`,);
  return expandAllowedFromFiles({ config: parsed, },);
}
