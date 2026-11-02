//! Benchmarks for building standard logging payloads on the proxy hot path.

use axum::http::{HeaderMap, HeaderValue};
use divan::{black_box, Bencher};
use litellm_rust::{
    callbacks::standard_logging::{response_value, StandardLoggingPayload},
    model_prices::ModelCostMap,
    sdk::routing::Deployment,
};
use serde_json::{json, Value};

static BACKUP_JSON: &str = include_str!("../src/model_prices_backup.json");

fn main() {
    divan::main();
}

fn deployment() -> Deployment {
    Deployment {
        provider_id: "anthropic".to_owned(),
        upstream_model: "claude-sonnet-4-5".to_owned(),
        api_base: "https://api.anthropic.com".to_owned(),
        api_key: "sk-test".to_owned(),
    }
}

fn headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("x-request-id", HeaderValue::from_static("req-123"));
    headers.insert("authorization", HeaderValue::from_static("Bearer sk-abc"));
    headers.insert("x-forwarded-for", HeaderValue::from_static("10.0.0.1"));
    headers
}

fn request(messages: usize) -> Value {
    let messages: Vec<Value> = (0..messages)
        .map(|i| json!({ "role": "user", "content": format!("message number {i} ").repeat(20) }))
        .collect();
    json!({
        "model": "claude",
        "max_tokens": 1024,
        "messages": messages,
        "secret_fields": { "raw_headers": { "authorization": "Bearer sk-abc" } },
    })
}

fn json_response() -> Value {
    json!({
        "id": "msg_01",
        "type": "message",
        "role": "assistant",
        "content": [{ "type": "text", "text": "Hello! ".repeat(200) }],
        "usage": { "input_tokens": 1200, "output_tokens": 340 },
    })
}

fn sse_response(events: usize) -> Value {
    let mut text = String::new();
    for i in 0..events {
        let delta =
            json!({ "type": "content_block_delta", "delta": { "text": format!("tok{i}") } });
        text.push_str(&format!("event: content_block_delta\ndata: {delta}\n\n"));
    }
    let usage =
        json!({ "type": "message_delta", "usage": { "input_tokens": 900, "output_tokens": 450 } });
    text.push_str(&format!(
        "event: message_delta\ndata: {usage}\n\ndata: [DONE]\n\n"
    ));
    Value::String(text)
}

#[divan::bench(args = [1, 10, 100])]
fn payload_new(bencher: Bencher, messages: usize) {
    let (deployment, headers) = (deployment(), headers());
    bencher
        .with_inputs(|| request(messages))
        .bench_local_values(|request| {
            StandardLoggingPayload::new(
                "anthropic_messages",
                false,
                request,
                "claude",
                &deployment,
                &headers,
            )
        });
}

#[divan::bench]
fn finish_success_json(bencher: Bencher) {
    let prices: ModelCostMap = serde_json::from_str(BACKUP_JSON).unwrap();
    let (deployment, headers) = (deployment(), headers());
    let base = StandardLoggingPayload::new(
        "anthropic_messages",
        false,
        request(1),
        "claude",
        &deployment,
        &headers,
    );
    bencher
        .with_inputs(|| (base.clone(), json_response()))
        .bench_local_values(|(mut payload, response)| {
            payload.finish_success(response, &prices);
            payload
        });
}

#[divan::bench(args = [10, 100, 1000])]
fn finish_success_sse(bencher: Bencher, events: usize) {
    let prices: ModelCostMap = serde_json::from_str(BACKUP_JSON).unwrap();
    let (deployment, headers) = (deployment(), headers());
    let base = StandardLoggingPayload::new(
        "anthropic_messages",
        true,
        request(1),
        "claude",
        &deployment,
        &headers,
    );
    bencher
        .with_inputs(|| (base.clone(), sse_response(events)))
        .bench_local_values(|(mut payload, response)| {
            payload.finish_success(response, &prices);
            payload
        });
}

#[divan::bench]
fn response_value_json(bencher: Bencher) {
    let body = serde_json::to_vec(&json_response()).unwrap();
    bencher.bench_local(|| response_value(black_box(&body), Some("application/json")));
}

#[divan::bench]
fn response_value_text(bencher: Bencher) {
    let Value::String(body) = sse_response(100) else {
        unreachable!()
    };
    bencher.bench_local(|| response_value(black_box(body.as_bytes()), Some("text/event-stream")));
}
