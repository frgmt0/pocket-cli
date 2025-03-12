//! Performance benchmarks for Pocket CLI
//!
//! This file contains benchmarks for measuring the performance of VCS and snippet operations.
//! Note: This is a placeholder file that will be implemented once the actual API is finalized.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tempfile::tempdir;
use std::path::PathBuf;
use rand::Rng;

/// Benchmark file operations as a baseline
fn bench_file_operations(c: &mut Criterion) {
    c.bench_function("create_file", |b| {
        b.iter(|| {
            let temp_dir = tempdir().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            std::fs::write(&file_path, black_box("Hello, world!")).unwrap();
            black_box(file_path)
        })
    });
    
    c.bench_function("read_file", |b| {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "Hello, world!").unwrap();
        
        b.iter(|| {
            let content = std::fs::read_to_string(black_box(&file_path)).unwrap();
            black_box(content)
        })
    });
}

/// Benchmark JSON serialization/deserialization as a baseline
fn bench_json_operations(c: &mut Criterion) {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct TestData {
        id: String,
        name: String,
        content: String,
        tags: Vec<String>,
    }
    
    let test_data = TestData {
        id: "test-id".to_string(),
        name: "test-name".to_string(),
        content: "fn main() {\n    println!(\"Hello, world!\");\n}".to_string(),
        tags: vec!["test".to_string(), "example".to_string()],
    };
    
    c.bench_function("json_serialize", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&test_data)).unwrap();
            black_box(json)
        })
    });
    
    let json = serde_json::to_string(&test_data).unwrap();
    
    c.bench_function("json_deserialize", |b| {
        b.iter(|| {
            let data: TestData = serde_json::from_str(black_box(&json)).unwrap();
            black_box(data)
        })
    });
}

/// Benchmark hash computation as a baseline for object storage
fn bench_hash_computation(c: &mut Criterion) {
    use sha2::{Sha256, Digest};
    
    let data = "Hello, world! This is a test string for hash computation.";
    
    c.bench_function("sha256_hash", |b| {
        b.iter(|| {
            let mut hasher = Sha256::new();
            hasher.update(black_box(data));
            let hash = hasher.finalize();
            black_box(hash)
        })
    });
}

/// Benchmark directory traversal as a baseline for repository operations
fn bench_directory_traversal(c: &mut Criterion) {
    let mut group = c.benchmark_group("directory_traversal");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("walkdir", size), size, |b, &size| {
            // Create a temporary directory with the specified number of files
            let temp_dir = tempdir().unwrap();
            let dir_path = temp_dir.path();
            
            for i in 0..size {
                let file_path = dir_path.join(format!("file_{}.txt", i));
                std::fs::write(&file_path, format!("Content of file {}", i)).unwrap();
            }
            
            b.iter(|| {
                let entries: Vec<_> = walkdir::WalkDir::new(black_box(dir_path))
                    .min_depth(1)
                    .into_iter()
                    .filter_map(Result::ok)
                    .collect();
                black_box(entries)
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_file_operations,
    bench_json_operations,
    bench_hash_computation,
    bench_directory_traversal
);
criterion_main!(benches); 