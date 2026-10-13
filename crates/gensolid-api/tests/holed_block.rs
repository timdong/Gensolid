//! Geometric and property checks for the specialised through-hole primitive.

#[cfg(test)]
mod tests {
    use gensolid_api::{Axis2, Precision, Session, SessionOptions};
    use gensolid_check::{CheckOptions, check_body};
    use gensolid_geom::{Curve, Surface};
    use gensolid_math::{Frame, Point3, Vec3};
    use gensolid_topo::{BodyId, Model, Sense};
    use proptest::prelude::*;

    fn basis(origin: [f64; 3], azimuth: f64, elevation: f64) -> Axis2 {
        let (s, c) = azimuth.sin_cos();
        let (se, ce) = elevation.sin_cos();
        Axis2 {
            location: origin,
            axis: [ce * c, ce * s, se],
            ref_direction: [-s, c, 0.0],
        }
    }

    fn sign(sense: Sense) -> f64 {
        match sense {
            Sense::Positive => 1.0,
            Sense::Negative => -1.0,
        }
    }

    // Independent geometric checks supplement the unchanged checker: it does not currently
    // test inner-loop winding or distinguish an inward hole wall from an outward boss.
    fn assert_hole_geometry(model: &Model, body: BodyId, frame: Frame, radius: f64, height: f64) {
        let precision = model.precision();
        let faces = model.body_faces(body).unwrap();
        let mut cylinders = 0;
        let mut plane_rings = 0;
        let mut wall_rings = 0;
        for face_id in faces {
            let face = model.face(face_id).unwrap();
            let surface = model.geom().surface(face.surface.unwrap()).unwrap();
            if let Surface::Cylinder(cylinder) = surface {
                cylinders += 1;
                assert!((cylinder.radius() - radius).abs() <= precision.linear());
                assert!(cylinder.frame().origin().distance(frame.origin()) <= precision.linear());
                assert_eq!(face.sense, Sense::Negative);
            }
            for &lp in &face.loops {
                for &fin_id in &model.loop_(lp).unwrap().fins {
                    let fin = model.fin(fin_id).unwrap();
                    let edge = model.edge(fin.edge).unwrap();
                    let Curve::Circle(circle) = model.geom().curve(edge.curve.unwrap()).unwrap()
                    else {
                        continue;
                    };
                    assert!(edge.start.is_none() && edge.end.is_none());
                    assert!((circle.radius() - radius).abs() <= precision.linear());
                    let local_center = frame.to_local(circle.frame().origin());
                    assert!(local_center.x.hypot(local_center.y) <= precision.linear());
                    assert!((local_center.z.abs() - height / 2.0).abs() <= precision.linear());
                    for angle in [0.0, std::f64::consts::FRAC_PI_2, std::f64::consts::PI] {
                        let point = circle.eval(angle);
                        let local = frame.to_local(point.point);
                        let radial = frame.vector_to_world(Vec3::new(local.x, local.y, 0.0));
                        let tangent = point.d1 * sign(fin.sense);
                        match surface {
                            Surface::Plane(plane) => {
                                let normal = plane.frame().z() * sign(face.sense);
                                // Material lies radially outside the inner circular loop.
                                assert!(normal.cross(tangent).dot(radial) > 0.0);
                            }
                            Surface::Cylinder(cylinder) => {
                                let normal = cylinder
                                    .eval(angle, local_center.z)
                                    .normal(precision)
                                    .unwrap()
                                    * sign(face.sense);
                                assert!(normal.dot(radial) < 0.0);
                                // Material on the wall lies towards its mid-height from either rim.
                                assert!(
                                    normal.cross(tangent).dot(frame.z()) * local_center.z < 0.0
                                );
                            }
                            _ => panic!("unexpected surface on circular rim"),
                        }
                    }
                    match surface {
                        Surface::Plane(_) => plane_rings += 1,
                        Surface::Cylinder(_) => wall_rings += 1,
                        _ => {}
                    }
                }
            }
        }
        assert_eq!((cylinders, plane_rings, wall_rings), (1, 2, 2));
    }

