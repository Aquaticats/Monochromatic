# Forbidden-strings tail-format rule files (normative spec)

Status:
 adopted 2026-07-20
(`doc/decision/forbidden-strings-rule-file-format.md`);
supersedes the block-form draft
(`doc/planning/forbidden-strings-block-rule-format.md`).

A rule file is what `tail --verbose -n +1` over per-rule files would
produce:
 named sections whose bodies are rules.
 The per-rule body has no
container grammar at all;
 the engine reads it directly.
 Authors edit the
single file "pretending to be tail".

## Header grammar

A section opens with a line matching exactly:

- `==> `,
   then a name,
   then ` <==`,
   then end of line;
- the name matches lowercase kebab-with-dots:
  first character `[a-z0-9]`,
   then any of `[a-z0-9.-]`,
  no adjacent dots or dashes at the spec level is not required,
  but names are unique within one loaded rule set (duplicate names are a
  fail-closed load error).

The constrained name alphabet defines valid names;
 collision defense
is the arrow rule below (maintainer ruling 2026-07-20,
 replacing an
earlier draft where out-of-alphabet arrow lines stayed content).

## Near-header fail-closed rule

Any line whose trimmed form starts with `==>` and is not exactly a
strict header line is a fail-closed load error (redacted,
 reporting the
line number only).
 This covers case-typo'd headers,
 indented would-be
headers,
 missing-space typos,
 and genuine content about tail-style
text alike;
 nothing arrow-leading is ever silently absorbed into a
section body,
 which closes the silent-split failure mode the vet
validated.
 Genuine content is written with the reshape convention
below.
 A `#`-leading comment line mentioning an arrow is unaffected
(its trimmed form starts with `#`).

## Collision mitigation (mandatory, from the vet)

A regex body that must genuinely match line-leading tail-style text
reshapes the first byte with a character class:
`[=]=> ` in place of
`==> `,
 compiling to the same match.
 A bare literal cannot begin with
`==>` at all;
 express such a rule in regex form instead.
 Inside a bare
literal,
`[=]` remains the literal three characters (the escaper
escapes brackets;
 there is no de-reshaping in literal context;
maintainer ruling 2026-07-20).

## Section body classification

After the header,
 body lines run to the next header or end of file.
Trailing blank lines of a section are insignificant (the separator
newline tail guarantees,
 plus any of the body's own).

- Every section is exactly one rule,
   and every rule,
   bare literal or
  regex,
   therefore carries its own name (maintainer correction
  2026-07-20:
   a literal-list section form was considered and rejected
  because uniform per-rule identity outweighs local-appendix
  compactness).
- A body containing exactly one significant line (non-blank,
   not
  `#`-leading):
   classified by the incumbent two-form rule,
   so a bare
  literal (escaped and word-boundary-gated exactly as today) and a
  single-line `/PATTERN/FLAGS` both work unchanged inside a named
  section.
- A body containing more than one significant line:
   the
  body is one verbatim regex pattern handed to the engine as-is
  (the engine's always-verbose mode ignores blank lines and consumes
  first-column `#` comment lines itself).
   No delimiters,
   no flags slot
  (the incumbent flags slot is vestigial:
   only the no-ops `m` and `x`
  were ever accepted).
- An empty section (no significant lines) is a fail-closed load error.

## File-level rules

- A leading UTF-8 BOM is stripped once.
- Blank lines and `#`-comment lines before the first header are
  ignored;
   any other content before the first header is a fail-closed
  load error in tail-format files.
- Format autodetection during the transition:
   a file whose first
  significant line is a strict header parses as tail-format;
   otherwise
  the whole file parses as the legacy line-based format.
   One file never
  mixes formats.
- Concatenating tail-format files (file-enforcer materialization) is
  plain byte append;
   name uniqueness is enforced over the concatenated
  result.

## Rule identity

The section name is the rule's stable identity.
 Sensitive local rules
receive deliberately opaque names,
 because names surface in findings
and CI logs.

DECIDED (maintainer,
 2026-07-20,
 resolving
`doc/planning/forbidden-strings-rule-identity-ux.md`):
 findings render
named rules as `rule=<name>`;
 unnamed legacy rules keep the offset
numeric `rule=N` fallback.
 Baseline names are the betterleaks ids,
embedded at build time as a name sidecar beside the precompiled set.
A runtime rule name that collides with a baseline name fails the load
closed.
 Local-appendix rules use opaque sequential names (`local-NNN`)
that reveal nothing about a rule's topic,
 per the maintainer's
instruction to hide even the subject area of the local rules.

## Migration sequencing (binary before data files)

1.  Parser lands with tests (#396);
     a release ships.
2.  The local gate and CI move to the released binary.
3.  The three live rule files convert
    (`forbidden-strings.append.txt`,
    `forbidden-strings.append.local.txt`,
    `package/forbidden-strings/cli/data/builtin-rules.txt`,
     the last
    regenerated by the porter with betterleaks ids as section names).
    The shared appendix's per-code comment block becomes per-branch
    comments inside the section bodies.

Until step 3,
 no live rule file may contain tail-format syntax:
 an old
binary would read headers as bare literals and the gate would misfire.
Legacy-format removal is a separate later decision.
