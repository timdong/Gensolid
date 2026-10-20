//! Capacity failures on real modelling paths, independently checked after rollback.
#[cfg(test)]
mod tests {
    use gensolid_api::{Axis2, ErrorCode, ModelLimits, Precision, Session, SessionOptions};
    use gensolid_check::{CheckOptions, check_body};
    use proptest::prelude::*;

    fn frame(origin: [f64; 3], azimuth: f64, elevation: f64) -> Axis2 {
        let (s, c) = azimuth.sin_cos();
        let (se, ce) = elevation.sin_cos();
        Axis2 {
            location: origin,
            axis: [ce * c, ce * s, se],
            ref_direction: [-s, c, 0.0],
        }
    }

    fn check_all(session: &Session) {
        for body in session.model().bodies() {
            let result = check_body(session.model(), body, &CheckOptions::default()).unwrap();
            assert!(result.faults.is_empty(), "{:?}", result.faults);
            assert_eq!(result.unchecked, 0);
        }
    }

    fn fail_and_recover(x: f64, y: f64, z: f64, radius: f64, basis: Axis2) {
        let mut session = Session::start_with_limits(
            SessionOptions::default(),
            ModelLimits {
                fins: 48,
                ..ModelLimits::default()
            },
        );
        let old = session
            .body_create_solid_block(x, y, z, Some(&basis))
            .unwrap();
        let old_faces = session.body_ask_faces(old).unwrap();
        let old_edges = session.body_ask_edges(old).unwrap();
        let old_vertices = session.body_ask_vertices(old).unwrap();
        let old_points = old_vertices
            .iter()
            .map(|&v| session.vertex_ask_point(v).unwrap())
            .collect::<Vec<_>>();
        let geometry = (
            session
                .model()
                .geom()
                .curves()
                .map(|(id, c)| (id, c.clone()))
                .collect::<Vec<_>>(),
            session
                .model()
                .geom()
                .surfaces()
                .map(|(id, s)| (id, s.clone()))
                .collect::<Vec<_>>(),
        );
        let expected_next_tag = *old_vertices.iter().max().unwrap() + 1;
        let error = session
            .body_create_solid_block_with_offset_hole(
                x,
                y,
                z,
                radius,
                [x * 0.1, -y * 0.1],
                Some(&basis),
            )
            .unwrap_err();
        assert_eq!(error.code(), ErrorCode::CapacityExceeded);
        assert_eq!(session.model().bodies().count(), 1);
        assert_eq!(session.body_ask_faces(old).unwrap(), old_faces);
        assert_eq!(session.body_ask_edges(old).unwrap(), old_edges);
        assert_eq!(session.body_ask_vertices(old).unwrap(), old_vertices);
        assert_eq!(
            old_vertices
                .iter()
                .map(|&v| session.vertex_ask_point(v).unwrap())
                .collect::<Vec<_>>(),
            old_points
        );
        assert_eq!(
            session
                .model()
                .geom()
                .curves()
                .map(|(id, c)| (id, c.clone()))
                .collect::<Vec<_>>(),
            geometry.0
        );
        assert_eq!(
            session
                .model()
                .geom()
                .surfaces()
                .map(|(id, s)| (id, s.clone()))
                .collect::<Vec<_>>(),
            geometry.1
        );
        check_all(&session);
        assert_eq!(
            session
                .body_create_solid_block(x, y, z, Some(&basis))
                .unwrap(),
            expected_next_tag
        );
        assert_eq!(session.model().bodies().count(), 2);
        check_all(&session);
    }

    #[test]
    fn late_hole_failure_preserves_queries_and_next_tag() {
        fail_and_recover(40.0, 25.0, 8.0, 4.0, frame([10.0, 20.0, 30.0], 0.8, -0.3));
    }

    #[test]
    fn geometry_failure_after_allocation_preserves_existing_body() {
        let mut session = Session::start(SessionOptions::default());
        let old = session
            .body_create_solid_block(40.0, 25.0, 8.0, None)
            .unwrap();
        // Finite coordinates large enough that local corners collapse when rounded.
        let huge = Precision::DEFAULT.size() / f64::EPSILON * Precision::DEFAULT.size();
        let basis = Axis2 {
            location: [huge; 3],
            axis: [0.0, 0.0, 1.0],
            ref_direction: [1.0, 0.0, 0.0],
        };
        let error = session
            .body_create_solid_block(30.0, 20.0, 5.0, Some(&basis))
            .unwrap_err();
        assert_eq!(error.code(), ErrorCode::Degenerate);
        assert_eq!(session.model().bodies().count(), 1);
        assert_eq!(session.model().geom().curve_count(), 12);
        assert_eq!(session.model().geom().surface_count(), 6);
        assert!(session.body_check(old).unwrap().is_empty());
        assert_eq!(
            session
                .body_create_solid_block(30.0, 20.0, 5.0, None)
                .unwrap(),
            2
        );
        check_all(&session);
    }

    proptest! {
        #[test]
        fn random_sizes_and_frames_recover(
            x in 1.0f64..100.0, y in 1.0f64..100.0, z in 1.0f64..100.0,
            ox in -50.0f64..50.0, oy in -50.0f64..50.0, oz in -50.0f64..50.0,
            azimuth in -3.0f64..3.0, elevation in -1.5f64..1.5,
        ) {
            fail_and_recover(x,y,z,x.min(y)*0.125,frame([ox,oy,oz],azimuth,elevation));
        }

        #[test]
        fn near_precision_height_failure_recovers(
            factor in 16.0f64..256.0, azimuth in -3.0f64..3.0, elevation in -1.5f64..1.5,
        ) {
            fail_and_recover(40.0,25.0,Precision::DEFAULT.linear()*factor,4.0,
                frame([0.0,0.0,0.0],azimuth,elevation));
        }
    }
}
