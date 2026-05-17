import React, { Suspense, useCallback, useMemo } from 'react';
import { Canvas } from '@react-three/fiber';
import { Center, GizmoHelper, GizmoViewport, Grid, Html, Line, OrbitControls, TransformControls } from '@react-three/drei';
import * as THREE from 'three';

import { ModelViewer } from '@/components/ModelViewer';
import { SmileDesignViewer } from '@/components/SmileDesignViewer';
import type { FileData, MeshMetadata, ThemeMode, ToolMode } from '@/types';
import type { TlantiDbPreferences } from '@/stores/tlantidb-case-store';

import { CadSceneLighting, CameraController, CropManager } from './CadScenePrimitives';

export interface CanvasCropPoint {
  x: number;
  y: number;
}

export interface CanvasMeasurementLine {
  start: THREE.Vector3;
  end: THREE.Vector3;
  mid: THREE.Vector3;
  dist: number;
  index: number;
}

export interface CanvasSceneProps {
  files: FileData[];
  selectedId: string | null;
  selectedMesh: THREE.Object3D | null;
  activeTool: ToolMode;
  themeMode: ThemeMode;
  gridVisible: boolean;
  zoomTrigger: number;
  cropStart: CanvasCropPoint | null;
  cropEnd: CanvasCropPoint | null;
  cropPlanes: THREE.Plane[];
  measurePoints: THREE.Vector3[];
  measurementLines: CanvasMeasurementLine[];
  transformSpace: 'local' | 'world';
  snapValue: number | null;
  navigationSensitivity: TlantiDbPreferences['navigationSensitivity'];
  controlScheme: TlantiDbPreferences['controlScheme'];
  orbitRef: React.MutableRefObject<any>;
  transformRef: React.MutableRefObject<any>;
  onSelectedMeshChange: (mesh: THREE.Object3D | null) => void;
  onSetCropPlanes: (planes: THREE.Plane[]) => void;
  onObjectClick: (id: string, point: THREE.Vector3, event: MouseEvent) => void;
  onCenterView: (point: THREE.Vector3) => void;
  onContextMenu: (event: React.MouseEvent | THREE.Event, type: 'object', targetId: string) => void;
  onMetadataLoaded: (id: string, metadata: MeshMetadata) => void;
  onTransformChange: () => void;
  /**
   * V210 — extension slot for overlay children rendered inside the R3F Canvas.
   * Features (jaw motion, distance gradient, crop handles…) can mount their
   * own primitives here without `CanvasScene` knowing about them.
   */
  overlayChildren?: React.ReactNode;
}

