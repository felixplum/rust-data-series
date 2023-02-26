
pub trait L1 {
    fn compute(v1: &Self, v2: &Self) -> f32;
}

impl L1 for f32 {
    fn compute(v1: &Self, v2: &Self) -> f32 {
        return (v1 - v2).abs();
    }
}

impl L1 for f64 {
    fn compute(v1: &Self, v2: &Self) -> f32 {
        return (v1 - v2).abs() as f32;
    }
}


