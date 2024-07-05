use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;
use stabby::alloc::{allocators, collections::arc_btree::ArcBTreeSet, vec::Vec, IAlloc};

fn bench<T: IAlloc + Clone + Default>(c: &mut Criterion, set: &[i32]) {
    c.bench_function(core::any::type_name::<T>(), |b| {
        b.iter(|| {
            let mut vec = Vec::new_in(T::default());
            let mut btree = ArcBTreeSet::<_, _, false, 5>::new_in(T::default());
            for &i in set {
                vec.push(i);
                btree.insert(i);
            }
            black_box((vec, btree));
        })
    });
}

fn bench_allocs(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    for n in [10, 100, 1000, 10000].into_iter() {
        let set = (0..n).map(|_| rng.gen()).collect::<Vec<i32>>();
        // bench::<allocators::FreelistGlobalAlloc>(c, &set);
        bench::<allocators::LibcAlloc>(c, &set);
        bench::<allocators::RustAlloc>(c, &set);
    }
}

criterion_group!(benches, bench_allocs);
criterion_main!(benches);
