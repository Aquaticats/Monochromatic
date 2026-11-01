/**
 Quota-overflow recognition shared by the web storage persistence engines.
 
 @module
 */

/**
 Error `name` values engines raise for a storage quota overflow: the DOM
 standard name every current browser and Node web storage use, plus Firefox's
 legacy alias. Quota overflow is matched by `name` rather than by class or
 numeric `code` because the concrete type differs by engine (a `DOMException`
 in Chromium and Node, a differently-branded object historically in Firefox),
 while the standard name is stable across them.
 */
const QUOTA_EXCEEDED_NAMES: ReadonlySet<string> = new Set([
  'QuotaExceededError',
  'NS_ERROR_DOM_QUOTA_REACHED',
],);

/**
 Reports whether a caught `setItem` value is a storage quota overflow, so
 eviction reclaims space only for a full store and never for an unrelated
 write fault such as a disabled-storage `SecurityError`.
 
 @param error - Caught value from a `setItem` failure.
 
 @returns Whether `error` names a quota overflow.
 
 @example
 ```ts
 try { sessionStorage.setItem(k, v); }
 catch (error: unknown) { if (isQuotaExceededError(error)) evictOldest(); }
 ```
 */
export function isQuotaExceededError(error: unknown,): boolean {
  return (
    ((typeof error) === 'object')
      && (error !== null)
      && ('name' in error)
      && ((typeof error.name) === 'string')
      && QUOTA_EXCEEDED_NAMES.has(error.name,)
  );
}
