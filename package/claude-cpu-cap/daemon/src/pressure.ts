import { readFile, } from 'node:fs/promises';

/**
 Prefix of the ten-second average in a pressure line.
 */
const AVG10_PREFIX = 'avg10=';

/**
 Reads the ten-second pressure average of a `/proc/pressure` file.

 
 @param file - pressure file such as `/proc/pressure/cpu`
 
 @param kind - `some` (any task stalled) or `full` (all tasks stalled)

 @returns the `avg10` percentage

 @throws when the file has no `avg10` value for the requested kind

 @example
 ```ts
 await readPressure({ file: '/proc/pressure/cpu', kind: 'some', },); // 0.52
 ```
 */
export async function readPressure({
  file,
  kind,
}: Readonly<{
  file: string;
  kind: 'some' | 'full';
}>,): Promise<number> {
  /**
   Contents of the pressure file.
   */
  const text = await readFile(
    file,
    'utf8',
  );
  /**
   The line for the requested kind.
   */
  const line = text.split('\n',)
    .find(function startsWithKind(candidate,) {
      return candidate.startsWith(`${kind} `,);
    },);
  /**
   The avg10 field of that line.
   */
  const field = line?.split(' ',)
    .find(function isAvg10(part,) {
      return part.startsWith(AVG10_PREFIX,);
    },);
  /**
   The avg10 percentage, NaN when the field is not a number.
   */
  const value = Number(field?.slice(AVG10_PREFIX.length,),);
  if ((field === undefined) || Number.isNaN(value,)) throw new Error(`no ${kind} avg10 in ${file}`,);
  return value;
}
