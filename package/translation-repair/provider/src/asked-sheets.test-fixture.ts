import {
  type ChatJsonRequest,
  messageText,
} from '../dist/final/node/index.mjs';

//region Asked sheets
// WHICH STAGE WAS ASKED WHAT, for cases that script a whole refinement flow
// and read which stage's sheet carried a block.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. A case that looks for a handed-in
// sentence on "some sheet" cannot say which stage carried it, and one that
// looks for the absence of a sentence nobody handed in cannot fail. These
// record each exchange under the stage that asked it, so a case names the
// stages whose sheet carries a block's heading and asserts that list whole.

/**
 One exchange a scripted client was asked.

 @example
 ```ts
 const exchange: AskedSheet = { stage: 'refine_report', sheet: 'ORIGINAL ...', };
 ```
 */
export type AskedSheet = {
  /**
   Stage that asked, by the name of its structured-output constraint.
   */
  readonly stage: string;

  /**
   Sheet the stage composed, which is the exchange's last message. The rules
   in the system message are left out: they name a block's heading whether or
   not the sheet prints the block.
   */
  readonly sheet: string;
};

/**
 Heading a sheet prints over the declared names, when any were handed in.
 */
export const DECLARED_NAMES_HEADING = 'DECLARED NAMES';

/**
 Heading a sheet prints over the cited references, when any were handed in.
 */
export const CITED_REFERENCES_HEADING = 'CITED REFERENCES, EVIDENCE ONLY';

/**
 Reads the stage and the sheet off one exchange a scripted client is asked.

 @param request - exchange as the stage sent it

 @returns Stage name and sheet text

 @throws {@link Error} when the exchange carries no message

 @example
 ```ts
 asked.push(askedSheetOf({ request, },),);
 ```
 */
export function askedSheetOf<ValueT,>(
  { request, }: { readonly request: ChatJsonRequest<ValueT>; },
): AskedSheet {
  /**
   Last message of the exchange, which is the sheet the stage composed.
   */
  const last = request.messages
    .at(-1,);
  if (last === undefined)
    throw new Error('the stage asked an exchange that carries no message',);
  return {
    stage: request.responseFormat
      ?.json_schema
      .name
      ?? '',
    sheet: messageText({ message: last, },),
  };
}

/**
 Stage of every exchange, in the order the flow asked.

 @param asked - exchanges a scripted client recorded

 @returns One stage name per exchange

 @example
 ```ts
 const stages = stagesAsked({ asked, },);
 ```
 */
export function stagesAsked(
  { asked, }: { readonly asked: readonly AskedSheet[]; },
): readonly string[] {
  return asked.map(function toStage(exchange,): string {
    return exchange.stage;
  },);
}

/**
 Stages whose sheet carries a text, one entry per exchange in the order the
 flow asked, so a case reads which stage's sheet carried what.

 @param asked - exchanges a scripted client recorded

 @param text - heading or handed-in wording to look for

 @returns Stage of every exchange whose sheet includes the text

 @example
 ```ts
 const stages = stagesCarrying({ asked, text: DECLARED_NAMES_HEADING, },);
 ```
 */
export function stagesCarrying(
  {
    asked,
    text,
  }: {
    readonly asked: readonly AskedSheet[];
    readonly text: string;
  },
): readonly string[] {
  return stagesAsked({
    asked: asked.filter(function carries(exchange,): boolean {
      return exchange.sheet
        .includes(text,);
    },),
  },);
}

//endregion Asked sheets
