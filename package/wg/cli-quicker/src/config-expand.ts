import { readFile, } from 'node:fs/promises';

import { generateAllowedIps, } from '@monochromatic-dev/wg-core-allowedips/ts';

import type {
  AllowedFromFiles,
  WireguardConfig,
} from './config-types.ts';

/**
 Generated line associated with forwarded-config insertion point.
 */
type ExpandedAllowedIps = {
  readonly line: string;
  readonly wgLineIndex: number;
};

/**
 Expands one peer's source files into optional WireGuard line.
 
 @param source - Peer-associated paths and insertion index.
 
 @returns Generated line text and original insertion index.
 
 @example
 ```ts
 await expandAllowedSource({ source });
 ```
 */
async function expandAllowedSource(
  { source, }: { readonly source: AllowedFromFiles; },
): Promise<ExpandedAllowedIps> {
  /**
   Complete allowed and disallowed texts read concurrently.
   */
  const [allowedText, disallowedText,] = await Promise.all([
    readFile(
      source.allowed,
      'utf8',
    ),
    readFile(
      source.disallowed,
      'utf8',
    ),
  ],);
  /**
   Minimized prefix text computed from both files.
   */
  const allowedIps = await generateAllowedIps({
    allowedText,
    disallowedText,
  },);
  /**
   Comma-separated value without module's output newline.
   */
  const value = allowedIps.trim();
  return {
    line: value === '' ? '' : `AllowedIPs = ${value}`,
    wgLineIndex: source.wgLineIndex,
  };
}

/**
 Expands every peer-scoped `AllowedIPsFromFiles` directive in place.
 
 Each generated line is inserted where its directive occurred,
 so multiple peers cannot exchange or inherit each other's address sets.
 Empty generation omits `AllowedIPs` from that peer.
 
 @param config - Parsed config carrying peer-associated source paths.
 
 @returns Config with generated lines inserted into their owning peers.
 
 @example
 ```ts
 await expandAllowedFromFiles({ config });
 ```
 */
export async function expandAllowedFromFiles(
  { config, }: { readonly config: WireguardConfig; },
): Promise<WireguardConfig> {
  if (config.allowedFromFiles
    .length
    === 0)
    return config;
  /**
   Pending independent peer expansions.
   */
  const pendingExpansions: Promise<ExpandedAllowedIps>[] = [];
  for (const source of config.allowedFromFiles)
    pendingExpansions.push(expandAllowedSource({ source, }),);
  /**
   Generated peer lines after concurrent file and lookup work.
   */
  const expansions = await Promise.all(pendingExpansions,);
  /**
   Generated lines keyed by original forwarded-config line position.
   */
  const lineByIndex = new Map<number, string>();
  for (const expansion of expansions) {
    lineByIndex.set(
      expansion.wgLineIndex,
      expansion.line,
    );
  }
  /**
   Original forwarded lines including terminal empty line.
   */
  const lines = config.wgConfig
    .split('\n',);
  /**
   Forwarded lines with each nonempty generation inserted before original successor.
   */
  const expandedLines: string[] = [];
  for (const [index, line,] of lines.entries()) {
    /**
     Generated line for this exact insertion point when present.
     */
    const generated = lineByIndex.get(index,);
    if ((generated !== undefined) && (generated !== ''))
      expandedLines.push(generated,);
    expandedLines.push(line,);
  }
  return {
    ...config,
    wgConfig: expandedLines.join('\n',),
  };
}
