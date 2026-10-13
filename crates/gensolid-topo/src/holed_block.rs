//! Analytic block with a positioned cylindrical through-hole.
//! Constructed from public analytic geometry, without translating another kernel's code.

use gensolid_core::{Error, Result};
use gensolid_geom::{Circle, Curve, Cylinder, Surface};
use gensolid_math::{Frame, Interval, Sign, Vec3};

use crate::{Model, entity::*, make_block};

/// Creates a block centred on `frame.origin()` with a centred through-hole along its
/// local Z axis. `radius` is the hole radius, not its diameter. The radius and the
/// remaining walls must exceed linear precision. This is a specialised primitive,
/// not a general Boolean operation. Full circular edges have no vertices or seams.
pub fn make_block_with_hole(
    model: &mut Model,
    x: f64,
    y: f64,
    z: f64,
    radius: f64,
    frame: &Frame,
) -> Result<BodyId> {
    make_block_with_offset_hole(model, x, y, z, radius, [0.0, 0.0], frame)
}

/// Creates a block with a cylindrical through-hole at local XY coordinates `offset`.
/// The block remains centred on `frame.origin()`; the hole runs parallel to local Z.
/// All four side walls and the radius must certifiably exceed linear precision.
/// Holes touching or crossing the outer boundary are rejected before topology allocation.
pub fn make_block_with_offset_hole(
    model: &mut Model,
    x: f64,
    y: f64,
    z: f64,
    radius: f64,
    offset: [f64; 2],
    frame: &Frame,
) -> Result<BodyId> {
    let precision = *model.precision();
    if ![x, y, z, radius, offset[0], offset[1]]
        .into_iter()
        .all(f64::is_finite)
    {
        return Err(Error::invalid_argument(
            "holed block dimensions and offset must be finite",
        ));
    }
    let r = Interval::point(radius);
    let hx = Interval::point(x) * Interval::point(0.5);
    let hy = Interval::point(y) * Interval::point(0.5);
    let ox = Interval::point(offset[0]);
    let oy = Interval::point(offset[1]);
    let tolerance = Interval::point(precision.linear());
    for (name, gap) in [
        ("radius", r),
        ("+X wall", hx - ox - r),
        ("-X wall", hx + ox - r),
        ("+Y wall", hy - oy - r),
        ("-Y wall", hy + oy - r),
    ] {
        // Reject ambiguous enclosures conservatively; a wall at the precision boundary
        // is not a reliably representable part. No rounded float decides connectivity.
        if (gap - tolerance).sign() != Some(Sign::Positive) {
            return Err(Error::invalid_argument(format!(
                "holed block {name} must certifiably exceed linear precision"
            )));
        }
    }

    // Validate analytic geometry before allocating topology.
    let hole_frame = Frame::new(
        frame.to_world(Vec3::new(offset[0], offset[1], 0.0)),
        frame.z(),
        frame.x(),
        &precision,
    )?;
    let cylinder = Cylinder::new(hole_frame, radius, &precision)?;
    let mut rings = Vec::with_capacity(2);
    for height in [-z / 2.0, z / 2.0] {
        let ring_frame = Frame::new(
            hole_frame.to_world(Vec3::new(0.0, 0.0, height)),
            frame.z(),
            frame.x(),
            &precision,
        )?;
        rings.push(Circle::new(ring_frame, radius, &precision)?);
    }
    let body = make_block(model, x, y, z, frame)?;
    // make_block emits bottom and top first, in shell order.
    let faces = model.body_faces(body)?;
    let bottom = faces
        .first()
        .copied()
        .ok_or_else(|| Error::not_found("block bottom"))?;
    let top = faces
        .get(1)
        .copied()
        .ok_or_else(|| Error::not_found("block top"))?;
    let outer = model
        .face(bottom)?
        .front
        .ok_or_else(|| Error::not_found("block front shell"))?;
    let inner = model
        .face(bottom)?
        .back
        .ok_or_else(|| Error::not_found("block back shell"))?;

    let mut raw = model.raw();
    let surface = raw.geom_mut().add_surface(Surface::Cylinder(cylinder))?;
    let wall = raw.insert_face(Face {
        front: Some(outer),
        back: Some(inner),
        loops: Vec::new(),
        surface: Some(surface),
        sense: Sense::Negative, // Hole normal points towards the empty axis.
    })?;
    raw.shell_mut(outer)?.faces.push(wall);
    raw.shell_mut(inner)?.faces.push(wall);
    for ((plate, plate_sense), ring) in [(bottom, Sense::Positive), (top, Sense::Negative)]
        .into_iter()
        .zip(rings)
    {
        let curve = raw.geom_mut().add_curve(Curve::Circle(ring))?;
        let edge = raw.insert_edge(Edge {
            fins: Vec::new(),
            start: None,
            end: None,
            curve: Some(curve),
            tolerance: None,
        })?;
        // Each circle bounds both an inner loop of the plate and an end of the wall.
        for (face, sense) in [(plate, plate_sense), (wall, plate_sense.reversed())] {
            let lp = raw.insert_loop(Loop {
                face,
                fins: Vec::new(),
            })?;
            let fin = raw.insert_fin(Fin {
                owner: lp,
                edge,
                sense,
            })?;
            raw.edge_mut(edge)?.fins.push(fin);
            raw.loop_mut(lp)?.fins.push(fin);
            raw.face_mut(face)?.loops.push(lp);
        }
    }
    Ok(body)
}
