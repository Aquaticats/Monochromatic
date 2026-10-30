/**
 Arguments and extra environment of one option spread.
 */
export type ArgumentSet = readonly [
  string,
  readonly string[],
  Readonly<Record<string, string>>
];

/**
 Scanner scenarios that fail on their options. A scanner run that gets past its options vendors the real dependencies
 with cargo, which a stand-in cannot answer, so only these are pinned.
 */
export const scannerArgumentSets: readonly ArgumentSet[] = [
  [
    'reject-unknown-scope',
    ['--bogus'],
    {}
  ],
  [
    'reject-two-scopes',
    [
      '--startup',
      '--pathname'
    ],
    {}
  ],
];
