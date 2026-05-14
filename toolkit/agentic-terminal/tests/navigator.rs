use agentic_terminal::manifest::default_tools;
use agentic_terminal::navigator::ToolNavigator;
use agentic_terminal::types::Tool;

#[test]
fn empty_query_lists_all_tools_and_selects_first() {
    let nav = ToolNavigator::new(default_tools());
    assert_eq!(nav.filtered_len(), 16);
    assert_eq!(nav.selected_tool().unwrap().name, "nix");
}

#[test]
fn exact_prefix_puts_match_first() {
    let mut nav = ToolNavigator::new(default_tools());
    nav.set_query("jq");
    assert!(nav.filtered_len() >= 1);
    assert_eq!(nav.selected_tool().unwrap().name, "jq");

    nav.set_query("fd");
    assert_eq!(nav.selected_tool().unwrap().name, "fd");
    
    nav.set_query("nvme");
    assert_eq!(nav.selected_tool().unwrap().name, "nvme");
}

#[test]
fn impossible_query_leaves_empty_and_no_selection() {
    let mut nav = ToolNavigator::new(default_tools());
    nav.set_query("zzznonexistentzzz");
    assert_eq!(nav.filtered_len(), 0);
    assert!(nav.selected_tool().is_none());
}

#[test]
fn next_previous_wraparound() {
    let mut nav = ToolNavigator::new(default_tools());
    nav.set_query("");
    nav.list_state.select(Some(0));
    nav.previous();
    assert_eq!(nav.list_state.selected(), Some(15));
    nav.next();
    assert_eq!(nav.list_state.selected(), Some(0));
}

#[test]
fn page_up_down_respect_bounds() {
    let mut nav = ToolNavigator::new(default_tools());
    nav.set_query("");
    nav.list_state.select(Some(0));
    nav.page_up(100);
    assert_eq!(nav.list_state.selected(), Some(0));

    nav.list_state.select(Some(15));
    nav.page_down(100);
    assert_eq!(nav.list_state.selected(), Some(15));
}

#[test]
fn query_is_trimmed_before_matching() {
    let mut nav = ToolNavigator::new(default_tools());
    nav.set_query("   jq   ");
    assert_eq!(nav.selected_tool().unwrap().name, "jq");
}

#[test]
fn duplicate_tool_names_preserve_distinct_selection() {
    let tools = vec![
        Tool {
            name: "dup".into(),
            description: "first duplicate".into(),
            examples: vec![],
            package: None,
            nix_attr: None,
            version: None,
            homepage: None,
        },
        Tool {
            name: "dup".into(),
            description: "second duplicate".into(),
            examples: vec![],
            package: None,
            nix_attr: None,
            version: None,
            homepage: None,
        },
    ];
    let nav = ToolNavigator::new(tools);
    assert_eq!(nav.filtered_len(), 2);
    assert_eq!(
        nav.tool_at_idx(nav.matches()[0].tool_idx).unwrap().description,
        "first duplicate"
    );
    assert_eq!(
        nav.tool_at_idx(nav.matches()[1].tool_idx).unwrap().description,
        "second duplicate"
    );
}
