//! SDK Benchmark Example
//! Run with: cargo run --example benchmark [--network]

use bittensor_rs::utils::balance::{rao_to_tao, tao_to_rao};
use bittensor_rs::utils::weights::{
    convert_weight_uids_and_vals_to_tensor, normalize_max_weight, normalize_weights,
    u16_normalized_float,
};
use bittensor_rs::BittensorClient;
use std::time::Instant;

struct BenchResult {
    name: String,
    mean_ms: f64,
    median_ms: f64,
    min_ms: f64,
    max_ms: f64,
}

fn benchmark<F>(name: &str, iterations: usize, warmup: usize, mut f: F) -> BenchResult
where
    F: FnMut(),
{
    // Warmup
    for _ in 0..warmup {
        f();
    }

    let mut times: Vec<f64> = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let start = Instant::now();
        f();
        let elapsed = start.elapsed();
        times.push(elapsed.as_secs_f64() * 1000.0);
    }

    times.sort_by(|a, b| a.partial_cmp(b).unwrap());

    BenchResult {
        name: name.to_string(),
        mean_ms: times.iter().sum::<f64>() / times.len() as f64,
        median_ms: times[times.len() / 2],
        min_ms: times[0],
        max_ms: times[times.len() - 1],
    }
}

async fn benchmark_async<F, Fut>(
    name: &str,
    iterations: usize,
    warmup: usize,
    mut f: F,
) -> BenchResult
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    for _ in 0..warmup {
        f().await;
    }

    let mut times: Vec<f64> = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let start = Instant::now();
        f().await;
        let elapsed = start.elapsed();
        times.push(elapsed.as_secs_f64() * 1000.0);
    }

    times.sort_by(|a, b| a.partial_cmp(b).unwrap());

    BenchResult {
        name: name.to_string(),
        mean_ms: times.iter().sum::<f64>() / times.len() as f64,
        median_ms: times[times.len() / 2],
        min_ms: times[0],
        max_ms: times[times.len() - 1],
    }
}

