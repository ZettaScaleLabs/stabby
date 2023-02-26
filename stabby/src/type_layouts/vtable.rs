use crate as stabby;

/// Implementation detail for stabby's version of dyn traits.
/// Any type that implements a trait `ITrait` must implement `IConstConstructor<VtITrait>` for `stabby::dyn!(Ptr<ITrait>)::from(value)` to work.
pub trait IConstConstructor<Vt> {
    const VTABLE: Vt;
}

/// Implementation detail for stabby's version of dyn traits.
pub trait TransitiveDeref<Head, N> {
    fn tderef<'a>(&'a self) -> &'a Head;
}
/// Implementation detail for stabby's version of dyn traits.
pub struct H;
/// Implementation detail for stabby's version of dyn traits.
pub struct T<T>(T);
/// A recursive type to define sets of v-tables.
/// You should _always_ use `stabby::vtable!(Trait1 + Trait2 + ...)` to generate this type,
/// as this macro will ensure that traits are ordered consistently in the vtable.
pub struct VTable<Head, Tail = VtDrop> {
    head: Head,
    tail: Tail,
}
impl<T, Head, Tail> IConstConstructor<VTable<Head, Tail>> for T
where
    T: IConstConstructor<Head> + IConstConstructor<Tail>,
{
    const VTABLE: VTable<Head, Tail> = VTable {
        head: T::VTABLE,
        tail: T::VTABLE,
    };
}
impl<Head> TransitiveDeref<VtDrop, T<()>> for VTable<Head> {
    fn tderef<'a>(&'a self) -> &'a VtDrop {
        &self.tail
    }
}
impl<Head, Tail> TransitiveDeref<Head, H> for VTable<Head, Tail> {
    fn tderef<'a>(&'a self) -> &'a Head {
        &self.head
    }
}
impl<Head, Tail: TransitiveDeref<Vt, N>, Vt, N> TransitiveDeref<Vt, T<N>> for VTable<Head, Tail> {
    fn tderef<'a>(&'a self) -> &'a Vt {
        self.tail.tderef()
    }
}

// DROP

#[stabby::stabby]
#[derive(Clone, Copy)]
pub struct VtDrop {
    drop: extern "C" fn(&mut ()),
}
impl<T> IConstConstructor<VtDrop> for T {
    const VTABLE: VtDrop = VtDrop {
        drop: unsafe {
            core::mem::transmute({
                extern "C" fn drop<T>(this: &mut T) {
                    unsafe { core::ptr::drop_in_place(this) }
                }
                drop::<T>
            } as extern "C" fn(&mut T))
        },
    };
}

// MYTRAIT

pub trait MyTrait {
    type Output;
    extern "C" fn do_stuff<'a>(&'a self, with: Self::Output) -> &'a Self;
    extern "C" fn gen_stuff(&mut self) -> Self::Output;
}

#[stabby::stabby]
#[derive(Clone, Copy)]
pub struct VtMyTrait<Output> {
    do_stuff: extern "C" fn(&(), Output) -> &'static (),
    gen_stuff: extern "C" fn(&mut ()) -> Output,
}
impl<T: MyTrait> IConstConstructor<VtMyTrait<T::Output>> for T {
    const VTABLE: VtMyTrait<T::Output> = unsafe {
        VtMyTrait {
            do_stuff: core::mem::transmute(
                <T as MyTrait>::do_stuff as extern "C" fn(&Self, T::Output) -> &Self,
            ),
            gen_stuff: core::mem::transmute(
                <T as MyTrait>::gen_stuff as extern "C" fn(&mut Self) -> T::Output,
            ),
        }
    };
}

// MYTRAIT2

pub trait MyTrait2 {
    extern "C" fn do_stuff2<'a>(&'a self) -> &'a Self;
}
#[stabby::stabby]
#[derive(Clone, Copy)]
pub struct VtMyTrait2 {
    do_stuff: extern "C" fn(&()) -> &'static (),
}
impl<T: MyTrait2> IConstConstructor<VtMyTrait2> for T {
    const VTABLE: VtMyTrait2 = VtMyTrait2 {
        do_stuff: unsafe {
            core::mem::transmute(<T as MyTrait2>::do_stuff2 as extern "C" fn(&Self) -> &Self)
        },
    };
}

// IMPL

impl MyTrait for Box<u8> {
    type Output = u8;
    extern "C" fn do_stuff<'a>(&'a self, _: Self::Output) -> &'a Self {
        self
    }
    extern "C" fn gen_stuff(&mut self) -> Self::Output {
        **self
    }
}
impl MyTrait2 for Box<u8> {
    extern "C" fn do_stuff2<'a>(&'a self) -> &'a Self {
        self
    }
}

// TEST
const AVT: VTable<VtMyTrait2, VTable<VtMyTrait<u8>>> = Box::<u8>::VTABLE;
fn test() {
    let DVT: &VtDrop = AVT.tderef();
    let MVT: &VtMyTrait<u8> = AVT.tderef();
    let M2VT: &VtMyTrait2 = AVT.tderef();
}
