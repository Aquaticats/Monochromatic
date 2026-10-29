/**
 Deterministic rendering of the plan's inline resolution data
 (`plan.artifacts.ts` rows and the `version` fields in `plan.tools.ts`),
 which `toolchain:update` rewrites after re-resolution.
 */

import type { ToolArtifacts, } from './plan.artifacts.ts';

//region plan.artifacts.ts

/**
 Header block of `plan.artifacts.ts`, preserved verbatim (it records how the
 first resolutions were captured).
 */
const ARTIFACTS_HEADER = `/**
 * Per-platform artifact provenance for the toolchain plan.
 *
 * Seeded from the \`mise.lock\` resolutions extracted to
 * \`\${HOME}/temp/agent/mise-lock-extract-2026-10-26.json\` on 2026-10-26.
 * Linux-only rows are carried over as-is
 * (doc/decision/mise-lock-linux-only-stopgap.md);
 * macOS and Windows artifact rows stay absent until they are resolved.
 *
 * Rewritten by \`toolchain:update\` when a re-resolution moves a version;
 * the header keeps the original capture record (2026-10-26).
 *
 * \`taplo\`'s \`mise.lock\` row recorded a blake3 digest for \`linux-x64\`; the
 * SHA-256 recorded here was computed from the same artifact bytes on
 * 2026-10-26, so every row stays a SHA-256 the sync can check.
 */

/** One downloadable artifact: its URL and the SHA-256 the lock records. */
export type ArtifactRow = {
  readonly url: string;
  readonly sha256: string;
};

/** Artifact rows keyed by platform id (\`linux-x64\`, \`linux-arm64-musl\`, ...). */
export type ToolArtifacts = Readonly<Record<string, ArtifactRow>>;

`;

/**
 Renders `plan.artifacts.ts` from one artifact map.
 
 @param options - Artifact tables keyed by tool id
 
 @returns Complete file text, byte-stable for the same map
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
    .sort();
  /**
   Rendered tool blocks.
   */
  const blocks: string[] = [];
  for (const id of ids) {
    /**
     Platform ids in ascending order.
     */
    const platforms = Object.keys(artifacts[id] ?? {},)
      .sort();
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
        `    '${platform}': { url: '${
          row.url
        }', sha256: '${
          row.sha256
        }' },`,
      );
    }
    blocks.push(`  '${id}': {\n${
      rows.join('\n',)
    }\n  },`,);
  }
  return `${
    ARTIFACTS_HEADER
  }/**
 * Artifact provenance keyed by tool id, then by platform id.
 *
 * Keys are sorted for deterministic rendering.
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
 
 @throws {Error} when the id has no artifact table
 */
export function artifacts(id: string,): ToolArtifacts {
  const table = ARTIFACTS[id];
  if (table === undefined)
    throw new Error(\`no artifact provenance recorded for \${JSON.stringify(id,)}\`);
  return table;
}
`;
}

//endregion plan.artifacts.ts

//region plan.tools.ts

/**
 Rewrites each named entry's resolution in `plan.tools.ts`, carrying the
 literals that pin it.
 
 @param options - File text and the new versions keyed by tool id
 
 @returns File text with each resolution and its pin literals updated
 
 @throws Error when an entry's version field cannot be found
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
   Text with each entry's resolution and pins replaced.
   */
  let patched = source;
  for (const [id, version,] of versions) {
    /**
     Pattern spanning one entry's object literal: its id field through the
     closing brace at entry depth (nested blocks close deeper indented).
     */
    const pattern = new RegExp(
      `(id: '${
        id.replaceAll(
          /[.*+?^${}()|[\]\\]/gu,
          '\\\\$&',
        )
      }',[\\s\\S]*?\\n  \\},)`,
      'u',
    );
    /**
     Entry text carrying the resolution.
     */
    const match = patched.match(pattern,);
    if (match === null) {
      throw new Error(`no entry found for plan entry ${id}`,);
    }
    /**
     Resolved version the entry pins before this update.
     */
    const versionMatch = match[0].match(/version: '([^']*)'/u,);
    if (versionMatch === null) {
      throw new Error(`no version field found for plan entry ${id}`,);
    }
    const oldVersion = versionMatch[1];
    if (oldVersion === undefined || oldVersion === '') {
      throw new Error(`no version field found for plan entry ${id}`,);
    }
    /**
     Entry text with the string literals ending in the old version moved to
     the new one: `version`, `packageId`, and env values like `java/<version>`.
     Comment lines keep their prose, so a comment citing the superseded
     version stays a historical note.
     */
    const moved = match[0]
      .split('\n',)
      .map((line,) => line.trimStart().startsWith('//',)
        ? line
        : line.replaceAll(`${
          oldVersion
        }'`, `${
          version
        }'`,),)
      .join('\n',);
    patched = patched.replace(pattern, () => moved,);
  }
  return patched;
}

//endregion plan.tools.ts
