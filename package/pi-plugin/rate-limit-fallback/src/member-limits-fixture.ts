/**
 The window and output limit each writer member reports, and the requests derived from them.
 @module
 */

/**
 Limits one member reports to Pi's registry.
 */
export type MemberLimits = Readonly<{
  contextWindow: number;
  maxTokens: number
}>;

/**
 Limits the four pool members report, in the pool's order.

 Measured over the members Pi 1.6.0 registers on this machine on 2026-11-27 and recorded in
 `doc/decision/writer-rounds-take-qwen-first-through-the-virtual-writer.md`, "The writer's members
 become the reviewer's": qwen's member 1,000,000 and 384,000, Synthetic's 524,288 and 65,536,
 opencode-go's 1,000,000 and 384,000, and Neuralwatt's flex lane 1,048,560 and 393,216. Spelled out
 here rather than read from a registry, because these cases must decide a request's size from a
 member's own window without a session.
 */
export const MEMBER_LIMITS: readonly (MemberLimits & {
  readonly provider: string;
  readonly id: string
})[] = [
  {
    provider: 'qwen-token-plan-individual',
    id: 'deepseek-v4.1-flash-1017',
    contextWindow: 1_000_000,
    maxTokens: 384_000,
  },
  {
    provider: 'synthetic',
    id: 'hf:deepseek-ai/DeepSeek-V4.1-Flash-1017',
    contextWindow: 524_288,
    maxTokens: 65_536,
  },
  {
    provider: 'opencode-go',
    id: 'deepseek-v4.1-flash',
    contextWindow: 1_000_000,
    maxTokens: 384_000,
  },
  {
    provider: 'neuralwatt',
    id: 'deepseek-v4.1-flash-flex',
    contextWindow: 1_048_560,
    maxTokens: 393_216,
  },
];

/**
 Limits one member reports, by its exact provider and model.

 @param provider - provider id the member belongs to

 @param id - physical model id of the member

 @returns the pair that member reports

 @throws Error when the fixture has no such member

 @example
 ```ts
 memberLimitsOf({ provider: 'synthetic', id: 'hf:deepseek-ai/DeepSeek-V4.1-Flash-1017' });
 ```
 */
export function memberLimitsOf({
  provider,
  id,
}: {
  readonly provider: string;
  readonly id: string;
},): MemberLimits {
  for (const member of MEMBER_LIMITS) {
    if ((member.provider === provider) && (member.id === id))
      return member;
  }
  throw new Error(`No fixture limits for ${provider}/${id}`,);
}

/**
 Characters of padding per token of the member window a request is sized against, for a request that
 lands above that window.

 Pi's estimator reads 3.5 characters per token (`@earendil-works/pi-ai` dist/utils/estimate.js,
 `CHARS_PER_TOKEN`), so a padding of four characters per window token estimates above the window for
 any ratio between three and four characters per token that the estimator might use, and one of three
 characters estimates below it. Deriving the two factors from that pair keeps these cases a statement
 about sizes rather than about the estimator's exact constant.
 */
const PADDING_CHARS_ABOVE_WINDOW = 4;

/**
 Characters of padding per window token for a request that lands below a member's window.
 */
const PADDING_CHARS_BELOW_WINDOW = 3;

/**
 Characters of padding per window token for a request whose input fits a member's window while
 leaving less room than that member's own output limit.

 Between the two factors above and the estimator's own ratio: the input lands close to the window,
 and each case asserts that its request really does fit the window and really does exceed it once the
 output limit the request carries is added, so a changed ratio fails the case instead of quietly
 testing nothing.
 */
const PADDING_CHARS_NEAR_THE_WINDOW = 3.4;

/**
 Text padding of a fixed number of characters, for a request of a known estimated size.

 Made up: the text stands for no program's output.
 Boundary: the estimated size of a request, which decides whether a member is asked
 (`doc/captures.md`, "Made-up texts").

 @param characters - length of the padding

 @returns the padding, one repeated letter

 @example
 ```ts
 paddingOf(3_500); // 1,000 estimated tokens
 ```
 */
export function paddingOf(characters: number,): string {
  return 'x'.repeat(characters,);
}

/**
 Padding whose estimated size lands above one member's window and below a larger one's.

 @param window - window the request must exceed

 @returns padding of four characters per token of that window

 @example
 ```ts
 paddingAboveWindow(524_288);
 ```
 */
export function paddingAboveWindow(window: number,): string {
  return paddingOf(window * PADDING_CHARS_ABOVE_WINDOW,);
}

/**
 Padding whose estimated size lands below one member's window.

 @param window - window the request must fit

 @returns padding of three characters per token of that window

 @example
 ```ts
 paddingBelowWindow(524_288);
 ```
 */
export function paddingBelowWindow(window: number,): string {
  return paddingOf(window * PADDING_CHARS_BELOW_WINDOW,);
}

/**
 Padding whose input fits one member's window while leaving less room than that member's own output
 limit.

 @param window - window the input must fit

 @returns padding of 3.4 characters per token of that window

 @example
 ```ts
 paddingNearWindow(524_288);
 ```
 */
export function paddingNearWindow(window: number,): string {
  return paddingOf(window * PADDING_CHARS_NEAR_THE_WINDOW,);
}
