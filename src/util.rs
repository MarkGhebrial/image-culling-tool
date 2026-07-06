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

pub fn merge_sort<T>(array: &[T], compare: impl Fn(&T, &T) -> bool + Clone) -> Vec<T>
where
    T: Clone,
{
    // Handle base case:
    if array.len() == 1 {
        return vec![array[0].clone()];
    }

    // Split array in two
    let split_index: usize = array.len() / 2;
    let (a, b) = array.split_at(split_index);

    // Sort each half
    let sorted_a = merge_sort(a, compare.clone());
    let sorted_b = merge_sort(b, compare.clone());

    // Merge the two halves
    return merge(&sorted_a, &sorted_b, &compare);
}

fn merge<T>(a: &[T], b: &[T], compare: impl Fn(&T, &T) -> bool + Clone) -> Vec<T>
where
    T: Clone,
{
    let mut out: Vec<T> = Vec::with_capacity(a.len() + b.len());

    let mut index_a = 0;
    let mut index_b = 0;
    loop {
        let item_a = a.get(index_a);
        let item_b = b.get(index_b);

        match (item_a, item_b) {
            (Some(item_a), Some(item_b)) => {
                if compare(item_a, item_b) {
                    out.push(item_a.clone());
                    index_a += 1;
                } else {
                    out.push(item_b.clone());
                    index_b += 1;
                }
            }
            (Some(item_a), None) => {
                out.push(item_a.clone());
                index_a += 1;
            }
            (None, Some(item_b)) => {
                out.push(item_b.clone());
                index_b += 1;
            }
            (None, None) => break,
        }
    }

    out
}