export function CanvasScene({
  files,
  selectedId,
  selectedMesh,
  activeTool,
  themeMode,
  gridVisible,
  zoomTrigger,
  cropStart,
  cropEnd,
  cropPlanes,
  measurePoints,
  measurementLines,
  transformSpace,
  snapValue,
  navigationSensitivity,
  controlScheme,
  orbitRef,
  transformRef,
  onSelectedMeshChange,
  onSetCropPlanes,
  onObjectClick,
  onCenterView,
  onContextMenu,
  onMetadataLoaded,
  onTransformChange,
  overlayChildren,
}: CanvasSceneProps) {
  const filesByParent = useMemo(() => {
    const map = new Map<string | null, FileData[]>();

    for (const file of files) {
      const parentId = file.parentId ?? null;
      const list = map.get(parentId) ?? [];
      list.push(file);
      map.set(parentId, list);
    }

    return map;
  }, [files]);

  const filesById = useMemo(() => {
    const map = new Map<string, FileData>();
    for (const file of files) map.set(file.id, file);
    return map;
  }, [files]);

  const isNodeVisible = useCallback((file: FileData): boolean => {
    if (!file.visible) return false;
    if (!file.parentId) return true;

    const parent = filesById.get(file.parentId);
    return parent ? isNodeVisible(parent) : true;
  }, [filesById]);

  const bindSelectedMeshRef = useCallback((fileId: string, node: THREE.Object3D | null) => {
    if (fileId === selectedId && node && selectedMesh?.uuid !== node.uuid) {
      onSelectedMeshChange(node);
    }
  }, [onSelectedMeshChange, selectedId, selectedMesh?.uuid]);

  const renderNode = useCallback((file: FileData, parentOpacity = 1, parentWireframe = false): React.ReactNode => {
    const visible = isNodeVisible(file);
    const effectiveOpacity = file.opacity * parentOpacity;
    const effectiveWireframe = file.wireframe || parentWireframe;

    if (file.type === 'GROUP') {
      return (
        <group
          key={file.id}
          position={file.position}
          rotation={file.rotation as any}
          scale={file.scale}
          ref={(node) => bindSelectedMeshRef(file.id, node)}
        >
          {(filesByParent.get(file.id) ?? []).map((child) => renderNode(child, effectiveOpacity, effectiveWireframe))}
        </group>
      );
    }

    const displayFile = { ...file, opacity: effectiveOpacity, wireframe: effectiveWireframe };

    if (file.type === 'MODEL') {
      return (
        <ModelViewer
          key={file.id}
          file={displayFile}
          ref={(node) => bindSelectedMeshRef(file.id, node)}
          visible={visible}
          selected={selectedId === file.id}
          onClick={(point, event) => onObjectClick(file.id, point, event)}
          onMiddleClick={onCenterView}
          onContextMenu={(event) => onContextMenu(event, 'object', file.id)}
          activeTool={activeTool}
          onMetadataLoaded={onMetadataLoaded}
          onSculptStart={() => {
            if (orbitRef.current) orbitRef.current.enabled = false;
          }}
          onSculptEnd={() => {
            if (orbitRef.current) orbitRef.current.enabled = true;
          }}
          cropPlanes={cropPlanes}
        />
      );
    }

    if (file.type === 'IMAGE') {
      return (
        <SmileDesignViewer
          key={file.id}
          file={displayFile}
          ref={(node) => bindSelectedMeshRef(file.id, node)}
          visible={visible}
          selected={selectedId === file.id}
          opacity={displayFile.opacity}
          onClick={(point, event) => onObjectClick(file.id, point, event)}
          onContextMenu={(event) => onContextMenu(event, 'object', file.id)}
          cropPlanes={cropPlanes}
        />
      );
    }

    return null;
  }, [
    activeTool,
    bindSelectedMeshRef,
    cropPlanes,
    filesByParent,
    isNodeVisible,
    onCenterView,
    onContextMenu,
    onMetadataLoaded,
    onObjectClick,
    orbitRef,
    selectedId,
  ]);

  return (
    <Canvas
      className="h-full min-h-[320px] w-full"
      style={{ width: '100%', height: 'calc(100dvh - 4rem)', minHeight: 320 }}
      shadows
      camera={{ position: [8, 8, 8], fov: 45, far: 100000, near: 0.1 }}
      dpr={[1, 2]}
      gl={{ preserveDrawingBuffer: false }}
    >
      <CameraController zoomTrigger={zoomTrigger} selectedMesh={selectedMesh} />
      <CropManager cropStart={cropStart} cropEnd={cropEnd} setCropPlanes={onSetCropPlanes} />
      <color attach="background" args={[themeMode === 'dark' ? '#111' : '#f0f0f0']} />
      <CadSceneLighting themeMode={themeMode} />
      <group>
        {gridVisible && (
          <Grid
            infiniteGrid
            fadeDistance={50}
            sectionColor={themeMode === 'dark' ? '#2dd4bf' : '#94a3b8'}
            cellColor={themeMode === 'dark' ? '#1f2937' : '#e2e8f0'}
          />
        )}
        <Suspense fallback={null}>
          <Center>
            {(filesByParent.get(null) ?? []).map((file) => renderNode(file))}
          </Center>
        </Suspense>
        {activeTool === 'MEASURE' && (
          <>
            {measurePoints.map((point, index) => (
              <mesh key={`pt-${index}`} position={point}>
                <sphereGeometry args={[0.05, 16, 16]} />
                <meshBasicMaterial color="yellow" depthTest={false} />
              </mesh>
            ))}
            {measurementLines.map((line) => (
              <group key={`line-${line.index}`}>
                <Line points={[line.start, line.end]} color="yellow" lineWidth={2} depthTest={false} />
                <Html position={line.mid}>
                  <div className="whitespace-nowrap rounded border border-yellow-400/50 bg-black/80 px-2 py-1 text-xs text-yellow-400">
                    {line.dist.toFixed(2)} units
                  </div>
                </Html>
              </group>
            ))}
          </>
        )}
        {selectedId && selectedMesh && ['MOVE', 'ROTATE', 'SCALE'].includes(activeTool) && (
          <TransformControls
            ref={transformRef}
            object={selectedMesh}
            mode={activeTool === 'ROTATE' ? 'rotate' : activeTool === 'SCALE' ? 'scale' : 'translate'}
            space={transformSpace}
            size={1.0}
            onMouseDown={() => {
              if (orbitRef.current) orbitRef.current.enabled = false;
            }}
            onMouseUp={() => {
              if (orbitRef.current) orbitRef.current.enabled = true;
              onTransformChange();
            }}
            translationSnap={snapValue || undefined}
            rotationSnap={snapValue ? THREE.MathUtils.degToRad(snapValue * 10) : undefined}
            scaleSnap={snapValue ? snapValue * 0.1 : undefined}
          />
        )}
        {activeTool === 'CLIP' && (
          <mesh position={[0, 0, 0]} rotation={[-Math.PI / 2, 0, 0]}>
            <planeGeometry args={[15, 15]} />
            <meshBasicMaterial color="#FA93FA" opacity={0.1} transparent side={THREE.DoubleSide} />
            <lineSegments>
              <edgesGeometry args={[new THREE.PlaneGeometry(15, 15)]} />
              <lineBasicMaterial color="#FA93FA" />
            </lineSegments>
          </mesh>
        )}
      </group>
      <OrbitControls
        ref={orbitRef}
        makeDefault
        enableDamping
        dampingFactor={0.05}
        zoomSpeed={navigationSensitivity.zoom}
        panSpeed={navigationSensitivity.pan}
        rotateSpeed={navigationSensitivity.rotation}
        minDistance={0.1}
        maxDistance={1000}
        enabled={activeTool !== 'CROP'}
        mouseButtons={{
          LEFT: THREE.MOUSE.ROTATE,
          MIDDLE: controlScheme === 'cad' ? THREE.MOUSE.PAN : THREE.MOUSE.DOLLY,
          RIGHT: controlScheme === 'cad' ? THREE.MOUSE.ROTATE : THREE.MOUSE.PAN,
        }}
      />
      <GizmoHelper alignment="top-right" margin={[80, 80]}>
        <GizmoViewport axisColors={['#FA93FA', '#C967E8', '#983AD6']} labelColor="white" />
      </GizmoHelper>
      {overlayChildren}
    </Canvas>
  );
}