    fn validate(x: f64, y: f64, z: f64, radius: f64, basis: Axis2) {
        let mut session = Session::start(SessionOptions::default());
        let tag = session
            .body_create_solid_block_with_hole(x, y, z, radius, Some(&basis))
            .unwrap();
        assert!(session.body_check(tag).unwrap().is_empty());
        let model = session.model();
        let body = model.bodies().next().unwrap();
        let report = check_body(model, body, &CheckOptions::default()).unwrap();
        assert!(report.is_valid(), "{report:?}");
        let [ox, oy, oz] = basis.location;
        let [ax, ay, az] = basis.axis;
        let [rx, ry, rz] = basis.ref_direction;
        let frame = Frame::new(
            Point3::new(ox, oy, oz),
            Vec3::new(ax, ay, az),
            Vec3::new(rx, ry, rz),
            model.precision(),
        )
        .unwrap();
        assert_hole_geometry(model, body, frame, radius, z);
        // The hole must preserve all eight outer corners under the same rigid transform.
        for vertex in model.body_vertices(body).unwrap() {
            let p = frame.to_local(model.vertex(vertex).unwrap().point);
            for (value, size) in [(p.x, x), (p.y, y), (p.z, z)] {
                assert!((value.abs() - size / 2.0).abs() <= model.precision().linear());
            }
        }
    }

    fn validate_offset(x: f64, y: f64, z: f64, radius: f64, offset: [f64; 2], basis: Axis2) {
        let mut session = Session::start(SessionOptions::default());
        let tag = session
            .body_create_solid_block_with_offset_hole(x, y, z, radius, offset, Some(&basis))
            .unwrap();
        assert!(session.body_check(tag).unwrap().is_empty());
        let model = session.model();
        let body = model.bodies().next().unwrap();
        let report = check_body(model, body, &CheckOptions::default()).unwrap();
        assert!(report.is_valid(), "{report:?}");
        let [ox, oy, oz] = basis.location;
        let [ax, ay, az] = basis.axis;
        let [rx, ry, rz] = basis.ref_direction;
        let block_frame = Frame::new(
            Point3::new(ox, oy, oz),
            Vec3::new(ax, ay, az),
            Vec3::new(rx, ry, rz),
            model.precision(),
        )
        .unwrap();
        let hole_frame = Frame::new(
            block_frame.to_world(Vec3::new(offset[0], offset[1], 0.0)),
            block_frame.z(),
            block_frame.x(),
            model.precision(),
        )
        .unwrap();
        assert_hole_geometry(model, body, hole_frame, radius, z);
        let vertices = model.body_vertices(body).unwrap();
        assert_eq!(vertices.len(), 8);
        for vertex in vertices {
            let point = block_frame.to_local(model.vertex(vertex).unwrap().point);
            for (value, size) in [(point.x, x), (point.y, y), (point.z, z)] {
                assert!((value.abs() - size / 2.0).abs() <= model.precision().linear());
            }
        }
    }

    #[test]
    fn offset_plate_preserves_block_and_positions_hole() {
        let basis = Axis2 {
            location: [0.0, 0.0, 4.0],
            axis: [0.0, 0.0, 1.0],
            ref_direction: [1.0, 0.0, 0.0],
        };
        validate_offset(40.0, 25.0, 8.0, 4.0, [6.0, -3.0], basis);
        validate_offset(40.0, 25.0, 8.0, 4.0, [0.0, 0.0], basis);
        // Local XY offsets must rotate with the block, not remain in world XY.
        validate_offset(
            40.0,
            25.0,
            8.0,
            4.0,
            [6.0, -3.0],
            self::basis([10.0, -20.0, 30.0], 1.0, 0.7),
        );
        let mut model = Model::new(Precision::DEFAULT);
        let body =
            gensolid_topo::make_block_with_hole(&mut model, 40.0, 25.0, 8.0, 4.0, &Frame::WORLD)
                .unwrap();
        assert!(
            check_body(&model, body, &CheckOptions::default())
                .unwrap()
                .is_valid()
        );
    }

