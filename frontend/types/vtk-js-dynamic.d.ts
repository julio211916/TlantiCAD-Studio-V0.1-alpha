declare module '@kitware/vtk.js/*' {
  const vtkModule: any;
  export default vtkModule;
  export const Mode: any;
  export const vtkImageData: any;
  export const vtkAlgorithm: any;
  export const vtkObject: any;
  export const vtkSubscription: any;
  export type Vector2 = [number, number];
  export type Vector3 = [number, number, number];
  export type Bounds = [number, number, number, number, number, number];
  export type RGBAColor = [number, number, number, number];
}
