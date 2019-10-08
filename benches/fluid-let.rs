// Copyright (c) 2019, ilammy
// Licensed under MIT license (see LICENSE)

use criterion::*;

use fluid_let::fluid_let;

fn get(c: &mut Criterion) {
    let mut group = c.benchmark_group("fluid_let");
    group.bench_function(BenchmarkId::new("get", "dynamic"), |b| {
        fluid_let!(static COUNTER: i32);
        let mut total = 0;
        COUNTER.set(1, || {
            b.iter(|| COUNTER.get(|value| total += value.unwrap_or(&0)));
        });
    });
    group.bench_function(BenchmarkId::new("get", "static"), |b| {
        static mut COUNTER: Option<&i32> = None;
        let mut total = 0;
        unsafe {
            COUNTER = Some(&1);
            b.iter(|| total += COUNTER.unwrap_or(&0));
            COUNTER = None;
        }
    });
    group.finish();
}

fn set(c: &mut Criterion) {
    let mut group = c.benchmark_group("fluid_let");
    group.bench_function(BenchmarkId::new("set", "dynamic"), |b| {
        fluid_let!(static COUNTER: i32);
        let mut total = 0;
        b.iter(|| COUNTER.set(total, || black_box(total += total)));
    });
    group.bench_function(BenchmarkId::new("set", "static"), |b| {
        static mut COUNTER: Option<&i32> = None;
        let mut total = 0;
        b.iter(|| unsafe {
            // It's safe to transmute reference lifetime like this:
            COUNTER = Some(std::mem::transmute(&total));
            black_box(total += total);
            COUNTER = None;
        });
    });
    group.finish();
}

criterion_group!(fluid_let, get, set);

criterion_main!(fluid_let);
