use agentic_terminal::types::Hash;

#[test]
fn hash_of_distinct_inputs_differs() {
    assert_ne!(Hash::of(b"a").0, Hash::of(b"b").0);
}
