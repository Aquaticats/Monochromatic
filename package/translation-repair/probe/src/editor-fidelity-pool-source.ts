import { REVIEWED_CORPUS_SHA, } from './fidelity-reference-manifest.ts';
import type {
  FidelityReferenceSpan,
  FidelityReferenceSpec,
} from './fidelity-reference-model.ts';

//region Editor fidelity pool source
// THE TEXT OF THE DRAWN PARTS OF THE MANIFEST, written by the draw so that the files in the repository are what the
// draw gives. IDS, OFFSETS AND HASHES ONLY: the corpus is unlicensed, so no passage of it is written here. The layout is
// the one the package's formatter leaves alone, numeric separators included, so a check that compares these bytes with
// the committed files does not fail on formatting.

/**
 One file the draw writes.

 @example
 ```ts
 const file: PoolFile = { name: 'editor-fidelity-pool-drawn-1.ts', text: '', };
 ```
 */
export type PoolFile = {
  /**
   File name, written beside the manifest.
   */
  readonly name: string;

  /**
   File contents.
   */
  readonly text: string;
};

/**
 Digits from which a number is written with separators every three, as the formatter writes it.
 */
const SEPARATED_FROM_DIGITS = 4;

/**
 Digits between two separators.
 */
const GROUP_DIGITS = 3;

/**
 Writes a count or offset the way the formatter does: digits in groups of three from the right, joined by underscores,
 from four digits up.

 @param value - non-negative whole number

 @returns The number as source text

 @example
 ```ts
 const written = numberText({ value: 12345, },);
 ```
 */
function numberText({ value, }: { readonly value: number; },): string {
  /**
   The digits.
   */
  const digits = String(value,);
  if (digits.length < SEPARATED_FROM_DIGITS)
    return digits;
  /**
   Groups from the right, the first one possibly shorter.
   */
  const groups = Array.from(
    { length: Math.ceil(digits.length / GROUP_DIGITS,), },
    function groupAt(
      _unused,
      index,
    ): string {
      return digits.slice(
        Math.max(
          0,
          digits.length - ((index + 1) * GROUP_DIGITS),
        ),
        digits.length - (index * GROUP_DIGITS),
      );
    },
  );
  return groups.toReversed()
    .join('_',);
}

/**
 Writes one span of a specification.

 @param name - field name

 @param span - span written

 @returns The field as source text, one indentation level in

 @example
 ```ts
 const written = spanText({ name: 'source', span, },);
 ```
 */
function spanText(
  {
    name,
    span,
  }: {
    readonly name: string;
    readonly span: FidelityReferenceSpan;
  },
): string {
  return `    ${name}: {
      startOffset: ${numberText({ value: span.startOffset, },)},
      endOffset: ${numberText({ value: span.endOffset, },)},
      hash: '${span.hash}',
    },
`;
}

/**
 Writes one specification.

 @param spec - specification written

 @returns The specification as source text

 @example
 ```ts
 const written = specText({ spec, },);
 ```
 */
function specText({ spec, }: { readonly spec: FidelityReferenceSpec; },): string {
  return `  {
    id: '${spec.id}',
    entryId: '${spec.entryId}',
    corpusSha: ${(spec.corpusSha === REVIEWED_CORPUS_SHA) ? 'REVIEWED_CORPUS_SHA' : `'${spec.corpusSha}'`},
${spanText({
  name: 'source',
  span: spec.source,
},)}${spanText({
  name: 'archive',
  span: spec.archive,
},)}    referenceHash: '${spec.referenceHash}',
    referenceChars: ${numberText({ value: spec.referenceChars, },)},
    edits: [],
${spanText({
  name: 'donor',
  span: spec.donor,
},)}    damages: [
${spec.damages
  .map(function damageText(damage,): string {
    return `      {
        kind: '${damage.kind}',
        hash: '${damage.hash}',
        changedChars: ${numberText({ value: damage.changedChars, },)},
      },
`;
  },)
  .join('',)}    ],
    reviewedOn: '${spec.reviewedOn}',
  },
`;
}

