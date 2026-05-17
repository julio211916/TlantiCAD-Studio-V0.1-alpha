import React, { useMemo, forwardRef } from 'react';
import { useLoader } from '@react-three/fiber';
import { DoubleSide, Event, Mesh, Plane, PlaneGeometry, TextureLoader, Vector3 } from 'three';
import { FileData, ToolMode } from '../types';

interface SmileDesignViewerProps {
  file: FileData;
  visible: boolean;
  selected: boolean;
  opacity: number;
  onClick: (point: Vector3, event: MouseEvent) => void;
  onContextMenu?: (e: Event) => void;
  cropPlanes?: Plane[];
}

export const SmileDesignViewer = forwardRef<Mesh, SmileDesignViewerProps>(({ file, visible, selected, opacity, onClick, onContextMenu, cropPlanes = [] }, ref) => {
  const texture = useLoader(TextureLoader, file.url || '');
  
  // Calculate aspect ratio to maintain image dimensions
  const aspect = useMemo(() => {
    if (texture.image) {
      return texture.image.width / texture.image.height;
    }
    return 1;
  }, [texture]);

  if (!visible) return null;

  return (
    <mesh 
      ref={ref}
      position={file.position}
      rotation={file.rotation as any}
      scale={[file.scale[0] * aspect, file.scale[1], 1]}
      onClick={(e) => {
        e.stopPropagation();
        onClick(e.point, e.nativeEvent);
      }}
      onContextMenu={(e) => {
        e.stopPropagation();
        if (onContextMenu) onContextMenu(e);
      }}
    >
      <planeGeometry args={[10, 10]} />
      <meshBasicMaterial 
        map={texture} 
        transparent 
        opacity={opacity} 
        side={DoubleSide} 
        depthWrite={false} // Allow seeing through to models behind if needed, or overlay behavior
        toneMapped={false}
        clippingPlanes={cropPlanes}
      />
      {selected && (
        <lineSegments>
          <edgesGeometry args={[new PlaneGeometry(10, 10)]} />
          <lineBasicMaterial color="#FA93FA" />
        </lineSegments>
      )}
    </mesh>
  );
});

SmileDesignViewer.displayName = 'SmileDesignViewer';