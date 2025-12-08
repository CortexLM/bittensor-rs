#!/usr/bin/env python3
"""
Full benchmark for Python Bittensor SDK - Local and Network operations.
"""

import sys
import time
import json
import asyncio
import statistics

print("Loading Python Bittensor SDK...")
load_start = time.perf_counter()

import bittensor
from bittensor.core.async_subtensor import AsyncSubtensor
from bittensor.utils import balance as balance_utils
from bittensor.utils import weight_utils
import numpy as np

load_time = time.perf_counter() - load_start
print(f"SDK loaded in {load_time*1000:.2f}ms (excluded from benchmark)\n")


def benchmark_function(func, iterations=100, warmup=5):
    """Benchmark a sync function."""
    for _ in range(warmup):
        func()
    
    times = []
    for _ in range(iterations):
        start = time.perf_counter()
        func()
        end = time.perf_counter()
        times.append((end - start) * 1000)
    
    return times


async def benchmark_async_function(func, iterations=10, warmup=2):
    """Benchmark an async function."""
    for _ in range(warmup):
        await func()
    
    times = []
    for _ in range(iterations):
        start = time.perf_counter()
        await func()
        end = time.perf_counter()
        times.append((end - start) * 1000)
    
    return times


class BenchmarkResult:
    def __init__(self, name, times, iterations):
        self.name = name
        self.times = times
        self.iterations = iterations
    
    @property
    def mean_ms(self):
        return statistics.mean(self.times)
    
    @property
    def median_ms(self):
        return statistics.median(self.times)
    
    @property
    def min_ms(self):
        return min(self.times)
    
    @property
    def max_ms(self):
        return max(self.times)


def run_local_benchmarks(iterations=100):
    """Run local (non-network) benchmarks."""
    results = []
    
    print("=" * 60)
    print("PYTHON SDK LOCAL OPERATIONS")
    print("=" * 60)
    
    # Balance Operations
    print("\n--- Balance Operations ---")
    
    def rao_to_tao_1000():
        for i in range(1000):
            _ = balance_utils.Balance.from_rao(1_000_000_000 + i).tao
    
    times = benchmark_function(rao_to_tao_1000, iterations)
    r = BenchmarkResult("rao_to_tao (1000 ops)", times, iterations)
    results.append(r)
    print(f"rao_to_tao: mean={r.mean_ms:.3f}ms, median={r.median_ms:.3f}ms")
    
    def tao_to_rao_1000():
        for i in range(1000):
            _ = balance_utils.Balance.from_tao(1.0 + i * 0.001).rao
    
    times = benchmark_function(tao_to_rao_1000, iterations)
    r = BenchmarkResult("tao_to_rao (1000 ops)", times, iterations)
    results.append(r)
    print(f"tao_to_rao: mean={r.mean_ms:.3f}ms, median={r.median_ms:.3f}ms")
    
    # Weight Operations
    print("\n--- Weight Operations ---")
    
    weights_256 = np.random.rand(256).astype(np.float32)
    def normalize_256():
        weight_utils.normalize_max_weight(weights_256.copy(), limit=0.1)
    
    times = benchmark_function(normalize_256, iterations)
    r = BenchmarkResult("normalize_max_weight (256 neurons)", times, iterations)
    results.append(r)
    print(f"normalize_max_weight (256): mean={r.mean_ms:.3f}ms, median={r.median_ms:.3f}ms")
    
    uids_256 = np.arange(256, dtype=np.int64)
    weights_f32 = np.random.rand(256).astype(np.float32)
    def convert_256():
        weight_utils.convert_weights_and_uids_for_emit(uids_256.copy(), weights_f32.copy())
    
    times = benchmark_function(convert_256, iterations)
    r = BenchmarkResult("convert_weights_and_uids (256 neurons)", times, iterations)
    results.append(r)
    print(f"convert_weights_and_uids (256): mean={r.mean_ms:.3f}ms, median={r.median_ms:.3f}ms")
    
    weights_1000 = np.random.rand(1000).astype(np.float32)
    def normalize_1000():
        weight_utils.normalize_max_weight(weights_1000.copy(), limit=0.01)
    
    times = benchmark_function(normalize_1000, iterations)
    r = BenchmarkResult("normalize_max_weight (1000 neurons)", times, iterations)
    results.append(r)
    print(f"normalize_max_weight (1000): mean={r.mean_ms:.3f}ms, median={r.median_ms:.3f}ms")
    
    # Normalization Operations
    print("\n--- Normalization Operations ---")
    
    def u16_normalize_10000():
        for i in range(10000):
            _ = i / 65535.0
    
    times = benchmark_function(u16_normalize_10000, iterations)
    r = BenchmarkResult("u16_normalized_float (10000 ops)", times, iterations)
    results.append(r)
    print(f"u16_normalized: mean={r.mean_ms:.3f}ms, median={r.median_ms:.3f}ms")
    
    def convert_tensor():
        n = 256
        uids = list(range(0, 256, 2))
        vals = [100] * len(uids)
        weight_utils.convert_weight_uids_and_vals_to_tensor(n, uids, vals)
    
    times = benchmark_function(convert_tensor, iterations)
    r = BenchmarkResult("convert_to_tensor (128 uids)", times, iterations)
    results.append(r)
    print(f"convert_to_tensor: mean={r.mean_ms:.3f}ms, median={r.median_ms:.3f}ms")
    
    return results


