# TOML v1.0.0 footguns: array-of-tables context shifts, dotted-key vs explicit-table conflicts, cross-parser disagreement, no value interpolation

This file documents the actual TOML specification pitfalls that
motivated writing the comment-preserving JSONC parser in
`package/jsonc-edit/core-ts` instead of adopting TOML for workspace
configuration.
 Each footgun gets its own canonical section.

The shape matches the rest of TROUBLESHOOTING,
 even though "the
bug" is a spec-level set of constraints rather than a single
implementation defect:
 the conclusion is the same kind of audit
trail.

## Symptom (top level)

When evaluating TOML for workspace configuration,
 four distinct
classes of failure mode surfaced:

1. Array-of-tables `[[thing]]` headers silently shift context.
2. Dotted keys and explicit `[table]` headers conflict
   asymmetrically.
3. Major parsers disagree on edge cases that the spec leaves
   underspecified.
4. No mechanism for value reuse or interpolation across sections.

A common misconception worth dismissing up front:
 repeating
`[database]` twice in a TOML file does **not** silently overwrite
the first definition.
 The TOML v1.0.0 spec explicitly prohibits
this:

> "Like keys,
>  you cannot define a table more than once.
>  Doing so
> is invalid.
> "

All compliant parsers (Python `tomllib`,
 Rust `toml`,
 JS
`smol-toml`) reject duplicate table headers with a parse error.
This is not a footgun:
 the spec and implementations handle it
correctly.
 The real footguns are subtler.

## Verification (versions surveyed)

- TOML spec v1.0.0 (current as of 2026-05)
- Python `tomllib` (Python 3.11+)
- Rust `toml` and `toml_edit` (latest crates.
  io as of 2026-03)
- JS `smol-toml` (latest npm as of 2026-03)

Reproductions for each footgun are inline below.
 Tested by
feeding the snippets to each parser and recording the behaviour.

---

## Footgun 1: Array-of-tables `[[thing]]` loses context silently

### Symptom

```toml
[[products]]
name = "Hammer"

[[products.variants]]
name = "small" # variant of Hammer

[[products]]
name = "Nail" # context silently shifts -- Hammer is closed

[[products.variants]]
name = "gold" # variant of Nail, not Hammer
```

There is no syntax to append a variant to a **previous** array
element once a new `[[products]]` has been opened.
 In a large
file,
 accidentally placing `[[products]]` in the wrong position
silently reassigns all subsequent sub-tables to the wrong parent.

### Root cause

The spec states:

> "Any reference to an array of tables points to the most recently
> defined table element of the array.
> "

This is correct per spec;
 the trap is that "most recently
defined" is a positional property of the file.
 Reorganising a
file by moving a `[[products]]` block can change the parent of
unrelated `[[products.variants]]` blocks below it.

### Verified workaround

Group every array element with its sub-tables physically
adjacent in the file;
 never let unrelated content separate them.
Tradeoff:
 file ordering is now load-bearing;
 a careless reorder
silently breaks data.
 JSONC avoids this because each variant is
an explicit property of its parent object literal.

### What does not work

- Inline tables:
   limited expression power,
   cannot contain
  multi-line arrays of tables.
- Comments warning about context:
   comments cannot affect parser
  behaviour;
   the trap survives any documentation effort.

### Why we do not file this upstream

The behaviour matches the spec exactly.
 Walking the 5
constraints:
 not a defect,
 no fix,
 decision:
 no upstream report.

---

## Footgun 2: Dotted keys and explicit `[table]` headers conflict asymmetrically

### Symptom

```toml
fruit.apple.color = "red" # implicitly creates [fruit] and [fruit.apple]

[fruit.apple] # INVALID -- table already defined via dotted key
taste = "sweet"
```

But this is **valid**:

```toml
fruit.apple.color = "red"

[fruit.apple.texture] # valid -- new sub-table, not a redefinition
smooth = true
```

You can extend an implicitly created table downward (deeper
nesting) but not sideways (same level).

### Root cause

> "Since tables cannot be defined more than once,
>  redefining such
> tables using a `[table]` header is not allowed.
>  Likewise,
>  using
> dotted keys to redefine tables already defined in `[table]` form
> is not allowed.
> "

The spec treats `[fruit.apple]` as a "redefinition" but
`[fruit.apple.texture]` as a "new sub-table";
 the asymmetry is
hard to remember without internalising the implicit-table
graph.

### Verification

Compliant parsers all reject the redefinition case identically.
The asymmetric behaviour is reproducible across `tomllib`,
`toml`,
 and `smol-toml`.

### Verified workaround

Pick one style per file:
 dotted keys throughout,
 or explicit
