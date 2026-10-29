/**
 Mesh-geometry constructors for the scatter and axis-arrow layers.
 
 Built once at module load; the same geometry instance is reused
 across every glyph (deck.gl's `SimpleMeshLayer` instances the mesh
 per-datum via `getPosition`/`getScale`/`getColor`/`getOrientation`).
 
 Three geometries:
 
 - {@link sphereGeometry}: leaf glyphs and the unknown cluster.
   Smooth-shaded per-vertex normals via {@link SphereGeometry}.
 - {@link octahedronGeometry}: non-leaf glyphs. Flat-shaded with one
   face normal per triangle (vertices duplicated 24 times so each
   face's three corners share a normal).
 - {@link coneGeometry}: axis arrowheads. Smooth-shaded cone with apex
   along +Y; rotate via `getOrientation` to align with +x/+y/+z axes.
 
 @example
 ```ts
 import { sphereGeometry } from './deck-geometries.ts';
 new SimpleMeshLayer({ mesh: sphereGeometry, getPosition, getScale });
 ```
 */

// oxlint-disable eslint/no-magic-numbers -- octahedron face/vertex tables and per-face stride offsets are intrinsic geometry data; naming each slot is absurd and obscures the layout.

import {
  ConeGeometry,
  Geometry,
  SphereGeometry,
} from '@luma.gl/engine';

//region Constants

/**
 `1/√3`. The octahedron's flat-face normals are the eight unit
 vectors with components `(±1/√3, ±1/√3, ±1/√3)`; precomputing the
 factor keeps the loop body free of `Math.sqrt`.
 */
const SQRT_3_INV = 1 / Math
  .sqrt(3,);

/**
 Latitude divisions for the sphere mesh; 16 keeps the silhouette smooth without exploding the vertex count.
 */
const SPHERE_NLAT = 16;
/**
 Longitude divisions for the sphere mesh; matches {@link SPHERE_NLAT}.
 */
const SPHERE_NLONG = 16;
/**
 Cone tessellation; 16 sides is enough for a clean arrowhead silhouette at any zoom.
 */
const CONE_RADIAL_SEGMENTS = 16;

/**
 Indices into {@link OCTAHEDRON_VERTICES} for the eight triangular faces, CCW from outside.
 */
const OCTAHEDRON_FACE_INDICES: readonly (readonly [
  number,
  number,
  number,
])[] = [
  [
    2,
    0,
    4,
  ],
  [
    2,
    4,
    1,
  ],
  [
    2,
    1,
    5,
  ],
  [
    2,
    5,
    0,
  ],
  [
    3,
    4,
    0,
  ],
  [
    3,
    1,
    4,
  ],
  [
    3,
    5,
    1,
  ],
  [
    3,
    0,
    5,
  ],
];

/**
 Six canonical octahedron vertices, on the unit axes.
 */
const OCTAHEDRON_VERTICES: readonly (readonly [
  number,
  number,
  number,
])[] = [
  [
    1,
    0,
    0,
  ],
  [
    -1,
    0,
    0,
  ],
  [
    0,
    1,
    0,
  ],
  [
    0,
    -1,
    0,
  ],
  [
    0,
    0,
    1,
  ],
  [
    0,
    0,
    -1,
  ],
];

/**
 Number of position/normal floats per face (3 vertices × 3 components).
 */
const FLOATS_PER_FACE_VEC3 = 9;
/**
 Number of texcoord floats per face (3 vertices × 2 components).
 */
const FLOATS_PER_FACE_VEC2 = 6;

//endregion Constants

//region Public geometries

/**
 Unit sphere for leaf glyphs and the unknown cluster.
 
 Smooth-shaded (per-vertex normals match position directions), 16x16
 tessellation. `SimpleMeshLayer` scales it per-datum via `getScale`.
 */
export const sphereGeometry: Geometry = new SphereGeometry({
  radius: 1,
  nlat: SPHERE_NLAT,
  nlong: SPHERE_NLONG,
},);

/**
 Unit octahedron for non-leaf glyphs.
 
 Vertices duplicated 24 times so each triangle's three corners share
 a single face normal; produces visible facets ("diamond" silhouette
 from every camera angle). Face normals are the eight `(±1/√3)³`
 unit vectors.
 
 @returns A `Geometry` ready to feed into `SimpleMeshLayer.mesh`.
 */
