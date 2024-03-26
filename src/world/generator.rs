use rand_core::SeedableRng;
use rand_pcg::Pcg32;

type Rng = Pcg32;

pub struct WorldGenerator {
    pub rng: Rng,
}

impl WorldGenerator {
    pub fn new(seed: Option<u64>) -> Self {
        let rng = if seed.is_some() {
            Rng::seed_from_u64(seed.unwrap())
        } else {
            Rng::from_entropy()
        };

        Self { rng }
    }
}

pub trait Generate {
    fn generate(&mut self, generator: &mut WorldGenerator);
}
