use rand::Rng;

pub fn random_float() -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0.0..1.0)
}
