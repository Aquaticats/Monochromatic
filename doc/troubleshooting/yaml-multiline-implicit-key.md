# `yaml` 2.9.0 via `pi-coding-agent` 1.2.0: wrapped `SKILL.md` frontmatter at column 0 parses as a multi-line implicit key and skips the skill

## Symptom

Pi's startup skill report prints a `[Skill conflicts]` block,
and each listed skill silently disappears
from the skill catalog (the loader drops the skill on any frontmatter parse error):

```text
[Skill conflicts]
  /var/home/user/Monochromatic/.agents/skills/android-emulator/SKILL.md
    Implicit keys need to be on a single line at line 3, column 1:

description: Android emulator or AVD work. Use before launching, scripting or b…
to use the Android emulator (device passes, screenshots, gfxinfo, adb on an emu…
^
```

Three frontmatter surface patterns reach the same failure family,
each with its own error code from the
`yaml` composer:

- Two or more plain wrap lines at column 0 (the wrapped-description habit):
  `MULTILINE_IMPLICIT_KEY`,
  "Implicit keys need to be on a single line".
- Exactly one plain wrap line at column 0:
  `MISSING_CHAR`,
  "Implicit map keys need to be followed by map values".
- `": "` inside a plain scalar value:
  `BLOCK_AS_IMPLICIT_KEY`,
  "Nested mappings are not allowed in compact mappings".

## Root cause

### Step 1: Pi parses the extracted frontmatter with the `yaml` package

The skill loader reads each `SKILL.md`,
takes the text between the `---` fences,
and hands it to `parse`:

```js
// node_modules/.pnpm/@earendil-works+pi-coding-agent@1.2.0/node_modules/@earendil-works/pi-coding-agent/dist/utils/frontmatter.js:1
import { parse } from "yaml";
```

`@earendil-works/pi-coding-agent@1.2.0` pins `yaml` to exactly 2.9.0.
Because the opening `---` line is
stripped before parsing,
error positions are body-relative:
the report's "line 3,
column 1" is file
line 4.
An earlier reading blamed file line 3 (the `description:` key line);
the report's own excerpt
disproves it,
since the caret sits under the first flush-left wrap line.

### Step 2: the plain value ends at the line end, so the wrap lands in key position

A plain scalar continues onto following lines only when those lines are indented deeper than the block
mapping.
In the broken files the wrap lines sit at column 0,
the same column as the keys,
so the
composer closes the value at end of line and reads the flush-left lines as mapping-level text.
Mapping-level text before a `:` is a key,
so it is evaluated as an implicit (colon-form) key.

### Step 3: YAML restricts implicit keys to a single line

```js
// node_modules/.pnpm/yaml@2.9.0/node_modules/yaml/dist/compose/resolve-block-map.js:48
onError(key ?? start[start.length - 1], 'MULTILINE_IMPLICIT_KEY', 'Implicit keys need to be on a single line');
```

The YAML 1.2 specification states the restriction:
"If the `?` indicator is omitted,
parsing needs to
see past the implicit key to recognize it as such.
To limit the amount of lookahead required,
the `:`
indicator must appear at most 1024 Unicode characters beyond the start of the key.
In addition,
the
key is restricted to a single line."
Explicit keys (`?` then `:`) are the multi-line key form;
nothing
in Pi requires descriptions to be one line.

### Step 4: Pi reports the parse error and skips the skill

```js
// node_modules/.pnpm/@earendil-works+pi-coding-agent@1.2.0/node_modules/@earendil-works/pi-coding-agent/dist/modes/interactive/interactive-mode.js:1436
this.loadedResourcesContainer.addChild(new ThemedText(() => `${theme.fg("warning", "[Skill conflicts]")}\n${warningLines()}`, 0, 0));
```

The failure therefore presents as a missing skill plus a startup warning.
Skill diagnostics render in
the interactive TUI and the rpc skills list only;
`pi -p` print mode does not surface them,
so headless
runs see just the missing skill with no explanation.

## Verification

### Versions under test

- `@earendil-works/pi-coding-agent@1.2.0`,
  whose bundled `pi` reports `pi --version` 1.2.0,
  pinning
  `yaml@2.9.0`.
- `yaml@2.9.1` and `yaml@3.0.0-2`,
  also installed in this workspace,
  emit the identical message and
  code for every shape below.

### Harness

Run from the repository root,
where `yaml` resolves:

