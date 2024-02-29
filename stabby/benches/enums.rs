use std::{hint::unreachable_unchecked, num::NonZeroU32};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{Rng, SeedableRng};

#[stabby::stabby]
pub struct Params {
    value: u32,
    repeats: u8,
}

pub enum StdOp {
    Add(Params),
    Sub(Params),
    Mul(Params),
}
pub enum Op {
    Add,
    Sub,
    Mul,
}
pub struct Manual {
    value: u32,
    repeats: u8,
    op: Op,
}

#[repr(u8)]
pub enum COp {
    Add(Params),
    Sub(Params),
    Mul(Params),
}

#[stabby::stabby]
pub enum StabbyOp {
    Add(Params),
    Sub(Params),
    Mul(Params),
}

#[repr(u8)]
pub enum COption<T> {
    Some(T),
    None,
}

const N: usize = 100000;
fn bench_dynptr(c: &mut Criterion) {
    macro_rules! assert_eq {
        ($a: expr, $b: expr) => {
            if $a != $b {
                panic!(":(");
            }
        };
    }
    const _: () = {
        // Surprisingly, Rust doesn't minimize the size here. Does it not see the niche?
        assert_eq!(std::mem::size_of::<StdOp>(), 12);
        // [`Manual`] uses the niche that [`StdOp`] should be able to use.
        assert_eq!(std::mem::size_of::<Manual>(), 8);
        // Good old repr(C) must be fat for historical reasons
        assert_eq!(std::mem::size_of::<COp>(), 12);
        // Stabby finds the niche automagically.
        assert_eq!(std::mem::size_of::<StabbyOp>(), 8);
        assert_eq!(std::mem::size_of::<COption<NonZeroU32>>(), 8);
        assert_eq!(std::mem::size_of::<Option<NonZeroU32>>(), 4);
        assert_eq!(std::mem::size_of::<stabby::option::Option<NonZeroU32>>(), 4);
    };
    let rng = rand::rngs::StdRng::seed_from_u64(0);
    let ops = (0..N)
        .map({
            let mut rng = rng.clone();
            move |_| {
                (
                    rng.gen_range(0..=2u8),
                    rng.gen_range(1..=5u8),
                    rng.gen_range(0..=100u32),
                )
            }
        })
        .collect::<Vec<_>>();
    let mut std_op = Vec::with_capacity(ops.len());
    let mut manual_op = Vec::with_capacity(ops.len());
    let mut c_op = Vec::with_capacity(ops.len());
    let mut stabby_op = Vec::with_capacity(ops.len());
    // Baseline (81.233 µs) for constructing 100K enum variants where niches might be available.
    c.bench_function("std_new", |b| {
        b.iter(|| {
            std_op.clear();
            for (op, repeats, value) in &ops {
                std_op.push(match op {
                    0 => StdOp::Add(Params {
                        value: *value,
                        repeats: *repeats,
                    }),
                    1 => StdOp::Sub(Params {
                        value: *value,
                        repeats: *repeats,
                    }),
                    2 => StdOp::Mul(Params {
                        value: *value,
                        repeats: *repeats,
                    }),
                    _ => unsafe { unreachable_unchecked() },
                });
            }
        });
    });
    // Faster (65.052 µs), likely thanks to the smaller memory footprint and constifyability.
    c.bench_function("manual_new", |b| {
        b.iter(|| {
            manual_op.clear();
            for (op, repeats, value) in &ops {
                manual_op.push(match op {
                    0 => Manual {
                        op: Op::Add,
                        value: *value,
                        repeats: *repeats,
                    },
                    1 => Manual {
                        op: Op::Sub,
                        value: *value,
                        repeats: *repeats,
                    },
                    2 => Manual {
                        op: Op::Mul,
                        value: *value,
                        repeats: *repeats,
                    },
                    _ => unsafe { unreachable_unchecked() },
                });
            }
        });
    });
    // Same performance, same memory footprint.
    c.bench_function("c_new", |b| {
        b.iter(|| {
            c_op.clear();
            for (op, repeats, value) in &ops {
                c_op.push(match op {
                    0 => COp::Add(Params {
                        value: *value,
                        repeats: *repeats,
                    }),
                    1 => COp::Sub(Params {
                        value: *value,
                        repeats: *repeats,
                    }),
                    2 => COp::Mul(Params {
                        value: *value,
                        repeats: *repeats,
                    }),
                    _ => unsafe { unreachable_unchecked() },
                });
            }
        });
    });
    // Much slower (379.25 µs), the smaller layout doesn't compensate for 2 branches that can't be constified.
    c.bench_function("stabby_new", |b| {
        b.iter(|| {
            stabby_op.clear();
            for (op, repeats, value) in &ops {
                stabby_op.push(match op {
                    0 => StabbyOp::Add(Params {
                        value: *value,
                        repeats: *repeats,
                    }),
                    1 => StabbyOp::Sub(Params {
                        value: *value,
                        repeats: *repeats,
                    }),
                    2 => StabbyOp::Mul(Params {
                        value: *value,
                        repeats: *repeats,
                    }),
                    _ => unsafe { unreachable_unchecked() },
                });
            }
        });
    });

    // Baseline (439.04 µs) for executing 100K operations
    c.bench_function("std_run", |b| {
        b.iter(|| {
            let mut result = 0;
            for op in &std_op {
                match op {
                    StdOp::Add(Params { value, repeats }) => {
                        for _ in 0..*repeats {
                            result += value
                        }
                    }
                    StdOp::Sub(Params { value, repeats }) => {
                        for _ in 0..*repeats {
                            result -= value
                        }
                    }
                    StdOp::Mul(Params { value, repeats }) => {
                        for _ in 0..*repeats {
                            result *= value
                        }
                    }
                }
            }
            black_box(result);
        });
    });
    // Slightly faster (432.38 µs), better optimization?
    c.bench_function("manual_run", |b| {
        b.iter(|| {
            let mut result = 0;
            for op in &manual_op {
                match op {
                    Manual {
                        value,
                        repeats,
                        op: Op::Add,
                    } => {
                        for _ in 0..*repeats {
                            result += value
                        }
                    }
                    Manual {
                        value,
                        repeats,
                        op: Op::Sub,
                    } => {
                        for _ in 0..*repeats {
                            result -= value
                        }
                    }
                    Manual {
                        value,
                        repeats,
                        op: Op::Mul,
                    } => {
                        for _ in 0..*repeats {
                            result *= value
                        }
                    }
                }
            }
            black_box(result);
        });
    });
    // Slower (458.91 µs), likely due to more memory churn
    c.bench_function("c_run", |b| {
        b.iter(|| {
            let mut result = 0;
            for op in &c_op {
                match op {
                    COp::Add(Params { value, repeats }) => {
                        for _ in 0..*repeats {
                            result += value
                        }
                    }
                    COp::Sub(Params { value, repeats }) => {
                        for _ in 0..*repeats {
                            result -= value
                        }
                    }
                    COp::Mul(Params { value, repeats }) => {
                        for _ in 0..*repeats {
                            result *= value
                        }
                    }
                }
            }
            black_box(result);
        });
    });
    /// Same performance
    c.bench_function("stabby_run", |b| {
        b.iter(|| {
            let mut result = 0;
            for op in &stabby_op {
                op.match_ref_ctx(
                    &mut result,
                    |result, Params { value, repeats }| {
                        for _ in 0..*repeats {
                            *result += *value
                        }
                    },
                    |result, Params { value, repeats }| {
                        for _ in 0..*repeats {
                            *result -= *value
                        }
                    },
                    |result, Params { value, repeats }| {
                        for _ in 0..*repeats {
                            *result *= *value
                        }
                    },
                )
            }
            black_box(result);
        });
    });

    let ops = (0..N)
        .map({
            let mut rng = rng.clone();
            move |_| {
                (
                    rng.gen_bool(0.7),
                    NonZeroU32::new(rng.gen_range(1..=100u32)).unwrap(),
                )
            }
        })
        .collect::<Vec<_>>();
    let mut std_op = Vec::with_capacity(ops.len());
    let mut c_op = Vec::with_capacity(ops.len());
    let mut stabby_op = Vec::with_capacity(ops.len());
    // Baseline (160μs) for `new_opt` bench series, where 100000 instances of `Option<NonZeroU32>` are pushed into a pre-allocated vector.
    c.bench_function("std_new_opt", |b| {
        b.iter(|| {
            std_op.clear();
            for (some, value) in &ops {
                std_op.push(some.then_some(*value));
            }
        });
    });
    // Slower (189μs), likely due to COpt<NonZeroU32> being twice as big. It has to churn through more memory, but construction is cheap and cache prediction easy.
    c.bench_function("c_new_opt", |b| {
        b.iter(|| {
            c_op.clear();
            for (some, value) in &ops {
                c_op.push(match some {
                    true => COption::Some(*value),
                    false => COption::None,
                });
            }
        });
    });
    // Much slower (398μs), likely due to the constructor for `None` being non-const due to trait limitations.
    c.bench_function("stabby_new_opt", |b| {
        b.iter(|| {
            stabby_op.clear();
            for (some, value) in &ops {
                stabby_op.push(match some {
                    true => stabby::option::Option::Some(*value),
                    false => stabby::option::Option::None(),
                });
            }
        });
    });

    // Baseline (152μs) for accessing all values of the 100000 element vector and adding the present ones to a result.
    c.bench_function("std_run_opt", |b| {
        b.iter(|| {
            let mut result = 0;
            for value in std_op.iter().filter_map(Option::as_ref) {
                result += value.get()
            }
            black_box(result);
        });
    });
    // Equivalent performance, the larger memory footprint is likely compensated by a better optimization of the operation (branchless?).
    c.bench_function("c_run_opt", |b| {
        b.iter(|| {
            let mut result = 0;
            for op in &c_op {
                match op {
                    COption::Some(value) => result += value.get(),
                    COption::None => {}
                }
            }
            black_box(result);
        });
    });
    // Faster (137μs) despite not having much of a reason to be, maybe this form let the optimizer find a better gen?
    c.bench_function("stabby_run_opt", |b| {
        b.iter(|| {
            let mut result = 0;
            for op in &stabby_op {
                op.match_ref(|value| result += value.get(), || ())
            }
            black_box(result);
        });
    });
}

criterion_group!(benches, bench_dynptr);
criterion_main!(benches);
