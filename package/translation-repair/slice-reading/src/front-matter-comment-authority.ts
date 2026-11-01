import {
  parseDocument as parseYamlDocument,
  Scalar,
  YAMLMap,
} from 'yaml';

//region Front matter comment authority

/**
 Comment or attribution lookup outcome.

 @example
 ```ts
 const reading: CommentReading = { kind: 'found', text: 'Qingyuan, by Mogu', };
 ```
 */
type CommentReading =
  | {
    /**
     Requested text exists.
     */
    readonly kind: 'found';

    /**
     Comment or attribution text.
     */
    readonly text: string;
  }
    | {
    /**
     Requested path or attribution marker does not exist.
     */
      readonly kind: 'not-found';
    };

/**
 Marker separating location prose from contributor attribution.
 */
const CONTRIBUTOR_MARKER = ', by ';

/**
 Removes front matter fences before YAML AST parsing.

 @param text - complete front matter block

 @returns YAML between opening and closing fences

 @example
 ```ts
 const yaml = yamlInsideFrontMatter({ text: '---\nname: Mittens\n---\n', });
 ```
 */
function yamlInsideFrontMatter({ text, }: { readonly text: string; },): string {
  /**
   Boundary after opening fence line.
   */
  const openingEnd = text.indexOf('\n',);
  /**
   Boundary before closing fence line.
   */
  const closingStart = text.lastIndexOf('\n---',);
  if ((!text.includes('\n',)) || (closingStart <= openingEnd))
    return text;
  return text.slice(
    openingEnd + 1,
    closingStart,
  );
}

/**
 Reads inline comment attached to `info.location` scalar by YAML path.

 @param text - complete front matter block

 @returns Comment text without hash marker, or named absence

 @example
 ```ts
 const comment = locationComment({ text, });
 ```
 */
function locationComment({ text, }: { readonly text: string; },): CommentReading {
  /**
   Parsed YAML document retaining node comments.
   */
  const document = parseYamlDocument(yamlInsideFrontMatter({ text, }),);
  if (!(document.value instanceof YAMLMap))
    return { kind: 'not-found', };
  /**
   Standard metadata info pair.
   */
  const infoPair = document.value
    .values
    .get('info',);
  if ((infoPair === undefined) || (!(infoPair.value instanceof YAMLMap)))
    return { kind: 'not-found', };
  /**
   Location pair whose scalar owns inline comment.
   */
  const locationPair = infoPair.value
    .values
    .get('location',);
  if ((locationPair === undefined) || (!(locationPair.value instanceof Scalar)))
    return { kind: 'not-found', };
  /**
   Inline comment as YAML parser records it.
   */
  const { comment, } = locationPair.value;
  if (((typeof comment) !== 'string') || (comment.trim() === ''))
    return { kind: 'not-found', };
  return {
    kind: 'found',
    text: comment,
  };
}

/**
 Reads contributor attribution from location comment.

 @param comment - location comment lookup

 @returns Contributor spelling after `, by `, or named absence

 @example
 ```ts
 const contributor = contributorAttribution({ comment, });
 ```
 */
function contributorAttribution(
  { comment, }: { readonly comment: CommentReading; },
): CommentReading {
  if (comment.kind !== 'found')
    return { kind: 'not-found', };
  /**
   Whether contributor delimiter exists in comment.
   */
  const hasContributor = comment.text
    .includes(CONTRIBUTOR_MARKER,);
  if (!hasContributor)
    return { kind: 'not-found', };
  /**
   Last contributor delimiter after presence was established. Using the last
   occurrence, not the first, lets the place portion of the comment carry its
   own commas without being mistaken for the marker.
   */
  const markerAt = comment.text
    .lastIndexOf(CONTRIBUTOR_MARKER,);
  /**
   Contributor spelling after delimiter.
   */
  const text = comment.text
    .slice(markerAt + CONTRIBUTOR_MARKER.length,)
    .trim();
  if (text === '')
    return { kind: 'not-found', };
  return {
    kind: 'found',
    text,
  };
}

/**
 Validates established target contributor spelling in location comment.

 SOURCE AND ARCHIVE COMMENTS AT SAME YAML PATH establish relation. When their
 contributor spellings differ, archive form is existing target-language name.
 Candidate may translate surrounding place comment but must not replace that
 established contributor with source-script form or another spelling.

 @param sourceText - source front matter

 @param pageText - archive front matter

 @param candidateText - candidate front matter

 @returns Model-facing findings, empty when no enforceable relation exists

 @example
 ```ts
 const findings = frontMatterCommentAuthorityFindings({ sourceText, pageText, candidateText, });
 ```
 */
export function frontMatterCommentAuthorityFindings(
  {
    sourceText,
    pageText,
    candidateText,
  }: {
    readonly sourceText: string;
    readonly pageText: string;
    readonly candidateText: string;
  },
): readonly string[] {
  /**
   Contributor spelling at source metadata path.
   */
  const source = contributorAttribution({
    comment: locationComment({ text: sourceText, }),
  },);
  /**
   Established target spelling at same metadata path.
   */
  const page = contributorAttribution({
    comment: locationComment({ text: pageText, }),
  },);
  if ((source.kind !== 'found')
    || (page.kind !== 'found')
    || (source.text === page.text))
    return [];
  /**
   Candidate spelling at same metadata path.
   */
  const candidate = contributorAttribution({
    comment: locationComment({ text: candidateText, }),
  },);
  if ((candidate.kind === 'found') && (candidate.text === page.text))
    return [];
  return [
    'Your translation must keep established target contributor spelling in info.location comment after `, by `.',
  ];
}

//endregion Front matter comment authority
