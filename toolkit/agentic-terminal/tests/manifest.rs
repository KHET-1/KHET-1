use agentic_terminal::manifest::canonical_tool_bytes;
use agentic_terminal::types::Tool;

fn mk_tool(examples: Vec<&str>, description: &str) -> Tool {
    Tool {
        name: "x".into(),
        description: description.into(),
        examples: examples.into_iter().map(str::to_owned).collect(),
        package: None,
        nix_attr: None,
        version: None,
        homepage: None,
    }
}

#[test]
fn canonical_bytes_disambiguate_example_list_boundaries() {
    let a = vec![mk_tool(vec!["a|b"], "desc")];
    let b = vec![mk_tool(vec!["a", "b"], "desc")];
    assert_ne!(canonical_tool_bytes(&a), canonical_tool_bytes(&b));
}

#[test]
fn canonical_bytes_disambiguate_newlines_in_fields() {
    let a = vec![mk_tool(vec!["one"], "line1\nline2")];
    let b = vec![mk_tool(vec!["one\ndesc:line2"], "line1")];
    assert_ne!(canonical_tool_bytes(&a), canonical_tool_bytes(&b));
}
