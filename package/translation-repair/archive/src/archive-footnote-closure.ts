import type { FootnoteRelabel, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  type FootnoteLabelRewrite,
  readFootnoteClosureInput,
  type RelabelClosure,
  retainedFootnoteLabels,
} from '@monochromatic-dev/translation-repair-page-pass/ts';
import {
  activeFootnoteMarkers,
  footnoteMarkerLabels,
  type ReadonlyEntry,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  normalizeFootnoteIdentifier,
  relabelsFootnote,
} from '@monochromatic-dev/translation-repair-document-structure/ts';


//region Archive footnote relabel closure
// Correspondences follow the original. An unmatched archive note keeps its own identity under a fresh label.
// Existing one-pair elimination stays distinct from collision avoidance and never gains extra votes.

/**
 Distinct labels a text carries, references and definition openers alike,
 in order of first appearance.

 @param text - document to read

 @returns Labels, each once

 @example
 ```ts
 documentLabels({ text: 'A[^2].\n\n[^1]: one\n\n[^2]: two\n' });
 ```
 */
export function documentLabels({ text, }: { readonly text: string; },): readonly string[] {
  return footnoteMarkerLabels({ markers: activeFootnoteMarkers({ text, },), },);
}

/**
 Builds an injective simultaneous label rewrite without inventing source correspondence for archive apparatus.
 The existing one-remaining-label elimination is allowed. Fresh displacement is allowed only when
 every original label is accounted for and destination occupancy is the remaining obstacle.

 INJECTIVE BY CONSTRUCTION, so no check of it runs here (ledger T8, eighteenth
 batch, which removed one no input reached). The supplied relations are
 injective (`readFootnoteClosureInput` refuses any that are not); the
 eliminated pair joins an archive label and an original label no relation
 names; each displaced label takes a fresh label outside both namespaces,
 none twice; and an archive label nothing moves is no relation's destination,
 or it would have been displaced. `archive-footnote-closure-injective.unit.test.ts`
 closes every map over every pair of small label sets and reads each result's
 destinations distinct, and applying a map that would merge identifiers is
 refused where it is applied (`footnote-rewrite-map.ts`, kind `collision`).

 @param map - supplied positive correspondences, including identity relations when known

 @param archiveLabels - complete archive identifier universe

 @param originalLabels - complete original identifier universe

 @returns Closed operational map with separate provenance, or an explicit refusal

 @example
 ```ts
 const closure = closeFootnoteRelabel({ map: [{ from: '2', to: '1' }], archiveLabels: ['1', '2'], originalLabels: ['2', '1'] });
 ```
 */
export function closeFootnoteRelabel(
  {
    map,
    archiveLabels,
    originalLabels,
  }: {
    readonly map: readonly FootnoteRelabel[];
    readonly archiveLabels: readonly string[];
    readonly originalLabels: readonly string[];
  },
): RelabelClosure {
  /**
   Membership and injectivity checked under the parser's identifier normalization.
   */
  const input = readFootnoteClosureInput({
    map,
    archiveLabels,
    originalLabels,
  },);
  if (input.kind === 'open')
    return input;
  /**
   Distinct supplied relations, including identities that establish source coverage.
   */
  const { correspondences, } = input;
  /**
   Archive identities already placed by supplied evidence.
   */
  const suppliedFrom = new Set(correspondences.map(function archiveKey(relation,): string {
    return normalizeFootnoteIdentifier({ identifier: relation.from, },);
  },),);
  /**
   Original identities already accounted for by supplied evidence.
   */
  const suppliedTo = new Set(correspondences.map(function originalKey(relation,): string {
    return normalizeFootnoteIdentifier({ identifier: relation.to, },);
  },),);
  /**
   Unmapped archive identities retain their first observed spellings.
   */
  const archiveLeft = [...input.archive
    .entries(),].filter(function unplaced([key,]: ReadonlyEntry<string, string>,): boolean {
      return !suppliedFrom.has(key,);
    },);
  /**
   Original identities still needing correspondence.
   */
  const originalLeft = [...input.original
    .entries(),].filter(function unaccounted([key,]: ReadonlyEntry<string, string>,): boolean {
      return !suppliedTo.has(key,);
    },);
  /**
   The sole remaining archive candidate, when one exists.
   */
  const [archiveOnly,] = archiveLeft;
  /**
   The sole remaining original candidate, when one exists.
   */
  const [originalOnly,] = originalLeft;
  /**
   Existing forced move; equal unmapped spellings do not invent identity correspondence.
   */
  const eliminated: readonly FootnoteRelabel[] = ((archiveLeft.length === 1) && (originalLeft.length === 1)
    && (archiveOnly !== undefined)
    && (originalOnly !== undefined)
    && (archiveOnly[0] !== originalOnly[0]))
    ? [{
      from: archiveOnly[1],
      to: originalOnly[1],
    },]
    : [];
  /**
   Positive relations after the existing forced elimination, never after displacement.
   */
  const established = [
    ...correspondences,
    ...eliminated,
  ];
  /**
   Archive identities accounted for by correspondence or elimination.
   */
  const establishedFrom = new Set(established.map(function archiveKey(relation,): string {
    return normalizeFootnoteIdentifier({ identifier: relation.from, },);
  },),);
  /**
   Original identities accounted for by those same relations.
   */
  const establishedTo = new Set(established.map(function originalKey(relation,): string {
    return normalizeFootnoteIdentifier({ identifier: relation.to, },);
  },),);
  /**
   Occupied destinations that no positive relation moves away, in archive order.
   */
  const collisions = [...input.archive
    .entries(),]
    .filter(function occupied([key,]: ReadonlyEntry<string, string>,): boolean {
      return establishedTo.has(key,) && (!establishedFrom.has(key,));
    },)
    .map(function spelling([, label,]: ReadonlyEntry<string, string>,): string { return label; },);
  /**
   Source identities not explained by any correspondence or existing elimination.
   */
  const unaccounted = [...input.original
    .keys(),].filter(function unknown(key,): boolean {
      return !establishedTo.has(key,);
    },);
  if ((collisions.length > 0) && (unaccounted.length > 0))
    return {
      kind: 'open',
      detail: `the map lands on ${collisions.map(function marker(label,): string { return `[^${label}]`; },)
        .join(', ',)} while original correspondence remains incomplete (${String(unaccounted.length,)} original labels unaccounted for)`,
    };
  /**
   Both logical namespaces are reserved, including labels whose current spelling will move.
   */
  const reserved = new Set([
    ...input.archive
      .keys(),
    ...input.original
      .keys(),
  ]);
  /**
   Operational displacements, deliberately excluded from source correspondence evidence.
   */
  const retained = retainedFootnoteLabels({
    labels: collisions,
    reserved,
  },);
  /**
   Actual simultaneous rewrites; identity relations carry evidence but change no bytes.
   */
  const rewrites: readonly FootnoteLabelRewrite[] = [
    ...established.filter(function changes(relation,): boolean {
      return relabelsFootnote({ relation, },);
    },),
    ...retained.map(function displaced(move,): FootnoteLabelRewrite {
      return {
        from: move.from,
        to: move.retainedAs,
      };
    },),
  ];
  return {
    kind: 'closed',
    map: rewrites,
    correspondences,
    eliminated,
    retained,
  };
}

//endregion Archive footnote relabel closure