`[table]` headers throughout.
 Mixing the two requires keeping
the implicit-table graph in mind.
 Tradeoff:
 stylistic constraint
enforced by convention,
 not by the parser.

### What does not work

- Reopening a dotted-key-created table with `[header]` in any
  configuration:
   the rule is symmetric across parsers and not
  configurable.

### Why we do not file this upstream

Spec-defined behaviour;
 no defect.
 No upstream report.

---

## Footgun 3: Parser implementations disagree on dotted-key + sub-table edge cases

### Symptom

```toml
[tool.hatch]
version.source = "vcs"

[tool.hatch.version.raw-options]
local_scheme = "no-local-version"
```

- Python `tomllib`:
   parses successfully (merges into `version`)
- Rust `toml` / `toml_edit`:
   rejects with "duplicate key:
  `version`"
- JS `smol-toml`:
   behaviour varies by version

See
[toml-rs/toml#439](https://github.com/toml-rs/toml/issues/439).
A file that works in one ecosystem silently breaks in another.

### Root cause

The interaction rules between dotted keys and explicit
sub-tables are ambiguous enough at the edges that the major
parsers diverge.
 The spec text is interpretable both ways.

### Verification

Feed the snippet above to each parser.
 Behaviours match the
table above.
 The divergence is reproducible against current
versions as of 2026-05.

### Verified workaround

Avoid the ambiguous construct:
 define `[tool.hatch.version]` and
`[tool.hatch.version.raw-options]` separately,
 never mixing a
dotted key (`version.source = "vcs"`) with a later
`[tool.hatch.version.raw-options]` header.
 Tradeoff:
 more
verbose;
 relies on author discipline.

### What does not work

- Picking the most permissive parser:
   file portability degrades;
  collaborators on other ecosystems hit parse errors.
- Lint rules:
   TOML has no first-class linter that warns about
  this pattern.

### Why we do not file this upstream

The defect is in the spec's underspecification,
 not in any one
parser.
 Filing against `toml-rs` would not bind `tomllib` or
`smol-toml`.
 The right venue is the TOML spec repo;
 the
discussion has been open and unresolved for years (see #439).
Walking the 5 constraints:
 filing yet another voice into the
same thread is unlikely to move it.
 No additional upstream
report from us.

---

## Footgun 4: No value reuse or interpolation

### Symptom

Repeated settings across environments must be duplicated:

```toml
[dev.deploy.parameters]
region = "us-east-1"
capabilities = "CAPABILITY_IAM"

[prod.deploy.parameters]
region = "us-east-1" # must repeat
capabilities = "CAPABILITY_IAM" # must repeat
```

There is no anchor,
 alias,
 or interpolation mechanism in TOML.

### Root cause

The TOML spec deliberately omits cross-reference features to
keep the format flat and predictable.
 The author's stated goal
is human-edited config,
 not DRY config-as-data.

### Verified workaround

When configuration needs reuse or logic,
 switch to TypeScript
(per the workspace's "config-as-data to TypeScript" rule).
 The
JSONC parser handles the simple case;
 a `.ts` file handles
anything that needs `if`,
 `map`,
 or `await`.

Tradeoff:
 introduces a code path for what would be data in
YAML's anchor model.
 Acceptable because TypeScript is the
project's primary language;
 YAML adds another grammar to
maintain.

### What does not work

- YAML-style anchors as an extension:
   not supported by any TOML
  parser;
   would not be portable.
- External pre-processing (templating engines):
   adds a build
  step and breaks editor tooling that expects valid TOML.

### Why we do not file this upstream

Spec design choice;
 no defect.
 The TOML maintainers have
declined feature requests for interpolation citing the
"easy-to-read" design goal.
 No upstream report.

---

## Why we chose JSONC

These four footguns,
 combined with TOML's inability to represent
the configuration patterns we needed,
 led us to write a
comment-preserving JSONC parser in `package/jsonc-edit/core-ts`:

- **No ambiguous table semantics**:
   JSON objects have one
  definition site.
- **Comments preserved**:
   `//` and `/* */` survive round-trip
  parsing.
- **Trailing commas**:
   easier to edit and diff.
- **Ecosystem alignment**:
   VS Code,
   TypeScript,
   ESLint,
   oxlint
  all use JSONC.
- **Escape hatch to TypeScript**:
   when config needs logic (`if`,
  `map`,
   `await`),
   switch from config-as-data to a `.ts` file
  that imports the same schema.

The comment-nesting limitation (`/* */` cannot nest) is a known
constraint we accepted;
 see
[`TROUBLESHOOTING.cLikeComments.md`](c-like-comments.md)
for how it affected the parser implementation.
