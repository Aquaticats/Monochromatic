# module-pnpm-workspace-catalog

Safe parsing and discovery for `pnpm-workspace.yaml` catalog blocks.

The package parses YAML with the `yaml` library,
validates catalog keys as npm package names,
and stores results in null-prototype maps.
Invalid keys and non-string values are warned about and skipped without discarding valid entries.

## API

- `parseCatalogFromYaml(content)` parses default `catalog:` and named `catalogs:` blocks.
- `readCatalogFile({ startDir? })` finds the nearest workspace file
  and returns both its original content and parsed catalogs.
- `flattenCatalogEntries({ document, includeNamedCatalogs? })` returns raw entries
  while preserving `npm:` alias values.
- `isValidPackageName(name)` validates the npm package-name shape used for catalog keys.

Named catalogs are excluded from flattening unless `includeNamedCatalogs: true` is provided.
The parsed document always retains both default and named blocks,
so callers can make that choice without reparsing.

## Usage

```ts
import {
  flattenCatalogEntries,
  readCatalogFile,
} from '@monochromatic-dev/dependency-catalog-core-pnpm';

const workspace = await readCatalogFile();
const entries = flattenCatalogEntries({
  document: workspace.catalogs,
  includeNamedCatalogs: true,
});
```

`workspace.content` is the exact source text used for parsing.
Tools that preserve comments,
ordering,
whitespace,
or quote style can use it for surgical write-back instead of serializing the parsed YAML again.

## Scope

This package intentionally supports the pnpm workspace YAML format only.
Other catalog source formats are tracked separately by the manager-specific catalog-tighten issues.
