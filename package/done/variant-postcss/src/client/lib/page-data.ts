/* oxlint-disable typescript/no-unnecessary-type-parameters -- TData is a return-only parameter on purpose: it centralizes the single JSON-hydration cast at this trusted server->client boundary. Inlining to `unknown` would push an unchecked `as` assertion (and its own suppression) to every one of the five call sites. */
/**
 Reads server-provided page data from the `\<script id="page-data" type="application/json"\>`
 element that {@link renderPage} (or the inline HTML shells) embed in every page.
 
 This is the bridge between server-side data and client-side rendering:
 the server serializes query results as JSON into the HTML, and the client
 deserializes them here to build the UI without an additional fetch.
 
 @returns Parsed page data of the requested type
 
 @example
 ```ts
 const data = readPageData<InboxPageData>();
 ```
 */
export function readPageData<TData,>(): TData {
  /**
   Embedded JSON `<script>` placed in the HTML by the server render.
   */
  const element = document.querySelector<HTMLScriptElement>('#page-data',);
  if (!(element instanceof HTMLScriptElement))
    throw new Error('Missing page data script element',);

  /**
   Raw JSON payload extracted from the script element.
   */
  const text = element.textContent;
  if (text.length
    === 0)
    throw new Error('Page data element is empty',);

  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- JSON.parse returns unknown; caller provides the expected shape via TData
  return JSON.parse(text,) as TData;
}
/* oxlint-enable typescript/no-unnecessary-type-parameters */
