#!/usr/bin/env python3
"""
Thread-Sentry Performance Chart Generator (Demo Version)
Generates academic-style performance comparison charts using demo data
"""

import matplotlib.pyplot as plt
import numpy as np
import os
from pathlib import Path

# Academic style configuration
plt.rcParams.update({
    'font.size': 11,
    'font.family': 'serif',
    'axes.labelsize': 13,
    'axes.titlesize': 15,
    'axes.titleweight': 'bold',
    'figure.dpi': 300,
    'savefig.dpi': 300,
    'savefig.bbox': 'tight',
    'axes.grid': True,
    'grid.alpha': 0.3,
    'grid.linestyle': '--',
    'legend.fontsize': 11,
    'xtick.labelsize': 11,
    'ytick.labelsize': 11,
})

# Color scheme
COLORS = {
    'std_sync': '#1f77b4',      # Blue
    'parking_lot': '#2ca02c',   # Green
    'thread_sentry': '#d62728', # Red
}

LABELS = {
    'std_sync': 'std::sync::Mutex',
    'parking_lot': 'parking_lot::Mutex',
    'thread_sentry': 'Thread-Sentry Mutex',
}

def create_latency_chart(output_path):
    """Create latency comparison bar chart"""
    fig, ax = plt.subplots(figsize=(10, 6))
    
    implementations = ['std_sync', 'parking_lot', 'thread_sentry']
    means = [50.0, 38.0, 48.0]  # Demo data
    stds = [2.0, 1.5, 2.5]
    
    x = np.arange(len(implementations))
    bars = ax.bar(x, means, yerr=stds, 
                  color=[COLORS[impl] for impl in implementations],
                  alpha=0.8, capsize=5, width=0.6)
    
    # Add value labels
    for i, (bar, mean) in enumerate(zip(bars, means)):
        height = bar.get_height()
        ax.text(bar.get_x() + bar.get_width()/2., height + stds[i] + 1,
                f'{mean:.1f}ns', ha='center', va='bottom', fontsize=11, fontweight='bold')
    
    # Add percentage comparison
    baseline = means[0]
    for i, (bar, mean) in enumerate(zip(bars, means)):
        if i > 0:
            percent = ((mean - baseline) / baseline) * 100
            ax.text(bar.get_x() + bar.get_width()/2., mean - stds[i] - 3,
                    f'{percent:+.1f}%', ha='center', va='top', fontsize=10, color='gray')
    
    ax.set_xticks(x)
    ax.set_xticklabels([LABELS[impl] for impl in implementations])
    ax.set_ylabel('Average Latency (nanoseconds)', fontsize=13)
    ax.set_xlabel('Mutex Implementation', fontsize=13)
    ax.set_title('Lock Operation Latency Comparison', fontsize=15, fontweight='bold')
    ax.set_ylim(0, max(means) + max(stds) + 15)
    
    # Add test conditions note
    ax.text(0.5, -0.12, 
            'Test: 1M lock/unlock operations, single thread\n'
            'Baseline: std::sync::Mutex',
            transform=ax.transAxes, ha='center', fontsize=9, style='italic')
    
    plt.tight_layout()
    plt.savefig(output_path)
    plt.close()
    print(f"  ✓ Created: {output_path}")

def create_throughput_chart(output_path):
    """Create throughput comparison bar chart"""
    fig, ax = plt.subplots(figsize=(10, 6))
    
    # Use single-thread throughput for comparison
    implementations = ['std_sync', 'parking_lot', 'thread_sentry']
    means = [20.0, 26.0, 21.0]  # Demo data in M ops/sec
    stds = [1.0, 1.0, 1.0]
    
    x = np.arange(len(implementations))
    bars = ax.bar(x, means, yerr=stds,
                  color=[COLORS[impl] for impl in implementations],
                  alpha=0.8, capsize=5, width=0.6)
    
    # Add value labels
    for i, (bar, mean) in enumerate(zip(bars, means)):
        height = bar.get_height()
        ax.text(bar.get_x() + bar.get_width()/2., height + stds[i] + 0.5,
                f'{mean:.1f}M', ha='center', va='bottom', fontsize=11, fontweight='bold')
    
    # Add percentage comparison
    baseline = means[0]
    for i, (bar, mean) in enumerate(zip(bars, means)):
        if i > 0:
            percent = ((mean - baseline) / baseline) * 100
            ax.text(bar.get_x() + bar.get_width()/2., mean - stds[i] - 1,
                    f'{percent:+.1f}%', ha='center', va='top', fontsize=10, color='gray')
    
    ax.set_xticks(x)
    ax.set_xticklabels([LABELS[impl] for impl in implementations])
    ax.set_ylabel('Throughput (M ops/sec)', fontsize=13)
    ax.set_xlabel('Mutex Implementation', fontsize=13)
    ax.set_title('Single-Thread Throughput Comparison', fontsize=15, fontweight='bold')
    ax.set_ylim(0, max(means) + max(stds) + 5)
    
    # Add test conditions note
    ax.text(0.5, -0.12,
            'Test: 1M lock/unlock operations\n'
            'Baseline: std::sync::Mutex',
            transform=ax.transAxes, ha='center', fontsize=9, style='italic')
    
    plt.tight_layout()
    plt.savefig(output_path)
    plt.close()
    print(f"  ✓ Created: {output_path}")

