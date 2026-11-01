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
import { listAllTags, } from '../../lib/db/tasks.ts';
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
   Safe default returned on any parse or validation failure.
   */
  const empty: AutofillResult = {
    tags: [],
    locations: [],
  };

  try {
    /* oxlint-disable typescript/no-unsafe-type-assertion -- JSON.parse returns unknown; shape validated below */
    /**
     Raw parsed payload narrowed field-by-field in the validation below.
     */
    const parsed = JSON.parse(raw,) as RawAutofillResponse;
    /* oxlint-enable typescript/no-unsafe-type-assertion */
    if ((typeof parsed) !== 'object')
      return empty;

    /**
     Validated string-only tag list from the response.
     */
    const tags = Array.isArray(parsed.tags,)
      ? parsed.tags
        .filter(function isString(tag,): tag is string {
          return (typeof tag) === 'string';
        },)
      : [];

    /**
     Validated string-only location list from the response.
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
    const priorityField: { priority?: TaskPriority; } = ((typeof parsed.priority) === 'string')
      && VALID_PRIORITIES
        .has(parsed.priority,)
      ? { priority: parsed.priority as TaskPriority, }
      : {};
    /* oxlint-enable typescript/no-unsafe-type-assertion */

    /* oxlint-disable typescript/no-unsafe-type-assertion -- validated by Set.has check */
    /**
     Complexity field, included only when the response carries a recognised value.
     */
    const complexityField: { complexity?: TaskComplexity; } = ((typeof parsed.complexity) === 'string')
      && VALID_COMPLEXITIES
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
  catch (error) {
    console.error(
      'AI autofill response JSON parse failed:',
      error,
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
   Distinct location strings extracted from `tasks.locations` JSON arrays.
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
 
 @param req - Incoming request with JSON body containing `title`
 
 @returns JSON response with inferred tags, locations, priority, and complexity
 
 @example
 ```ts
 const response = await handleAutofill(event.req);
 ```
 */
export async function handleAutofill(req: Request,): Promise<Response> {
  try {
    /* oxlint-disable typescript/no-unsafe-type-assertion -- request body is expected to be a JSON object */
    /**
     Parsed JSON body; field types are validated below before use.
     */
    const body = (await req.json()) as Record<string, unknown>;
    /* oxlint-enable typescript/no-unsafe-type-assertion */
    /**
     Trimmed title from the body; empty-string fallback yields the empty result below.
     */
    const title = ((typeof body.title) === 'string') ? body.title
      .trim() : '';

    if (title.length
      === 0) {
      return Response.json({
        tags: [],
        locations: [],
      },);
    }

    /**
     Pre-existing tags supplied as consistency hints to the AI prompt.
     */
    const existingTags = await listAllTags();
    /**
     Pre-existing locations supplied as consistency hints to the AI prompt.
     */
    const existingLocations = await listAllLocations();
    /**
     Final chat-completion messages with the title and the existing-metadata hints.
     */
    const messages = buildAutofillMessages({
      title,
      existingTags,
      existingLocations,
    },);

    /**
     AI completion outcome; `ok: false` triggers the empty-payload degraded response.
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
     Validated metadata produced by the parser; returned directly as the JSON response.
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
