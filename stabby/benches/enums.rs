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
        assert_eq!(std::mem::size_of::<StdOp>(), 12);
        assert_eq!(std::mem::size_of::<Manual>(), 8);
        assert_eq!(std::mem::size_of::<COp>(), 12);
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
    c.bench_function("std_new_opt", |b| {
        b.iter(|| {
            std_op.clear();
            for (some, value) in &ops {
                std_op.push(some.then_some(*value));
            }
        });
    });
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
    c.bench_function("std_run_opt", |b| {
        b.iter(|| {
            let mut result = 0;
            for value in std_op.iter().filter_map(Option::as_ref) {
                result += value.get()
            }
            black_box(result);
        });
    });
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
