/**
 AI autofill API handler.
 
 POST /api/ai/autofill
 Accepts `\{ title: string \}` and returns inferred metadata (tags, locations,
 priority, complexity) using the configured chat completions endpoint.
 
 Degrades gracefully: when the AI is unavailable or returns garbage,
 the response carries empty/null fields so the client can still function.
 
 Exceeds 100 lines: borderline at ~105 lines, but `parseAutofillResponse`,
 `listAllLocations`, and `handleAutofill` share the same types and validation
 constants; splitting would add a module for fewer than 40 lines of code.
 */
import { chatCompletion, } from '../../lib/ai/client.ts';
import { buildAutofillMessages, } from '../../lib/ai/prompts.ts';
import db from '../../lib/db.ts';
import { listAllTags, } from '../../lib/db/tasks-queries.ts';
import {
  TASK_COMPLEXITIES,
  TASK_PRIORITIES,
  type TaskComplexity,
  type TaskPriority,
} from '../../lib/types.ts';

//region Types

/**
 Result of AI-powered metadata inference for a task title.
 */
type AutofillResult = {
  /**
   Suggested tags.
   */
  tags: string[];
  /**
   Suggested locations.
   */
  locations: string[];
  /**
   Suggested priority level; absent when none was inferred.
   */
  priority?: TaskPriority;
  /**
   Suggested complexity level; absent when none was inferred.
   */
  complexity?: TaskComplexity;
};

/**
 Raw shape of the AI response before validation.
 */
type RawAutofillResponse = {
  /**
   Possibly-valid tags array.
   */
  tags?: unknown;
  /**
   Possibly-valid locations array.
   */
  locations?: unknown;
  /**
   Possibly-valid priority string.
   */
  priority?: unknown;
  /**
   Possibly-valid complexity string.
   */
  complexity?: unknown;
};

//endregion Types

//region Validation

/**
 Set of valid priority values for input validation.
 */
const VALID_PRIORITIES = new Set<string>(TASK_PRIORITIES,);

/**
 Set of valid complexity values for input validation.
 */
const VALID_COMPLEXITIES = new Set<string>(TASK_COMPLEXITIES,);

/**
 Maximum tokens for AI autofill response.
 */
const MAX_TOKENS = 256;

/**
 Best-effort extraction of an autofill result from possibly malformed AI output.
 
 @param raw - Raw JSON string from the AI completion
 
 @returns Validated autofill result with safe defaults
 */
function parseAutofillResponse(raw: string,): AutofillResult {
  /**
   Safe default returned whenever parsing or validation fails.
   */
  const empty: AutofillResult = {
    tags: [],
    locations: [],
  };

  try {
    /* oxlint-disable typescript/no-unsafe-type-assertion -- JSON.parse returns unknown; shape validated below */
    /**
     Raw object asserted to the loose shape; every field is rechecked individually.
     */
    const parsed = JSON.parse(raw,) as RawAutofillResponse;
    /* oxlint-enable typescript/no-unsafe-type-assertion */
    if ((typeof parsed) !== 'object')
      return empty;

    /**
     String-only tag list filtered defensively against malformed AI output.
     */
    const tags = Array.isArray(parsed.tags,)
      ? parsed.tags
        .filter(function isString(tag,): tag is string {
          return (typeof tag) === 'string';
        },)
      : [];

    /**
     String-only location list filtered defensively against malformed AI output.
     */
    const locations = Array.isArray(parsed.locations,)
      ? parsed.locations
        .filter(function isString(location,): location is string {
          return (typeof location) === 'string';
        },)
      : [];

    /* oxlint-disable typescript/no-unsafe-type-assertion -- validated by Set.has check */
    /**
     Priority field, included only when the response carries a recognised value.
     */
    const priorityField: { priority?: TaskPriority; } = ((typeof parsed
      .priority) === 'string') && VALID_PRIORITIES
        .has(parsed.priority,)
      ? { priority: parsed.priority as TaskPriority, }
      : {};

    /**
     Complexity field, included only when the response carries a recognised value.
     */
    const complexityField: { complexity?: TaskComplexity; } = ((typeof parsed
      .complexity) === 'string') && VALID_COMPLEXITIES
        .has(parsed.complexity,)
      ? { complexity: parsed.complexity as TaskComplexity, }
      : {};
    /* oxlint-enable typescript/no-unsafe-type-assertion */

    return {
      tags,
      locations,
      ...priorityField,
      ...complexityField,
    };
  }
  catch (autofillParseError: unknown) {
    // AI output was not valid JSON; log the cause and return the safe empty default.
    console.error(
      'parseAutofillResponse could not parse AI JSON output; using empty result:',
      autofillParseError,
    );
    return empty;
  }
}

