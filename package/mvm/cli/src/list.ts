import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { VM_PREFIX, } from './config.ts';
import { virsh, } from './virsh.ts';

/**
 Logger root for mvm after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

/**
 Single VM entry with its display name and current libvirt state.
 */
export type VmInfo = {
  name: string;
  state: string;
};

/**
 Sentinel returned by {@link parseVirshRow} for header, separator, and other
 non-data rows. A unique symbol models "not a row" without a nullish union.
 */
const NOT_A_DATA_ROW: unique symbol = Symbol(
  'returned for a virsh table line that is not a VM data row',
);

/**
 Checks whether `c` is a single ASCII whitespace character.
 
 @param c - single-character string to inspect
 
 @returns whether `c` is space, tab, newline, carriage return, form feed, or vertical tab
 
 @example
 ```ts
 isWhitespaceChar(' ');  // true
 isWhitespaceChar('a');  // false
 ```
 */
function isWhitespaceChar(c: string,): boolean {
  return (c === ' ')
    || (c === '\t')
    || (c === '\n')
    || (c === '\r')
    || (c === '\f')
    || (c === '\v');
}

/**
 Checks whether `s` consists only of ASCII digits and is non-empty.
 
 @param s - string to inspect
 
 @returns whether every character of `s` is in `[0-9]` and `s` is non-empty
 
 @example
 ```ts
 isDigitString('42'); // true
 isDigitString('');   // false
 isDigitString('1a'); // false
 ```
 */
function isDigitString(s: string,): boolean {
  if (s.length
    === 0)
    return false;
  for (const c of s) {
    if ((c < '0') || (c > '9'))
      return false;
  }
  return true;
}

/**
 Splits `s` on runs of whitespace, dropping empty fragments.
 
 Equivalent to `s.trim().split(/\s+/)` for non-empty results; written as a
 single linear index pass to avoid a regex literal under the `no-regex`
 rule.
 
 @param s - input string
 
 @returns ordered non-empty tokens
 
 @example
 ```ts
 splitOnWhitespace('  a  b\tc'); // ['a', 'b', 'c']
 splitOnWhitespace('   ');       // []
 ```
 */
export function splitOnWhitespace(s: string,): readonly string[] {
  return (function collect(): readonly string[] {
    /**
     Tokens in order; appended once per non-whitespace run so the result is never rebuilt each step.
     */
    const tokens: string[] = [];
    /**
     Scan cursor over `s`; advances monotonically to `s.length` (single linear pass: O(n) time, O(1) stack, no recursion).
     */
    let idx = 0;
    while (idx < s
      .length) {
      if (isWhitespaceChar(s.charAt(idx,),)) {
        idx += 1;
      }
      else {
        /**
         Inclusive start of the non-whitespace run under the cursor; the run is sliced out once its end is reached.
         */
        const start = idx;
        while (idx < s
          .length) {
          if (isWhitespaceChar(s.charAt(idx,),))
            break;
          idx += 1;
        }
        /**
         Non-whitespace run from `start` to the cursor; one slice per token keeps total work linear.
         */
        const token = s.slice(
          start,
          idx,
        );
        tokens.push(token,);
      }
    }
    return tokens;
  })();
}

/**
 Parses a single row of `virsh list --all` output into structured fields.
 
 Mirrors the original `/\s+(?:\d+|-)\s+(\S+)\s+(.+)/` shape: rows have
 leading whitespace, an id column (digits or `-`), a name column
 (non-whitespace), then a state column (one or more whitespace-separated
 tokens). Returns {@link NOT_A_DATA_ROW} for header rows, separator rows, and
 any non-data row.
 
 @param line - single line of virsh tabular output
 
 @returns parsed row or {@link NOT_A_DATA_ROW} when `line` is not a data row
 
 @example
 ```ts
 parseVirshRow(' 1    mvm-foo    running');  // { name: 'mvm-foo', state: 'running' }
 parseVirshRow(' -    mvm-bar    shut off'); // { name: 'mvm-bar', state: 'shut off' }
 parseVirshRow(' Id   Name   State');        // NOT_A_DATA_ROW (header)
 ```
 */
function parseVirshRow(
  line: string,
): VmInfo | typeof NOT_A_DATA_ROW {
  /**
   Minimum token count for a data row: id, name, and at least one state token.
   Header (`Id Name State`) and separator (`---`) rows produce fewer or
   differently-shaped tokens.
   */
  const MIN_DATA_ROW_TOKENS = 3;
  /**
   Whitespace-separated tokens of `line`.
   */
  const tokens = splitOnWhitespace(line,);
  if (tokens.length
    < MIN_DATA_ROW_TOKENS)
    return NOT_A_DATA_ROW;
  /**
   Id column; data rows have digits or a literal `-`.
   */
  const [idToken, vmName, ...stateTokens] = tokens;
  if ((idToken === undefined) || (vmName === undefined))
    return NOT_A_DATA_ROW;
  if ((idToken !== '-') && (!isDigitString(idToken,)))
    return NOT_A_DATA_ROW;
  return {
    name: vmName,
    state: stateTokens.join(' ',),
  };
}

/**
 Queries libvirt for all managed VMs and returns structured info.
 Parses `virsh list --all` output and filters for VMs with the `mvm-` prefix.
 
 @returns Array of VM entries with name (without prefix) and state
 
 @example
 ```ts
 const vms = await list();
 // [{ name: 'dev-01', state: 'running' }, { name: 'dev-02', state: 'shut off' }]
 ```
 */
export async function list(): Promise<readonly VmInfo[]> {
  /**
   Logger scoped to this call so debug output is attributable.
   */
  const rl = tagged({
    tag: list.name,
    l,
  },);
  rl.debug('querying virsh for all VMs',);

  /**
   Raw multi-line output from `virsh list --all`; parsed line by line below.
   */
  const output = await virsh({ args: [
    'list',
    '--all',
  ], },);
  /**
   Each row of the virsh table, including the header and separator rows the parser below filters out.
   */
  const lines = output.split('\n',);

  /**
   Accumulator for prefixed VMs found in the virsh table; returned as the result.
   */
  const vms: VmInfo[] = [];

  for (const line of lines) {
    /**
     Parsed row fields, or {@link NOT_A_DATA_ROW} for header/separator/non-data rows.
     */
    const row = parseVirshRow(line,);
    if (row !== NOT_A_DATA_ROW) {
      /**
       VM name column from the parsed data row.
       */
      const vmName = row.name;
      /**
       State column from the parsed data row; trimmed when emitted because it carries trailing spaces.
       */
      const vmState = row.state;
      if (vmName.startsWith(VM_PREFIX,)) {
        vms.push({
          name: vmName.slice(VM_PREFIX.length,),
          state: vmState.trim(),
        },);
      }
    }
  }

  rl.debug(`found ${String(vms.length,)} managed VMs`,);
  return vms;
}
