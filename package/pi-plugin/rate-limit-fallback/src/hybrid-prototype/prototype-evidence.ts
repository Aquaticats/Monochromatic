/**
 The prototype's own evidence stream: one JSON record per decision, written to stderr beside Pi's.
 @module
 */

/**
 Prefix every evidence line starts with, so a capture keeps Pi's own diagnostics out of the way.
 */
export const EVIDENCE_PREFIX = '[hybrid-prototype]';

/**
 Characters of a provider message an evidence line keeps, enough to recognize a refusal.
 */
export const EVIDENCE_TEXT_CHARS = 200;

/**
 Model identity and limits, read-only, as the prototype reads them.

 Project-owned so a catalog model with writable fields is assignable to it and no function here
 claims ownership of a Pi value.
 */
export type CatalogLimitsView = {
  readonly provider: string;
  readonly id: string;
  readonly contextWindow: number;
  readonly maxTokens: number
};

/**
 Record one prototype decision on stderr.

 The session capture redirects stderr into the run folder, so the record sits beside the session's
 own event stream. Records name models, limits and router reasons; they never carry credentials or
 message bodies beyond a refusal excerpt.

 @param record - fields to serialize as one line

 @example
 ```ts
 note({ event: 'route', reason: 'user' });
 ```
 */
export function note(record: Readonly<Record<string, unknown>>,): void {
  process.stderr
    .write(`${EVIDENCE_PREFIX} ${JSON.stringify(record)}\n`,);
}

/**
 Describe the limit pair Pi would apply for one model, without the model's other fields.

 @param model - catalog model the prototype reads

 @returns provider, id and both limits

 @example
 ```ts
 limitsOf({ model: registry.find('rate-limit-fallback', 'writer') });
 ```
 */
export function limitsOf({
  model,
}: {
  readonly model: CatalogLimitsView;
},): Readonly<Record<string, unknown>> {
  return {
    model: `${model.provider}/${model.id}`,
    contextWindow: model.contextWindow,
    maxTokens: model.maxTokens,
  };
}

/**
 Keep the start of a provider message for an evidence line.

 @param text - provider message to shorten

 @returns the first {@link EVIDENCE_TEXT_CHARS} characters

 @example
 ```ts
 excerpt('429: {"type":"rate_limit_error"}');
 ```
 */
export function excerpt(text: string,): string {
  return text.slice(
    0,
    EVIDENCE_TEXT_CHARS,
  );
}
