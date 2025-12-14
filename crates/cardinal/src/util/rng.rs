use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

pub struct GameRng {
    rng: Pcg64,
}

impl GameRng {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Pcg64::seed_from_u64(seed),
        }
    }

    pub fn generate<T>(&mut self) -> T
    where
        rand::distributions::Standard: rand::distributions::Distribution<T>,
    {
        self.rng.r#gen()
    }
}
