/**
 Deterministic rendering of the plan's inline resolution data
 (`plan.artifacts.data.ts` rows and the `version` fields in `plan.tools.data.ts`),
 which `toolchain:update` rewrites after re-resolution.
 */

import type { ToolArtifacts, } from './plan.artifacts.data.ts';

//region plan.artifacts.data.ts

/**
 Header block of `plan.artifacts.data.ts`, preserved verbatim (it records how the
 first resolutions were captured).
 */
const ARTIFACTS_HEADER = `/**
 Per-platform artifact provenance for the toolchain plan.

 Seeded from the \`mise.lock\` resolutions extracted to
 \`\${HOME}/temp/agent/mise-lock-extract-2026-10-26.json\` on 2026-10-26.
 Linux-only rows are carried over as-is
 (doc/decision/mise-lock-linux-only-stopgap.md);
 macOS and Windows artifact rows stay absent until they are resolved.

 Rewritten by \`toolchain:update\` when a re-resolution moves a version;
 the header keeps the original capture record (2026-10-26).

 \`taplo\`'s \`mise.lock\` row recorded a blake3 digest for \`linux-x64\`; the
 SHA-256 recorded here was computed from the same artifact bytes on
 2026-10-26, so every row stays a SHA-256 the sync can check.
 */

/**
 One downloadable artifact: its URL and the SHA-256 the lock records.
 */
export type ArtifactRow = {
  readonly url: string;
  readonly sha256: string;
};

/**
 Artifact rows keyed by platform id (\`linux-x64\`, \`linux-arm64-musl\`, ...).
 */
export type ToolArtifacts = Readonly<Record<string, ArtifactRow>>;

`;

/**
 Renders `plan.artifacts.data.ts` from one artifact map.

 @param artifacts - artifact tables keyed by tool id

 @returns complete file text, byte-stable for the same map

 @example
 ```ts
 renderPlanArtifactsTs({ artifacts: { age: { 'linux-x64': { url: 'https://example/age.tgz', sha256: 'a'.repeat(64,), }, }, }, },);
 ```
 */
export function renderPlanArtifactsTs(
  {
    artifacts,
  }: {
    readonly artifacts: Readonly<Record<string, ToolArtifacts>>;
  },
): string {
  /**
   Tool ids in ascending order.
   */
  const ids = Object.keys(artifacts,)
    .toSorted();
  /**
   Rendered tool blocks.
   */
  const blocks: string[] = [];
  for (const id of ids) {
    /**
     Platform ids in ascending order.
     */
    const platforms = Object.keys(artifacts[id] ?? {},)
      .toSorted();
    /**
     Rendered rows.
     */
    const rows: string[] = [];
    for (const platform of platforms) {
      /**
       One artifact row.
       */
      const row = artifacts[id]?.[platform];
      if (row === undefined)
        continue;
      rows.push(
        `    '${platform}': {\n      url: '${
          row.url
        }',\n      sha256: '${
          row.sha256
        }',\n    },`,
      );
    }
    blocks.push(rows.length === 0
      ? `  '${id}': {},`
      : `  '${id}': {\n${
        rows.join('\n',)
      }\n  },`,);
  }
  return `${
    ARTIFACTS_HEADER
  }/**
 Artifact provenance keyed by tool id, then by platform id.

 Keys are sorted for deterministic rendering.
 */
export const ARTIFACTS: Readonly<Record<string, ToolArtifacts>> = {
${
  blocks.join('\n',)
}
};

/**
 Looks one tool's artifact table up by tool id.

 @param id - Stable tool id

 @returns Artifact table recorded for the tool

 @throws Error when the id has no artifact table

 @example
 \`\`\`ts
 artifacts('age');
 \`\`\`
 */
export function artifacts(id: string,): ToolArtifacts {
  /**
   Recorded table, absent for an unknown id.
   */
  const table = ARTIFACTS[id];
  if (table === undefined)
    throw new Error(\`no artifact provenance recorded for \${JSON.stringify(id,)}\`,);
  return table;
}
`;
}

//endregion plan.artifacts.data.ts

//region plan.tools.data.ts

/**
 Marker that closes one plan entry object literal at entry depth (nested
 blocks close deeper indented).
 */
const ENTRY_CLOSE = '\n  },';

/**
 Marker that opens the resolved version literal inside one plan entry.
 */
const VERSION_OPEN = `version: '`;

/**
 Reads the version a plan entry pins before an update.

 @param entryText - entry object literal text

 @param id - tool id for error messages

 @returns version literal content

 @throws Error when the entry carries no non-empty `version: '...'` field
 */
function recordedVersion(
  {
    entryText,
    id,
  }: {
    readonly entryText: string;
    readonly id: string;
  },
): string {
  /**
   Index of the version field marker, or -1 when the entry has none.
   */
  const markerAt = entryText.indexOf(VERSION_OPEN,);
  /**
   Index of the version literal's first character.
   */
  const openAt = markerAt + VERSION_OPEN.length;
  /**
   Index of the closing quote, or -1 when the field is absent or unterminated.
   */
  const closeAt = markerAt === (-1)
    ? -1
    : entryText.indexOf(
      `'`,
      openAt,
    );
  if ((closeAt === (-1)) || (closeAt === openAt)) {
    throw new Error(`no version field found for plan entry ${id}`,);
  }
  return entryText.slice(
    openAt,
    closeAt,
  );
}

/**
 Text that opens the id field of a plan entry, up to the id's opening quote.
 */
const ID_OPEN = `id: '`;

/**
 Start index of the first `id: '<id>',` field for every id the source declares,
 found in one left-to-right pass.

 @param source - plan source text

 @returns first start index keyed by id
 */
