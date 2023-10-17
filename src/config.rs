pub struct MonteCarloConfig {
    pub iterations: i64,
}

impl MonteCarloConfig {
    const ITERATIONS: &str = "MONTE_CARLO_ITERATIONS";
    pub fn load() -> Self {
        Self {
            iterations: std::env::var(MonteCarloConfig::ITERATIONS)
                .unwrap()
                .parse()
                .unwrap(),
        }
    }

    #[cfg(test)]
    pub fn default() -> Self {
        Self { iterations: 6000 }
    }
}
pub struct MiniMaxConfig {
    pub depth: usize,
}

impl MiniMaxConfig {
    const MINIMAX_DEPTH: &str = "MINIMAX_DEPTH";
    pub fn load() -> Self {
        Self {
            depth: std::env::var(MiniMaxConfig::MINIMAX_DEPTH)
                .unwrap()
                .parse()
                .unwrap(),
        }
    }

    #[cfg(test)]
    pub fn default() -> Self {
        Self { depth: 11 }
    }
}

pub enum Engine {
    MonteCarlo(MonteCarloConfig),
    MiniMax(MiniMaxConfig),
}

impl Engine {
    const ENGINE: &str = "ENGINE";
    const MONTECARLO: &str = "monte_carlo";
    const MINIMAX: &str = "mini_max";

    pub fn load() -> Self {
        let engine_type = std::env::var(Engine::ENGINE).unwrap();

        if engine_type == Engine::MONTECARLO {
            return Engine::MonteCarlo(MonteCarloConfig::load());
        }
        if engine_type == Engine::MINIMAX {
            return Engine::MiniMax(MiniMaxConfig::load());
        }
        panic!("Invalid engine type configured")
    }
}

pub struct Config {
    pub engine: Engine,
}

impl Config {
    pub fn load() -> Self {
        Config {
            engine: Engine::load(),
        }
    }
}
