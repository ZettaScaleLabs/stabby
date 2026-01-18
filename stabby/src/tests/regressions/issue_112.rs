#[test]
fn issue_112() {
    use crate::boxed::Box;

    struct Deadbeef(u32);

    impl Drop for Deadbeef {
        fn drop(&mut self) {
            assert_eq!(self.0, 0xdeadbeef, "not deadbeef?!")
        }
    }
    let _deadbeef: crate::dynptr!(Box<dyn Send>) = Box::new(Deadbeef(0xdeadbeef)).into();
}
