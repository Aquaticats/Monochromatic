/**
 Static shell-element lookups for the renderer.
 
 @example
 ```ts
 const strip = getShellElement({ id: 'strip' });
 ```
 
 @packageDocumentation
 */

/**
 Error thrown when expected static markup is missing or has an unexpected tag.
 
 @example
 ```ts
 new MissingShellElementError({ id: 'strip' });
 ```
 */
export class MissingShellElementError extends Error {
  /**
   Builds a descriptive DOM lookup error.
   
   @param id - Missing element id.
   
   @example
   ```ts
   new MissingShellElementError({ id: 'strip' });
   ```
   */
  public constructor({ id, }: { readonly id: string; },) {
    super(`Missing file-manager shell element: ${id}`,);
    this.name = 'MissingShellElementError';
  }
}

/**
 Returns a shell element by id after checking its runtime type.
 
 @param id - Static element id expected in `index.html`.
 
 @returns Element narrowed to `HTMLElement`.
 
 @throws MissingShellElementError when the element is absent.
 
 @example
 ```ts
 getShellElement({ id: 'strip' });
 ```
 */
export function getShellElement({ id, }: { readonly id: string; },): HTMLElement {
  /**
   Element found in the static markup.
   */
  const element = document.querySelector<HTMLElement>(`#${id}`,);

  if (!(element instanceof HTMLElement))
    throw new MissingShellElementError({ id, },);

  return element;
}

/**
 Shows a user-visible status line (errors included) in the shell footer.
 
 @param text - Status text to show.
 
 @example
 ```ts
 showStatus({ text: 'listing failed' });
 ```
 */
export function showStatus({ text, }: { readonly text: string; },): void {
  getShellElement({ id: 'status', },)
    .textContent = text;
}
