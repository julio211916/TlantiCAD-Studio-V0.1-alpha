import React from 'react';

export interface CadOverlaysProps {
  preloader?: React.ReactNode;
  moduleDialogs?: React.ReactNode;
  sceneOverlays?: React.ReactNode;
  drawers?: React.ReactNode;
  navigation?: React.ReactNode;
  dicom?: React.ReactNode;
}

export function CadOverlays({
  preloader,
  moduleDialogs,
  sceneOverlays,
  drawers,
  navigation,
  dicom,
}: CadOverlaysProps) {
  return (
    <>
      {preloader}
      {moduleDialogs}
      {sceneOverlays}
      {drawers}
      {navigation}
      {dicom}
    </>
  );
}
