/** Per-rule relaxations applied around {@link pcreToResharp}. Each entry
  maps a rule id to functions that rewrite the upstream regex before
  conversion or the converted regex after conversion. The relaxation
  comment travels with the rule in the emitted output so the lossy step
  is visible in the example file. */
export type Relaxation = {
  /**
   Human-readable explanation emitted above generated rule.
   */
  readonly note: string;
  /**
   Optional rewrite applied before PCRE-to-resharp conversion.
   */
  readonly transform?: (regex: string,) => string;
  /**
   Optional rewrite applied after PCRE-to-resharp conversion.
   */
  readonly convertedTransform?: (regex: string,) => string;
};

/**
 Rule-id to relaxation lookup applied around {@link pcreToResharp}.
 
 Each entry pairs the upstream betterleaks rule id with rewrite hooks plus
 a human-readable note explaining what changed. The note is emitted
 alongside the rule in the generated example file so the change stays
 visible.
 */
export const RELAXATIONS: ReadonlyMap<string, Relaxation> = new Map([
  // `(?:^|[X])` start-anchor alternation: drop the `^|` arm. Loses the
  // unusual case where the secret starts at byte 0 of the file.
  [
    'azure-ad-client-secret',
    {
      note:
        'dropped `^|` start-anchor arm; rule no longer fires when the secret starts at byte 0 of a file.',
      transform: function dropStartAnchor(r,): string {
        return r.replaceAll(
          '(?:^|',
          '(?:',
        );
      },
    },
  ],
  // Trailing `\b` arm in `(?:[X]|...|\b)`: resharp rejects \b as a
  // standalone alternation arm. Drop it; the other arms cover every
  // realistic terminator.
  [
    'perplexity-api-key',
    {
      note: 'dropped trailing `\\b` alternation arm.',
      transform: function dropTrailingBoundary(r,): string {
        return r.replaceAll(
          String.raw`|\b)`,
          ')',
        );
      },
    },
  ],
  // Mid-pattern `(?i)` toggle. Resharp does not accept late case toggles.
  // Hoist `(?i)` to the front; that makes the `EZAK` / `EZTK` literal
  // case-insensitive as well, a tolerable broadening (false-positive
  // risk: a stray `eZaK` followed by 54 alnum chars is vanishingly rare).
  [
    'easypost-api-token',
    {
      note:
        'hoisted `(?i)` to the front and dropped trailing `\\b` (resharp rejects `\\b` immediately after a case-folded class).',
      transform: function relaxEasyPostA(_r,): string {
        return String.raw`(?i)\bEZAK[a-z0-9]{54}`;
      },
    },
  ],
  [
    'easypost-test-api-token',
    {
      note:
        'hoisted `(?i)` to the front and dropped trailing `\\b` (resharp rejects `\\b` immediately after a case-folded class).',
      transform: function relaxEasyPostB(_r,): string {
        return String.raw`(?i)\bEZTK[a-z0-9]{54}`;
      },
    },
  ],
  // `\b` placement inside nested alternation. Drop the inner anchors and
  // collapse the variant length to `{20,74}`; the upstream form fixes
  // {20,58,74} exactly which is too brittle to translate cleanly.
  [
    'openai-api-key',
    {
      note:
        'collapsed length cohort {20,58,74} to {20,74}, removed inner `\\b`, dropped trailing `\\b` (resharp rejects `\\b` after a class containing both word and non-word chars), dropped trailing punctuation requirement.',
      transform: function relaxOpenAi(_r,): string {
        return String
          .raw`\bsk-(?:[a-zA-Z0-9]{20}|(?:proj|svcacct|admin)-[A-Za-z0-9_-]{20,74})T3BlbkFJ[A-Za-z0-9_-]{20,74}`;
      },
    },
  ],
  // Doubled `\b(\b(...)\b)` boundary nesting. Drop the outer `\b(...)`
  // wrapper. Behavior unchanged because the inner group already
  // word-boundary-anchors.
  [
    'sourcegraph-access-token',
    {
      note:
        'removed redundant outer `\\b(\\b...\\b)` wrapping AND dropped the bare `[a-fA-F0-9]{40}` arm. The bare-hex arm fires on any 40-char hex string -- e.g. GitHub Actions SHA pins (`uses: x/y@<sha>`) -- which is unacceptable false-positive density. Only `sgp_`-prefixed forms remain.',
      transform: function relaxSourcegraph(_r,): string {
        return String
          .raw`(?i)\b(sgp_(?:[a-fA-F0-9]{16}|local)_[a-fA-F0-9]{40}|sgp_[a-fA-F0-9]{40})\b(?:\\?['"\x60]|[\s;]|\\[nr]|$)`;
      },
    },
  ],
  // Top-level alternation with disjoint head/tail structure. Keep only
  // the ATATT3-prefixed arm (the high-signal half); drop the label-and-
  // shape half (which has many label permutations and a `[a-z0-9]{20}
  // [a-f0-9]{4}` body that resharp's algebra can't lower).
  [
    'atlassian-api-token',
    {
      note:
        'kept only the `ATATT3...` prefix arm; the label-based ([Aa]tlassian / [Jj]ira / [Cc]onfluence + 24-char body) arm is dropped.',
      transform: function relaxAtlassian(_r,): string {
        return String.raw`\b(ATATT3[A-Za-z0-9_\-=]{186})(?:\\?['"\x60]|[\s;]|\\[nr]|$)`;
      },
    },
  ],
  // Resharp set algebra can express inline placeholder exceptions without
  // introducing a scanner-level allowlist. Keep the upstream AWS access key
  // shape, then subtract the documented all-2s fixture used in tests.
  [
    'aws-access-token',
    {
      note:
        'excluded documented placeholder `AKIA2222222222222222` via resharp intersection/complement.',
      convertedTransform: function excludeAwsPlaceholder(r,): string {
        return String.raw`${r}&~(AKIA2{16})`;
      },
    },
  ],
  // Strip the optional query-string-and-fragment tail (the `(?:\?\w+=
  // ...)?` portion) plus the trailing `\b` arm. The connection-string
  // shape `mongodb://user:pass@host[/authdb]` still fires.
  [
    'mongodb-connection-string',
    {
      note:
        'dropped optional query-string tail (everything after `[/authdb]?`) and the trailing `\\b` alternation arm.',
      transform: function relaxMongo(_r,): string {
        return String
          .raw`\b(mongodb(?:\+srv)?://(?:[!-9;-~]{3,50}):(?:[!-?A-~]{3,88})@(?:(?:[a-zA-Z0-9][\w.-]+|\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})(?::\d{1,5})?(?:,(?:[a-zA-Z0-9][\w.-]+|\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})(?::\d{1,5})?)*)/?(?:[\w-]+)?)(?:['"\s;\x60]|\\[nr]|$)`;
      },
    },
  ],
  // Keep the upstream GitHub PAT shape, then subtract the all-zeros fixture.
  [
    'github-pat',
    {
      note:
        'excluded documented placeholder `ghp_000000000000000000000000000000000000` via resharp intersection/complement.',
      convertedTransform: function excludeGitHubPatPlaceholder(r,): string {
        return String.raw`${r}&~(ghp\_0{36})`;
      },
    },
  ],
],);
