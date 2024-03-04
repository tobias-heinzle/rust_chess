fn generate_shuffled_data() -> Vec<u64> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..100000).map(|_| rng.gen::<u64>()).collect()
}

pub fn no_sort<T: Ord>(vec: Vec<T>) -> Vec<T> {
    vec
}

pub fn sort_1<T: Ord>(mut vec: Vec<T>) -> Vec<T> {
    vec.sort_by(|a, b| b.cmp(a));
    vec
}

pub fn sort_2<T: Ord + Copy>(mut vec: Vec<T>) -> Vec<T> {
    vec.sort_by_key(|&w| std::cmp::Reverse(w));
    vec
}

pub fn sort_3<T: Ord>(mut vec: Vec<T>) -> Vec<T> {
    vec.sort();
    vec.reverse();
    vec
}

pub fn sort_4(mut vec: Vec<u64>) -> Vec<u64> {
    vec.sort_by_key(|n| std::u64::MAX - n);
    vec
}

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn comparison_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Sorting");
    let data = generate_shuffled_data();

    group.bench_function("no_sort", |b| b.iter(|| black_box(no_sort(data.clone()))));

    group.bench_function("sort_1", |b| b.iter(|| black_box(sort_1(data.clone()))));

    group.bench_function("sort_2", |b| b.iter(|| black_box(sort_2(data.clone()))));

    group.bench_function("sort_3", |b| b.iter(|| black_box(sort_3(data.clone()))));

    group.bench_function("sort_4", |b| b.iter(|| black_box(sort_3(data.clone()))));

    group.finish()
}

criterion_group!(benches, comparison_benchmark);
criterion_main!(benches);
