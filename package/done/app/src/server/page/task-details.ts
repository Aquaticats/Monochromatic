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
} from '../../lib/db/tasks-queries.ts';
import { TASK_NOT_FOUND, } from '../../lib/types.ts';
import { serializePageData, } from './layout.ts';

/**
 Renders the task detail page for a single task (via {@link getTaskById})
 with its blocker summaries (candidates from {@link listTasksForBlockerPicker}),
 serialized for the client via {@link serializePageData}.
 
 @param taskId - Task UUID from the route parameter
 
 @returns HTML response, or 404 when the task does not exist
 
 @example
 ```ts
 const response = await taskDetailsPage('abc-123');
 ```
 */
export async function taskDetailsPage(taskId: string,): Promise<Response> {
  /**
   Target task; the not-found sentinel short-circuits to a 404 response below.
   */
  const task = await getTaskById(taskId,);
  if (task === TASK_NOT_FOUND) {
    return new Response(
      'Task not found',
      { status: 404, },
    );
  }

  /**
   Candidates eligible to be picked as a new blocker (everything but this task).
   */
  const blockerCandidates = await listTasksForBlockerPicker(taskId,);
  /**
   Lookup keyed by id so `blockerSummaries` can be built without a second query.
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
   Already-set blockers reshaped to the summary view the client expects.
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
   Bundled payload serialised into the embedded `#page-data` script.
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
   Full HTML document string; returned wrapped in a Response below.
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
