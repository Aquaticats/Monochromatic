import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import { frontMatterCommentAuthorityFindings, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  isJsonRecord,
  requireFrontMatterRefusal,
  splitFrontMatter,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { readFrontMatterGround, } from './translate-floor-ground.ts';
import type { SliceValidation, } from './translate-validate.ts';

//region Front matter translation
// YAML metadata is syntax-bearing localized content. Candidate shape is checked
// against archive metadata while model ensemble decides semantic translation.

/**
 Decision addendum shared by final candidate comparisons.
 */
export const FRONT_MATTER_DECISION_RULE: string = 'The candidates are complete YAML front matter. A candidate is '
  + 'flawed if it breaks YAML fences, field names, nesting, container lengths, or scalar kinds. ORIGINAL metadata '
  + 'values are source facts. The visible name field must identify the source person and must not be replaced by an '
  + 'entry directory id. When ORIGINAL name and info.alias are the same identity, the translated name must appear '
  + 'among the comma-separated renderings in translated info.alias (the alias may carry the original script and '
  + 'other renderings beside it); a candidate whose name appears nowhere in its alias is invalid. In info.location '
  + 'comments, keep established target contributor spelling after `, by ` where source and archive spell that '
  + 'contributor differently.';

/**
 Separator an alias list is written with in this corpus.

 MEASURED 2026-09-04 over the pinned archives: 70 alias values carry a
 comma, one a slash, none a Chinese comma or an enumeration mark.

 THE COMMA ALONE, where `ALIAS_SEPARATORS` (`corpus-run/directory-id-name.ts`)
 also splits on the full-width comma and the enumeration mark: that reader
 takes originals, which write both, and this one takes a translation, whose
 rule tells a model to separate renderings with commas (ledger B97).
 */
const ALIAS_SEPARATOR = ',';

/**
 Whether an alias carries the name among its renderings.

 THE OWNER'S DECISION OF 2026-09-04: where the ORIGINAL declares name and
 alias the same, the translated alias may carry the name beside other
 renderings ("鲵鲵, Nini" for the name "Nini"), because seven of the fourteen
 such archives at the pinned corpus already do, and equality would have forced
 every one of them to drop the original-script alias it publishes. Equality
 refused the luxuanwen3 page of that day after a full run.

 @param alias - alias value as the candidate writes it

 @param name - visible name the alias must carry

 @returns Whether some comma-separated rendering equals the name exactly

 @example
 ```ts
 const carried = aliasCarriesName({ alias: '鲵鲵, Nini', name: 'Nini', },);
 ```
 */
function aliasCarriesName(
  {
    alias,
    name,
  }: {
    readonly alias: string;
    readonly name: string;
  },
): boolean {
  return alias
    .split(ALIAS_SEPARATOR,)
    .some(function isName(rendering,): boolean {
      return rendering.trim() === name.trim();
    },);
}

/**
 Visible identity read from standard fields, or another metadata schema.

 @example
 ```ts
 const identity: VisibleIdentityReading = { kind: 'present', name: 'Mittens', alias: 'Mittens', };
 ```
 */
type VisibleIdentityReading =
  | {
    /**
     Standard visible identity fields are present.
     */
    readonly kind: 'present';

    /**
     Primary visible name.
     */
    readonly name: string;

    /**
     Alias nested under metadata info, a list's items joined on the
     separator.
     */
    readonly alias: string;
  }
    | {
    /**
     Metadata uses another schema and carries no enforceable relation here.
     */
      readonly kind: 'other-schema';
    };

/**
 Reads standard visible identity fields from parsed metadata, taking an alias
 written as a string or as a list of strings.

 @param value - parsed YAML document

 @returns Identity pair, or nothing when document uses another schema

 @example
 ```ts
 const identity = visibleIdentityOf({ value: { name: 'Mittens', info: { alias: 'Mittens', }, }, });
 ```
 */
function visibleIdentityOf({ value, }: { readonly value: unknown; },): VisibleIdentityReading {
  if (!isJsonRecord(value,))
    return { kind: 'other-schema', };
  /**
   Nested metadata containing declared alias.
   */
  const { info, } = value;
  if (!isJsonRecord(info,))
    return { kind: 'other-schema', };
  /**
   Primary value whose relationship carries source identity.
   */
  const { name, } = value;
  /**
   Alias value whose relationship carries source identity.
   */
  const { alias, } = info;
  if ((typeof name) !== 'string')
    return { kind: 'other-schema', };
  if ((typeof alias) === 'string') {
    return {
      kind: 'present',
      name,
      alias,
    };
  }
  // A LIST READS AS ITS ITEMS JOINED ON THE SEPARATOR, as `aliasesOf`
  // (`corpus-run/directory-id-name.ts`) reads one: read as another schema,
  // a list-shaped alias silently skipped the rule that the name appear among
  // the alias renderings (ledger B97).
  if (Array.isArray(alias,) && alias.every(function isText(item,): item is string {
    return (typeof item) === 'string';
  },)) {
    return {
      kind: 'present',
      name,
      alias: alias.join(ALIAS_SEPARATOR,),
    };
  }
  return { kind: 'other-schema', };
}

/**
 Structural signature for parsed YAML value.

 @param value - parsed YAML value

 @returns Stable signature of keys, containers and scalar kinds

 @example
 ```ts
 const shape = yamlShape({ value: { name: 'Mittens', }, });
 ```
 */
function yamlShape({ value, }: { readonly value: unknown; }): string {
  if (value === null)
    return 'null';
  if (Array.isArray(value,)) {
    /**
     Child signatures in container order.
     */
    const children = value.map(function child(item,): string {
      return yamlShape({ value: item, });
    },);
    return `[${children.join(',',)}]`;
  }
  if (isJsonRecord(value,)) {
    return `{${textsInCodePointOrder({ texts: Object.keys(value,), },)
      .map(function field(key,): string {
        return `${JSON.stringify(key,)}:${yamlShape({ value: value[key], })}`;
      },)
      .join(',',)}}`;
  }
  return typeof value;
}

/**
 Validates syntax and archive-compatible key shape of front matter candidate.

 @param sourceText - source front matter whose identity relationships govern

 @param pageText - archive front matter candidate replaces

 @param candidateText - proposed localized front matter

 @returns Translation validation result

 @throws Whatever the front-matter splitter throws that is not a YAML
 refusal, since that is a fault in this code rather than a fact about any
 text

 @example
 ```ts
 const validation = validateFrontMatterTranslation({ sourceText, pageText, candidateText, });
 ```
 */
export function validateFrontMatterTranslation(
  {
    sourceText,
    pageText,
    candidateText,
  }: {
    readonly sourceText: string;
    readonly pageText: string;
    readonly candidateText: string;
  },
): SliceValidation {
  /**
   The original's metadata and the page's, read by the one definition of
   whether this floor can compare anything (`translate-floor-ground.ts`,
   ledger B43), and read OUTSIDE the candidate's try: a refusal of either
   side's YAML is not the candidate's fault, and it was charged to the
   candidate as one (ledger B44).
   */
  const ground = readFrontMatterGround({
    sourceText,
    pageText,
  },);
  try {
    /**
     Parsed candidate metadata under review.
     */
    const candidate = splitFrontMatter({ text: candidateText, },);
    if (candidate.frontMatter === undefined) {
      return {
        kind: 'invalid',
        findings: ['Your translation must remain one YAML front matter block fenced by --- lines.',],
      };
    }
    /**
     Candidate body outside metadata fences.
     */
    const { body, } = candidate;
    /**
     Candidate body after insignificant whitespace is removed.
     */
    const candidateBody = body.trim();
    if (candidateBody.length > 0) {
      return {
        kind: 'invalid',
        findings: ['Your translation added text outside YAML front matter block.',],
      };
    }
    // AFTER THE CANDIDATE'S OWN FAULTS, which are its author's to fix
    // whatever either side is.
    if (ground.kind === 'blind') {
      return {
        kind: 'unknown',
        detail: ground.detail,
      };
    }
    /**
     Parsed source metadata.
     */
    const { data: sourceData, } = ground.source;
    /**
     Parsed candidate metadata.
     */
    const { data: candidateData, } = candidate.frontMatter;
    /**
     Parsed archive metadata when target already carries it.
     */
    const { page: pageFrontMatter, } = ground;
    /**
     Structural authority:
     archive metadata when it holds any,
     otherwise the original's shape:
     for a new insertion,
     and for an archive whose fence pair holds nothing (parsed as null),
     since an empty block has no established keys to keep.
     */
    const pageData = pageFrontMatter?.data ?? sourceData;
    if (yamlShape({ value: candidateData, }) !== yamlShape({ value: pageData, })) {
      return {
        kind: 'invalid',
        findings: ['Your translation changed YAML field names, nesting, container lengths, or scalar kinds.',],
      };
    }
    /**
     Comment attribution findings grounded at same YAML path.
     */
    const commentFindings = frontMatterCommentAuthorityFindings({
      sourceText,
      pageText,
      candidateText,
    },);
    if (commentFindings.length > 0) {
      return {
        kind: 'invalid',
        findings: commentFindings,
      };
    }
    /**
     Source identity pair, when standard fields expose one.
     */
    const sourceIdentity = visibleIdentityOf({ value: sourceData, },);
    /**
     Candidate identity pair under same standard fields.
     */
    const candidateIdentity = visibleIdentityOf({ value: candidateData, },);
    if ((sourceIdentity.kind === 'present')
      && (sourceIdentity.name === sourceIdentity.alias)
      && ((candidateIdentity.kind !== 'present')
        || (!aliasCarriesName({
          alias: candidateIdentity.alias,
          name: candidateIdentity.name,
        },)))) {
      return {
        kind: 'invalid',
        findings: [
          'Your translation must carry the name among the comma-separated renderings in info.alias because ORIGINAL declares name and info.alias as the same identity.',
        ],
      };
    }
    return {
      kind: 'valid',
      pageGrammar: 'strict',
    };
  }
  catch (error) {
    /**
     The candidate's YAML refusal, the only thing caught here that is its
     author's to fix: anything else is a fault in this code, and charging
     it to the candidate would send a model to revise text with nothing
     wrong in it (ledger B44).
     */
    const refusal = requireFrontMatterRefusal({ error, },);
    return {
      kind: 'invalid',
      findings: [`Your translation could not be parsed as YAML front matter: ${String(refusal,)}`,],
    };
  }
}

//endregion Front matter translation
