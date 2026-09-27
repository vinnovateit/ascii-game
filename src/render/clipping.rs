use glam::Vec4;
fn distance(v: Vec4, plane: usize) -> f32 {
    match plane {
        0 => v.w + v.x,
        1 => v.w - v.x,
        2 => v.w + v.y,
        3 => v.w - v.y,
        4 => v.z,
        _ => v.w - v.z,
    }
}
/// Sutherland-Hodgman against the RH zero-to-one homogeneous frustum.
/// Fixed scratch arrays avoid per-triangle allocations (triangle + six planes <= 9 vertices).
pub fn clip(triangle: [Vec4; 3]) -> ([Vec4; 12], usize) {
    let mut input = [Vec4::ZERO; 12];
    input[..3].copy_from_slice(&triangle);
    let mut count = 3;
    if triangle.iter().any(|v| !v.is_finite()) {
        return (input, 0);
    }
    for plane in 0..6 {
        if count == 0 {
            break;
        }
        let mut output = [Vec4::ZERO; 12];
        let mut n = 0;
        let mut a = input[count - 1];
        let mut da = distance(a, plane);
        for &b in &input[..count] {
            let db = distance(b, plane);
            if (da >= 0.) != (db >= 0.) {
                output[n] = a.lerp(b, da / (da - db));
                n += 1;
            }
            if db >= 0. {
                output[n] = b;
                n += 1;
            }
            a = b;
            da = db;
        }
        input = output;
        count = n;
    }
    (input, count)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn near_crossing_becomes_quad() {
        let (v, n) = clip([
            Vec4::new(-0.5, 0., -0.5, 1.),
            Vec4::new(0.5, 0., 0.5, 1.),
            Vec4::new(0., 0.5, 0.5, 1.),
        ]);
        assert_eq!(n, 4);
        assert!(v[..n].iter().all(|v| v.z >= 0.));
    }
    #[test]
    fn outside_rejected() {
        assert_eq!(clip([Vec4::new(0., 0., -1., 1.); 3]).1, 0);
    }
    #[test]
    fn all_planes_bound_projection() {
        let (v, n) = clip([
            Vec4::new(-100., 0., 0.5, 1.),
            Vec4::new(100., 0., 0.5, 1.),
            Vec4::new(0., 100., 0.5, 1.),
        ]);
        assert!(n >= 3);
        for p in &v[..n] {
            for plane in 0..6 {
                assert!(distance(*p, plane) >= -1e-5);
            }
        }
    }
}
