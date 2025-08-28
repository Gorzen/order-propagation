use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    /// Latency between nodes in milliseconds
    latency_ms: u64,
    /// Packets' time to live
    pub time_to_live: u64,
    /// Number of nodes in the network
    pub num_nodes: u64,
    /// Number of connected neighbors in the network
    pub num_neighbors: u64,
    /// Number of nodes to gossip to
    pub num_peers: u64,
    /// Number of packets to send from main to the network
    pub num_runs: u64,
}

impl Config {
    pub fn load() -> Result<Config, config::ConfigError> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config.toml"))
            .build()?;

        Config::validate_config(config.try_deserialize()?)
    }

    fn validate_config(config: Config) -> Result<Config, config::ConfigError> {
        if config.num_peers == 0 {
            Err(config::ConfigError::Message(
                "num_peers can't be 0".to_string(),
            ))
        } else if config.num_peers > config.num_neighbors {
            Err(config::ConfigError::Message(format!(
                "More peers (= {}) than neighbors available (= {})",
                config.num_peers, config.num_neighbors
            )))
        } else if config.num_nodes <= config.num_neighbors {
            Err(config::ConfigError::Message(format!(
                "There must be more nodes (= {}) than neighbors (= {})",
                config.num_nodes, config.num_neighbors
            )))
        } else {
            Ok(config)
        }
    }

    pub fn latency(&self) -> Duration {
        Duration::from_millis(self.latency_ms)
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;

    #[test]
    fn test_config() {
        let config = Config {
            latency_ms: 0,
            time_to_live: 0,
            num_nodes: 1_000,
            num_neighbors: 24,
            num_peers: 8,
            num_runs: 0,
        };

        assert!(Config::validate_config(config.clone()).is_ok());

        // Not enough nodes for neighbors
        let mut config_0 = config.clone();
        config_0.num_nodes = 24;
        let e_0 = Config::validate_config(config_0.clone()).err().unwrap();
        assert_eq!(
            e_0.to_string(),
            "There must be more nodes (= 24) than neighbors (= 24)"
        );

        // Not enough neighbors for peers
        let mut config_1 = config.clone();
        config_1.num_neighbors = 7;
        let e_1 = Config::validate_config(config_1.clone()).err().unwrap();
        assert_eq!(
            e_1.to_string(),
            "More peers (= 8) than neighbors available (= 7)"
        );

        // Not enough peers
        let mut config_2 = config.clone();
        config_2.num_peers = 0;
        let e_2 = Config::validate_config(config_2.clone()).err().unwrap();
        assert_eq!(e_2.to_string(), "num_peers can't be 0");
    }
}
