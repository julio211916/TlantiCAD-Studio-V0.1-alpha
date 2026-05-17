import React, { memo, useEffect, useMemo } from 'react';
import { Canvas, useThree } from '@react-three/fiber';
import { Grid, Line, OrbitControls } from '@react-three/drei';
import * as THREE from 'three';

import { useCadSceneStore } from '../state/cad-scene-store';
import type { CadSceneEntity } from '../domain/cad-scene';

const ARCH_POSITIONS = new Float32Array([
  -2.6, 0, -0.25, -2.0, 0.2, -0.72, -1.45, 0.24, -0.95,
  -0.82, 0.18, -1.08, -0.25, 0.1, -1.12, 0.25, 0.1, -1.12,
  0.82, 0.18, -1.08, 1.45, 0.24, -0.95, 2.0, 0.2, -0.72,
  2.6, 0, -0.25, 2.15, -0.18, 0.34, 1.42, -0.28, 0.68,
  0.72, -0.32, 0.86, 0, -0.34, 0.92, -0.72, -0.32, 0.86,
  -1.42, -0.28, 0.68, -2.15, -0.18, 0.34, -2.6, 0, -0.25,
  -1.45, 0.74, -0.25, -0.72, 0.58, -0.46, 0, 0.5, -0.52,
  0.72, 0.58, -0.46, 1.45, 0.74, -0.25,
]);

const ARCH_INDICES = new Uint16Array([
  0, 1, 18, 1, 2, 19, 1, 19, 18, 2, 3, 19,
  3, 4, 20, 3, 20, 19, 4, 5, 20, 5, 6, 20,
  6, 7, 21, 6, 21, 20, 7, 8, 21, 8, 9, 22,
  8, 22, 21, 9, 10, 22, 10, 11, 22, 11, 12, 21,
  11, 21, 22, 12, 13, 20, 12, 20, 21, 13, 14, 20,
  14, 15, 19, 14, 19, 20, 15, 16, 18, 15, 18, 19,
  16, 17, 18,
]);

const MARGIN_POINTS: Array<[number, number, number]> = [
  [-2.2, 0.32, -0.44],
  [-1.5, 0.42, -0.72],
  [-0.65, 0.37, -0.86],
  [0, 0.32, -0.9],
  [0.65, 0.37, -0.86],
  [1.5, 0.42, -0.72],
  [2.2, 0.32, -0.44],
];

function useFixtureGeometry() {
  const geometry = useMemo(() => {
    const next = new THREE.BufferGeometry();
    next.setAttribute('position', new THREE.BufferAttribute(ARCH_POSITIONS, 3));
    next.setIndex(new THREE.BufferAttribute(ARCH_INDICES, 1));
    next.computeVertexNormals();
    next.computeBoundingSphere();
    return next;
  }, []);

  useEffect(() => () => geometry.dispose(), [geometry]);

  return geometry;
}

function useFixtureMaterial(entity: CadSceneEntity, selected: boolean) {
  const material = useMemo(() => new THREE.MeshStandardMaterial({
    color: entity.clinicalRole === 'antagonist' ? '#8c6f6a' : '#9fcdbc',
    metalness: 0.05,
    roughness: 0.48,
    transparent: entity.clinicalRole === 'antagonist',
    opacity: entity.clinicalRole === 'antagonist' ? 0.82 : 1,
    emissive: selected ? '#0284c7' : '#000000',
    emissiveIntensity: selected ? 0.28 : 0,
  }), [entity.clinicalRole, selected]);

  useEffect(() => () => material.dispose(), [material]);

  return material;
}

function SceneInvalidator({ revision }: { revision: number }) {
  const invalidate = useThree((state) => state.invalidate);

  useEffect(() => {
    invalidate();
  }, [invalidate, revision]);

  return null;
}

function disposeMaterial(material: THREE.Material | THREE.Material[]) {
  if (Array.isArray(material)) {
    material.forEach((entry) => entry.dispose());
    return;
  }

  material.dispose();
}

function SceneDisposalGuard() {
  const scene = useThree((state) => state.scene);
  const gl = useThree((state) => state.gl);

  useEffect(() => () => {
    scene.traverse((object) => {
      const mesh = object as THREE.Mesh;
      mesh.geometry?.dispose();
      if (mesh.material) {
        disposeMaterial(mesh.material);
      }
    });
    gl.renderLists.dispose();
  }, [gl, scene]);

  return null;
}

const FixtureMesh = memo(function FixtureMesh({
  entity,
  selected,
  onSelect,
}: {
  entity: CadSceneEntity;
  selected: boolean;
  onSelect: (id: string) => void;
}) {
  const geometry = useFixtureGeometry();
  const material = useFixtureMaterial(entity, selected);

  return (
    <mesh
      geometry={geometry}
      material={material}
      position={entity.transform.position}
      rotation={entity.transform.rotation}
      scale={entity.transform.scale}
      onPointerDown={(event) => {
        event.stopPropagation();
        onSelect(entity.id);
      }}
    />
  );
});

const MarginPreview = memo(function MarginPreview({ entity }: { entity: CadSceneEntity | undefined }) {
  if (!entity?.visible) {
    return null;
  }

  return (
    <Line
      points={MARGIN_POINTS}
      color="#22d3ee"
      lineWidth={2}
      dashed={false}
      position={entity.transform.position}
      rotation={entity.transform.rotation}
      scale={entity.transform.scale}
    />
  );
});

function CadSceneContent() {
  const entities = useCadSceneStore((state) => state.entities);
  const selectedEntityId = useCadSceneStore((state) => state.selectedEntityId);
  const gridVisible = useCadSceneStore((state) => state.gridVisible);
  const sceneRevision = useCadSceneStore((state) => state.sceneRevision);
  const selectEntity = useCadSceneStore((state) => state.selectEntity);
  const invalidate = useThree((state) => state.invalidate);
  const meshEntities = entities.filter((entity) => entity.kind === 'mesh');
  const marginEntity = entities.find((entity) => entity.id === 'fixture-margin-preview');

  return (
    <>
      <SceneDisposalGuard />
      <SceneInvalidator revision={sceneRevision} />
      <color attach="background" args={['#f7f9fc']} />
      <ambientLight intensity={1.2} />
      <directionalLight position={[3, 5, 4]} intensity={1.9} />
      <directionalLight position={[-4, 3, -2]} intensity={0.65} />
      {gridVisible && (
        <Grid
          infiniteGrid
          fadeDistance={42}
          cellSize={0.5}
          sectionSize={2}
          sectionColor="#b7c9ea"
          cellColor="#dbe8fb"
        />
      )}
      {meshEntities.map((entity) => (
        entity.visible ? (
          <FixtureMesh
            key={entity.id}
            entity={entity}
            selected={selectedEntityId === entity.id}
            onSelect={selectEntity}
          />
        ) : null
      ))}
      <MarginPreview entity={marginEntity} />
      <OrbitControls
        makeDefault
        enableDamping
        dampingFactor={0.08}
        minDistance={1.2}
        maxDistance={18}
        onChange={() => invalidate()}
      />
    </>
  );
}

export function CadViewportShell() {
  const performance = useCadSceneStore((state) => state.performance);

  return (
    <Canvas
      frameloop="demand"
      dpr={[1, performance.dprMax]}
      gl={{ antialias: true, powerPreference: 'high-performance', preserveDrawingBuffer: false }}
      camera={{ position: [0.25, 2.4, 7.2], fov: 35, near: 0.05, far: 1000 }}
      data-testid="cad-three-viewport"
      data-render-loop="demand"
    >
      <CadSceneContent />
    </Canvas>
  );
}
