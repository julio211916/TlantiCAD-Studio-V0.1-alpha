import { BufferGeometry, Vector3 } from 'three';
import { MeshMetadata } from '../types';
import { TessellateModifier } from 'three-stdlib';

export const calculateMeshStats = (geometry: BufferGeometry): MeshMetadata => {
  let volume = 0;
  let area = 0;
  
  const pos = geometry.attributes.position;
  const index = geometry.index;
  
  const p1 = new Vector3();
  const p2 = new Vector3();
  const p3 = new Vector3();
  const a = new Vector3();
  const b = new Vector3();
  const c = new Vector3();

  const processTriangle = (ia: number, ib: number, ic: number) => {
    p1.fromBufferAttribute(pos, ia);
    p2.fromBufferAttribute(pos, ib);
    p3.fromBufferAttribute(pos, ic);

    // Signed Volume calculation
    // Volume contribution = (p1 . (p2 x p3)) / 6
    a.copy(p1);
    b.copy(p2);
    c.copy(p3);
    
    // b.cross(c) modifies b, so we do it in place
    volume += a.dot(b.cross(c)) / 6.0;
    
    // Surface Area calculation
    // Area = 0.5 * |(p2 - p1) x (p3 - p1)|
    b.copy(p2).sub(p1);
    c.copy(p3).sub(p1);
    const cross = new Vector3().copy(b).cross(c);
    area += 0.5 * cross.length();
  };

  if (index) {
    for (let i = 0; i < index.count; i += 3) {
      processTriangle(index.getX(i), index.getY(i), index.getZ(i));
    }
  } else {
    for (let i = 0; i < pos.count; i += 3) {
      processTriangle(i, i + 1, i + 2);
    }
  }
  
  return {
    volume: Math.abs(volume),
    area: area,
    vertices: pos.count,
    triangles: index ? index.count / 3 : pos.count / 3
  };
};

export const subdivideGeometry = (geometry: BufferGeometry): BufferGeometry => {
    const modifier = new TessellateModifier(0.1, 6); // Max edge length, max iterations
    // Note: TessellateModifier in three-stdlib might expect Geometry or BufferGeometry depending on version, 
    // but typically modifies BufferGeometry in newer versions.
    // If it fails, we return original.
    try {
        const tessellated = modifier.modify(geometry);
        return tessellated;
    } catch (e) {
        console.warn("Subdivision failed", e);
        return geometry;
    }
};