/**
 Prompt templates for AI-powered task features.
 
 All prompts follow a strict pattern:
 - System message contains instructions and schema
 - User data is always delimited inside XML-style tags
 - Output is constrained to JSON so we can validate with a schema
 */
import type { ChatMessage, } from './client.ts';

//region Autofill: infer metadata from a task title

/**
 Builds the message pair for the autofill endpoint.
 
 The AI infers tags, locations, priority, and complexity from the task title.
 Existing tags and locations are provided so the model prefers consistency.
 
 @param title - Raw task title typed by the user
 
 @param existingTags - Tags already used in the database
 
 @param existingLocations - Locations already used in the database
 
 @returns Chat messages ready for {@link chatCompletion}
 
 @mutates existingTags - `JSON.stringify` may invoke array accessors or proxy traps.
 
 @mutates existingLocations - `JSON.stringify` may invoke array accessors or proxy traps.
 
 @example
 ```ts
 const messages = buildAutofillMessages({
   title: 'Buy groceries',
   existingTags: ['errand'],
   existingLocations: ['Walmart'],
 });
 ```
 */
export function buildAutofillMessages({
  title,
  existingTags,
  existingLocations,
}: {
  readonly title: string;
  existingTags: readonly string[];
  existingLocations: readonly string[];
},): ChatMessage[] {
  /**
   Schema-bearing system message; passed back as the first element of the chat array.
   */
  const systemPrompt =
    `You are a task metadata assistant. Given a task title, infer metadata.
Return ONLY valid JSON matching this schema, no other text:
{
  "tags": string[],
  "locations": string[],
  "priority": "low" | "medium" | "high" | null,
  "complexity": "low" | "medium" | "high" | null
}

Rules:
- tags: short, lowercase labels describing the task category (e.g. "shopping", "errands", "code-review")
- locations: physical places where the task can be done (e.g. "Walmart", "Home"); omit if location-agnostic
- priority: null unless clearly implied by urgency words
- complexity: null unless clearly implied by scope

For consistency, prefer these existing tags when applicable: ${
      JSON.stringify(existingTags,)
    }
For consistency, prefer these existing locations when applicable: ${
      JSON.stringify(existingLocations,)
    }`;

  return [
    {
      role: 'system',
      content: systemPrompt,
    },
    {
      role: 'user',
      content: `<task_title>${title}</task_title>`,
    },
  ];
}

//endregion Autofill

//region Suggestion ranking: rank tasks by relevance to user context

/**
 Builds the message pair for the suggestion engine.
 
 The AI returns an ordered array of task IDs, most relevant first.
 
 @param tasks - Serialisable task summaries (id, title, tags, locations, priority, dueDate, complexity)
 
 @param currentLocation - User's current or pinned location, or null
 
 @param focusDirective - Free-text focus instruction, or null
 
 @returns Chat messages ready for {@link chatCompletion}
 
 @mutates tasks - `JSON.stringify` may invoke `toJSON`, getters, or proxy traps on task summaries.
 
 @example
 ```ts
 const messages = buildSuggestionMessages({
   tasks,
   currentLocation: 'home',
   focusDirective: 'focus on errands',
 });
 ```
 */
export function buildSuggestionMessages({
  tasks,
  currentLocation,
  focusDirective,
}: {
  tasks: readonly {
    readonly id: string;
    readonly title: string;
    readonly tags: readonly string[];
    readonly locations: readonly string[];
    readonly priority?: string;
    readonly dueDate?: string;
    readonly complexity?: string;
  }[];
  readonly currentLocation?: string;
  readonly focusDirective?: string;
},): ChatMessage[] {
  /**
   Snapshot of the request time so the model can reason about due-date proximity.
   */
  const currentTime = new Date().toISOString();

  /**
   Ranking instructions kept separate so the user-context block remains compact.
   */
  const systemPrompt =
    `You are a task prioritization assistant. Given a list of tasks and the user's current context, return the task IDs ranked by what the user should do next.
Return ONLY a JSON array of task ID strings, most important first. No other text.

Ranking factors (in rough priority order):
1. Location match: tasks doable at the user's current location rank higher
2. Focus directive: respect the user's stated priority intent
3. Due date: approaching deadlines rank higher
4. Priority: high > medium > low
5. Complexity: prefer lower-complexity tasks when other factors are equal`;

  /**
   XML-delimited payload so the model can distinguish context from task data.
   */
  const userContent = `<user_context>
Location: ${currentLocation ?? 'unknown'}
Focus: ${focusDirective ?? 'none'}
Time: ${currentTime}
</user_context>

<user_tasks>
${JSON.stringify(tasks,)}
</user_tasks>`;

  return [
    {
      role: 'system',
      content: systemPrompt,
    },
    {
      role: 'user',
      content: userContent,
    },
  ];
}

//endregion Suggestion ranking
