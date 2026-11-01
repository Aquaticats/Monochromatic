# @monochromatic-dev/mutation-test-cli-unused-export

Workspace-wide unused-export detection built on yuku-analyzer's
cross-file semantic model.

The tool feeds every workspace package's `src/**/*.ts(x)` sources into
one `Analyzer`,
links the module graph,
and reports every export whose backing symbol has zero resolved
references anywhere in the workspace.

## Usage

```bash
# report findings for the current workspace
mise run //package/mutation-test/cli-unused-export:run

# machine-readable output
node src/cli.ts --json

# non-zero exit when findings exist, for CI gates
node src/cli.ts --check

# analyze another workspace root
node src/cli.ts /path/to/workspace
```

Each plain-output line is
`<file>:<line>:<column> [type ]<name>`.

## Semantics

- A use anywhere in the workspace counts,
  because yuku-analyzer follows import,
  named re-export,
  and `export *` chains back to the defining symbol.
  An export re-exported through `index.ts` and consumed from another
  package is used;
  the same export with no eventual consumer is reported at its
  declaration site.
- Test files import package behavior from built `dist` entry points.
  The resolver maps `dist/final/<flavor>/index.mjs` specifiers back to
  the owning package's `src/index.ts`,
  so helpers exported for built-artifact tests count as used.
- Export records with no backing local symbol
  (`export *`,
  `export default <expression>`,
  re-export specifiers)
  are never findings themselves;
  their defining symbols are judged where they are declared.

## Caveats

- An export used only by its own package's tests still counts as used,
  which hides code that is dead outside its tests.
- Specifier forms other than relative paths and
  `@monochromatic-dev/<name>/ts[/subpath]` are treated as external and
  never produce usage.
