use crate::DynamicSizeArray;

#[test]
fn push_pop() {
    let mut array = DynamicSizeArray::new();
    let expected = ["123", "abc"];

    array.push("abc");
    array.push("123");

    let mut i = 0;
    while let Some(s) = array.pop() {
        assert_eq!(s, expected[i]);
        i += 1;
    }
}
