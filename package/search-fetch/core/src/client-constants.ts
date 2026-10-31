/**
 Linkup client constants.
 
 @module
 */

/**
 Default Linkup API base URL.
 */
const DEFAULT_LINKUP_BASE_URL = 'https://api.linkup.so/v1' as const;

/**
 Linkup search endpoint path.
 */
const LINKUP_SEARCH_ENDPOINT = '/search' as const;

/**
 Linkup fetch endpoint path.
 */
const LINKUP_FETCH_ENDPOINT = '/fetch' as const;

/**
 Fixed Linkup search depth selected for this package.
 */
const LINKUP_SEARCH_DEPTH = 'standard' as const;

/**
 Fixed Linkup search output type selected for this package.
 */
const LINKUP_SEARCH_OUTPUT_TYPE = 'searchResults' as const;

/**
 Fixed Linkup fetch JavaScript rendering mode.
 */
const LINKUP_FETCH_RENDER_JS = true as const;

/**
 Fixed Linkup fetch image extraction mode.
 */
const LINKUP_FETCH_EXTRACT_IMAGES = false as const;

/**
 Fixed Linkup fetch raw HTML mode.
 */
const LINKUP_FETCH_INCLUDE_RAW_HTML = false as const;

/**
 HTTP POST method.
 */
const HTTP_POST = 'POST' as const;

/**
 Authorization header name.
 */
const AUTHORIZATION_HEADER = 'Authorization' as const;

/**
 JSON content type header name.
 */
const CONTENT_TYPE_HEADER = 'Content-Type' as const;

/**
 JSON media type sent to Linkup.
 */
const JSON_CONTENT_TYPE = 'application/json' as const;

/**
 User-Agent header name.
 */
const USER_AGENT_HEADER = 'User-Agent' as const;

/**
 User-Agent value sent by this package.
 */
const USER_AGENT_VALUE = '@monochromatic-dev/search-fetch-pi-plugin' as const;

/**
 AbortError name used by fetch implementations.
 */
const ABORT_ERROR_NAME = 'AbortError' as const;

export {
  ABORT_ERROR_NAME,
  AUTHORIZATION_HEADER,
  CONTENT_TYPE_HEADER,
  DEFAULT_LINKUP_BASE_URL,
  HTTP_POST,
  JSON_CONTENT_TYPE,
  LINKUP_FETCH_ENDPOINT,
  LINKUP_FETCH_EXTRACT_IMAGES,
  LINKUP_FETCH_INCLUDE_RAW_HTML,
  LINKUP_FETCH_RENDER_JS,
  LINKUP_SEARCH_DEPTH,
  LINKUP_SEARCH_ENDPOINT,
  LINKUP_SEARCH_OUTPUT_TYPE,
  USER_AGENT_HEADER,
  USER_AGENT_VALUE,
};