/**
 Writes the parts of the drawn specifications and the file that gathers them.

 @param specs - drawn specifications, in the order of the draw

 @param seed - seed that drew them

 @param perPart - specifications to a part

 @returns The part files in order, then the file that gathers them

 @example
 ```ts
 const files = poolSourceFiles({ specs, seed: 'round-4', perPart: 6, },);
 ```
 */
export function poolSourceFiles(
  {
    specs,
    seed,
    perPart,
  }: {
    readonly specs: readonly FidelityReferenceSpec[];
    readonly seed: string;
    readonly perPart: number;
  },
): readonly PoolFile[] {
  /**
   Number of parts.
   */
  const parts = Math.ceil(specs.length / perPart,);
  /**
   The part files.
   */
  const partFiles = Array.from(
    { length: parts, },
    function partAt(
      _unused,
      index,
    ): PoolFile {
      /**
       Specifications of this part.
       */
      const own = specs.slice(
        index * perPart,
        (index + 1) * perPart,
      );
      /**
       Position of the part among the parts, from one.
       */
      const number = String(index + 1,);
      return {
        name: `editor-fidelity-pool-drawn-${number}.ts`,
        text: `import { REVIEWED_CORPUS_SHA, } from './fidelity-reference-manifest.ts';
import type { FidelityReferenceSpec, } from './fidelity-reference-model.ts';

//region Editor fidelity pool, drawn specifications ${number}
// Ids, offsets and hashes only; no corpus text. Drawn by the rule in \`editor-fidelity-pool-manifest.ts\` with seed \`${seed}\`:
// entries \`${own[0]
  ?.entryId
  ?? ''}\` to \`${own.at(-1,)
    ?.entryId
    ?? ''}\` in the order of the draw.

/**
 Specifications ${number} of ${String(parts,)} drawn from the pinned corpus for the editor fidelity pool, each a heading section of
 an entry with the hashes of its source, its archive text, its donor and its damaged twins.

 @example
 \`\`\`ts
 const drawn: readonly FidelityReferenceSpec[] = DRAWN_EDITOR_SPECS_${number};
 \`\`\`
 */
export const DRAWN_EDITOR_SPECS_${number}: readonly FidelityReferenceSpec[] = [
${own.map(function each(spec,): string {
          return specText({ spec, },);
        },)
  .join('',)}];

//endregion Editor fidelity pool, drawn specifications ${number}
`,
      };
    },
  );
  /**
   The names of the parts, one to a line of imports and of spreads.
   */
  const numbers = partFiles.map(function numberOf(
    _file,
    index,
  ): string {
    return String(index + 1,);
  },);
  return [
    ...partFiles,
    {
      name: 'editor-fidelity-pool-drawn.ts',
      text: `${numbers.map(function importOf(number,): string {
        return `import { DRAWN_EDITOR_SPECS_${number}, } from './editor-fidelity-pool-drawn-${number}.ts';\n`;
      },)
        .join('',)}import type { FidelityReferenceSpec, } from './fidelity-reference-model.ts';

//region Editor fidelity pool, drawn specifications
// Every specification the draw of seed \`${seed}\` took, the parts in order. Written by \`editor-fidelity-pool-draw\`; do not
// edit by hand.

/**
 The drawn specifications of the editor fidelity pool.

 @example
 \`\`\`ts
 const drawn: readonly FidelityReferenceSpec[] = DRAWN_EDITOR_SPECS;
 \`\`\`
 */
export const DRAWN_EDITOR_SPECS: readonly FidelityReferenceSpec[] = [
${numbers.map(function spreadOf(number,): string {
        return `  ...DRAWN_EDITOR_SPECS_${number},\n`;
      },)
  .join('',)}];

//endregion Editor fidelity pool, drawn specifications
`,
    },
  ];
}

//endregion Editor fidelity pool source
