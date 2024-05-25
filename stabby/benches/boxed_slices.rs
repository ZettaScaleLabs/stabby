use std::mem::MaybeUninit;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_slices(c: &mut Criterion) {
    c.bench_function("std_box_new", |b| {
        b.iter(|| Box::new(black_box(15)));
    });
    c.bench_function("stabby_box_new", |b| {
        b.iter(|| stabby::boxed::Box::new(black_box(15)));
    });
    c.bench_function("stabby_box_make", |b| {
        b.iter(|| {
            stabby::boxed::Box::make(|slot| {
                slot.write(black_box(15));
            })
        });
    });
    c.bench_function("std_box_big_new", |b| {
        b.iter(|| Box::<[usize; 10000]>::new([15; 10000]));
    });
    c.bench_function("stabby_box_big_new", |b| {
        b.iter(|| stabby::boxed::Box::<[usize; 10000]>::new([15; 10000]));
    });
    c.bench_function("stabby_box_big_make", |b| {
        b.iter(|| {
            stabby::boxed::Box::<[usize; 10000]>::make(|slot| {
                for slot in unsafe {
                    core::mem::transmute::<
                        &mut MaybeUninit<[usize; 10000]>,
                        &mut [MaybeUninit<usize>; 10000],
                    >(slot)
                } {
                    slot.write(15);
                }
            })
        });
    });
    for n in [10, 100, 1000, 10000, 100000].into_iter() {
        c.bench_function(&format!("collect_std_vec_{n}"), |b| {
            b.iter(|| black_box(0..n).map(black_box).collect::<Vec<_>>())
        });
        c.bench_function(&format!("collect_stabby_vec_{n}"), |b| {
            b.iter(|| {
                black_box(0..n)
                    .map(black_box)
                    .collect::<stabby::vec::Vec<_>>()
            })
        });
        c.bench_function(&format!("push_std_vec_{n}"), |b| {
            b.iter(|| {
                let mut v = Vec::new();
                for i in 0..n {
                    v.push(black_box(i))
                }
            })
        });
        c.bench_function(&format!("push_stabby_vec_{n}"), |b| {
            b.iter(|| {
                let mut v = stabby::vec::Vec::new();
                for i in 0..n {
                    v.push(black_box(i))
                }
            })
        });
        let std_vec = (0..n).collect::<Vec<_>>();
        let stabby_vec = (0..n).collect::<stabby::vec::Vec<_>>();
        c.bench_function(&format!("arc_std_vec_{n}"), |b| {
            b.iter_custom(|it| {
                let mut t = std::time::Duration::new(0, 0);
                for _ in 0..it {
                    let clone = std_vec.clone();
                    let start = std::time::Instant::now();
                    let arc: std::sync::Arc<[_]> = black_box(clone.into());
                    t += start.elapsed();
                    core::mem::drop(arc);
                }
                t
            })
        });
        c.bench_function(&format!("arc_stabby_vec_{n}"), |b| {
            b.iter_custom(|it| {
                let mut t = std::time::Duration::new(0, 0);
                for _ in 0..it {
                    let clone = stabby_vec.clone();
                    let start = std::time::Instant::now();
                    let arc: stabby::sync::ArcSlice<_> = black_box(clone.into());
                    t += start.elapsed();
                    core::mem::drop(arc);
                }
                t
            })
        });
        if n == 100000 {
            c.bench_function(&format!("box_std_vec_{n}"), |b| {
                b.iter_custom(|it| {
                    let mut t = std::time::Duration::new(0, 0);
                    for _ in 0..it {
                        let clone = std_vec.clone();
                        let start = std::time::Instant::now();
                        let arc: Box<[_]> = black_box(clone.into());
                        t += start.elapsed();
                        core::mem::drop(arc);
                    }
                    t
                })
            });
            c.bench_function(&format!("box_stabby_vec_{n}"), |b| {
                b.iter_custom(|it| {
                    let mut t = std::time::Duration::new(0, 0);
                    for _ in 0..it {
                        let clone = stabby_vec.clone();
                        let start = std::time::Instant::now();
                        let arc: stabby::boxed::BoxedSlice<_> = black_box(clone.into());
                        t += start.elapsed();
                        core::mem::drop(arc);
                    }
                    t
                })
            });
        }
    }
}

criterion_group!(benches, bench_slices);
criterion_main!(benches);
