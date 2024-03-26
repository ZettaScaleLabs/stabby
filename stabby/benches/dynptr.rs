use std::hint::unreachable_unchecked;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{Rng, SeedableRng};

#[stabby::stabby]
pub trait Op {
    extern "C" fn run(&self, lhs: u32, rhs: u32) -> u32;
}
pub trait OpNoExtern {
    fn run(&self, lhs: u32, rhs: u32) -> u32;
}

macro_rules! impl_op {
    ($t: ty, $lhs: ident, $rhs: ident: $e: expr) => {
        impl Op for $t {
            extern "C" fn run(&self, $lhs: u32, $rhs: u32) -> u32 {
                $e
            }
        }
        impl OpNoExtern for $t {
            fn run(&self, $lhs: u32, $rhs: u32) -> u32 {
                $e
            }
        }
    };
}

#[allow(dead_code)]
#[derive(Default)]
pub struct Add(u8);
impl_op!(Add, lhs, rhs: lhs.wrapping_add(rhs));
#[allow(dead_code)]
#[derive(Default)]
pub struct Sub(u8);
impl_op!(Sub, lhs, rhs: lhs.wrapping_sub(rhs));
#[allow(dead_code)]
#[derive(Default)]
pub struct Mul(u8);
impl_op!(Mul, lhs, rhs: lhs.wrapping_mul(rhs));

const N: usize = 1000;
fn bench_dynptr(c: &mut Criterion) {
    let rng = rand::rngs::StdRng::seed_from_u64(0);
    let mut stabby_arc: Vec<stabby::dynptr!(stabby::sync::Arc<dyn Op>)> = Vec::with_capacity(N);
    let mut stabby_box: Vec<stabby::dynptr!(stabby::boxed::Box<dyn Op>)> = Vec::with_capacity(N);
    let mut std_arc: Vec<std::sync::Arc<dyn Op>> = Vec::with_capacity(N);
    let mut std_box: Vec<Box<dyn Op>> = Vec::with_capacity(N);
    let mut std_arc_noext: Vec<std::sync::Arc<dyn OpNoExtern>> = Vec::with_capacity(N);
    let mut std_box_noext: Vec<Box<dyn OpNoExtern>> = Vec::with_capacity(N);
    let ops = (0..N)
        .map({
            let mut rng = rng.clone();
            move |_| rng.gen_range(0..=2u8)
        })
        .collect::<Vec<_>>();
    // Baseline (14.734 µs) for constructing 1K arc traits.
    c.bench_function("stabby_arc_new", |b| {
        b.iter(|| {
            stabby_arc.clear();
            for i in &ops {
                stabby_arc.push(match i {
                    0 => stabby::sync::Arc::new(Add::default()).into(),
                    1 => stabby::sync::Arc::new(Sub::default()).into(),
                    2 => stabby::sync::Arc::new(Mul::default()).into(),
                    _ => unsafe { unreachable_unchecked() },
                });
            }
        });
    });
    c.bench_function("stabby_box_new", |b| {
        b.iter(|| {
            stabby_box.clear();
            for i in &ops {
                stabby_box.push(match i {
                    0 => stabby::boxed::Box::new(Add::default()).into(),
                    1 => stabby::boxed::Box::new(Sub::default()).into(),
                    2 => stabby::boxed::Box::new(Mul::default()).into(),
                    _ => unsafe { unreachable_unchecked() },
                });
            }
        });
    });
    c.bench_function("std_arc_new", |b| {
        b.iter(|| {
            std_arc.clear();
            for i in &ops {
                std_arc.push(match i {
                    0 => std::sync::Arc::new(Add::default()),
                    1 => std::sync::Arc::new(Sub::default()),
                    2 => std::sync::Arc::new(Mul::default()),
                    _ => unsafe { unreachable_unchecked() },
                });
            }
        });
    });
    c.bench_function("std_box_new", |b| {
        b.iter(|| {
            std_box.clear();
            for i in &ops {
                #[allow(clippy::box_default)]
                std_box.push(match i {
                    0 => Box::new(Add::default()),
                    1 => Box::new(Sub::default()),
                    2 => Box::new(Mul::default()),
                    _ => unsafe { unreachable_unchecked() },
                });
            }
        });
    });
    c.bench_function("std_arc_noext_new", |b| {
        b.iter(|| {
            std_arc_noext.clear();
            for i in &ops {
                std_arc_noext.push(match i {
                    0 => std::sync::Arc::new(Add::default()),
                    1 => std::sync::Arc::new(Sub::default()),
                    2 => std::sync::Arc::new(Mul::default()),
                    _ => unsafe { unreachable_unchecked() },
                });
            }
        });
    });
    c.bench_function("std_box_noext_new", |b| {
        b.iter(|| {
            std_box_noext.clear();
            for i in &ops {
                #[allow(clippy::box_default)]
                std_box_noext.push(match i {
                    0 => Box::new(Add::default()),
                    1 => Box::new(Sub::default()),
                    2 => Box::new(Mul::default()),
                    _ => unsafe { unreachable_unchecked() },
                });
            }
        });
    });
    c.bench_function("stabby_arc_run", |b| {
        let mut rng = black_box(rng.clone());
        b.iter(|| {
            black_box(
                stabby_arc
                    .iter()
                    .fold(rng.gen(), |acc, it| it.run(acc, rng.gen())),
            );
        });
    });
    c.bench_function("stabby_box_run", |b| {
        let mut rng = black_box(rng.clone());
        b.iter(|| {
            black_box(
                stabby_box
                    .iter()
                    .fold(rng.gen(), |acc, it| it.run(acc, rng.gen())),
            );
        });
    });
    c.bench_function("std_arc_run", |b| {
        let mut rng = black_box(rng.clone());
        b.iter(|| {
            black_box(
                std_arc
                    .iter()
                    .fold(rng.gen(), |acc, it| it.run(acc, rng.gen())),
            );
        });
    });
    c.bench_function("std_box_run", |b| {
        let mut rng = black_box(rng.clone());
        b.iter(|| {
            black_box(
                std_box
                    .iter()
                    .fold(rng.gen(), |acc, it| it.run(acc, rng.gen())),
            );
        });
    });
    c.bench_function("std_arc_noext_run", |b| {
        let mut rng = black_box(rng.clone());
        b.iter(|| {
            black_box(
                std_arc_noext
                    .iter()
                    .fold(rng.gen(), |acc, it| it.run(acc, rng.gen())),
            );
        });
    });
    c.bench_function("std_box_noext_run", |b| {
        let mut rng = black_box(rng.clone());
        b.iter(|| {
            black_box(
                std_box_noext
                    .iter()
                    .fold(rng.gen(), |acc, it| it.run(acc, rng.gen())),
            );
        });
    });
}

criterion_group!(benches, bench_dynptr);
criterion_main!(benches);
