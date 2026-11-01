/**
 * Rim mesh lifecycle: the surviving cracked glass around the hole.
 *
 * While a pane holds in the cracked stage, its intact sheet hides and
 * this merged mesh of the rim cells stands in, so the blasted-out hole
 * is a real opening in real geometry rather than a texture trick.
 */
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  BufferAttribute,
  BufferGeometry,
  Mesh,
} from 'three/webgpu';

import type { Pane, } from './pane-model.ts';
import { rimMeshArrays, } from './rim-mesh.ts';

/**
 * Components per position or normal vector in the flat buffers.
 */
const XYZ = 3;

/**
 * Builds the merged rim mesh from a pane's stored rim cells and mounts
 * it where the glass sheet was, sharing the sheet's material so the rim
 * shades exactly like the pane it survives from.
 *
 * @param pane - cracked pane whose rim cells are already stored
 *
 * @mutates pane - `geometry.setAttribute` and `geometry.setIndex` fill a fresh geometry, `mesh.position.copy(pane.glass.position)` reads the sheet position through a three.js method the analyzer cannot inspect, `pane.group.add(mesh)` mounts the rim, and the rimMesh slot fills.
 *
 * @throws when called before rim cells are stored
 *
 * @example
 * ```ts
 * buildPaneRim(pane,);
 * ```
 */
export function buildPaneRim(pane: Pane,): void {
  /**
   * Merged pane-local arrays for every surviving rim cell.
   */
  const arrays = rimMeshArrays({
    cells: nonNullishOrThrow(pane.rimCells,),
    thickness: pane.glass
      .scale
      .z,
  },);
  /**
   * Fresh geometry owned by this rim; disposed by {@link dropPaneRim}.
   */
  const geometry = new BufferGeometry();
  geometry.setAttribute(
    'position',
    new BufferAttribute(
      arrays.positions,
      XYZ,
    ),
  );
  geometry.setAttribute(
    'normal',
    new BufferAttribute(
      arrays.normals,
      XYZ,
    ),
  );
  geometry.setIndex(new BufferAttribute(
    arrays.indices,
    1,
  ),);
  /**
   * Rim mesh sharing the sheet's glass material.
   */
  const mesh = new Mesh(
    geometry,
    pane.glass
      .material,
  );
  mesh.position
    .copy(pane.glass
      .position,);
  pane.group
    .add(mesh,);
  pane.rimMesh = mesh;
}

/**
 * Disposes a pane's rim mesh resources, when present. The material is
 * the shared sheet glass and stays alive.
 *
 * @param pane - pane whose rim should be dropped
 *
 * @mutates pane - `pane.rimMesh.geometry.dispose()` frees the merged buffers, `pane.group.remove(pane.rimMesh)` unmounts the rim, and the rimMesh slot clears.
 *
 * @example
 * ```ts
 * dropPaneRim(pane,);
 * ```
 */
export function dropPaneRim(pane: Pane,): void {
  if (pane.rimMesh === undefined)
    return;
  pane.rimMesh
    .geometry
    .dispose();
  pane.group
    .remove(pane.rimMesh,);
  delete pane.rimMesh;
}
