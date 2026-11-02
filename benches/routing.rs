//! Benchmarks for model routing.

use divan::{black_box, Bencher};
use litellm_rust::{
    proxy::config::{GatewayConfig, LiteLlmParams, ModelEntry},
    sdk::{
        providers::{self, ProviderRegistry},
        routing::Router,
    },
};

fn main() {
    divan::main();
}

fn registry() -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();
    providers::register_all(&mut registry);
    registry
}

fn entry(model_name: String, model: String) -> ModelEntry {
    ModelEntry {
        model_name,
        litellm_params: LiteLlmParams {
            model,
            api_key: Some("sk".to_owned()),
            api_base: None,
            extra: Default::default(),
        },
    }
}

fn config(models: usize) -> GatewayConfig {
    let mut model_list: Vec<ModelEntry> = (0..models)
        .map(|i| entry(format!("claude-{i}"), format!("anthropic/claude-model-{i}")))
        .collect();
    model_list.push(entry("anthropic/*".to_owned(), "anthropic/*".to_owned()));
    GatewayConfig {
        model_list,
        mcp_servers: Default::default(),
        general_settings: Default::default(),
        slack: Default::default(),
        agents: Vec::new(),
    }
}

#[divan::bench]
fn register_all_providers() -> ProviderRegistry {
    registry()
}

#[divan::bench(args = [10, 100, 1000])]
fn router_from_config(bencher: Bencher, models: usize) {
    let (config, providers) = (config(models), registry());
    bencher.bench_local(|| Router::from_config(black_box(&config), &providers).unwrap());
}

#[divan::bench]
fn resolve_exact(bencher: Bencher) {
    let router = Router::from_config(&config(100), &registry()).unwrap();
    bencher.bench_local(|| router.resolve(black_box("claude-42")).unwrap());
}

#[divan::bench]
fn resolve_wildcard(bencher: Bencher) {
    let router = Router::from_config(&config(100), &registry()).unwrap();
    bencher.bench_local(|| {
        router
            .resolve(black_box("anthropic/claude-opus-4-8"))
            .unwrap()
    });
}

#[divan::bench]
fn resolve_and_build_messages_url(bencher: Bencher) {
    let router = Router::from_config(&config(100), &registry()).unwrap();
    bencher.bench_local(|| {
        let route = router.resolve(black_box("claude-sonnet-4-5")).unwrap();
        route.deployment.messages_url()
    });
}