fn print_result(r: &BenchResult) {
    println!(
        "{:<50} mean={:>8.3}ms  median={:>8.3}ms  min={:>8.3}ms  max={:>8.3}ms",
        r.name, r.mean_ms, r.median_ms, r.min_ms, r.max_ms
    );
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let run_network = args.iter().any(|a| a == "--network");
    let iterations = 100;

    println!("{}", "=".repeat(100));
    println!("BITTENSOR RUST SDK BENCHMARK");
    println!("{}", "=".repeat(100));

    // =========================================================================
    // LOCAL BENCHMARKS
    // =========================================================================
    println!("\n{}", "=".repeat(60));
    println!("LOCAL OPERATIONS (no network)");
    println!("{}", "=".repeat(60));

    let mut local_results = Vec::new();

    // Balance operations
    println!("\n--- Balance Operations ---");

    let r = benchmark("rao_to_tao (1000 conversions)", iterations, 5, || {
        for i in 0..1000u128 {
            std::hint::black_box(rao_to_tao(1_000_000_000 + i));
        }
    });
    print_result(&r);
    local_results.push(r);

    let r = benchmark("tao_to_rao (1000 conversions)", iterations, 5, || {
        for i in 0..1000 {
            std::hint::black_box(tao_to_rao(1.0 + i as f64 * 0.001));
        }
    });
    print_result(&r);
    local_results.push(r);

    // Weight operations
    println!("\n--- Weight Normalization ---");

    let weights_256: Vec<f32> = (0..256).map(|i| (i as f32) / 256.0).collect();
    let r = benchmark("normalize_max_weight (256 neurons)", iterations, 5, || {
        std::hint::black_box(normalize_max_weight(&weights_256, 0.1));
    });
    print_result(&r);
    local_results.push(r);

    let weights_1000: Vec<f32> = (0..1000).map(|i| (i as f32) / 1000.0).collect();
    let r = benchmark("normalize_max_weight (1000 neurons)", iterations, 5, || {
        std::hint::black_box(normalize_max_weight(&weights_1000, 0.01));
    });
    print_result(&r);
    local_results.push(r);

    let uids: Vec<u64> = (0..256).collect();
    let weights: Vec<f32> = (0..256).map(|i| (i as f32 + 1.0) / 256.0).collect();
    let r = benchmark("normalize_weights (256 neurons)", iterations, 5, || {
        let _ = std::hint::black_box(normalize_weights(&uids, &weights));
    });
    print_result(&r);
    local_results.push(r);

    // Tensor operations
    println!("\n--- Tensor Conversions ---");

    let r = benchmark("u16_normalized_float (10000 ops)", iterations, 5, || {
        for i in 0..10000u16 {
            std::hint::black_box(u16_normalized_float(i));
        }
    });
    print_result(&r);
    local_results.push(r);

    let uids_sparse: Vec<u16> = (0..256).step_by(2).map(|i| i as u16).collect();
    let vals_sparse: Vec<u16> = vec![100; 128];
    let r = benchmark("convert_to_tensor (128 sparse uids)", iterations, 5, || {
        std::hint::black_box(convert_weight_uids_and_vals_to_tensor(
            256,
            &uids_sparse,
            &vals_sparse,
        ));
    });
    print_result(&r);
    local_results.push(r);

    // =========================================================================
    // NETWORK BENCHMARKS
    // =========================================================================
    let mut network_results = Vec::new();

    if run_network {
        println!("\n{}", "=".repeat(60));
        println!("NETWORK OPERATIONS");
        println!("{}", "=".repeat(60));

        println!("\nConnecting to Bittensor network...");
        let connect_start = Instant::now();
        let client = BittensorClient::with_default().await?;
        println!(
            "Connected in {:.2}ms\n",
            connect_start.elapsed().as_secs_f64() * 1000.0
        );

        // Block operations
        println!("--- Block Operations ---");

        let r = benchmark_async("block_number", 10, 2, || async {
            std::hint::black_box(client.block_number().await.ok());
        })
        .await;
        print_result(&r);
        network_results.push(r);

        // Subnet operations
        println!("\n--- Subnet Operations ---");

        let r = benchmark_async("total_subnets", 10, 2, || async {
            std::hint::black_box(
                bittensor_rs::queries::subnets::total_subnets(&client)
                    .await
                    .ok(),
            );
        })
        .await;
        print_result(&r);
        network_results.push(r);

        let r = benchmark_async("subnet_exists (netuid=1)", 10, 2, || async {
            std::hint::black_box(
                bittensor_rs::queries::subnets::subnet_exists(&client, 1)
                    .await
                    .ok(),
            );
        })
        .await;
        print_result(&r);
        network_results.push(r);

        let r = benchmark_async("tempo (netuid=1)", 10, 2, || async {
            std::hint::black_box(bittensor_rs::queries::subnets::tempo(&client, 1).await.ok());
        })
        .await;
        print_result(&r);
        network_results.push(r);

        let r = benchmark_async("difficulty (netuid=1)", 10, 2, || async {
            std::hint::black_box(
                bittensor_rs::queries::subnets::difficulty(&client, 1)
                    .await
                    .ok(),
            );
        })
        .await;
        print_result(&r);
        network_results.push(r);

        let r = benchmark_async("subnet_n (netuid=1)", 10, 2, || async {
            std::hint::black_box(
                bittensor_rs::queries::subnets::subnet_n(&client, 1)
                    .await
                    .ok(),
            );
        })
        .await;
        print_result(&r);
        network_results.push(r);

        // Balance operations
        println!("\n--- Balance Operations ---");

        let test_account = sp_core::crypto::AccountId32::from([0u8; 32]);
        let r = benchmark_async("get_balance", 10, 2, || async {
            std::hint::black_box(
                bittensor_rs::queries::balances::get_balance(&client, &test_account)
                    .await
                    .ok(),
            );
        })
        .await;
        print_result(&r);
        network_results.push(r);

        // Delegate operations
        println!("\n--- Delegate Operations ---");

        let r = benchmark_async("get_delegates", 3, 1, || async {
            std::hint::black_box(
                bittensor_rs::queries::delegates::get_delegates(&client)
                    .await
                    .ok(),
            );
        })
        .await;
        print_result(&r);
        network_results.push(r);
    } else {
        println!("\nSkipping network benchmarks (use --network to enable)");
    }

    // =========================================================================
    // SUMMARY
    // =========================================================================
    println!("\n{}", "=".repeat(100));
    println!("SUMMARY");
    println!("{}", "=".repeat(100));

    let total_local: f64 = local_results.iter().map(|r| r.mean_ms).sum();
    println!("\nLocal operations total mean time: {:.3}ms", total_local);

    if !network_results.is_empty() {
        let total_network: f64 = network_results.iter().map(|r| r.mean_ms).sum();
        println!("Network operations total mean time: {:.3}ms", total_network);
    }

    println!("\n{:<50} {:>12}", "Operation Category", "Mean (ms)");
    println!("{}", "-".repeat(65));

    // Group by category
    let balance_ops: f64 = local_results
        .iter()
        .filter(|r| r.name.contains("rao") || r.name.contains("tao"))
        .map(|r| r.mean_ms)
        .sum();
    println!("{:<50} {:>12.3}", "Balance conversions", balance_ops);

    let weight_ops: f64 = local_results
        .iter()
        .filter(|r| r.name.contains("normalize") || r.name.contains("weights"))
        .map(|r| r.mean_ms)
        .sum();
    println!("{:<50} {:>12.3}", "Weight operations", weight_ops);

    let tensor_ops: f64 = local_results
        .iter()
        .filter(|r| r.name.contains("u16") || r.name.contains("tensor"))
        .map(|r| r.mean_ms)
        .sum();
    println!("{:<50} {:>12.3}", "Tensor conversions", tensor_ops);

    if !network_results.is_empty() {
        let block_ops: f64 = network_results
            .iter()
            .filter(|r| r.name.contains("block"))
            .map(|r| r.mean_ms)
            .sum();
        println!("{:<50} {:>12.3}", "Block queries", block_ops);

        let subnet_ops: f64 = network_results
            .iter()
            .filter(|r| {
                r.name.contains("subnet")
                    || r.name.contains("tempo")
                    || r.name.contains("difficulty")
            })
            .map(|r| r.mean_ms)
            .sum();
        println!("{:<50} {:>12.3}", "Subnet queries", subnet_ops);
    }

    Ok(())
}
