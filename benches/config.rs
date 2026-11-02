//! Benchmarks for gateway config parsing.

use divan::black_box;
use litellm_rust::proxy::config::GatewayConfig;

static EXAMPLE_CONFIG: &str = include_str!("../config.yaml.example");

fn main() {
    divan::main();
}

/// Deserialize the example gateway config (models, MCP servers, agents).
#[divan::bench]
fn parse_example_config() -> GatewayConfig {
    serde_yaml::from_str(black_box(EXAMPLE_CONFIG)).unwrap()
}
