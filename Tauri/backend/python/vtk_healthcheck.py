#!/usr/bin/env python3
import json
import sys

import vtk


def main() -> int:
    sphere = vtk.vtkSphereSource()
    sphere.SetThetaResolution(12)
    sphere.SetPhiResolution(8)
    sphere.Update()
    polydata = sphere.GetOutput()

    payload = {
        "ok": True,
        "engine": "vtk",
        "vtkVersion": vtk.vtkVersion.GetVTKVersion(),
        "pythonExecutable": sys.executable,
        "points": int(polydata.GetNumberOfPoints()),
        "polys": int(polydata.GetNumberOfPolys()),
        "supportsPolyData": polydata.GetNumberOfPoints() > 0 and polydata.GetNumberOfPolys() > 0,
    }
    print(json.dumps(payload))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
