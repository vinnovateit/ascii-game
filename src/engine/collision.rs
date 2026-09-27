use crate::world::test_scene::Solid;
use glam::Vec3;
const RADIUS: f32 = 0.23;
pub const EYE: f32 = 1.65;
const HEIGHT: f32 = 1.8;
fn horizontal(p: Vec3, s: &Solid) -> bool {
    p.x + RADIUS > s.min.x
        && p.x - RADIUS < s.max.x
        && p.z + RADIUS > s.min.z
        && p.z - RADIUS < s.max.z
}
fn occupied(p: Vec3, feet: f32, s: &Solid) -> bool {
    horizontal(p, s) && feet + HEIGHT > s.min.y + 1e-4 && feet < s.max.y - 1e-4
}
/// Axis-separated movement with 25cm step-up, a player box, and gravity.
/// Called at 120Hz; movement is subdivided to prevent wall tunnelling.
pub fn move_player(
    position: &mut Vec3,
    velocity: &mut f32,
    delta: Vec3,
    dt: f32,
    solids: &[Solid],
) {
    let steps = (delta.length() / 0.1).ceil().max(1.) as usize;
    for _ in 0..steps {
        for axis in [0, 2] {
            let mut candidate = *position;
            candidate[axis] += delta[axis] / steps as f32;
            let feet = position.y - EYE;
            let mut raised = feet;
            for s in solids {
                if horizontal(candidate, s) && s.max.y > feet && s.max.y <= feet + 0.25 {
                    raised = raised.max(s.max.y);
                }
            }
            if !solids.iter().any(|s| occupied(candidate, raised, s)) {
                candidate.y = raised + EYE;
                *position = candidate;
            }
        }
    }
    let feet = position.y - EYE;
    *velocity -= 9.81 * dt;
    let next = feet + *velocity * dt;
    let floor = solids
        .iter()
        .filter(|s| horizontal(*position, s) && s.max.y <= feet + 1e-4)
        .map(|s| s.max.y)
        .fold(f32::NEG_INFINITY, f32::max);
    if next <= floor {
        position.y = floor + EYE;
        *velocity = 0.;
    } else {
        position.y = next + EYE;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn walk_doorway_climb_and_descend_stairs() {
        let world = crate::world::test_scene::test_scene();
        let mut p = Vec3::new(0., EYE, 5.);
        let mut v = 0.;
        for _ in 0..760 {
            move_player(
                &mut p,
                &mut v,
                Vec3::new(0., 0., -0.025),
                1. / 120.,
                &world.solids,
            );
        }
        assert!(p.z < -13.);
        assert!((p.y - EYE - 1.2).abs() < 0.001);
        for _ in 0..500 {
            move_player(
                &mut p,
                &mut v,
                Vec3::new(0., 0., 0.025),
                1. / 120.,
                &world.solids,
            );
        }
        assert!((p.y - EYE).abs() < 0.001);
    }
    #[test]
    fn walls_stop_large_motion() {
        let world = crate::world::test_scene::test_scene();
        let mut p = Vec3::new(0., EYE, 5.);
        let mut v = 0.;
        move_player(
            &mut p,
            &mut v,
            Vec3::new(20., 0., 0.),
            1. / 120.,
            &world.solids,
        );
        assert!(p.x < 6. - RADIUS);
        assert!(p.x > 5.);
    }
    #[test]
    fn crates_block_walking() {
        let world = crate::world::test_scene::test_scene();
        let mut p = Vec3::new(-3., EYE, 1.);
        let mut v = 0.;
        for _ in 0..200 {
            move_player(
                &mut p,
                &mut v,
                Vec3::new(0., 0., -0.025),
                1. / 120.,
                &world.solids,
            );
        }
        assert!(p.z > -1.);
    }
}
