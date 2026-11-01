/**
 In-progress page handler.
 
 Queries tasks with active timers, then delegates to the shared `renderPage()` shell.
 Client entry: `/dist/client/in-progress.js` (src/client/in-progress.ts)
 */
import { listInProgressTasks, } from '../../lib/db/tasks.ts';
import { renderPage, } from './layout.ts';

/**
 Renders the in-progress page listing tasks with active timers.
 
 @returns HTML response for the in-progress page
 
 @example
 ```ts
 const response = await inProgressPage();
 ```
 */
export async function inProgressPage(): Promise<Response> {
  /**
   Tasks with active timers, queried fresh on every request for accurate elapsed times.
   */
  const tasks = await listInProgressTasks();

  return renderPage({
    title: 'In Progress - Done',
    heading: 'In Progress',
    entryScriptPath: '/dist/client/in-progress.js',
    pageData: { tasks, },
  },);
}
