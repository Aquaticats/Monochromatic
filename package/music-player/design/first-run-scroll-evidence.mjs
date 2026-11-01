//region Bounded native scroll-proof gate, not gesture delivery or accessibility acceptance
/** Rejects a proposed scroll witness that lacks matched content identity and both movement signals. */
export class FirstRunScrollEvidenceError extends Error {
  constructor(message) {
    super(message);
    this.name = 'FirstRunScrollEvidenceError';
  }
}
/**
 Checks explicitly selected application-body anchors, not arbitrary changed rectangles.
 Inspection of rendered paragraph visibility remains a separate publication requirement.
 */
export function assertFirstRunScrollEvidence({
  appRgbChanged,
  beforeNodes,
  afterNodes,
  sameEnvironment
}) {
  if (!sameEnvironment) throw new FirstRunScrollEvidenceError('Scroll evidence environment differs.');
  if ((beforeNodes.length !== 1) || (afterNodes.length !== 1)) {
    throw new FirstRunScrollEvidenceError('Scroll evidence requires exactly one identified application body in each hierarchy.');
  }
  const before = beforeNodes[0];
  const after = afterNodes[0];
  if ((before.text !== after.text) || (before.package !== 'dev.monochromatic.musicplayer')
    || (before.package !== after.package)) {
    throw new FirstRunScrollEvidenceError('Scroll evidence body identity differs.');
  }
  for (const node of [
    before,
    after
  ]) {
    if ((!Array.isArray(node.bounds))
      || (node.bounds
        .length
        !== 4)
      || node.bounds
        .some(value => !Number.isFinite(value))) {
      throw new FirstRunScrollEvidenceError('Scroll evidence body rectangle is absent or malformed.');
    }
  }
  if ((!appRgbChanged) || (before.bounds[1] === after.bounds[1])) {
    throw new FirstRunScrollEvidenceError('Scroll evidence requires changed app RGB and identified body displacement.');
  }
  return {
    bodyText: before.text,
    beforeBounds: before.bounds,
    afterBounds: after.bounds,
    verticalDisplacement: after.bounds[1] - before.bounds[1],
    appRgbChanged: true
  };
}
//endregion
