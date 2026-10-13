#!/usr/bin/env python3
"""Independent STEP measurements through public OCP APIs, never linked into the kernel."""
import argparse
import hashlib
import importlib.metadata
import json
from pathlib import Path

from OCP.Bnd import Bnd_Box
from OCP.BRepAdaptor import BRepAdaptor_Surface
from OCP.BRepBndLib import BRepBndLib
from OCP.BRepCheck import BRepCheck_Analyzer
from OCP.BRepGProp import BRepGProp
from OCP.GeomAbs import GeomAbs_Cylinder
from OCP.GProp import GProp_GProps
from OCP.IFSelect import IFSelect_RetDone
from OCP.Interface import Interface_Static
from OCP.STEPControl import STEPControl_Reader
from OCP.TopAbs import TopAbs_SOLID, TopAbs_SHELL, TopAbs_FACE, TopAbs_EDGE
from OCP.TopExp import TopExp
from OCP.TopoDS import TopoDS
from OCP.TopTools import TopTools_IndexedMapOfShape


def measure(path):
    path = Path(path).resolve()
    reader = STEPControl_Reader()
    if not Interface_Static.SetCVal_s('xstep.cascade.unit', 'MM'):
        raise RuntimeError('Could not select millimetres for independent STEP import')
    if reader.ReadFile(str(path)) != IFSelect_RetDone or reader.TransferRoots() == 0:
        raise RuntimeError(f'STEP import failed: {path}')
    shape = reader.OneShape()
    counts = {}
    cylinders = []
    for name, kind in [('solids', TopAbs_SOLID), ('shells', TopAbs_SHELL),
                       ('faces', TopAbs_FACE), ('edges', TopAbs_EDGE)]:
        indexed = TopTools_IndexedMapOfShape()
        TopExp.MapShapes_s(shape, kind, indexed)
        counts[name] = indexed.Extent()
        if name == 'faces':
            for i in range(1, indexed.Extent() + 1):
                surface = BRepAdaptor_Surface(TopoDS.Face_s(indexed.FindKey(i)))
                if surface.GetType() == GeomAbs_Cylinder:
                    cylinder = surface.Cylinder()
                    location, direction = cylinder.Location(), cylinder.Axis().Direction()
                    cylinders.append(dict(radius_mm=cylinder.Radius(),
                                          axis_location_mm=list(location.Coord()),
                                          axis_direction=list(direction.Coord())))
    valid = BRepCheck_Analyzer(shape).IsValid()
    bounds = Bnd_Box()
    BRepBndLib.AddOptimal_s(shape, bounds, False, False)
    props = GProp_GProps()
    BRepGProp.VolumeProperties_s(shape, props)
    return dict(file=str(path), sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                valid=valid, counts=counts, bounds_mm=list(bounds.Get()), volume_mm3=props.Mass(),
                cylinders=cylinders, ocp_version=importlib.metadata.version('cadquery-ocp'),
                source='Independent OCP STEP reader; not Creo or Gensolid mass property')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('files', nargs='+')
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    results = [measure(path) for path in args.files]
    Path(args.output).write_text(json.dumps(results, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(results, ensure_ascii=False))
    raise SystemExit(0 if all(r['valid'] and r['counts']['solids'] == 1 and r['counts']['shells'] == 1
                             for r in results) else 1)
