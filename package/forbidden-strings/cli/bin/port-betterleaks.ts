// What:     Stage one of the two-stage builtin-baseline generation chain.
//           Input is the upstream betterleaks configuration TOML stored
//           verbatim under `data/`; output is the tail-format intermediate
//           at `<repo>/.cache/forbidden-strings-builtin-stage1.txt`, whose
//           rule bodies are still in the PCRE-leaning pre-port dialect.
//           Stage two (the `dialectport` bin in
//           `package/forbidden-strings/core.bench/src/bin/`) rewrites
//           each `/PATTERN/FLAGS` body into the forbidden-regex dialect,
//           passes every other line through byte-identically (including the
//           `==> name <==` section headers this stage emits), and writes the
//           final `<package>/data/builtin-rules.txt` that `lib.rs` embeds
//           (see doc/decision/gitignore-negations.md). Run both stages via
//           `mise run //package/forbidden-strings/cli:generate:rules`.
//           This module holds the entry point and the section renderer; the
//           TOML walker lives in `port-betterleaks-parse.ts`, the pattern
//           conversion in `port-betterleaks-pcre.ts`, and the fixed baseline
//           text in `port-betterleaks-baseline.data.ts`.
// Why:      Forbidden-strings is rules-out-of-band by design, but most
//           teams want a sane starting deny-list of common credential
//           shapes. Rather than maintain that list ourselves, we port
//           betterleaks' default config and document every conversion
//           and intentional omission. Re-port by replacing
//           `data/betterleaks-default-config.toml` with a fresh upstream
//           dump and re-running the generation chain.
//
// Source TOML attribution:
//   Repo:    https://github.com/betterleaks/betterleaks
//   File:    config/betterleaks.toml
//   Commit:  007410ecca31fcbaff3a3de027cf6eeca59310f1 (2026-04-29)
//   License: MIT
//
// Conversions performed during port:
//   - `(?P<name>...)`         -> `(?:...)`            (resharp parses
//                                                      named groups but
//                                                      has no captures)
//   - lazy quantifiers        -> greedy form          (resharp errors on
//     `*?` `+?` `??` `{n,m}?`                          lazy quantifiers)
//   - `path = '''...'''`      -> dropped (with note)  (forbidden-strings
//                                                      has no per-rule
//                                                      path scoping; the
//                                                      rule fires on
//                                                      every scanned file)
//   - `keywords = [...]`      -> dropped              (resharp engine
//                                                      builds its own
//                                                      Aho-Corasick gate
//                                                      from the regex
//                                                      literal prefix)
//   - `filter = '''...'''`    -> dropped (with note)  (entropy / string
//                                                      allowlists have
//                                                      no equivalent)
//   - `validate = '''...'''`  -> dropped              (CEL validation has
//                                                      no equivalent)
//   - `[[rules.allowlists]]`  -> dropped              (allowlist-by-regex
//                                                      no equivalent)
//   - `[[rules.required]]`    -> kept regex, lost     (cannot enforce
//                                composite intent     "must be near rule
//                                                      X" in this engine)
//   - `secretGroup = N`       -> note emitted; the    (engine reports
//                                redacted match span  the whole match
//                                widens to the full   span; secretGroup
//                                regex                narrowing lost)
//
// Rules dropped wholesale (not portable; see comments inline):
//   - `generic-api-key`              relies on a ~1000-entry word
//                                    allowlist + entropy <= 3.5; firing
//                                    without them produces near-100%
//                                    false positives.
//   - rules with `skipReport = true` upstream marks them as
//                                    composite-helpers only (e.g.
//                                    `aws-secret-access-key`,
//                                    `ovh-application-key`,
//                                    `polymarket-api-secret`): they
//                                    have no useful standalone signal.
//   - `pkcs12-file`                  path-only rule (no regex field);
//                                    forbidden-strings has no path-only
//                                    mode.

import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  DROPPED_BY_ID,
  FOOTER,
  HEADER,
} from './port-betterleaks-baseline.data.ts';
import {
  parseRules,
  type RawRule,
} from './port-betterleaks-parse.ts';
import { pcreToResharp, } from './port-betterleaks-pcre.ts';

import { RELAXATIONS, } from '../src/port-betterleaks-relaxations.ts';

/**
 Reports whether an id fits the tail-format strict section-name grammar.
 
 Rejecting here fails the generation loudly instead of shipping a baseline
 whose header the scanner would refuse as a near-header at load time.
 
 @param id - the section name the scanner will print in findings
 
 @returns whether the id is inside the strict section-name grammar
 
 @example
 ```ts
 isStrictSectionName({ id: '1password-secret-key' }); // true
 isStrictSectionName({ id: 'Bad_Name' }); // false
 ```
 */
function isStrictSectionName({ id, }: { readonly id: string; },): boolean {
  if (id.length === 0)
    return false;
  /**
   Leading character; the grammar restricts it to `[a-z0-9]`.
   */
  const first = id.charAt(0,);
  if (!(((first >= 'a') && (first <= 'z')) || ((first >= '0') && (first <= '9'))))
    return false;
  // Indexed UTF-16 walk instead of string spread: the alphabet is pure ASCII,
  // so any surrogate half fails the range checks and rejects correctly.
  for (let index = 0; index < id.length; index += 1) {
    /**
     Single UTF-16 unit under the cursor.
     */
    const ch = id.charAt(index,);
    /**
     Whether the unit sits inside the strict section-name alphabet.
     */
    const isNameChar = ((ch >= 'a') && (ch <= 'z'))
      || ((ch >= '0') && (ch <= '9'))
      || (ch === '.')
      || (ch === '-');
    if (!isNameChar)
      return false;
  }
  return true;
}

