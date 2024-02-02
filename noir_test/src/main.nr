fn main(x: Field, y: pub Field) -> pub Field { 
    let z = x + y;
    z * x
}

#[test]
fn test_main() {
    let c = main(1, 2);

    assert(c == 3);
}
