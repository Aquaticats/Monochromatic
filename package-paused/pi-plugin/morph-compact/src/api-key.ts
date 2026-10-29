/**
 Morph API key resolution.
 Checks environment variable first, then falls back to
 reading from ~/.pi/agent/mcp.json where Morph MCP servers
 may store their keys.
 */

import { readFile, } from 'node:fs/promises';
import { homedir, } from 'node:os';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

/**
 Module logger tagged for Morph API key resolution.
 */
const l = tagged({ tag: 'morph-compact:api-key', },);

/**
 Path to pi's MCP configuration file.
 */
const MCP_CONFIG_PATH = join(
  homedir(),
  '.pi',
  'agent',
  'mcp.json',
);

/**
 Shape of the MCP configuration file.
 Keys are stored under `mcpServers.<name>.env.MORPH_API_KEY`.
 */
type McpConfig = {
  mcpServers?: Record<
    string,
    {
      env?: Record<string, string>;
    }
    >;
};

/**
 Sentinel for "resolution ran and found no Morph key". Distinct from the
 literal absence of a cache entry (not yet resolved) and never a nullish
 value, so it survives the no-nullish-union rule while still letting callers
 branch on "no key configured".
 */
export const NO_MORPH_KEY: unique symbol = Symbol('morph api key missing from config',);

/**
 Cache of the resolved Morph API key for the session.
 Map key is the literal `'value'`; presence indicates the cache has been
 populated, and the stored value is {@link NO_MORPH_KEY} when mcp.json
 contained no Morph entry. Using `Map.has` lets us distinguish "not yet
 resolved" from "resolved to no key" without a separate flag.
 */
const apiKeyCache = new Map<'value', string | typeof NO_MORPH_KEY>();

/**
 Read `MORPH_API_KEY` from `~/.pi/agent/mcp.json`.
 Walks all MCP server entries looking for an `env.MORPH_API_KEY` value.
 
 @returns the API key found in mcp.json, or {@link NO_MORPH_KEY}
 
 @example
 ```typescript
 const key = await readKeyFromMcpConfig();
 // key === "sk-..." or NO_MORPH_KEY
 ```
 */
async function readKeyFromMcpConfig(): Promise<string | typeof NO_MORPH_KEY> {
  try {
    /**
     Raw JSON bytes from the user's mcp.json; parsed below.
     */
    const contents = await readFile(
      MCP_CONFIG_PATH,
      'utf8',
    );
    /**
     Parsed mcp.json payload before structural validation.
     */
    const config: unknown = JSON.parse(contents,);
    if (((typeof config) !== 'object') || (config === null))
      return NO_MORPH_KEY;
    /**
     MCP server entries that may carry the Morph key under env.
     */
    const servers = (config as McpConfig).mcpServers;
    if (servers === undefined)
      return NO_MORPH_KEY;

    /**
     Iteration keys for walking each configured server entry.
     */
    const serverNames = Object.keys(servers,);
    for (const serverName of serverNames) {
      /**
       Current server entry; may be missing env block.
       */
      const server = servers[serverName];
      /**
       First env-stored key encountered short-circuits the walk.
       */
      const key = server?.env
        ?.MORPH_API_KEY;
      if ((key !== undefined) && (key !== ''))
        return key;
    }
    return NO_MORPH_KEY;
  }
  catch (error: unknown) {
    // File doesn't exist, unreadable, or invalid JSON: not an error, so the
    // absence of a Morph key is reported.
    tagged({
      tag: readKeyFromMcpConfig.name,
      l,
    },)
      .debug(`Could not read Morph key from mcp.json: ${String(error,)}`,);
    return NO_MORPH_KEY;
  }
}

/**
 Resolve the Morph API key.
 Checks `MORPH_API_KEY` environment variable first,
 then falls back to `~/.pi/agent/mcp.json`.
 Result is cached for the session.
 
 @returns the resolved API key, or {@link NO_MORPH_KEY} if not found
 
 @example
 ```typescript
 const key = await resolveMorphApiKey();
 if (key !== NO_MORPH_KEY) {
   const client = createMorphCompactClient({ morphApiKey: key });
 }
 ```
 */
export async function resolveMorphApiKey(): Promise<string | typeof NO_MORPH_KEY> {
  // Check env var first
  /**
   Direct env override takes precedence over mcp.json.
   */
  const envKey = process.env
    .MORPH_API_KEY;
  if ((envKey !== undefined) && (envKey !== ''))
    return envKey;

  // Return cached value if already resolved
  if (apiKeyCache.has('value',))
    return apiKeyCache.get('value',)
      ?? NO_MORPH_KEY;

  // Read from mcp.json and cache
  /**
   Resolved key from mcp.json; NO_MORPH_KEY when no Morph entry exists.
   */
  const resolved = await readKeyFromMcpConfig();
  apiKeyCache.set(
    'value',
    resolved,
  );
  return resolved;
}

/**
 Reset cached API key.
 Called on session start to allow re-reading if the config changed.
 
 @example
 ```typescript
 pi.on("session_start", () => resetApiKeyCache());
 ```
 */
export function resetApiKeyCache(): void {
  apiKeyCache.clear();
}
