//! Benchmarks for provider credential encryption.

use divan::{black_box, Bencher};
use litellm_rust::proxy::credential_crypto::{decrypt_value, encrypt_value};

const SIGNING_KEY: &str = "sk-master-key-for-benchmarks";

fn main() {
    divan::main();
}

fn secret(len: usize) -> String {
    "s".repeat(len)
}

#[divan::bench(args = [64, 4096])]
fn encrypt(bencher: Bencher, len: usize) {
    let value = secret(len);
    bencher.bench_local(|| encrypt_value(black_box(&value), SIGNING_KEY).unwrap());
}

#[divan::bench(args = [64, 4096])]
fn decrypt(bencher: Bencher, len: usize) {
    let encrypted = encrypt_value(&secret(len), SIGNING_KEY).unwrap();
    bencher.bench_local(|| decrypt_value(black_box(&encrypted), SIGNING_KEY).unwrap());
}