/**
 Render one rule as a tail-format section (header + comments + regex line),
 applying any {@link RELAXATIONS} entry before converting via {@link pcreToResharp}.
 
 @param rule - one parsed rule, with every field the extractors matched
 
 @returns the rule's tail-format section: header, notes, and regex line
 
 @throws Error when the betterleaks id falls outside the strict section-name
 grammar the scanner enforces on headers.
 */
function renderRule({ rule, }: { readonly rule: RawRule; },): string {
  if (!isStrictSectionName({ id: rule.id, },))
    throw new Error(`betterleaks id '${rule.id}' is not a valid tail-format section name`,);
  // The betterleaks id doubles as the tail-format section name: the scanner
  // renders it in findings (`rule=<name>`), so every baseline match is
  // self-identifying. Ids are lowercase kebab, inside the strict name grammar.
  /**
   Accumulator for the output block; joined with newlines at the end.
   */
  const lines: string[] = [
    `==> ${rule.id} <==`,
    `# ${rule.description}`,
  ];
  if (rule.pathScope
    !== undefined) {
    lines.push(
      `# NOTE: upstream restricts this rule to files matching: ${rule.pathScope}`,
      '#       Forbidden-strings has no per-rule path scoping; the rule fires on every scanned file.'
    );
  }
  if (rule.secretGroup
    !== undefined) {
    lines.push(
      `# NOTE: upstream extracts capture group ${rule.secretGroup} as the secret for redaction.`,
      '#       Forbidden-strings reports the full match span; the narrowing is lost.'
    );
  }
  if (rule.hasRequired) {
    lines.push(
      "# NOTE: upstream requires another rule's match nearby ([[rules.required]]).",
      '#       Forbidden-strings cannot enforce composite proximity rules; the regex fires standalone.'
    );
  }
  /**
   Optional relaxation for this rule id; rewrites the regex before conversion.
   */
  const relaxation = RELAXATIONS.get(rule.id,);
  /**
   Working regex: the upstream form, transformed when the relaxation carries a
   transform.
   */
  const pattern = (relaxation?.transform === undefined)
    ? rule.regex
    : relaxation.transform(rule.regex,);
  if (relaxation !== undefined)
    lines.push(`# RELAXATION: ${relaxation.note}`,);
  /**
   Resharp-compatible form of the pattern, ready for final relaxation.
   */
  const convertedBase = pcreToResharp({ pattern, },);
  /**
   Final emitted resharp form, including engine-specific algebra relaxations.
   */
  const converted = relaxation?.convertedTransform?.(convertedBase,)
    ?? convertedBase;
  lines.push(
    `/${converted}/`,
    ''
  );
  return lines.join('\n',);
}

/**
 Directory holding this script, used as the anchor for path resolution.
 */
const here = import.meta.dirname;
/**
 Path to the verbatim upstream TOML; input to the port.
 */
const tomlPath = join(
  here,
  '..',
  'data',
  'betterleaks-default-config.toml',
);
/**
 Path to the repo-root `.cache` scratch directory holding the stage-one
 intermediate; gitignored wholesale, shared with the file-enforcer
 generated rules file.
 */
const cacheDir = join(
  here,
  '..',
  '..',
  '..',
  '..',
  '.cache',
);
/**
 Path to the stage-one intermediate: tail-format sections whose bodies are
 still PCRE. The dialectport bin (stage two) reads exactly this path and
 writes the final embedded baseline into the package `data/` directory.
 */
const outPath = join(
  cacheDir,
  'forbidden-strings-builtin-stage1.txt',
);

/**
 Verbatim upstream TOML contents read from disk.
 */
const toml = await readFile(
  tomlPath,
  'utf8',
);
/**
 Every rule parsed from the TOML, pre-filter.
 */
const all = parseRules({ toml, },);

/**
 Rules that survive the drop filter and will be rendered into the output.
 */
const kept: RawRule[] = [];
/**
 Rules excluded from output, paired with the reason for the exclusion.
 */
const dropped: {
  rule: RawRule;
  reason: string;
}[] = [];
for (const rule of all) {
  if (rule.skipReport) {
    dropped.push({
      rule,
      reason: 'skipReport',
    },);
    continue;
  }
  /**
   Lookup of the rule id against {@link DROPPED_BY_ID}; non-undefined means dropped.
   */
  const dropReason = DROPPED_BY_ID.get(rule.id,);
  if (dropReason !== undefined) {
    dropped.push({
      rule,
      reason: dropReason,
    },);
    continue;
  }
  kept.push(rule,);
}

/**
 Every kept id, used for the duplicate check below; duplicate section
 names are a fail-closed load error in the scanner, so catch them here.
 */
const seenIds = new Set<string>();
for (const rule of kept) {
  if (seenIds.has(rule.id,))
    throw new Error(`duplicate betterleaks id '${rule.id}' would collide as a section name`,);
  seenIds.add(rule.id,);
}

/**
 Concatenated rule blocks rendered between the header and footer.
 */
const body = kept
  .map(function rulePass(rule,): string {
    return renderRule({ rule, },);
  },)
  .join('',);

/**
 Full output file contents written to disk.
 */
const content = `${HEADER}${body}${FOOTER}`;
await mkdir(
  cacheDir,
  { recursive: true, },
);
await writeFile(
  outPath,
  content,
  'utf8',
);
console.log(
  `wrote ${outPath} (${kept.length} rules kept, ${dropped.length} dropped)`,
);
if (dropped.length
  > 0) {
  console.log('  dropped:',);
  for (const {
    rule,
    reason,
  } of dropped) {
    console.log(`    ${rule.id} (${reason})`,);
  }
}