async def run_network_benchmarks(iterations=10):
    """Run network operation benchmarks."""
    results = []
    
    print("\n" + "=" * 60)
    print("PYTHON SDK NETWORK OPERATIONS")
    print("=" * 60)
    
    print("\nConnecting to Bittensor network...")
    connect_start = time.perf_counter()
    subtensor = AsyncSubtensor()
    await subtensor.initialize()
    connect_time = (time.perf_counter() - connect_start) * 1000
    print(f"Connected in {connect_time:.2f}ms")
    
    # Block number
    print("\n--- Block Operations ---")
    
    async def get_block():
        return await subtensor.get_current_block()
    
    times = await benchmark_async_function(get_block, iterations, warmup=2)
    r = BenchmarkResult("get_current_block", times, iterations)
    results.append(r)
    print(f"get_current_block: mean={r.mean_ms:.3f}ms, median={r.median_ms:.3f}ms")
    
    # Subnet info
    print("\n--- Subnet Operations ---")
    
    async def get_total():
        return await subtensor.get_total_subnets()
    
    times = await benchmark_async_function(get_total, iterations, warmup=2)
    r = BenchmarkResult("get_total_subnets", times, iterations)
    results.append(r)
    print(f"get_total_subnets: mean={r.mean_ms:.3f}ms, median={r.median_ms:.3f}ms")
    
    async def subnet_exists():
        return await subtensor.subnet_exists(netuid=1)
    
    times = await benchmark_async_function(subnet_exists, iterations, warmup=2)
    r = BenchmarkResult("subnet_exists", times, iterations)
    results.append(r)
    print(f"subnet_exists: mean={r.mean_ms:.3f}ms, median={r.median_ms:.3f}ms")
    
    async def get_tempo():
        return await subtensor.tempo(netuid=1)
    
    times = await benchmark_async_function(get_tempo, iterations, warmup=2)
    r = BenchmarkResult("tempo", times, iterations)
    results.append(r)
    print(f"tempo: mean={r.mean_ms:.3f}ms, median={r.median_ms:.3f}ms")
    
    # Neuron info
    print("\n--- Neuron Operations ---")
    
    async def get_n():
        return await subtensor.subnetwork_n(netuid=1)
    
    times = await benchmark_async_function(get_n, iterations, warmup=2)
    r = BenchmarkResult("subnetwork_n", times, iterations)
    results.append(r)
    print(f"subnetwork_n: mean={r.mean_ms:.3f}ms, median={r.median_ms:.3f}ms")
    
    # Delegates (limited iterations)
    print("\n--- Delegate Operations ---")
    
    async def get_delegates():
        return await subtensor.get_delegates()
    
    times = await benchmark_async_function(get_delegates, min(iterations, 3), warmup=1)
    r = BenchmarkResult("get_delegates", times, min(iterations, 3))
    results.append(r)
    print(f"get_delegates: mean={r.mean_ms:.3f}ms, median={r.median_ms:.3f}ms")
    
    await subtensor.close()
    return results


async def main():
    print("=" * 60)
    print("BITTENSOR PYTHON SDK FULL BENCHMARK")
    print("=" * 60)
    
    # Local benchmarks
    local_results = run_local_benchmarks(iterations=100)
    
    # Network benchmarks
    network_results = []
    try:
        network_results = await run_network_benchmarks(iterations=10)
    except Exception as e:
        print(f"\nNetwork benchmark failed: {e}")
    
    # Summary and save
    print("\n" + "=" * 80)
    print("SUMMARY")
    print("=" * 80)
    
    print(f"\n{'Operation':<50} {'Mean (ms)':>12} {'Median (ms)':>12}")
    print("-" * 75)
    
    print("\nLocal Operations:")
    for r in local_results:
        print(f"{r.name:<50} {r.mean_ms:>12.3f} {r.median_ms:>12.3f}")
    
    if network_results:
        print("\nNetwork Operations:")
        for r in network_results:
            print(f"{r.name:<50} {r.mean_ms:>12.3f} {r.median_ms:>12.3f}")
    
    # Save results
    all_results = {
        "local": [
            {
                "name": r.name,
                "mean_ms": r.mean_ms,
                "median_ms": r.median_ms,
                "min_ms": r.min_ms,
                "max_ms": r.max_ms,
                "iterations": r.iterations
            }
            for r in local_results
        ],
        "network": [
            {
                "name": r.name,
                "mean_ms": r.mean_ms,
                "median_ms": r.median_ms,
                "min_ms": r.min_ms,
                "max_ms": r.max_ms,
                "iterations": r.iterations
            }
            for r in network_results
        ]
    }
    
    output_path = "python_full_benchmark_results.json"
    with open(output_path, "w") as f:
        json.dump(all_results, f, indent=2)
    print(f"\nResults saved to {output_path}")
    
    # Calculate totals
    total_local = sum(r.mean_ms for r in local_results)
    print(f"\nLocal operations total: {total_local:.3f}ms")
    
    if network_results:
        total_network = sum(r.mean_ms for r in network_results)
        print(f"Network operations total: {total_network:.3f}ms")


if __name__ == "__main__":
    asyncio.run(main())