    #[test]
    fn rejects_offset_holes_touching_or_crossing_all_four_sides() {
        let mut session = Session::start(SessionOptions::default());
        for offset in [
            [16.0, 0.0],
            [-16.0, 0.0],
            [0.0, 8.5],
            [0.0, -8.5],
            [17.0, 0.0],
            [-17.0, 0.0],
            [0.0, 9.0],
            [0.0, -9.0],
            [f64::NAN, 0.0],
            [0.0, f64::INFINITY],
        ] {
            assert!(
                session
                    .body_create_solid_block_with_offset_hole(40.0, 25.0, 8.0, 4.0, offset, None)
                    .is_err()
            );
            assert_eq!(session.model().bodies().count(), 0);
        }
        let invalid_basis = Axis2 {
            location: [0.0; 3],
            axis: [0.0; 3],
            ref_direction: [1.0, 0.0, 0.0],
        };
        assert!(
            session
                .body_create_solid_block_with_offset_hole(
                    40.0,
                    25.0,
                    8.0,
                    4.0,
                    [6.0, -3.0],
                    Some(&invalid_basis)
                )
                .is_err()
        );
        assert_eq!(session.model().bodies().count(), 0);
        // Rejected requests must not consume tags or contaminate subsequent valid bodies.
        let tag = session
            .body_create_solid_block_with_offset_hole(40.0, 25.0, 8.0, 4.0, [6.0, -3.0], None)
            .unwrap();
        assert_eq!(tag, 1);
        assert!(session.body_check(tag).unwrap().is_empty());
    }

    #[test]
    fn creo_plate_matches_requested_geometry() {
        validate(
            40.0,
            25.0,
            8.0,
            4.0,
            Axis2 {
                location: [0.0, 0.0, 4.0],
                axis: [0.0, 0.0, 1.0],
                ref_direction: [1.0, 0.0, 0.0],
            },
        );
        // API world default, and multiple bodies in a session, must also work.
        let mut session = Session::start(SessionOptions::default());
        let first = session
            .body_create_solid_block(2.0, 2.0, 2.0, None)
            .unwrap();
        let second = session
            .body_create_solid_block_with_hole(40.0, 25.0, 8.0, 4.0, None)
            .unwrap();
        assert!(session.body_check(first).unwrap().is_empty());
        assert!(session.body_check(second).unwrap().is_empty());
        assert_eq!(session.body_ask_faces(second).unwrap().len(), 7);
        assert_eq!(session.body_ask_edges(second).unwrap().len(), 14);
        assert_eq!(session.body_ask_vertices(second).unwrap().len(), 8);
    }

    #[test]
    fn rejects_degenerate_inputs_without_allocating_bodies() {
        let p = Precision::DEFAULT.linear();
        let mut session = Session::start(SessionOptions::default());
        for (x, y, z, radius) in [
            (40.0, 25.0, 8.0, 0.0),
            (40.0, 25.0, 8.0, p),
            (40.0, 25.0, 8.0, -4.0),
            (40.0, 25.0, 8.0, 12.5),
            (40.0, 25.0, 8.0, 13.0),
            (0.0, 25.0, 8.0, 4.0),
            (40.0, -25.0, 8.0, 4.0),
            (40.0, 25.0, p, 4.0),
            (40.0, 25.0, f64::INFINITY, 4.0),
            (40.0, 25.0, 8.0, f64::NAN),
            (Precision::DEFAULT.size(), 25.0, 8.0, 4.0),
        ] {
            assert!(
                session
                    .body_create_solid_block_with_hole(x, y, z, radius, None)
                    .is_err()
            );
            assert_eq!(session.model().bodies().count(), 0);
        }
    }

