import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { CoverageControlCase, } from '../../dist/final/node/index.mjs';

//region Coverage control cases and client
// ONE DAMAGEABLE CASE AT A NAMED LOCATION, and a client answering every
// coverage round with full coverage and a quote really present in the
// translation, refusing to answer any other structured-output stage.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The coverage-control-cap and
// coverage-control-vote-change tests kept their own copy of this case
// builder and client; both now import them from here.

/**
 Name the coverage control's structured-output constraint carries.
 */
const COVERAGE_REPORT_STAGE = 'coverage_report';

/**
 Builds one damageable case, naming where it sits.

 @param where - case's own location label

 @param sourcePassage - source text the case asks about

 @param translationText - translation text the case parses

 @returns Case shaped as the coverage control accepts one

 @example
 ```ts
 const one = coverageControlCaseAt({ where: 'slice-0', sourcePassage: SOURCE_PASSAGE, translationText: TRANSLATION, },);
 ```
 */
function coverageControlCaseAt(
  {
    where,
    sourcePassage,
    translationText,
  }: {
    readonly where: string;
    readonly sourcePassage: string;
    readonly translationText: string;
  },
): CoverageControlCase {
  return {
    where,
    sourcePassage,
    translation: parseDocument({ text: translationText, },),
  };
}

/**
 Builds every damageable case named, all parsed from the same translation.

 @param where - location label per case, in the order offered

 @param sourcePassage - source text every case asks about

 @param translationText - translation text every case parses

 @returns Cases shaped as the coverage control accepts them

 @example
 ```ts
 const cases = coverageControlCasesAt({ where: WHERE, sourcePassage: SOURCE_PASSAGE, translationText: TRANSLATION, },);
 ```
 */
export function coverageControlCasesAt(
  {
    where,
    sourcePassage,
    translationText,
  }: {
    readonly where: readonly string[];
    readonly sourcePassage: string;
    readonly translationText: string;
  },
): readonly CoverageControlCase[] {
  return where.map(function toCase(oneWhere,): CoverageControlCase {
    return coverageControlCaseAt({
      where: oneWhere,
      sourcePassage,
      translationText,
    },);
  },);
}

/**
 Refuses a text call, which the coverage control never makes.

 @throws Error on any invocation
 */
function unexpectedCoverageText(): never {
  throw new Error('chatText unused by the coverage control',);
}

/**
 Refuses a quota read, which the coverage control never makes.

 @throws Error on any invocation
 */
function unexpectedCoverageQuotas(): never {
  throw new Error('quotas unused by the coverage control',);
}

/**
 Client answering every coverage round with full coverage and the given
 quote.

 @param quote - quoted text every scripted reply carries, real in the translation

 @returns Client honoring that script

 @example
 ```ts
 const client = coverageControlClient({ quote: QUOTED, },);
 ```
 */
export function coverageControlClient(
  { quote, }: { readonly quote: string; },
): ModelClient {
  return {
    chatText: unexpectedCoverageText,
    chatJson: function chatJson<ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> {
      /**
       Stage name from the structured-output constraint.
       */
      const stage = request.responseFormat
        ?.json_schema
        .name
        ?? '';
      if (stage !== COVERAGE_REPORT_STAGE)
        return Promise.reject(
          new Error(`the coverage control asks about coverage and nothing else, and this asked ${stage}`,),
        );

      /**
       Reply claiming the passage is carried, quoting text really present.
       */
      const scripted: unknown = {
        coverage: 'full',
        copiedSpan: quote,
        assessment: 'fixture',
      };
      if (!request.validate(scripted,))
        return Promise.reject(new Error('scripted reply failed the coverage guard',),);
      return Promise.resolve({
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      },);
    },
    quotas: unexpectedCoverageQuotas,
  };
}

//endregion Coverage control cases and client
