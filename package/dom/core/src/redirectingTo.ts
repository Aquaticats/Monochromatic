/**
 Automatically redirects to a target URL found in an anchor element with class `redirectingTo`.
 
 Searches for an anchor element (`<a>`) with CSS class `redirectingTo` and schedules
 a redirect to its `href` URL after a specified delay. Uses `location.replace()` to
 replace the current page in browser history, preventing back navigation to the
 original page. Useful for implementing delayed redirects, landing pages, or
 automatic navigation flows.
 
 @param delayTime - Time in milliseconds to wait before redirecting. Defaults to 5000ms (5 seconds).
 
 @example
 ```ts
 // HTML: <a href="https://example.com/dashboard" class="redirectingTo">Go to Dashboard</a>
 
 // Redirect after default 5 seconds
 onLoadRedirectingTo();
 
 // Redirect after 3 seconds
 onLoadRedirectingTo(3000);
 
 // Redirect immediately (not recommended for UX)
 onLoadRedirectingTo(0);
 
 // Common use case: Landing page with delayed redirect
 // HTML: <a href="/welcome" class="redirectingTo">Redirecting to welcome page...</a>
 onLoadRedirectingTo(2000); // 2 second delay
 
 // No redirect occurs if no element with class "redirectingTo" exists
 onLoadRedirectingTo(1000); // Safe to call even without target element
 ```
 */
export function onLoadRedirectingTo(delayTime: number = 5_000,): void {
  /**
   Anchor opted into auto-redirect via the `redirectingTo` class, or null when none exists.
   */
  const potentialRedirectingToElement = document
    .querySelector<HTMLAnchorElement>('a.redirectingTo',);
  if (potentialRedirectingToElement) {
    setTimeout(
      function redirect() {
        globalThis.location
          .replace(
          potentialRedirectingToElement.href,
          );
      },
      delayTime,
    );
  }
}