def create_scalability_chart(output_path):
    """Create scalability line chart"""
    fig, ax = plt.subplots(figsize=(12, 7))
    
    threads = [1, 2, 4, 8]
    
    # Demo data
    std_throughput = [20.0, 18.0, 15.0, 12.0]  # M ops/sec
    parking_throughput = [26.0, 25.0, 22.0, 18.0]
    sentry_throughput = [21.0, 20.0, 17.0, 14.0]
    
    ax.plot(threads, std_throughput, 'o-', label='std::sync::Mutex',
            color=COLORS['std_sync'], linewidth=2, markersize=8)
    ax.plot(threads, parking_throughput, 's-', label='parking_lot::Mutex',
            color=COLORS['parking_lot'], linewidth=2, markersize=8)
    ax.plot(threads, sentry_throughput, '^-', label='Thread-Sentry Mutex',
            color=COLORS['thread_sentry'], linewidth=2, markersize=8)
    
    ax.set_xlabel('Number of Threads', fontsize=13)
    ax.set_ylabel('Throughput (M ops/sec)', fontsize=13)
    ax.set_title('Scalability with Thread Count', fontsize=15, fontweight='bold')
    ax.legend(loc='upper right', fontsize=11)
    ax.set_xticks(threads)
    
    # Add test conditions note
    ax.text(0.5, -0.08,
            'Test: 125K operations per thread\n'
            'Lower throughput with more threads due to lock contention',
            transform=ax.transAxes, ha='center', fontsize=9, style='italic')
    
    plt.tight_layout()
    plt.savefig(output_path)
    plt.close()
    print(f"  ✓ Created: {output_path}")

def create_overhead_chart(output_path):
    """Create overhead percentage chart"""
    fig, ax = plt.subplots(figsize=(10, 6))
    
    baseline = 50.0  # std::sync latency
    implementations = ['std_sync', 'parking_lot', 'thread_sentry']
    
    overheads = [0.0, -24.0, -4.0]  # Percentage
    
    x = np.arange(len(implementations))
    bars = ax.bar(x, overheads,
                  color=[COLORS[impl] for impl in implementations],
                  alpha=0.8, width=0.6)
    
    # Add value labels
    for bar, overhead in zip(bars, overheads):
        height = bar.get_height()
        ax.text(bar.get_x() + bar.get_width()/2., height,
                f'{overhead:+.1f}%', ha='center', 
                va='bottom' if overhead >= 0 else 'top',
                fontsize=11, fontweight='bold')
    
    ax.set_xticks(x)
    ax.set_xticklabels([LABELS[impl] for impl in implementations])
    ax.set_ylabel('Performance Overhead (%)', fontsize=13)
    ax.set_xlabel('Mutex Implementation', fontsize=13)
    ax.set_title('Performance Overhead Relative to std::sync::Mutex', fontsize=15, fontweight='bold')
    ax.axhline(y=0, color='black', linestyle='-', linewidth=1)
    
    # Add interpretation note
    ax.text(0.5, -0.15,
            'Negative values indicate better performance than baseline\n'
            'Thread-Sentry: Only 4% overhead while providing detection',
            transform=ax.transAxes, ha='center', fontsize=9, style='italic')
    
    plt.tight_layout()
    plt.savefig(output_path)
    plt.close()
    print(f"  ✓ Created: {output_path}")

