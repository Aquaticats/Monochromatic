/**
 Task detail page handler.
 
 Renders its own HTML inline (not via `renderPage()`) because the task detail
 page omits the `<top-nav>` entirely; the `<task-detail>` web component
 provides its own back-button header.
 
 Client entry: `/dist/client/task-details.js` (src/client/task-details.ts)
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';
import {
  getTaskById,
  listTasksForBlockerPicker,
} from '../../lib/db/tasks.ts';
import { TASK_NOT_FOUND, } from '../../lib/types.ts';
import { serializePageData, } from './layout.ts';

/**
 Renders the task detail page for a single task with its blocker summaries.
 
 @param taskId - Task UUID from the route parameter
 
 @returns HTML response, or 404 when the task does not exist
 
 @example
 ```ts
 const response = await taskDetailsPage('uuid-123');
 ```
 */
export async function taskDetailsPage(taskId: string,): Promise<Response> {
  /**
   Existing task; a missing ID short-circuits with a 404 response.
   */
  const task = await getTaskById(taskId,);
  if (task === TASK_NOT_FOUND) {
    return new Response(
      'Task not found',
      { status: 404, },
    );
  }

  /**
   All tasks eligible to be picked as new blockers (excludes the current task).
   */
  const blockerCandidates = await listTasksForBlockerPicker(taskId,);
  /**
   Lookup table for resolving `blockedBy` IDs to candidate records.
   */
  const blockerCandidatesById = Object.fromEntries(
    blockerCandidates.map(function toEntry(candidate,) {
      return [
        candidate.id,
        candidate,
      ];
    },),
  );
  /**
   Existing blockers, resolved to candidate records and projected to the summary shape.
   */
  const blockerSummaries = task
    .blockedBy
    .map(function lookupBlocker(blockerId,) {
      return blockerCandidatesById[blockerId];
    },)
    .filter(function isDefined(candidate,) {
      return candidate !== undefined;
    },)
    .map(function toSummary(candidate,) {
      return {
        id: candidate.id,
        title: candidate.title,
        status: candidate.status,
      };
    },);

  /**
   Serialized into the embedded JSON `<script>` for client-side hydration.
   */
  const pageData = {
    task,
    blockerCandidates: blockerCandidates.map(function toMinimal(candidate,) {
      return {
        id: candidate.id,
        title: candidate.title,
      };
    },),
    blockerSummaries,
  };

  /**
   Rendered HTML response body; built via `hHtml` and wrapped with `<!DOCTYPE html>`.
   */
  const html = `<!DOCTYPE html>
${
    h({
      tag: 'html',
      attrs: { lang: 'en', },
      children: [
        h({
          tag: 'head',
          children: [
            h({
              tag: 'meta',
              attrs: { charset: 'utf8', },
            },),
            h({
              tag: 'meta',
              attrs: {
                name: 'viewport',
                content: 'width=device-width, initial-scale=1',
              },
            },),
            h({
              tag: 'title',
              text: `Task - ${task.title}`,
            },),
          ],
        },),
        h({
          tag: 'body',
          children: [
            h({
              tag: 'side-drawer',
              attrs: { id: 'drawer', },
            },),
            h({
              tag: 'div',
              class: 'page-wrapper',
              children: [
                h({
                  tag: 'main',
                  attrs: { id: 'app', },
                },),
              ],
            },),
            h({
              tag: 'script',
              attrs: {
                type: 'application/json',
                id: 'page-data',
              },
              html: serializePageData(pageData,),
            },),
            h({
              tag: 'script',
              attrs: {
                type: 'module',
                src: '/dist/client/task-details.js',
              },
            },),
          ],
        },),
      ],
    },)
  }`;

  return new Response(
    html,
    {
      headers: { 'Content-Type': 'text/html; charset=utf-8', },
    },
  );
}
