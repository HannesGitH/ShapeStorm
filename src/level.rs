use fastrand;

struct LevelManager {
    hardness : f32,
    rng : fastrand::Rng,
}

impl LevelManager {
    pub fn new(hardness:f32, seed: u64) -> Self {
        let rng = fastrand::Rng::with_seed(seed);
        Self {
            hardness,
            rng,
        }
    }
}