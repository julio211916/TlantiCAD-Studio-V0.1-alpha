import React, { useMemo, useEffect, forwardRef, useState, useRef } from 'react';
import { useLoader, useThree } from '@react-three/fiber';
import { STLLoader, OBJLoader, PLYLoader, SimplifyModifier } from 'three-stdlib';
import * as THREE from 'three';
import { FileData, ToolMode, MeshMetadata } from '../types';
import { calculateMeshStats } from '../utils/meshUtils';
import { resolveDentalMaterialProfile } from '@/lib/dental-materials';

interface ModelViewerProps {
  file: FileData;
  visible: boolean;
  selected: boolean;
  onClick: (point: THREE.Vector3, event: MouseEvent) => void;
  onMiddleClick?: (point: THREE.Vector3) => void;
  onContextMenu?: (e: THREE.Event) => void;
  activeTool: ToolMode;
  onMetadataLoaded?: (id: string, metadata: MeshMetadata) => void;
  onSculptStart?: () => void;
  onSculptEnd?: () => void;
  cropPlanes?: THREE.Plane[];
}

const SELECTION_EMISSIVE = '#5b9bf6';
const SEGMENT_OVERLAY = '#5b9bf6';

const Voxelizer = ({ geometry, file, selected }: { geometry: THREE.BufferGeometry, file: FileData, selected: boolean }) => {
  const voxelSize = 0.5;
  const instancedMeshRef = React.useRef<THREE.InstancedMesh>(null);
  const materialProfile = useMemo(() => resolveDentalMaterialProfile(file), [file]);
  
  const voxelData = useMemo(() => {
    geometry.computeBoundingBox();
    const positions: THREE.Vector3[] = [];
    const posAttribute = geometry.getAttribute('position');
    const step = Math.max(1, Math.floor(posAttribute.count / 10000)); 
    for (let i = 0; i < posAttribute.count; i += step) {
      const v = new THREE.Vector3().fromBufferAttribute(posAttribute, i);
      v.x = Math.round(v.x / voxelSize) * voxelSize;
      v.y = Math.round(v.y / voxelSize) * voxelSize;
      v.z = Math.round(v.z / voxelSize) * voxelSize;
      positions.push(v);
    }
    
    const unique = [];
    const set = new Set();
    for (const p of positions) {
      const key = `${p.x},${p.y},${p.z}`;
      if (!set.has(key)) {
        set.add(key);
        unique.push(p);
      }
    }
    return unique;
  }, [geometry, voxelSize]);

  useEffect(() => {
    if (instancedMeshRef.current) {
      const dummy = new THREE.Object3D();
      voxelData.forEach((pos, i) => {
        dummy.position.copy(pos);
        dummy.updateMatrix();
        instancedMeshRef.current!.setMatrixAt(i, dummy.matrix);
      });
      instancedMeshRef.current.instanceMatrix.needsUpdate = true;
    }
  }, [voxelData]);

  return (
    <instancedMesh 
      ref={instancedMeshRef} 
      args={[undefined as any, undefined as any, voxelData.length]}
      position={file.position}
      rotation={file.rotation as any}
      scale={file.scale}
    >
      <boxGeometry args={[voxelSize * 0.9, voxelSize * 0.9, voxelSize * 0.9]} />
      <meshStandardMaterial color={materialProfile.color} metalness={materialProfile.metalness} roughness={materialProfile.roughness} emissive={selected ? SELECTION_EMISSIVE : '#000000'} emissiveIntensity={selected ? 0.12 : 0} />
    </instancedMesh>
  );
};

