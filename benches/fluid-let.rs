// Copyright (c) 2019, ilammy
// Licensed under MIT license (see LICENSE)

use criterion::*;

use fluid_let::fluid_let;

#[allow(clippy::trivially_copy_pass_by_ref)]
#[inline]
fn read_and_add(sum: &mut i32, value: &i32) {
    // Make sure the compiler does not optimize this and emits actual loads & stores.
    // This is safe since we convert valid Rust references to pointers here.
    unsafe {
        let a = std::ptr::read_volatile(sum);
        let b = std::ptr::read_volatile(value);
        std::ptr::write_volatile(sum, a + b);
    }
}

fn get(c: &mut Criterion) {
    let mut group = c.benchmark_group("fluid_let");
    group.bench_function(BenchmarkId::new("get", "dynamic"), |b| {
        fluid_let!(static COUNTER: i32);
        let mut total = 0;
        COUNTER.set(&1, || {
            b.iter(|| COUNTER.get(|value| read_and_add(&mut total, value.unwrap_or(&0))));
        });
    });
    group.bench_function(BenchmarkId::new("get", "static"), |b| {
        static mut COUNTER: Option<&i32> = None;
        let mut total = 0;
        unsafe {
            COUNTER = Some(&1);
            b.iter(|| read_and_add(&mut total, COUNTER.unwrap_or(&0)));
            COUNTER = None;
        }
    });
    group.finish();
}

fn set(c: &mut Criterion) {
    let mut group = c.benchmark_group("fluid_let");
    group.bench_function(BenchmarkId::new("set", "dynamic"), |b| {
        fluid_let!(static COUNTER: i32);
        static MAGIC_VALUE: i32 = 42;
        let mut total = 0;
        b.iter(|| COUNTER.set(&MAGIC_VALUE, || read_and_add(&mut total, &MAGIC_VALUE)));
    });
    group.bench_function(BenchmarkId::new("set", "static"), |b| {
        static mut COUNTER: Option<&i32> = None;
        static MAGIC_VALUE: i32 = 42;
        let mut total = 0;
        b.iter(|| unsafe {
            COUNTER = Some(&MAGIC_VALUE);
            read_and_add(&mut total, &MAGIC_VALUE);
            COUNTER = None;
        });
    });
    group.finish();
}

criterion_group!(fluid_let, get, set);

criterion_main!(fluid_let);
