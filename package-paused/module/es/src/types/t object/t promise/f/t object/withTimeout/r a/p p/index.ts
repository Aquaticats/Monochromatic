/**
 * Re-export of `withTimeout` from `@monochromatic-dev/async-time-core`,
 * aliased to `$` to preserve the taxonomy convention.
 *
 * The implementation lives in `module-async-time`; this file keeps the
 * `t object/t promise/.../withTimeout` slot in the `module-es` taxonomy so
 * the namespace export
 * `types.object.promise.from.object.withTimeout.async.portable.$`
 * continues to resolve.
 *
 * @see {@link import('@monochromatic-dev/async-time-core/ts').withTimeout}
 */
export { withTimeout as $, } from '@monochromatic-dev/async-time-core/ts';
