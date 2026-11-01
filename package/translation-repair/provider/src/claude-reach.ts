import type {
  AnthropicServedId,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { CLAUDE_MODELS, } from './claude-catalog.ts';

//region Claude reach
// WHERE ONE ROSTER MODEL CAN BE REACHED ON THE OWNER'S CLAUDE KEY: the eight Claude cards, read off the catalog,
// which is read off the cards. No withholding set, as for the other direct routes: the cards' own holds decide which
// roles a card fills, and this file decides only whether a call already addressed to the model can go to the route.

/**
 Where one roster model can be reached on the Claude key.

 @example
 ```ts
 const served = claudeIdFor({ modelId: 'claude-haiku-5-5', },);
 ```
 */
export type ClaudeSpelling =
  | {
    /**
     Discriminator marking a model this route serves.
     */
    readonly served: true;

    /**
     Card spelling of the model, which names its wire model and effort in the catalog.
     */
    readonly id: AnthropicServedId;
  }
    | {
    /**
     Discriminator marking a model this route does not serve.
     */
      readonly served: false;
    };

/**
 How the Claude route spells one roster model, where it serves it at all.

 @param modelId - roster model to look up

 @returns Card spelling, or that this route does not serve it

 @example
 ```ts
 const spelling = claudeIdFor({ modelId, },);
 ```
 */
export function claudeIdFor({ modelId, }: { readonly modelId: RosterModelId; },): ClaudeSpelling {
  /**
   Entry standing in for this roster seat, if this route has one.
   */
  const entry = Object
    .values(CLAUDE_MODELS,)
    .find(function serves(info,): boolean {
      return info.sharedWith === modelId;
    },);
  if (entry === undefined)
    return { served: false, };
  return {
    served: true,
    id: entry.id,
  };
}

/**
 Whether the Claude route will show one roster model a picture.

 @param modelId - roster model to look up

 @returns Whether this route serves it AND its card reports image input

 @example
 ```ts
 const shows = claudeShowsPictures({ modelId, },);
 ```
 */
export function claudeShowsPictures({ modelId, }: { readonly modelId: RosterModelId; },): boolean {
  /**
   The route's spelling, which is also whether it serves this model at all.
   */
  const spelling = claudeIdFor({ modelId, },);
  if (!spelling.served)
    return false;
  return CLAUDE_MODELS[spelling.id]
    .readsImages;
}

//endregion Claude reach
