import type { VoyageModel, } from './types.ts';

//region Voyage API types

/**
 Shape of the Voyage AI multimodal embeddings API response.
 */
export type VoyageApiResponse = {
  readonly object: 'list';
  readonly data: readonly {
    readonly object: 'embedding';
    readonly embedding: readonly number[];
    readonly index: number;
  }[];
  readonly model: string;
  readonly usage: {
    readonly text_tokens: number;
    readonly image_pixels: number;
    readonly video_pixels: number;
    readonly total_tokens: number;
  };
};

/**
 Content item for the Voyage AI multimodal embeddings API request.
 */
export type VoyageContentItem =
  | {
    readonly type: 'image_url';
    readonly image_url: string;
  }
    | {
      readonly type: 'image_base64';
      readonly image_base64: string;
    };

/**
 Single input entry for the Voyage AI multimodal embeddings API request.
 */
export type VoyageInput = {
  readonly content: readonly VoyageContentItem[];
};

/**
 Request body for the Voyage AI multimodal embeddings API.
 */
export type VoyageApiRequest = {
  readonly inputs: readonly VoyageInput[];
  readonly model: VoyageModel;
  readonly input_type: 'document';
  readonly truncation: boolean;
};

//endregion Voyage API types