    proptest! {
        #[test]
        fn random_offset_sizes_and_frames_are_valid(
            x in 2.0f64..100.0, y in 2.0f64..100.0, z in 1.0f64..50.0,
            radius_fraction in 0.05f64..0.95,
            fractions in prop::array::uniform2(-0.95f64..0.95),
            origin in prop::array::uniform3(-100.0f64..100.0),
            azimuth in -std::f64::consts::PI..std::f64::consts::PI,
            elevation in -std::f64::consts::FRAC_PI_2..std::f64::consts::FRAC_PI_2,
        ) {
            let radius = x.min(y) * 0.5 * radius_fraction;
            let offset = [(x * 0.5 - radius) * fractions[0], (y * 0.5 - radius) * fractions[1]];
            validate_offset(x, y, z, radius, offset, basis(origin, azimuth, elevation));
        }

        #[test]
        fn offset_holes_with_near_degenerate_walls_are_valid(
            factor in 2.0f64..32.0, signs in prop::array::uniform2(any::<bool>()),
            thin_radius in any::<bool>(), thin_height in any::<bool>(),
            origin in prop::array::uniform3(-10.0f64..10.0),
            azimuth in -std::f64::consts::PI..std::f64::consts::PI,
            elevation in -std::f64::consts::FRAC_PI_2..std::f64::consts::FRAC_PI_2,
        ) {
            let gap = Precision::DEFAULT.linear() * factor;
            let radius = if thin_radius { gap } else { 4.0 };
            let height = if thin_height { gap } else { 8.0 };
            let offset = [(20.0 - radius - gap) * if signs[0] { 1.0 } else { -1.0 }, (12.5 - radius - gap) * if signs[1] { 1.0 } else { -1.0 }];
            validate_offset(40.0, 25.0, height, radius, offset, basis(origin, azimuth, elevation));
        }

        #[test]
        fn offset_walls_within_precision_are_rejected(
            fraction in 0.0f64..0.95, side in 0usize..4,
        ) {
            let gap = Precision::DEFAULT.linear() * fraction;
            let offset = match side { 0 => [16.0 - gap, 0.0], 1 => [-16.0 + gap, 0.0], 2 => [0.0, 8.5 - gap], _ => [0.0, -8.5 + gap] };
            let mut session = Session::start(SessionOptions::default());
            prop_assert!(session.body_create_solid_block_with_offset_hole(40.0, 25.0, 8.0, 4.0, offset, None).is_err());
            prop_assert_eq!(session.model().bodies().count(), 0);
        }

        #[test]
        fn random_sizes_and_frames_are_valid(
            x in 1.0f64..100.0, y in 1.0f64..100.0, z in 1.0f64..50.0,
            fraction in 0.05f64..0.95,
            origin in prop::array::uniform3(-100.0f64..100.0),
            azimuth in -std::f64::consts::PI..std::f64::consts::PI,
            elevation in -std::f64::consts::FRAC_PI_2..std::f64::consts::FRAC_PI_2,
        ) {
            validate(x, y, z, x.min(y) * 0.5 * fraction, basis(origin, azimuth, elevation));
        }

        #[test]
        fn near_degenerate_valid_radius_wall_and_height(
            factor in 2.0f64..32.0, thin_wall in any::<bool>(), thin_height in any::<bool>(),
            origin in prop::array::uniform3(-10.0f64..10.0),
            azimuth in -std::f64::consts::PI..std::f64::consts::PI,
            elevation in -std::f64::consts::FRAC_PI_2..std::f64::consts::FRAC_PI_2,
        ) {
            let length = Precision::DEFAULT.linear() * factor;
            let radius = if thin_wall { 5.0 - length } else { length };
            let height = if thin_height { length } else { 8.0 };
            validate(10.0, 12.0, height, radius, basis(origin, azimuth, elevation));
        }

        #[test]
        fn walls_within_precision_are_rejected(fraction in 0.0f64..0.95, size in 2.0f64..100.0) {
            let mut session = Session::start(SessionOptions::default());
            let radius = size * 0.5 - Precision::DEFAULT.linear() * fraction;
            prop_assert!(session.body_create_solid_block_with_hole(size, size, 8.0, radius, None).is_err());
            prop_assert_eq!(session.model().bodies().count(), 0);
        }
    }
}
