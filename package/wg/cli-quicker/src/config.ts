/**
 Public config parsing and loading surface.
 
 @module
 */

export { loadConfig, } from './config-load.ts';
export { parseConfigText, } from './config-parse.ts';

/**
 Exposed so built-artifact tests reach the DNS token classifier behind
 `parseConfigText`.
 
 @internal
 */
export { isIpLiteral, } from './config-parse-values.ts';
export type {
  AllowedFromFiles,
  WireguardConfig,
} from './config-types.ts';
