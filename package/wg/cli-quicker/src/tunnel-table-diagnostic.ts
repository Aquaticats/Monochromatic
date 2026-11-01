/**
 Address-family flag accepted by `ip` table commands.
 */
export type TableProto = '-4' | '-6';

/**
 Exact stderr emitted when requested family has not instantiated table.
 */
const ABSENT_TABLE_DIAGNOSTIC: Readonly<Record<TableProto, string>> = {
  '-4': 'Error: ipv4: FIB table does not exist.\nDump terminated',
  '-6': 'Error: ipv6: FIB table does not exist.\nDump terminated',
};

/**
 Recognizes only `ip route show` missing-family-table response.
 
 @param proto - Requested address family.
 
 @param exitCode - `ip` process exit status.
 
 @param stderr - Captured command diagnostic.
 
 @returns Whether response means requested family has no table object.
 
 @example
 ```ts
 isAbsentTableDiagnostic({
   proto: '-6',
   exitCode: 2,
   stderr: 'Error: ipv6: FIB table does not exist.\nDump terminated\n',
 });
 ```
 */
export function isAbsentTableDiagnostic(
  {
    proto,
    exitCode,
    stderr,
  }: {
    readonly proto: TableProto;
    readonly exitCode: number;
    readonly stderr: string;
  },
): boolean {
  return (exitCode === 2)
    && (stderr.trimEnd() === ABSENT_TABLE_DIAGNOSTIC[proto]);
}
