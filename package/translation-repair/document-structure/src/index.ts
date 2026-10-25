//region Public index
// The names `@monochromatic-dev/translation-repair-document-structure` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export {
  alignBlocks,
  type AlignmentStep,
} from './align-blocks-walk.ts';
export {
  estimateExpansion,
  FALLBACK_EXPANSION,
  scorePairing,
  tokenize,
} from './align-blocks.ts';
export {
  alignHeadingsForced,
  type ForcedAlignStep,
  type InsertionAnchor,
  type UnpairedReason,
} from './align-headings-forced.ts';
export {
  buildGrid,
  type Grid as HeadingAlignGrid,
} from './align-headings-grid.ts';
export {
  type OptimalPaths,
  scanOptimalPaths,
} from './align-headings-optimal.ts';
export { allInInputOrder, } from './all-in-input-order.ts';
export { longestRunOf, } from './character-run.ts';
export {
  type BlockExtent,
  widenExtentsToContainers,
  widenNodesToContainers,
} from './container-extents.ts';
export {
  buildDocumentNodes,
  type DocumentNode,
  type DocumentZone,
  hashContent,
  UnpositionedNodeError,
} from './document-node.ts';
export {
  CREDIT_BY_RULE,
  GAME_JARGON_RULE,
  GRAMMATICAL_ENGLISH_RULE,
  IDIOMATIC_ENGLISH_RULE,
  KEPT_SUBJECT_RULE,
  QUOTED_WORDS_REMINDER,
  QUOTED_WORDS_RULE,
} from './english-usage-policy.ts';
export { definitionStartsOf, } from './footnote-definition-starts.ts';
export {
  buildFootnoteGraph,
  scanFullwidthMarkers,
  scanGfmReferenceLiterals,
  type TextMarkerHit,
} from './footnote-graph.ts';
export {
  normalizeFootnoteIdentifier,
  relabelsFootnote,
} from './footnote-identifier.ts';
export {
  footnoteIdentifiers,
  type FootnoteMention,
  footnoteMentions,
  FootnoteOverflowError,
  MAX_SLICE_IDENTIFIERS,
} from './footnote-mentions.ts';
export {
  type FootnoteConvention,
  type FootnoteDefinitionHit,
  type FootnoteGraph,
  type FootnoteGraphFinding,
  type FootnoteReferenceHit,
} from './footnote-model.ts';
export {
  fragmentReadingOf,
  insideParsedSpan,
} from './footnote-parsed-spans.ts';
export {
  FootnoteRewriteError,
  type FootnoteRewriteFailure,
  requireFootnoteRewriteRefusal,
} from './footnote-rewrite-error.ts';
export { disjointSpans, } from './footnote-span-union.ts';
export {
  isAutolinkLiteral,
  NO_NODE_BOUNDS,
  nodeBounds,
  type TreeNode,
  unpositionedRuns,
} from './footnote-unpositioned-runs.ts';
export {
  type FrontMatterBlock,
  FrontMatterParseError,
  leadingMarkWidth,
  requireFrontMatterRefusal,
  splitFrontMatter,
  type SplitMdxDocument,
  yamlRefusalSite,
} from './front-matter.ts';
export {
  gfmMarkerAt,
  type GfmMarkerSpan,
  gfmMarkerSpans,
} from './gfm-marker-spans.ts';
export {
  isJsonArray,
  isJsonRecord,
  requireJsonSyntaxRefusal,
} from './json-guard.ts';
export {
  type ParsedBlock,
  parsedTopLevelBlocks,
} from './markdown-blocks.ts';
export {
  fenceForMarkdown,
  longestBacktickRun,
} from './markdown-fence.ts';
export {
  type LoneContainerTag,
  maskLoneContainerTags,
} from './mask-container-tags.ts';
export {
  commentRegionsOf,
  type MaskedCommentRegion,
  maskHtmlComments,
} from './mask-html-comments.ts';
export { maskInvisibleLines, } from './mask-invisible-lines.ts';
export {
  headingFloorOf,
  noteHeadingFloor,
} from './heading-floor-store.ts';
export { noteHeadingFloors, } from './heading-floors.ts';
export { moduleLineStartsOf, } from './module-line-starts.ts';
export {
  callParseEndHook,
  PARSE_END_HOOK_KEY,
} from './parse-end-hook.ts';
export {
  MarkdownParseError,
  MdxParseError,
  parseMarkdownBody,
  parseMdxBody,
  requireMarkdownRefusal,
  requireMdxRefusal,
  withMarkdownExhaustionRefused,
  withMdxExhaustionRefused,
} from './parse-mdx.ts';
export { parseMarkdownMathBody, } from './parse-markdown-math.ts';
export { parseSliceBody, } from './parse-slice-body.ts';
export {
  type AtomKind,
  type ProtectedAtom,
  scanTextAtoms,
} from './protected-atom.ts';
export { type DeepReadonlyData, } from './readonly-data.ts';
export {
  namesWithoutQuoting,
  type NamingError,
  refusalText,
} from './refusal-text.ts';
export {
  type BlockRange,
  footnoteDefinitionRangesOf,
  pageFootnoteDefinitionRangesOf,
  tableRangesOf,
} from './gfm-block-ranges.ts';
export {
  archiveNotesMissingFrom,
  type TranslatorsNote,
  translatorsNotesIn,
  withoutTranslatorsNotes,
} from './translators-note.ts';
export {
  type KeptStretch,
  type ShownTranslation,
  shownTranslation,
  type StoredStretch,
  storedStretchesOf,
} from './translators-note-view.ts';
export { UnhandledMemberInvariantError, } from './unhandled-member.ts';
export { withoutRawHtml, } from './without-raw-html.ts';
export {
  type ContainerSpan,
  flattenContainers,
  UnpositionedContainerError,
} from './unwrap-container.ts';

//endregion Public index
