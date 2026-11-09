/**
 Query parameter helpers for import attribute rewriting.
 
 Encodes and decodes the `__importattr` query parameter
 used to tag rewritten specifiers.
 
 @module
 */

//region Constants

/**
 Query parameter name used to encode the attribute type in rewritten specifiers.
 */
export const ATTR_QUERY_KEY = '__importattr';

/**
 Sentinel returned by {@link extractAttrType} when a module ID carries no
 `?__importattr=` marker. A `unique symbol` so it can never collide with a
 real attribute-type string; callers narrow with `=== NO_QUERY_ATTR`.
 */
export const NO_QUERY_ATTR: unique symbol = Symbol('import-attributes/no-query-attr',);

//endregion Constants

//region Query helpers

/**
 Extracts the attribute type from a module ID's query parameter.
 
 @param id - Module ID potentially containing `?__importattr=<type>`
 
 @returns Attribute type string if present, {@link NO_QUERY_ATTR} otherwise
 
 The value is returned as written, registered handler or not: whether a type can
 actually be loaded is the load hook's check, which leaves an unknown type to other
 loaders.
 
 @example
 ```ts
 extractAttrType('./file.sql?__importattr=text'); // 'text'
 extractAttrType('./file.sql'); // NO_QUERY_ATTR
 ```
 */
export function extractAttrType(id: string,): string | typeof NO_QUERY_ATTR {
  /**
   Offset of the attribute marker; -1 signals the ID has no encoded attribute.
   */
  const queryIndex = id.indexOf(`?${ATTR_QUERY_KEY}=`,);
  if (queryIndex === (-1))
    return NO_QUERY_ATTR;
  /**
   Position immediately after `?<key>=`, where the value substring begins.
   */
  const valueStart = queryIndex + ATTR_QUERY_KEY
    .length
    + 2;
  /**
   Boundary `&` introducing a subsequent query parameter, if any.
   */
  const ampIndex = id.indexOf(
    '&',
    valueStart,
  );
  if (ampIndex === (-1))
    return id.slice(valueStart,);
  return id.slice(
    valueStart,
    ampIndex,
  );
}

/**
 Strips the `__importattr` query parameter from a module ID,
 returning the clean file path.
 
 @param id - Module ID with `?__importattr=<type>`
 
 @returns File path without the attribute query parameter
 
 Everything from the marker onward is removed, later query parameters included, so a
 tagged ID always yields the file path the handler reads.
 
 @example
 ```ts
 stripAttrQuery('./file.sql?__importattr=text'); // './file.sql'
 ```
 */
export function stripAttrQuery(id: string,): string {
  /**
   Offset of the attribute marker; -1 means the ID is already clean.
   */
  const queryIndex = id.indexOf(`?${ATTR_QUERY_KEY}=`,);
  if (queryIndex === (-1))
    return id;
  return id.slice(
    0,
    queryIndex,
  );
}

//endregion Query helpers
