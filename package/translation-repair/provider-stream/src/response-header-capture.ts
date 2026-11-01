//region Response header capture
// THE RESPONSE HEADERS A CALLER ASKED FOR, handed back on the reply. The reply carried a status and a body alone
// until the Claude route (2026-10-22), which needs two kinds of header: `retry-after`, which Anthropic's 429 and 529
// replies carry and the retry ladder honours, and the `anthropic-ratelimit-*` family, which the client logs once on
// the first reply of a run. The caller names what it wants (an exact name, or a prefix ending in an asterisk), so a
// reply that was not asked for headers keeps the shape every older caller and test reads, and no header the caller
// did not name is held.

/**
 Suffix that turns a wanted name into a prefix.
 */
const PREFIX_MARK = '*';

/**
 Whether one header name is one the caller asked for.

 @param name - header name as the response spelled it, in any case

 @param wanted - names, or prefixes ending in an asterisk, the caller asked for

 @returns Whether the caller asked for it

 @example
 ```ts
 const asked = isWantedHeader({ name: 'Retry-After', wanted: ['retry-after',], },);
 ```
 */
function isWantedHeader(
  {
    name,
    wanted,
  }: {
    readonly name: string;
    readonly wanted: readonly string[];
  },
): boolean {
  /**
   The name in the case the wanted list is written in.
   */
  const lower = name.toLowerCase();
  return wanted.some(function matches(candidate,): boolean {
    return candidate.endsWith(PREFIX_MARK,)
      ? lower.startsWith(candidate.slice(
        0,
        -PREFIX_MARK.length,
      ),)
      : lower === candidate;
  },);
}

/**
 Picks the headers a caller asked for off a response.

 @param headers - the response's headers

 @param wanted - names, or prefixes ending in an asterisk, the caller asked for

 @returns The matching headers by lower-case name, none where nothing matched

 @example
 ```ts
 const captured = capturedHeadersOf({ headers: response.headers, wanted: ['retry-after',], },);
 ```
 */
export function capturedHeadersOf(
  {
    headers,
    wanted,
  }: {
    readonly headers: Headers;
    readonly wanted: readonly string[];
  },
): Readonly<Record<string, string>> {
  return Object.fromEntries([...headers.keys(),]
    .filter(function isAsked(name,): boolean {
      return isWantedHeader({
        name,
        wanted,
      },);
    },)
    .map(function pairOf(name,): readonly [
      string,
      string
    ] {
      return [
        name,
        headers.get(name,) ?? '',
      ];
    },),);
}

//endregion Response header capture
