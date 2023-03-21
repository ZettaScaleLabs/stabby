pub mod boxed;
pub mod sync;
pub mod borrow {
    use crate::abi::IStable;

    #[crate::stabby]
    pub enum Cow<Borrowed: IStable, Owned: IStable> {
        Borrowed(Borrowed),
        Owned(Owned),
    }
}
