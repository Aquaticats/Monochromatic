/**
 * HTTP status code constants.
 *
 * Spec-defined codes are re-exported from
 * `@monochromatic-dev/const-core`. Codes used only by this app
 * (302/303/410/413) stay local because they have no second consumer in
 * the workspace yet; promote to the shared package when one appears.
 */

export {
  HTTP_BAD_REQUEST,
  HTTP_CONFLICT,
  HTTP_CREATED,
  HTTP_FORBIDDEN,
  HTTP_INTERNAL_SERVER_ERROR,
  HTTP_NO_CONTENT,
  HTTP_NOT_FOUND,
  HTTP_NOT_MODIFIED,
  HTTP_OK,
} from '@monochromatic-dev/const-core/ts';

/**
 * 302 Found: temporary redirect that may change method on follow.
 */
export const HTTP_FOUND = 302;

/**
 * 303 See Other: response is at the URL in `Location`.
 */
export const HTTP_SEE_OTHER = 303;

/**
 * 410 Gone: resource existed and was deleted.
 */
export const HTTP_GONE = 410;

/**
 * 413 Payload Too Large: single chunk exceeded the hard cap.
 */
export const HTTP_PAYLOAD_TOO_LARGE = 413;
