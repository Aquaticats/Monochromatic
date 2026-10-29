//region Gemini API types

/**
 Inline image data for the Gemini embedContent API.
 */
export type GeminiInlineData = {
  /**
   MIME type (e.g. `image/png`, `image/jpeg`).
   */
  readonly mime_type: string;
  /**
   Raw base64-encoded image data (no data URI prefix).
   */
  readonly data: string;
};

/**
 A content part for the Gemini API; either inline image data or text.
 */
export type GeminiPart = {
  readonly inline_data: GeminiInlineData;
};

/**
 Request body for the Gemini embedContent API.
 The `model` field is optional for single-embed (inferred from URL path)
 but required in each entry of a batchEmbedContents request.
 */
export type GeminiEmbedContentRequest = {
  readonly model?: string;
  readonly content: {
    readonly parts: readonly GeminiPart[];
  };
};

/**
 Response from the Gemini embedContent API.
 */
export type GeminiEmbedContentResponse = {
  readonly embedding: {
    readonly values: readonly number[];
  };
};

/**
 Request body for the Gemini batchEmbedContents API.
 */
export type GeminiBatchEmbedRequest = {
  readonly requests: readonly GeminiEmbedContentRequest[];
};

/**
 Response from the Gemini batchEmbedContents API.
 */
export type GeminiBatchEmbedResponse = {
  readonly embeddings: readonly {
    readonly values: readonly number[];
  }[];
};

//endregion Gemini API types
