/**
 The generated baseline's fixed text: the banner, the trailer of set-algebra
 demonstration sections, and the rules this port drops by id.

 Declarations only, which is why the file carries the `*.data.ts` name the
 repository oxlint config exempts from the code-line budget
 (`package/config/oxlint/src/overrides.ts`).
 */

/**
 Rules to drop unconditionally.
 */
export const DROPPED_BY_ID: ReadonlyMap<string, string> = new Map([
  // Without the ~1000-entry word allowlist + entropy filter this rule
  // fires on practically anything labeled `key=` / `token=`.
  [
    'generic-api-key',
    'no-allowlist-no-entropy',
  ],
  // Pattern combines `(?s:.){0,N}` scope-flag dot, four nested
  // alternation arms, and shared inner structure. Resharp's algebra
  // pass refuses with `Algebra(UnsupportedPattern)`. Upstream gates
  // this rule with a `*.ya?ml$` path filter we already lose; without
  // path scoping the rule is also low signal.
  [
    'kubernetes-secret-yaml',
    'algebra-unsupported',
  ],
  // Multi-arm alternation with multiple `(?i)` toggles, single-quote
  // and double-quote variants, and four label sub-arms. Restructuring
  // into a resharp-acceptable form would lose more signal than it
  // preserves; the curl deny-list shape `curl ... -H "Authorization:
  // Bearer ..."` is also covered indirectly by the per-vendor token
  // rules (github-pat, openai-api-key, etc.) firing on the bearer
  // value itself.
  [
    'curl-auth-header',
    'multi-arm-(?i)-not-relaxable',
  ],
],);

/**
 Banner prepended to the generated built-in baseline file.
 
 Documents that the file is generated, names the source TOML and converter
 script, and reminds readers of forbidden-strings' line-level grammar.
 */
export const HEADER: string = `# forbidden-strings built-in baseline deny-list.
#
# THIS FILE IS GENERATED. Do not edit by hand. It is the output of the
# two-stage generation chain:
#   1. package/forbidden-strings/cli/bin/port-betterleaks.ts reads the
#      upstream TOML at
#      package/forbidden-strings/cli/data/betterleaks-default-config.toml
#      and emits a tail-format intermediate whose bodies are still PCRE.
#   2. The dialectport bin in
#      package/forbidden-strings/core.bench/src/bin/ rewrites each body
#      into the forbidden-regex dialect and writes this file.
# Re-generate via:
#   mise run //package/forbidden-strings/cli:generate:rules
#
# Composition:
#   - This file ports the betterleaks default ruleset into the
#     forbidden-strings tail rule-file format. It is a sane baseline of
#     common credential shapes (PEM, AWS, Slack, GitHub PAT, etc.) plus
#     two forbidden-regex set-algebra demonstrations.
#   - The build precompiles it into the forbidden-strings binary; the
#     \`--builtin-rules\` flag appends it to whatever rules file resolves
#     at scan time (and scans with it alone when the implicit default
#     rules file is absent). Without the flag the scanner never reads
#     these rules.
#
# Attribution:
#   Rules are ported from betterleaks' default configuration
#   (https://github.com/betterleaks/betterleaks, MIT-licensed). The port
#   is mechanical and lossy -- entropy filters, CEL validate steps,
#   keyword prefilters, and allowlists are dropped because the
#   forbidden-strings engine has no equivalent. Expect more false
#   positives than betterleaks would produce; consult the converter
#   source for the full list of conversions and intentional omissions.
#
# Format reminder (tail rule-file format):
#   - \`==> name <==\`  = section header; the section name is the rule's
#     identity, rendered in findings as \`rule=<name>\`. Baseline names are
#     the upstream betterleaks ids.
#   - Every section is exactly one rule. A single significant body line is
#     a bare literal or a \`/PATTERN/FLAGS\` regex; \`#\` lines and blanks
#     are insignificant.
#
# A literal that itself looks like /.../flags must be expressed as a regex
# (escape the slashes), e.g. ban literal \`/etc/passwd\` as \`/\\/etc\\/passwd/\`.

`;

/**
 Trailer appended to the generated built-in baseline file.
 
 Includes two engine-specific set-algebra demonstration sections
 (intersection and complement) so consumers can see capabilities that
 pure-PCRE engines lack. Each demonstration is its own named section
 because the tail format gives every rule its own identity.
 */
export const FOOTER: string = `==> set-algebra-demo-build-tag <==
# The two demonstration sections here are engine-specific (forbidden-regex
# set algebra), not betterleaks ports. The engine extends standard regex
# with two top-level set operators that pure-PCRE engines lack:
#   - A&B   intersection: matches strings matched by both A and B
#   - ~(A)  complement:   matches strings that do NOT match A
# Combined, these express "match X but not Y" without lookaround. PCRE
# engines (gitleaks, trufflehog, secretlint, plain RE2) cannot do this;
# the workaround is per-rule allowlists, which scale badly.
#
# Reads as: "match any 6-digit BUILD_ tag, EXCEPT the all-zeros placeholder."
/BUILD_[0-9]{6}&~(BUILD_000000)/

==> set-algebra-demo-release-tag <==
# Intersection composed with two complements: ban any 32-char hex hash
# under \`RELEASE_TAG_\`, except the documented placeholders.
# The complements inline their quantifier bodies: \`0{32}\` is a
# quantified literal (not a quantified group), and the deadbeef
# placeholder is written as 16 concatenated unquantified
# \`(de|ad|be|ef)\` groups (32 chars total). The inlined shapes date from
# the resharp era; they compile fine on forbidden-regex and are kept
# byte-stable so the compiled baseline does not shift.
/RELEASE_TAG_[a-f0-9]{32}&~(RELEASE_TAG_0{32})&~(RELEASE_TAG_${
  '(de|ad|be|ef)'.repeat(16,)
})/
`;