//endregion Validation

//region Existing metadata for consistency hints

/**
 Collects unique locations across all tasks via a full scan.
 
 @returns Sorted array of unique location strings
 */
async function listAllLocations(): Promise<string[]> {
  /* oxlint-disable typescript/no-unsafe-type-assertion -- database query returns rows with loc column */
  /**
   Single-column projection; the location string is unwrapped from each row below.
   */
  const rows = (await (await db
    .prepare(
    'SELECT DISTINCT loc.value AS loc FROM tasks, json_each(tasks.locations) AS loc ORDER BY loc.value ASC',
    ))
    .all()) as { readonly loc: string; }[];
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  return rows.map(function extractLoc(row,) {
    return row.loc;
  },);
}

//endregion Existing metadata

//region Handler

/**
 POST /api/ai/autofill; infers task metadata from a title using AI.
 Gathers existing tags via {@link listAllTags} and locations via
 {@link listAllLocations}, composes the prompt with {@link buildAutofillMessages},
 sends it through {@link chatCompletion}, and validates the result with
 {@link parseAutofillResponse}.
 
 @param req - Incoming request with JSON body containing `title`
 
 @returns JSON response with inferred tags, locations, priority, and complexity
 
 @example
 ```ts
 const response = await handleAutofill(request);
 ```
 */
export async function handleAutofill(req: Request,): Promise<Response> {
  try {
    /* oxlint-disable typescript/no-unsafe-type-assertion -- request body is expected to be a JSON object */
    /**
     Loose object so each field can be validated individually before use.
     */
    const body = (await req.json()) as Record<string, unknown>;
    /* oxlint-enable typescript/no-unsafe-type-assertion */
    /**
     Trimmed title; empty string short-circuits to the empty-response branch below.
     */
    const title = (typeof body.title) === 'string' ? body.title
      .trim() : '';

    if (title.length
      === 0) {
      return Response.json({
        tags: [],
        locations: [],
      },);
    }

    /**
     Tags already in use, forwarded so the model prefers consistency.
     */
    const existingTags = await listAllTags();
    /**
     Locations already in use, forwarded so the model prefers consistency.
     */
    const existingLocations = await listAllLocations();
    /**
     Composed chat messages ready for the completion endpoint.
     */
    const messages = buildAutofillMessages({
      title,
      existingTags,
      existingLocations,
    },);

    /**
     Discriminated result wrapping the model output or a transport error.
     */
    const result = await chatCompletion({
      messages,
      temperature: 0,
      jsonMode: true,
      maxTokens: MAX_TOKENS,
    },);

    if (!result.ok) {
      console.error(
        'AI autofill failed:',
        result.error,
      );
      return Response.json({
        tags: [],
        locations: [],
      },);
    }

    /**
     Validated metadata; serialised straight to the response body below.
     */
    const autofill = parseAutofillResponse(result.content,);
    return Response.json(autofill,);
  }
  catch (error) {
    console.error(
      'AI autofill handler error:',
      error,
    );
    return Response.json({
      tags: [],
      locations: [],
    },);
  }
}

//endregion Handler
