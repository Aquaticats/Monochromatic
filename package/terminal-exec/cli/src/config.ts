/**
 Parses `xdg-terminals.list` config files to extract explicit terminal preferences,
 exclusions, and TerminalArgExec defaults.
 
 @module
 */

import { readFile, } from 'node:fs/promises';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

/**
 Logger root for terminal-exec after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: parentLogger, },);
 ```
 */
const parentLogger = tagged({ tag: 'terminal-exec', },);

/**
 Tagged logger for this module.
 */
const l = tagged({
  tag: 'config',
  l: parentLogger,
},);

/**
 Sentinel returned by {@link parseDirective} for a line that is not a
 well-formed `/execarg_default:` directive. A `unique symbol`; callers narrow
 with `=== MALFORMED_DIRECTIVE`.
 */
const MALFORMED_DIRECTIVE: unique symbol = Symbol('terminal-exec/config directive cannot be parsed',);

/**
 Result of parsing all config files.
 */
export type ConfigResult = {
  /**
   Explicitly listed entry IDs in priority order.
   */
  readonly entryIds: readonly string[];
  /**
   Entry IDs excluded from fallback scanning.
   */
  readonly excludedIds: ReadonlySet<string>;
  /**
   Default TerminalArgExec values keyed by entry ID.
   */
  readonly execArgDefaults: ReadonlyMap<string, string>;
};

/**
 Reads and parses all xdg-terminals.list config files.
 Lines starting with `#` or empty lines are ignored.
 Lines starting with `-` exclude an entry ID from fallback.
 Lines starting with `/` are directives (e.g. `/execarg_default:id:arg`),
 handed to {@link parseDirective} for parsing.
 All other lines are explicit entry ID preferences.
 Every setting follows the path order: the first file naming an id decides its
 execarg default, its fallback exclusion, or its inclusion, so a system file
 never overrides the user's own configuration.
 
 @param paths - Config file paths in descending priority order.
 
 @returns Parsed configuration with entries, exclusions, and defaults.
 
 @example
 ```ts
 const config = await parseConfigFiles({ paths: ['/home/user/.config/xdg-terminals.list'] })
 // config.entryIds === ['com.mitchellh.ghostty.desktop']
 ```
 */
export async function parseConfigFiles(
  { paths, }: { readonly paths: readonly string[]; },
): Promise<ConfigResult> {
  /**
   Mutable accumulator of explicit entry preferences in priority order.
   */
  const entryIds: string[] = [];
  /**
   Ids removed from fallback by the first `-` marker that names them.
   */
  const excludedIds = new Set<string>();
  /**
   Ids whose fallback exclusion the first `+` marker that names them prevents.
   */
  const includedIds = new Set<string>();
  /**
   Per-id execarg defaults from the first `/execarg_default:` directive naming the id.
   */
  const execArgDefaults = new Map<string, string>();

  for (const path of paths) {
    /**
     Empty default lets the catch path continue to the next config file without restructuring.
     */
    let text = '';
    try {
      /* oxlint-disable-next-line no-await-in-loop -- sequential: config files override in priority order */
      text = await readFile(
        path,
        'utf8',
      );
    }
    catch (error) {
      if (!(Error.isError(error,)))
        throw error;

      continue;
    }

    l.debug(`reading config '${path}'`,);

    for (const rawLine of text.split('\n',)) {
      /**
       Whitespace tolerance before prefix-character dispatch.
       */
      const line = rawLine.trim();
      if ((line.length
        === 0) || line
          .startsWith('#',))
        continue;

      if (line.startsWith('/',)) {
        /**
         Parsed execarg-default entry; MALFORMED_DIRECTIVE for non-matching or malformed directives.
         */
        const directive = parseDirective({ line, },);
        if (directive !== MALFORMED_DIRECTIVE) {
          /**
           Entry id and default arg from the matched directive.
           */
          const [entryId, defaultArg,] = directive;
          if (execArgDefaults.has(entryId,)) {
            l.debug(`kept the higher-priority TerminalArgExec default for '${entryId}'`,);
            continue;
          }
          execArgDefaults.set(
            entryId,
            defaultArg,
          );
          l.debug(`added TerminalArgExec default '${defaultArg}' for '${entryId}'`,);
        }
        continue;
      }

      if (line.startsWith('-',)) {
        /**
         Entry id without the leading `-` exclusion marker.
         */
        const id = line.slice(1,);
        if (!includedIds.has(id,)) {
          excludedIds.add(id,);
          l.debug(`excluding entry '${id}' from fallback`,);
        }
        continue;
      }

      if (line.startsWith('+',)) {
        /**
         Entry id without the leading `+` inclusion marker.
         */
        const id = line.slice(1,);
        if (excludedIds.has(id,)) {
          l.debug(`kept the higher-priority exclusion of entry '${id}'`,);
          continue;
        }
        includedIds.add(id,);
        continue;
      }

      entryIds.push(line,);
      l.debug(`explicit entry '${line}'`,);
    }
  }

  return {
    entryIds,
    excludedIds,
    execArgDefaults,
  };
}

/**
 Parses a `/`-prefixed directive line into an execarg-default entry.
 
 @param line - Raw directive line including the leading `/`.
 
 @returns `[entryId, defaultArg]` for a well-formed `/execarg_default:` directive; {@link MALFORMED_DIRECTIVE} for other or malformed directives.
 
 @example
 ```ts
 parseDirective({ line: '/execarg_default:org.xterm:-e' }); // ['org.xterm', '-e']
 parseDirective({ line: '/unknown' });                      // MALFORMED_DIRECTIVE
 ```
 */
function parseDirective(
  { line, }: { readonly line: string; },
): readonly [
  string,
  string,
] | typeof MALFORMED_DIRECTIVE {
  /**
   Directive prefix lifted to a name for the slice math below.
   */
  const EXECARG_PREFIX = '/execarg_default:';
  if (!line.startsWith(EXECARG_PREFIX,))
    return MALFORMED_DIRECTIVE;

  /**
   Directive payload; format is `<entryId>:<defaultArg>`.
   */
  const rest = line.slice(EXECARG_PREFIX.length,);
  /**
   Separator between entry id and default arg; -1 means malformed and skipped.
   */
  const colonIdx = rest.indexOf(':',);
  if (colonIdx === (-1))
    return MALFORMED_DIRECTIVE;

  /**
   Target entry id for the default.
   */
  const entryId = rest.slice(
    0,
    colonIdx,
  );
  /**
   Default execarg value associated with the entry id.
   */
  const defaultArg = rest.slice(colonIdx + 1,);
  return [
    entryId,
    defaultArg,
  ];
}
