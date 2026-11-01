//region Public index
// The names `@monochromatic-dev/translation-repair-rendering-audit` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export {
  readSignature,
  type Signature,
  signaturesOf,
} from './corpus-run/attribution-line.ts';
export {
  commentBody,
  commentNoteLines,
  entryNoteLines,
  foldedLine,
  footnoteNoteLines,
  headingWords,
  type NoteSide,
} from './entry-notes.ts';
export { pageIdentityLines, } from './page-identity-lines.ts';
export {
  type Link,
  linksOf,
  pageNameLines,
  pairedPageNames,
} from './page-name-glossary.ts';
export { renderedLinks, } from './page-name-link-rendering.ts';
export { type ProseNamePair, } from './prose-name-han.ts';
export {
  proseNamePairs,
  respelledRendering,
} from './prose-name-pairs.ts';
export {
  type RepeatedTitleSpan,
  repeatedTitleSpans,
} from './page-title-spans.ts';
export {
  type AnchoredSpan,
  anchorLocatedSpan,
  type RenderingAuditSpanAnchor,
} from './rendering-audit-anchor.ts';
export {
  type AuditMemberClaim,
  corroborate,
  corroborateByOverlap,
  type CorroboratedDefect,
  CORROBORATION_VOICES,
  type NearMiss,
  nearMisses,
  type OverlapAgreement,
} from './rendering-audit-corroborate.ts';
export { RenderingAuditInvariantError, } from './rendering-audit-invariant.ts';
export { buildRenderingAuditMessages, } from './rendering-audit-prompt.ts';
export {
  quotesRequired,
  type ScreenedFinding,
  type ScreenedReport,
  screenRenderingAudit,
  type SideReading,
} from './rendering-audit-screen.ts';
export {
  CANDIDATE_ONLY_CATEGORIES,
  FINDING_FIELDS,
  isRenderingAuditReportWire,
  PAIRED_CATEGORIES,
  RENDERING_AUDIT_CATEGORIES,
  RENDERING_AUDIT_RESPONSE_FORMAT,
  RENDERING_AUDIT_VERDICTS,
  type RenderingAuditCategory,
  type RenderingAuditFindingWire,
  type RenderingAuditReportWire,
  type RenderingAuditSubject,
  type RenderingAuditVerdict,
  SOURCE_ONLY_CATEGORIES,
} from './rendering-audit-wire.ts';
export {
  type AuditVoiceRow,
  type RenderingAuditReport,
  runRenderingAudit,
} from './rendering-audit.ts';
export {
  RENDERING_GLOSSARY,
  renderingTermLines,
} from './rendering-glossary.ts';

//endregion Public index