```bash
node --input-type=module-typescript -e '
import { parse } from "yaml";
const shapes = {
  "two wrap lines at column 0": "name: x\ndescription: one two\nthree four\nfive six\n",
  "one wrap line at column 0": "name: x\ndescription: one two\nthree four\n",
  "colon-space in a plain value": "name: x\ndescription: use when a: b matters\n",
  "folded block scalar >-": "name: x\ndescription: >-\n  one two\n  three four\n",
  "plain continuation indented 2": "name: x\ndescription: one two\n  three four\n",
  "single line": "name: x\ndescription: one two three four\n",
};
for (const [label, source] of Object.entries(shapes)) {
  try {
    parse(source);
    console.log("PASS  " + label);
  } catch (error) {
    console.log("FAIL  " + label + " -> " + error.code + ": " + String(error.message).split("\n")[0]);
  }
}
'
```

### Failing catalog, split by variant

- `MULTILINE_IMPLICIT_KEY` (this incident):
  two or more wrap lines at column 0,
  as in
  `.agents/skills/android-emulator/SKILL.md` before the fix (description spanning file lines 3 to 5).
- `MISSING_CHAR`:
  exactly one wrap line at column 0,
  as in the first repaired fixture attempt.
- `BLOCK_AS_IMPLICIT_KEY`:
  `": "` inside the plain value,
  as in the same fixture attempt
  ("the pre-fix failure shape:
  wrapped description lines").

Harness output:

```text
FAIL  two wrap lines at column 0 -> MULTILINE_IMPLICIT_KEY: Implicit keys need to be on a single line at line 3, column 1:
FAIL  one wrap line at column 0 -> MISSING_CHAR: Implicit map keys need to be followed by map values at line 3, column 1:
FAIL  colon-space in a plain value -> BLOCK_AS_IMPLICIT_KEY: Nested mappings are not allowed in compact mappings at line 2, column 14:
PASS  folded block scalar >-
PASS  plain continuation indented 2
PASS  single line
```

### Clean catalog

- `description: >-` folded block scalar with wrapped,
  indented lines (used by
  `.agents/skills/choosing-technology/SKILL.md` and by the three repaired skills).
- `description: >` folded block scalar (used by several `~/.agents/skills/depot-*` skills).
- Plain scalar with wrap lines indented two spaces.
- Plain scalar on one line.
- Every remaining `SKILL.md` frontmatter in the two skill roots,
  verified with Pi's own loader.

### Consumer verification

Pi's `loadSkills` over the repo and user skill roots,
before and after the repair:

```bash
# node_modules/.pnpm/@earendil-works+pi-coding-agent@1.2.0/node_modules/@earendil-works/pi-coding-agent/dist/core/skills.js
node --input-type=module -e "import { loadSkills } from 'node_modules/.pnpm/@earendil-works+pi-coding-agent@1.2.0/node_modules/@earendil-works/pi-coding-agent/dist/core/skills.js'; const r = loadSkills({ cwd: process.cwd(), skillPaths: ['.agents/skills', '/var/home/user/.agents/skills'], includeDefaults: false }); console.log('skills loaded:', r.skills.length); console.log('diagnostics:', r.diagnostics.length);"
```

Before the repair:
`skills loaded: 36`,
`diagnostics: 3` (the three skills in this incident).
After:
`skills loaded: 39`,
`diagnostics: 0`.

## Verified workarounds

### Fold the description into a `>-` block scalar (applied in this repo)

```yaml
# .agents/skills/android-emulator/SKILL.md
description: >-
  Android emulator or AVD work. Use before launching, scripting or briefing an agent
  to use the Android emulator (device passes, screenshots, gfxinfo, adb on an emulator).
```

Tradeoffs:
wrap lines fold to single spaces,
so the value reads as the intended prose and diffs stay
narrow.
A later re-wrap must keep the two-space indent or the block closes and the bug returns;
the
trailing `-` strips the final newline so the value is exactly the joined prose.

### Indent the plain continuation lines

```yaml
description: Android emulator or AVD work. Use before launching, scripting or briefing an agent
  to use the Android emulator (device passes, screenshots, gfxinfo, adb on an emulator).
```

Tradeoffs:
smallest possible diff and no block-scalar indicator,
but the multi-line intent is
invisible,
so an editor or a markdown-style re-wrap back to column 0 silently reintroduces the failure.

### Collapse the description to one line

```yaml
description: Android emulator or AVD work. Use before launching, scripting or briefing an agent to use the Android emulator.
```

Tradeoffs:
always parses and needs no indentation discipline,
but produces long lines that fight the
repo's semantic-line-break habit and make frontmatter diffs hard to read.
Reserved for the case where
both wrapped forms are rejected by some consumer.

### Guard the shape (landed)

`mise run lint:skills` (defined in `mise.no-env.toml`,
part of `mise run lint`) parses every
`.agents/skills/*/SKILL.md` frontmatter with `yaml` and enforces the loader-spec fields:
kebab-case
`name` matching the directory,
non-empty `description` of at most 1024 characters.
It exits 1 on the
failing shapes catalogued above and warns (without failing) on unknown frontmatter keys,
since other
skill consumers legitimately carry their own keys.
Other roots check on demand with
`mise run lint:skills -- ~/.agents/skills`.

## What does not work

- Explicit `?` keys.
  The message hints at them,
  but the misread node is a value,
  not a key we want to
  write;
  `?` would restructure the mapping and break every loader expecting `name:` and `description:`.
- Treating this as a "descriptions must be one line" loader rule.
  Pi enforces no such rule;
  folded,
  literal,
  indented and single-line values all load.
- Repairing the wrap to a single continuation line.
  Still fails,
  with `MISSING_CHAR` instead.
- Blaming a formatter.
  dprint's markdown plugin is commented out in `package/config/dprint/index.json`,
  and `dprint check --list-different` lists zero `.md` paths,
  so nothing in the repo re-wraps
  frontmatter.
  The column 0 wraps came from applying the markdown semantic-line-break habit to YAML.
- Reproducing by parsing the whole `SKILL.md` as one YAML document.
  The extra `---` shifts positions
  (the same failure then reports "line 4,
  column 1");
  reproduce against the extracted frontmatter
  body the way `parseFrontmatter` does.

## Upstream filing artifact (do not file without an upstream bug)

### Upstream filing decision

`.out-of-scope/` check:
no exemption in `.out-of-scope/` names `yaml`,
frontmatter,
skills,
or this
message (searched every file in the directory),
so no exemption applies and the constraint walk runs.

Duplicate search:
`gh search issues "Implicit keys need to be on a single line"` finds the same trap
reported against other consumers,
notably `anthropics/claude-code` issue 94301 ("Unindented multi-line
frontmatter description silently fails to parse,
falls back to first body heading") and
`litmuschaos/litmus` issue 4907.
No matching issue exists in `eemeli/yaml` or the `pi` tracker.
The
claude-code thread documents the same YAML trap behind a different symptom (silent fallback to the
body heading);
nothing in this incident adds to that thread,
so no comment is kept.

Constraint walk,
targeting `eemeli/yaml` with `pi` as the consumer:

1. Upstream fault:
   no. The composer enforces the YAML 1.2 implicit-key rule,
   Pi reports the parse
   error as designed,
   and our `SKILL.md` files were malformed.
   Fails.
2. Could upstream fix it:
   the emitting library cannot relax a specification rule;
   a consumer
   auto-repairing wrapped values would be a feature,
   not a bug fix.
   Fails as a bug.
3. Use case support:
   both tools document block scalars as the multi-line value form.
   Not a gap.
4. Contribution welcome:
   not reached,
   since the gate closes at constraint 1.
5. Likely fix:
   behavior is intended and identical across `yaml` 2.9.0 to 3.0.0-2.
   No.
6. Minimal upstream fix prototyped:
   not applicable.
   The consumer-side fix is prototyped and landed,
   see Verified workarounds.

Decision:
do not file upstream (default policy).

### Draft (do not file as-is)

~~~md
Title: skill frontmatter parse errors carry body-relative lines and are invisible in `pi -p`

Description: When loadSkills diagnostics carry a yaml YAMLParseError, the error text is
body-relative ("line 3, column 1" is file line 4 because the opening --- fence is stripped
first), and pi -p does not surface skill diagnostics at all, so a headless run sees only a
missing skill while the interactive TUI renders "[Skill conflicts]". A consumer-side
improvement would prefix each diagnostic with its skill path and map positions to file lines.

Reproduction and shape catalog: doc/troubleshooting/yaml-multiline-implicit-key.md in the
reporting repository. This draft records the audit trail only; see the constraint walk above.
~~~
