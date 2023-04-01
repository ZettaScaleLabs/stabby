#[stabby::export]
extern "C" fn stable_fn(_: u8) {}

#[test]
fn report() {
    use stabby::IStable;
    let report = <extern "C" fn(u8) as IStable>::REPORT;
    panic!("{:?}", report);
}
