pub fn wrap(mut num: isize, min: isize, max: isize) -> isize {
    let difference = max - min;
    while num < min {
        num += difference
    }
    num % max
}

#[test]
fn test_wrap() {
    let min = 0;
    let max = 42;

    for i in -1000..=1000 {
        let i = wrap(i, min, max);
        assert!(i < max);
        assert!(i >= min);
    }

    assert_eq!(wrap(0, 0, 30), 0);
    assert_eq!(wrap(-1, 0, 30), 29);
    assert_eq!(wrap(30, 0, 30), 0);
}
