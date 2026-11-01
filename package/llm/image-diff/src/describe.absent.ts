/**
 Shared sentinel for an absent image-difference description.
 
 The describe chain (native Gemini, then OpenRouter fallback) yields a
 description string when a backend API key is configured, or this sentinel
 when no key is available so callers can skip the description gracefully
 instead of widening to a banned nullish union.
 
 @module
 */

/**
 Marks a missing description or a missing backend API key.
 
 A unique `Symbol` keeps the absent case out of a nullish union; consumers
 narrow with `value !== ABSENT` before treating the value as a string.
 
 @example
 ```ts
 const description = await describeImageDifference({ imageA, imageB });
 if (description !== ABSENT) console.log(description);
 ```
 */
export const ABSENT: unique symbol = Symbol('image difference description absent because no backend key configured',);