export const ModelViewer = forwardRef<THREE.Mesh, ModelViewerProps>(({ file, visible, selected, onClick, onMiddleClick, onContextMenu, activeTool, onMetadataLoaded, onSculptStart, onSculptEnd, cropPlanes = [] }, ref) => {
  const extension = file.name.split('.').pop()?.toLowerCase();
  const [hoverPoint, setHoverPoint] = useState<THREE.Vector3 | null>(null);

  // Determine Loader
  const Loader = useMemo(() => {
    if (extension === 'stl') return STLLoader;
    if (extension === 'obj') return OBJLoader;
    if (extension === 'ply') return PLYLoader;
    return null;
  }, [extension]);

  // Load Geometry
  const geometry = useLoader(Loader as any, file.url || '') as THREE.BufferGeometry | THREE.Group;

  // Prepare geometry for rendering
  const [meshGeometry, setMeshGeometry] = useState<THREE.BufferGeometry | null>(null);
  const historyRef = useRef<THREE.BufferGeometry[]>([]);
  const historyIndexRef = useRef<number>(-1);
  const [sourceMaterialColor, setSourceMaterialColor] = useState<string | null>(null);
  const materialProfile = useMemo(() => resolveDentalMaterialProfile(file), [file]);

  useEffect(() => {
    if (!geometry) return;
    let finalGeo: THREE.BufferGeometry | null = null;

    if (geometry instanceof THREE.BufferGeometry) {
      const cloned = geometry.clone();
      cloned.computeVertexNormals();
      cloned.center(); 
      finalGeo = cloned;
      setSourceMaterialColor(null);
    } else if (geometry instanceof THREE.Group) {
       geometry.traverse((child) => {
         if ((child as THREE.Mesh).isMesh && !finalGeo) {
           const childMesh = child as THREE.Mesh;
           finalGeo = childMesh.geometry.clone();
           finalGeo.center();

           const childMaterial = Array.isArray(childMesh.material) ? childMesh.material[0] : childMesh.material;
           const candidateColor = childMaterial && 'color' in childMaterial && childMaterial.color instanceof THREE.Color
             ? `#${childMaterial.color.getHexString()}`
             : null;
           setSourceMaterialColor(candidateColor);
         }
       });
    }
    
    if (finalGeo) {
      setMeshGeometry(finalGeo);
      historyRef.current = [finalGeo.clone()];
      historyIndexRef.current = 0;
    }
  }, [geometry]);

  const saveHistory = (geo: THREE.BufferGeometry) => {
    const newHistory = historyRef.current.slice(0, historyIndexRef.current + 1);
    newHistory.push(geo.clone());
    historyRef.current = newHistory;
    historyIndexRef.current = newHistory.length - 1;
  };

  useEffect(() => {
    if (file.action && meshGeometry) {
      if (file.action.type === 'OPTIMIZE') {
         try {
           const modifier = new SimplifyModifier();
           const count = meshGeometry.attributes.position.count;
           const simplified = modifier.modify(meshGeometry, Math.floor(count * 0.5));
           setMeshGeometry(simplified);
           saveHistory(simplified);
         } catch (e) {
           console.error("Optimization failed", e);
         }
      } else if (file.action.type === 'UNDO') {
         if (historyIndexRef.current > 0) {
           historyIndexRef.current -= 1;
           setMeshGeometry(historyRef.current[historyIndexRef.current].clone());
         }
      } else if (file.action.type === 'REDO') {
         if (historyIndexRef.current < historyRef.current.length - 1) {
           historyIndexRef.current += 1;
           setMeshGeometry(historyRef.current[historyIndexRef.current].clone());
         }
      }
    }
  }, [file.action]);

  const [isSculpting, setIsSculpting] = useState(false);
  const sculptRadius = 1.0;
  const sculptStrength = 0.1;

  const sculptMesh = (point: THREE.Vector3) => {
    if (!meshGeometry) return;
    const positionAttribute = meshGeometry.getAttribute('position');
    const normalAttribute = meshGeometry.getAttribute('normal');
    if (!positionAttribute || !normalAttribute) return;

    // Convert point to local space
    const localPoint = point.clone();
    if (ref && typeof ref !== 'function' && ref.current) {
      ref.current.worldToLocal(localPoint);
    }

    const v = new THREE.Vector3();
    const n = new THREE.Vector3();
    let modified = false;

    for (let i = 0; i < positionAttribute.count; i++) {
      v.fromBufferAttribute(positionAttribute, i);
      const dist = v.distanceTo(localPoint);
      
      if (dist < sculptRadius) {
        n.fromBufferAttribute(normalAttribute, i);
        // Falloff
        const intensity = (1 - dist / sculptRadius) * sculptStrength;
        v.addScaledVector(n, intensity);
        positionAttribute.setXYZ(i, v.x, v.y, v.z);
        modified = true;
      }
    }

    if (modified) {
      positionAttribute.needsUpdate = true;
      meshGeometry.computeVertexNormals();
    }
  };

  // Calculate Statistics
  useEffect(() => {
    if (meshGeometry && onMetadataLoaded && !file.metadata) {
       const stats = calculateMeshStats(meshGeometry);
       onMetadataLoaded(file.id, stats);
    }
  }, [meshGeometry, file.id, file.metadata, onMetadataLoaded]);

  // Clipping Planes
  const clippingPlanes = useMemo(() => {
     let planes: THREE.Plane[] = [...cropPlanes];
     if (activeTool === 'CLIP') {
       // Simple Y-plane clip
       planes.push(new THREE.Plane(new THREE.Vector3(0, -1, 0), 0));
     }
     if (activeTool === 'BOOLEAN_CUT') {
       // Simple visual box cut
       planes.push(
         new THREE.Plane(new THREE.Vector3(1, 0, 0), 1),
         new THREE.Plane(new THREE.Vector3(-1, 0, 0), 1),
         new THREE.Plane(new THREE.Vector3(0, 1, 0), 1),
         new THREE.Plane(new THREE.Vector3(0, -1, 0), 1)
       );
     }
     return planes;
  }, [activeTool, cropPlanes]);

  const hasVertexColors = useMemo(() => {
    if (!meshGeometry) {
      return false;
    }

    const colorAttribute = meshGeometry.getAttribute('color');
    return Boolean(colorAttribute && colorAttribute.count > 0);
  }, [meshGeometry]);

  const resolvedBaseColor = useMemo(() => {
    if (hasVertexColors) {
      return '#ffffff';
    }

    return sourceMaterialColor ?? materialProfile.color;
  }, [hasVertexColors, materialProfile.color, sourceMaterialColor]);

  if (!visible || !Loader || !meshGeometry) return null;

  return (
    <>
      {file.isVoxelized ? (
        <Voxelizer geometry={meshGeometry} file={file} selected={selected} />
      ) : file.isPointCloud ? (
        <points
          ref={ref as any}
          geometry={meshGeometry}
          position={file.position}
          rotation={file.rotation as any}
          scale={file.scale}
          onClick={(e) => { 
            e.stopPropagation(); 
            onClick(e.point, e.nativeEvent); 
          }}
        >
          <pointsMaterial color={resolvedBaseColor} size={0.05} sizeAttenuation={true} vertexColors={hasVertexColors} />
        </points>
      ) : (
        <mesh 
          ref={ref}
          geometry={meshGeometry} 
          position={file.position}
          rotation={file.rotation as any}
          scale={file.scale}
          onClick={(e) => { 
            e.stopPropagation(); 
            onClick(e.point, e.nativeEvent); 
          }}
          onPointerMove={(e) => {
            if (activeTool === 'SCULPT' && selected) {
              e.stopPropagation();
              setHoverPoint(e.point);
              if (isSculpting) {
                sculptMesh(e.point);
              }
            }
          }}
          onPointerOut={() => {
            setHoverPoint(null);
            if (isSculpting) {
              setIsSculpting(false);
              if (onSculptEnd) onSculptEnd();
            }
          }}
          onPointerDown={(e) => {
            if (activeTool === 'SCULPT' && selected && e.button === 0) {
              e.stopPropagation();
              setIsSculpting(true);
              if (onSculptStart) onSculptStart();
              sculptMesh(e.point);
            } else if (e.button === 1 && onMiddleClick) { // Middle button
                e.stopPropagation();
                onMiddleClick(e.point);
            }
          }}
          onPointerUp={() => {
            if (isSculpting) {
              setIsSculpting(false);
              if (onSculptEnd) onSculptEnd();
              if (meshGeometry) saveHistory(meshGeometry);
            }
          }}
          onContextMenu={(e) => {
            e.stopPropagation(); 
            if (onContextMenu) onContextMenu(e);
          }}
          castShadow
          receiveShadow
        >
          <meshPhysicalMaterial 
            color={resolvedBaseColor}
            vertexColors={hasVertexColors}
            metalness={materialProfile.metalness}
            roughness={materialProfile.roughness}
            clearcoat={materialProfile.clearcoat}
            clearcoatRoughness={materialProfile.clearcoatRoughness}
            transmission={materialProfile.transmission ?? 0}
            thickness={materialProfile.thickness ?? 0}
            ior={materialProfile.ior ?? 1.45}
            attenuationColor={materialProfile.attenuationColor ?? resolvedBaseColor}
            attenuationDistance={materialProfile.attenuationDistance ?? 1.5}
            emissive={selected ? SELECTION_EMISSIVE : '#000000'}
            emissiveIntensity={selected ? 0.12 : 0}
            side={THREE.DoubleSide}
            clippingPlanes={clippingPlanes}
            clipIntersection={activeTool === 'BOOLEAN_CUT'}
            clipShadows={true}
            transparent={file.opacity < 1 || file.isSegmented || (materialProfile.opacity ?? 1) < 1 || (materialProfile.transmission ?? 0) > 0}
            opacity={file.opacity * (materialProfile.opacity ?? 1)}
            wireframe={file.wireframe}
          />
          {(activeTool === 'SEGMENT' || file.isSegmented) && selected && (
             <meshBasicMaterial color={file.isSegmented ? SEGMENT_OVERLAY : '#00ff00'} wireframe opacity={0.3} transparent />
          )}
        </mesh>
      )}
      
      {/* Sculpt Brush Visualizer */}
      {activeTool === 'SCULPT' && selected && hoverPoint && (
        <mesh position={hoverPoint}>
          <sphereGeometry args={[0.2, 16, 16]} />
          <meshBasicMaterial color="red" transparent opacity={0.5} depthTest={false} />
        </mesh>
      )}
    </>
  );
});

ModelViewer.displayName = 'ModelViewer';