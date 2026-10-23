import type { ChatMessage, } from '@monochromatic-dev/module-llm-type/ts';
import type { JsonSchemaResponseFormat, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  buildBlockPairingMessages,
  type NumberedBlock,
} from '@monochromatic-dev/translation-repair-stage/ts';

//region Block-pairing protocol
// The messages and schema the pairing stage sends, built without a client so tests can read them.

/**
 Structured response contract the pairing stage asks for.
 Each call receives its own copy, so a caller that mutates one cannot change the next question.
 */
const PAIRING_RESPONSE_FORMAT: JsonSchemaResponseFormat = {
  type: 'json_schema',
  json_schema: {
    name: 'block_pairing',
    schema: {
      type: 'object',
      properties: {
        pairs: {
          type: 'array',
          items: {
            type: 'object',
            properties: {
              source: { type: 'integer', },
              target: { type: 'integer', },
            },
            required: [
              'source',
              'target',
            ],
            additionalProperties: false,
          },
        },
      },
      required: ['pairs',],
      additionalProperties: false,
    },
  },
};

/**
 Messages and response format for one pairing question, the same whichever model is asked.
 Which models are asked, and each provider's request body, stay with the stage and the client.

 @example
 ```ts
 const protocol = blockPairingProtocol({ sourceBlocks, targetBlocks, });
 ```
 */
export type BlockPairingProtocol = {
  /**
   Ordered messages, whose listing fence is longer than any run of `=` in the blocks.
   */
  readonly messages: readonly ChatMessage[];
  /**
   Schema name and shape, sent without `strict`; every object is closed (`additionalProperties: false`), which the
   Claude route requires and the other routes accept.
   */
  readonly responseFormat: JsonSchemaResponseFormat;
};

/**
 Constructs the pairing question's messages and response format for `pairBlocksWithRoster`.
 Definition-order exemptions affect how replies are read, not the messages sent.

 @param sourceBlocks - current original blocks in their emitted numbering

 @param targetBlocks - current archive blocks in their emitted numbering

 @param pictureContext - transcripts of the section's pictures for the sheet, absent when it shows none

 @returns Messages and a schema copy the caller owns; no client is created and nothing is bought

 @example
 ```ts
 const { messages, responseFormat, } = blockPairingProtocol({ sourceBlocks, targetBlocks, });
 ```
 */
export function blockPairingProtocol({
  sourceBlocks,
  targetBlocks,
  pictureContext,
}: {
  readonly sourceBlocks: readonly NumberedBlock[];
  readonly targetBlocks: readonly NumberedBlock[];
  readonly pictureContext?: string;
},): BlockPairingProtocol {
  return {
    messages: buildBlockPairingMessages({
      sourceBlocks,
      targetBlocks,
      ...((pictureContext === undefined) ? {} : { pictureContext, }),
    },),
    responseFormat: structuredClone(PAIRING_RESPONSE_FORMAT,),
  };
}

//endregion Block-pairing protocol