function firstIdStarts(
  { source, }: { readonly source: string; },
): ReadonlyMap<string, number> {
  /**
   First start index keyed by id.
   */
  const starts = new Map<string, number>();
  for (let at = source.indexOf(ID_OPEN,); at !== (-1); at = source.indexOf(
    ID_OPEN,
    at + 1,
  )) {
    /**
     Index of the closing quote of the id literal; the next search starts past it.
     */
    const quoteAt = source.indexOf(
      `'`,
      at + ID_OPEN.length,
    );
    if (quoteAt === (-1))
      break;
    /**
     Id literal, kept only when a comma follows the closing quote.
     */
    const id = source.slice(
      at + ID_OPEN.length,
      quoteAt,
    );
    if ((source.charAt(quoteAt + 1,) === ',') && (!starts.has(id,)))
      starts.set(
        id,
        at,
      );
  }
  return starts;
}

/**
 One entry to rewrite: its id, new version, and where its object literal starts.
 */
type EntryEdit = {
  readonly id: string;
  readonly version: string;
  readonly start: number;
};

/**
 Tool id with its new version, as iterating a version map lists them.
 */
type VersionPair = readonly [
  string,
  string,
];

/**
 One rewritten entry: the source range it replaces and its new text.
 */
type EntryPatch = {
  readonly start: number;
  readonly end: number;
  readonly moved: string;
};

/**
 Rewrites one entry's text so the literals ending in its old version end in the new one.

 @param source - plan source text

 @param edit - entry to rewrite

 @returns source range of the entry and its rewritten text

 @throws Error when the entry has no closing brace or no version field
 */
function patchEntry(
  {
    source,
    edit,
  }: {
    readonly source: string;
    readonly edit: EntryEdit;
  },
): EntryPatch {
  /**
   Index of the closing brace at entry depth, or -1 when the entry never closes.
   */
  const closeAt = source.indexOf(
    ENTRY_CLOSE,
    edit.start,
  );
  if (closeAt === (-1)) {
    throw new Error(`no entry found for plan entry ${edit.id}`,);
  }
  /**
   Index just past the entry's closing brace.
   */
  const end = closeAt + ENTRY_CLOSE.length;
  /**
   Entry text carrying the resolution: its id field through the closing brace.
   */
  const entryText = source.slice(
    edit.start,
    end,
  );
  /**
   Resolved version the entry pins before this update.
   */
  const oldVersion = recordedVersion({
    entryText,
    id: edit.id,
  },);
  /**
   Entry text with the string literals ending in the old version moved to
   the new one: `version`, `packageId`, and env values like `java/<version>`.
   Comment lines keep their prose, so a comment citing the superseded
   version stays a historical note.
   */
  const moved = entryText
    .split('\n',)
    .map(function moveLine(line,): string {
      return line.trimStart()
        .startsWith('//',)
        ? line
        : line.replaceAll(
          `${oldVersion}'`,
          `${edit.version}'`,
        );
    },)
    .join('\n',);
  return {
    start: edit.start,
    end,
    moved,
  };
}

/**
 Rewrites each named entry's resolution in `plan.tools.data.ts`, carrying the
 literals that pin it.

 Every entry is located in one scan and rewritten in one ordered pass, so the
 work is linear in the source length plus a sort of the edited entries.

 @param source - file text

 @param versions - new versions keyed by tool id

 @returns file text with each resolution and its pin literals updated

 @throws Error when an entry or its version field cannot be found, or when two edited entries overlap

 @example
 ```ts
 patchPlanVersions({ source: "  {\n    id: 'age',\n    version: '1.0.0',\n  },", versions: new Map([['age', '1.1.0',],],), },);
 ```
 */
export function patchPlanVersions(
  {
    source,
    versions,
  }: {
    readonly source: string;
    readonly versions: ReadonlyMap<string, string>;
  },
): string {
  /**
   First start index of every id the source declares.
   */
  const starts = firstIdStarts({ source, },);
  /**
   Entries to rewrite, in source order.
   */
  const edits = [...versions,].map(function toEdit(pair: VersionPair,): EntryEdit {
    /**
     Tool id and its new version.
     */
    const [id, version,] = pair;
    /**
     Start index of this entry, absent when the source never declares the id.
     */
    const start = starts.get(id,);
    if (start === undefined) {
      throw new Error(`no entry found for plan entry ${id}`,);
    }
    return {
      id,
      version,
      start,
    };
  },)
    .toSorted(function bySourceOrder(
      left,
      right,
    ): number {
      return left.start - right.start;
    },);
  /**
   Rewritten entries in source order, each searched only past the previous one.
   */
  const patches: EntryPatch[] = [];
  for (const edit of edits) {
    /**
     Previous rewritten entry.
     */
    const previous = patches.at(-1,);
    if ((previous !== undefined) && (edit.start < previous.end)) {
      throw new Error(`plan entry ${edit.id} starts inside the previous edited entry`,);
    }
    patches.push(patchEntry({
      source,
      edit,
    },),);
  }
  /**
   Source text between the entries, each gap followed by the entry that ends it.
   */
  const pieces = patches.map(function gapThenEntry(
    patch,
    at,
  ): string {
    /**
     Entry before this one, absent for the first.
     */
    const before = patches[at - 1];
    return `${
      source.slice(
        before === undefined
          ? 0
          : before.end,
        patch.start,
      )
    }${patch.moved}`;
  },);
  /**
   Last rewritten entry, absent when nothing was edited.
   */
  const last = patches.at(-1,);
  return `${
    pieces.join('',)
  }${
    source.slice(last === undefined
      ? 0
      : last.end,)
  }`;
}

//endregion plan.tools.data.ts
