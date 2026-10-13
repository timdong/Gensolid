//! Analytic 40 x 25 x 8 mm plate with a diameter-8 through-hole.
//! Optional CLI arguments specify the local XY hole offset: `-- 6 -3`.
//! Creo imported an externally generated plate through J-Link and exported it to STEP;
//! this example constructs the corresponding geometry directly in Gensolid.

use gensolid_api::{Axis2, Session, SessionOptions};
use gensolid_geom::{Curve, Surface};
use gensolid_topo::Sense;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const WIDTH: f64 = 40.0;
    const DEPTH: f64 = 25.0;
    const HEIGHT: f64 = 8.0;
    const RADIUS: f64 = 4.0;
    let args: Vec<String> = std::env::args().skip(1).collect();
    let offset = match args.as_slice() {
        [] => [0.0, 0.0],
        [x, y] => [x.parse::<f64>()?, y.parse::<f64>()?],
        _ => return Err("Usage: creo_compare [hole_offset_x_mm hole_offset_y_mm]".into()),
    };

    let mut session = Session::start(SessionOptions::default());
    // Creo's probe uses the footprint on Z=0 and the top face on Z=8.
    let basis = Axis2 {
        location: [0.0, 0.0, HEIGHT / 2.0],
        axis: [0.0, 0.0, 1.0],
        ref_direction: [1.0, 0.0, 0.0],
    };
    let body = session.body_create_solid_block_with_offset_hole(
        WIDTH,
        DEPTH,
        HEIGHT,
        RADIUS,
        offset,
        Some(&basis),
    )?;
    let faces = session.body_ask_faces(body)?;
    let edges = session.body_ask_edges(body)?;
    let vertices = session.body_ask_vertices(body)?;
    let faults = session.body_check(body)?;

    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for vertex in &vertices {
        let point = session.vertex_ask_point(*vertex)?;
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }

    let model = session.model();
    let body_id = model.bodies().next().ok_or("demo body missing")?;
    let mut cylinder_count = 0;
    let mut ring_count = 0;
    let tolerance = model.precision().linear();
    let mut hole_ok = true;
    for face_id in model.body_faces(body_id)? {
        let face = model.face(face_id)?;
        let surface = model
            .geom()
            .surface(face.surface.ok_or("face surface missing")?)?;
        if let Surface::Cylinder(cylinder) = surface {
            cylinder_count += 1;
            hole_ok &= (cylinder.radius() - RADIUS).abs() <= tolerance
                && face.sense == Sense::Negative
                && cylinder
                    .frame()
                    .origin()
                    .distance(gensolid_math::Point3::new(
                        offset[0],
                        offset[1],
                        HEIGHT / 2.0,
                    ))
                    <= tolerance
                && (cylinder.frame().z() - gensolid_math::Vec3::Z).length()
                    <= model.precision().angular();
        }
    }
    for edge_id in model.body_edges(body_id)? {
        let edge = model.edge(edge_id)?;
        if let Curve::Circle(circle) = model
            .geom()
            .curve(edge.curve.ok_or("edge curve missing")?)?
        {
            ring_count += 1;
            let center = circle.frame().origin();
            hole_ok &= (circle.radius() - RADIUS).abs() <= tolerance
                && (center.x - offset[0]).hypot(center.y - offset[1]) <= tolerance
                && center.z.abs().min((center.z - HEIGHT).abs()) <= tolerance;
        }
    }
    hole_ok &= cylinder_count == 1 && ring_count == 2;

    println!("demo=creo_compare");
    println!("body_tag={body}");
    println!("expected_size_mm={WIDTH}x{DEPTH}x{HEIGHT}");
    println!("expected_hole_diameter_mm={}", RADIUS * 2.0);
    println!("hole_offset_xy_mm=[{:.3},{:.3}]", offset[0], offset[1]);
    println!("bbox_min_mm=[{:.3},{:.3},{:.3}]", min[0], min[1], min[2]);
    println!("bbox_max_mm=[{:.3},{:.3},{:.3}]", max[0], max[1], max[2]);
    println!("face_count={}", faces.len());
    println!("edge_count={}", edges.len());
    println!("vertex_count={}", vertices.len());
    println!("body_check_faults={}", faults.len());
    println!("cylindrical_wall_count={cylinder_count}");
    println!("circular_rim_count={ring_count}");
    println!("hole_matches_expected={hole_ok}");
    // Reference formula, not a call to a kernel mass-properties implementation.
    println!(
        "analytical_reference_volume_mm3={:.6}",
        (WIDTH * DEPTH - std::f64::consts::PI * RADIUS * RADIUS) * HEIGHT
    );
    for fault in &faults {
        println!("fault={fault:?}");
    }

    let size_ok = min
        .iter()
        .zip([-WIDTH / 2.0, -DEPTH / 2.0, 0.0])
        .chain(max.iter().zip([WIDTH / 2.0, DEPTH / 2.0, HEIGHT]))
        .all(|(actual, expected)| (actual - expected).abs() <= tolerance);
    println!("bbox_matches_expected={size_ok}");
    if !size_ok || !hole_ok || !faults.is_empty() {
        return Err("Gensolid plate did not satisfy the expected geometry checks".into());
    }
    Ok(())
}