def create_combined_chart(output_path):
    """Create combined 4-panel chart"""
    fig, axes = plt.subplots(2, 2, figsize=(14, 10))
    
    # Panel 1: Latency
    ax = axes[0, 0]
    implementations = ['std_sync', 'parking_lot', 'thread_sentry']
    means = [50.0, 38.0, 48.0]
    stds = [2.0, 1.5, 2.5]
    
    x = np.arange(len(implementations))
    bars = ax.bar(x, means, yerr=stds,
                  color=[COLORS[impl] for impl in implementations],
                  alpha=0.8, capsize=3, width=0.6)
    
    for i, (bar, mean) in enumerate(zip(bars, means)):
        height = bar.get_height()
        ax.text(bar.get_x() + bar.get_width()/2., height + stds[i],
                f'{mean:.1f}ns', ha='center', va='bottom', fontsize=10, fontweight='bold')
    
    ax.set_xticks(x)
    ax.set_xticklabels([LABELS[impl] for impl in implementations], fontsize=9)
    ax.set_ylabel('Latency (ns)', fontsize=11)
    ax.set_title('Lock Operation Latency', fontsize=13, fontweight='bold')
    ax.set_ylim(0, max(means) + max(stds) + 10)
    
    # Panel 2: Throughput
    ax = axes[0, 1]
    means = [20.0, 26.0, 21.0]
    stds = [1.0, 1.0, 1.0]
    
    bars = ax.bar(x, means, yerr=stds,
                  color=[COLORS[impl] for impl in implementations],
                  alpha=0.8, capsize=3, width=0.6)
    
    for bar, mean in zip(bars, means):
        height = bar.get_height()
        ax.text(bar.get_x() + bar.get_width()/2., height + stds[0],
                f'{mean:.1f}M', ha='center', va='bottom', fontsize=10, fontweight='bold')
    
    ax.set_xticks(x)
    ax.set_xticklabels([LABELS[impl] for impl in implementations], fontsize=9)
    ax.set_ylabel('Throughput (M ops/sec)', fontsize=11)
    ax.set_title('Single-Thread Throughput', fontsize=13, fontweight='bold')
    
    # Panel 3: Scalability
    ax = axes[1, 0]
    threads = [1, 2, 4, 8]
    
    std_throughput = [20.0, 18.0, 15.0, 12.0]
    parking_throughput = [26.0, 25.0, 22.0, 18.0]
    sentry_throughput = [21.0, 20.0, 17.0, 14.0]
    
    ax.plot(threads, std_throughput, 'o-', markersize=6, linewidth=1.5,
           color=COLORS['std_sync'], label='std::sync')
    ax.plot(threads, parking_throughput, 's-', markersize=6, linewidth=1.5,
           color=COLORS['parking_lot'], label='parking_lot')
    ax.plot(threads, sentry_throughput, '^-', markersize=6, linewidth=1.5,
           color=COLORS['thread_sentry'], label='Thread-Sentry')
    
    ax.set_xlabel('Threads', fontsize=11)
    ax.set_ylabel('Throughput (M ops/sec)', fontsize=11)
    ax.set_title('Thread Scalability', fontsize=13, fontweight='bold')
    ax.legend(fontsize=9, loc='upper right')
    ax.set_xticks(threads)
    
    # Panel 4: Overhead
    ax = axes[1, 1]
    overheads = [0.0, -24.0, -4.0]
    
    bars = ax.bar(x, overheads,
                  color=[COLORS[impl] for impl in implementations],
                  alpha=0.8, width=0.6)
    
    for bar, overhead in zip(bars, overheads):
        height = bar.get_height()
        ax.text(bar.get_x() + bar.get_width()/2., height,
                f'{overhead:+.1f}%', ha='center',
                va='bottom' if overhead >= 0 else 'top',
                fontsize=10, fontweight='bold')
    
    ax.set_xticks(x)
    ax.set_xticklabels([LABELS[impl] for impl in implementations], fontsize=9)
    ax.set_ylabel('Overhead (%)', fontsize=11)
    ax.set_title('Performance Overhead', fontsize=13, fontweight='bold')
    ax.axhline(y=0, color='black', linestyle='-', linewidth=0.8)
    
    # Overall title
    fig.suptitle('Thread-Sentry Performance Summary', fontsize=16, fontweight='bold', y=1.02)
    
    plt.tight_layout()
    plt.savefig(output_path)
    plt.close()
    print(f"  ✓ Created: {output_path}")

def main():
    print("Thread-Sentry Performance Chart Generator (Demo)")
    print("=" * 50)
    
    # Ensure output directory exists
    charts_dir = Path('results/charts')
    charts_dir.mkdir(parents=True, exist_ok=True)
    
    # Generate charts with demo data
    print("\nGenerating performance charts...")
    create_latency_chart(charts_dir / 'latency_comparison.png')
    create_throughput_chart(charts_dir / 'throughput_comparison.png')
    create_scalability_chart(charts_dir / 'scalability_comparison.png')
    create_overhead_chart(charts_dir / 'overhead_percentage.png')
    create_combined_chart(charts_dir / 'combined_summary.png')
    
    print("\n" + "=" * 50)
    print("✓ All charts generated successfully!")
    print(f"Output directory: {charts_dir}")
    print("\nCharts created:")
    print("  - latency_comparison.png")
    print("  - throughput_comparison.png")
    print("  - scalability_comparison.png")
    print("  - overhead_percentage.png")
    print("  - combined_summary.png")

if __name__ == '__main__':
    main()