function buildOctahedronGeometry(): Geometry {
  /**
   Eight-face count; drives every typed-array size below.
   */
  const faceCount = OCTAHEDRON_FACE_INDICES.length;
  /**
   Interleaved per-face vertex positions; written face-by-face inside the loop.
   */
  const positions = new Float32Array(faceCount * FLOATS_PER_FACE_VEC3,);
  /**
   Per-vertex normals; each face's three corners share the same face normal.
   */
  const normals = new Float32Array(faceCount * FLOATS_PER_FACE_VEC3,);
  /**
   Per-vertex UVs; every face uses the same triangle so the canvas texture renders identically per side.
   */
  const texCoords = new Float32Array(faceCount * FLOATS_PER_FACE_VEC2,);
  /**
   Looks up an octahedron vertex by index; throws on out-of-range.
   
   @param i - Octahedron vertex index in `[0, 5]`.
   
   @returns The 3-tuple at that index.
   
   @throws When `i` is outside the vertex table bounds.
   */
  function vertexAt(i: number,): readonly [
    number,
    number,
    number,
  ] {
    /**
     Vertex tuple from the canonical table; out-of-range becomes an explicit error rather than a silent `undefined`.
     */
    const v = OCTAHEDRON_VERTICES[i];
    if (v === undefined)
      throw new Error(`octahedron vertex index out of range: ${i.toString()}`,);
    return v;
  }
  OCTAHEDRON_FACE_INDICES.forEach(function emitFace(
    [a, b, c,],
    faceIndex,
  ) {
    /**
     First corner of the triangle being emitted.
     */
    const [
      vAx,
      vAy,
      vAz,
    ] = vertexAt(a,);
    /**
     Second corner of the triangle being emitted.
     */
    const [
      vBx,
      vBy,
      vBz,
    ] = vertexAt(b,);
    /**
     Third corner of the triangle being emitted.
     */
    const [
      vCx,
      vCy,
      vCz,
    ] = vertexAt(c,);
    /**
     Face-centroid sum component × {@link SQRT_3_INV} = octahedral unit normal.
     */
    const nx = (vAx + vBx
      + vCx) * SQRT_3_INV;
    /**
     Y component of the same face normal.
     */
    const ny = (vAy + vBy
      + vCy) * SQRT_3_INV;
    /**
     Z component of the same face normal.
     */
    const nz = (vAz + vBz
      + vCz) * SQRT_3_INV;
    /**
     Byte-flat write cursor for the position buffer; advances three vec3 slots per face.
     */
    const posOffset = faceIndex * FLOATS_PER_FACE_VEC3;
    positions[posOffset + 0] = vAx;
    positions[posOffset + 1] = vAy;
    positions[posOffset + 2] = vAz;
    positions[posOffset + 3] = vBx;
    positions[posOffset + 4] = vBy;
    positions[posOffset + 5] = vBz;
    positions[posOffset + 6] = vCx;
    positions[posOffset + 7] = vCy;
    positions[posOffset + 8] = vCz;
    /**
     Write cursor for the normal buffer; identical stride to {@link posOffset}.
     */
    const norOffset = faceIndex * FLOATS_PER_FACE_VEC3;
    normals[norOffset + 0] = nx;
    normals[norOffset + 1] = ny;
    normals[norOffset + 2] = nz;
    normals[norOffset + 3] = nx;
    normals[norOffset + 4] = ny;
    normals[norOffset + 5] = nz;
    normals[norOffset + 6] = nx;
    normals[norOffset + 7] = ny;
    normals[norOffset + 8] = nz;
    /**
     Every face shares the same UV triangle `(0, 0), (1, 0), (0.5, 1)`
     so the per-probe canvas texture appears the same on each face;
     the name baked into that triangle is then readable on whichever
     face is currently camera-facing.
     */
    const uvOffset = faceIndex * FLOATS_PER_FACE_VEC2;
    texCoords[uvOffset + 0] = 0;
    texCoords[uvOffset + 1] = 0;
    texCoords[uvOffset + 2] = 1;
    texCoords[uvOffset + 3] = 0;
    texCoords[uvOffset + 4] = 0.5;
    texCoords[uvOffset + 5] = 1;
  },);
  return new Geometry({
    topology: 'triangle-list',
    attributes: {
      POSITION: {
        size: 3,
        value: positions,
      },
      NORMAL: {
        size: 3,
        value: normals,
      },
      TEXCOORD_0: {
        size: 2,
        value: texCoords,
      },
    },
  },);
}

/**
 Built once at module load and reused for every non-leaf glyph.
 */
export const octahedronGeometry: Geometry = buildOctahedronGeometry();

/**
 Unit cone with apex along +X.
 
 Used for the +X axis arrowhead. Built via luma.gl's
 {@link ConeGeometry} with `verticalAxis: 'x'`, so the cone's vertical
 dimension is the world-X axis; no runtime rotation needed.
 */
export const coneGeometryX: Geometry = new ConeGeometry({
  radius: 1,
  height: 1,
  nradial: CONE_RADIAL_SEGMENTS,
  verticalAxis: 'x',
},);

/**
 Unit cone with apex along +Y; +Y axis arrowhead.
 */
export const coneGeometryY: Geometry = new ConeGeometry({
  radius: 1,
  height: 1,
  nradial: CONE_RADIAL_SEGMENTS,
  verticalAxis: 'y',
},);

/**
 Unit cone with apex along +Z; +Z axis arrowhead.
 */
export const coneGeometryZ: Geometry = new ConeGeometry({
  radius: 1,
  height: 1,
  nradial: CONE_RADIAL_SEGMENTS,
  verticalAxis: 'z',
},);

//endregion Public geometries
