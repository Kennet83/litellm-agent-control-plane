//! Benchmarks for parsing the LiteLLM model cost map.

use divan::{black_box, Bencher};
use litellm_rust::model_prices::ModelCostMap;

static BACKUP_JSON: &str = include_str!("../src/model_prices_backup.json");

fn main() {
    divan::main();
}

/// Parse the full embedded model cost map, as done at startup when the
/// upstream map cannot be fetched.
#[divan::bench(sample_count = 10)]
fn parse_full_cost_map() -> ModelCostMap {
    serde_json::from_str(black_box(BACKUP_JSON)).unwrap()
}

/// Look up pricing for a batch of common models in a loaded map.
#[divan::bench]
fn lookup_models(bencher: Bencher) {
    let map: ModelCostMap = serde_json::from_str(BACKUP_JSON).unwrap();
    let models = [
        "claude-sonnet-4-5",
        "gpt-4o",
        "gpt-4o-mini",
        "gemini/gemini-2.5-pro",
        "unknown-model",
    ];
    bencher.bench_local(|| {
        models
            .iter()
            .filter_map(|model| map.get(black_box(*model)))
            .filter_map(|info| info.input_cost_per_token)
            .sum::<f64>()
    });
}
