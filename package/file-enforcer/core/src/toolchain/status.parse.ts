/**
 Parsers for the listing output the `status.ts` route observers read: whitespace-delimited rows from
 `uv tool list` and the JSON from `pnpm list --global`.
 */

//region Text helpers

/**
 Whether a string is exactly one whitespace character.

 @param ch - Text to test

 @returns `true` for a single whitespace character
 */
function isWhitespaceCharacter(ch: string,): boolean {
  return (ch.length > 0) && (ch.trim() === '');
}

/**
 Splits a line at its first whitespace character.

 @param line - Text to split

 @returns The text before the first whitespace and the text from it onward; the second part is empty when the line
 has no whitespace

 @example
 ```ts
 splitAtFirstWhitespace('rustc 1.9.0',); // ['rustc', ' 1.9.0']
 ```
 */
export function splitAtFirstWhitespace(line: string,): readonly [
  string,
  string,
] {
  for (let index = 0; index < line.length; index += 1) {
    if (isWhitespaceCharacter(line.charAt(index,),))
      return [
        line.slice(
          0,
          index,
        ),
        line.slice(index,),
      ];
  }
  return [
    line,
    '',
  ];
}

//endregion Text helpers

//region uv

/**
 Reads one `name vVERSION` row of `uv tool list`.

 @param line - One output line

 @returns The registry name and the version without its leading `v`; empty when the line is not a row
 */
function uvToolRow(line: string,): readonly string[] {
  /**
   The name token and the remainder of the line.
   */
  const [name, rest,] = splitAtFirstWhitespace(line,);
  if (name === '')
    return [];
  /**
   Version token that follows the whitespace.
   */
  const [token,] = splitAtFirstWhitespace(rest.trimStart(),);
  if (token === '')
    return [];
  return [
    name,
    // A lone `v` is a version, not a prefix.
    (token.length > 1) && token.startsWith('v',)
      ? token.slice(1,)
      : token,
  ];
}

/**
 Reads `uv tool list` output into versions keyed by name with hyphens spelled as underscores.

 `uv tool list` prints registry-normalized names (`huggingface-hub`) where the plan may spell the PyPI form with
 underscores, so both sides compare in the underscore spelling. The first row for a name wins.

 @param output - Standard output of `uv tool list`

 @returns Version per underscore-spelled name

 @example
 ```ts
 parseUvToolList('huggingface-hub v2.2.0\n',).get('huggingface_hub',); // '2.2.0'
 ```
 */
export function parseUvToolList(output: string,): ReadonlyMap<string, string> {
  /**
   Versions keyed by underscore-spelled name.
   */
  const versions = new Map<string, string>();
  for (const line of output.split('\n',)) {
    /**
     Name and version of this row, or nothing for other lines.
     */
    const [name, version,] = uvToolRow(line,);
    if ((name === undefined) || (version === undefined))
      continue;
    /**
     Underscore-spelled name that keys the row.
     */
    const key = name.replaceAll(
      '-',
      '_',
    );
    if (!versions.has(key,))
      versions.set(
        key,
        version,
      );
  }
  return versions;
}

//endregion uv

//region pnpm

/**
 Whether a parsed JSON value is a non-array object.

 @param value - Parsed JSON value

 @returns `true` for a plain object
 */
function isJsonObject(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object')
    && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Reads the global dependency versions out of `pnpm list --global --json` output.

 pnpm emits one root object per entry in an array; older versions emitted the object directly.

 @param parsed - Parsed JSON output

 @returns Installed version per package name

 @example
 ```ts
 globalPnpmVersions([{ dependencies: { typescript: { version: '6.0.3', }, }, },],).get('typescript',); // '6.0.3'
 ```
 */
export function globalPnpmVersions(parsed: unknown,): ReadonlyMap<string, string> {
  /**
   Installed version per package name, later roots overriding earlier ones.
   */
  const versions = new Map<string, string>();
  /**
   Root objects of the output.
   */
  const roots: readonly unknown[] = Array.isArray(parsed,)
    ? parsed
    : [parsed];
  for (const root of roots) {
    if (!isJsonObject(root,))
      continue;
    /**
     Dependency map of one root.
     */
    const { dependencies, } = root;
    if (!isJsonObject(dependencies,))
      continue;
    for (const [name, dependency,] of Object.entries(dependencies,)) {
      if (!isJsonObject(dependency,))
        continue;
      /**
       Version text pnpm recorded for this package.
       */
      const { version, } = dependency;
      if ((typeof version) === 'string')
        versions.set(
          name,
          version,
        );
    }
  }
  return versions;
}

//endregion pnpm
