//! Deterministic capacity failure and successful recovery through the public Rust API.
use gensolid_api::{Axis2, ErrorCode, ModelLimits, Session, SessionOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = Session::start_with_limits(
        SessionOptions::default(),
        ModelLimits {
            fins: 48,
            ..ModelLimits::default()
        },
    );
    let basis = Axis2 {
        location: [0.0, 0.0, 4.0],
        axis: [0.0, 0.0, 1.0],
        ref_direction: [1.0, 0.0, 0.0],
    };
    let old = session.body_create_solid_block(40.0, 25.0, 8.0, Some(&basis))?;
    let vertices = session.body_ask_vertices(old)?;
    let points = vertices
        .iter()
        .map(|&v| session.vertex_ask_point(v))
        .collect::<Result<Vec<_>, _>>()?;
    let expected_next = vertices
        .iter()
        .max()
        .copied()
        .ok_or("old body has no vertices")?
        .checked_add(1)
        .ok_or("tag overflow")?;
    let error = match session.body_create_solid_block_with_offset_hole(
        40.0,
        25.0,
        8.0,
        4.0,
        [6.0, -3.0],
        Some(&basis),
    ) {
        Err(error) if error.code() == ErrorCode::CapacityExceeded => error,
        Err(error) => return Err(error.into()),
        Ok(_) => return Err("expected controlled capacity failure".into()),
    };
    let preserved = session.model().bodies().count() == 1
        && session.model().geom().curve_count() == 12
        && session.model().geom().surface_count() == 6
        && session.body_check(old)?.is_empty()
        && session.body_ask_vertices(old)? == vertices
        && vertices
            .iter()
            .map(|&v| session.vertex_ask_point(v))
            .collect::<Result<Vec<_>, _>>()?
            == points;
    if !preserved {
        return Err("failed modelling operation changed existing state".into());
    }
    let later = session.body_create_solid_block(40.0, 25.0, 8.0, Some(&basis))?;
    if later != expected_next || !session.body_check(later)?.is_empty() {
        return Err("recovery failed or a rejected request consumed a tag".into());
    }
    println!("demo=transaction_recovery");
    println!("injected_limit=fins:48");
    println!("failure_code={:?}", error.code());
    println!("existing_body_preserved={preserved}");
    println!("next_tag_preserved={}", later == expected_next);
    println!("committed_body_count={}", session.model().bodies().count());
    println!("recovered_body_faults={}", session.body_check(later)?.len());
    Ok(())
}
