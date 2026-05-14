use agentic_terminal::manifest::default_tools;
use agentic_terminal::navigator::ToolNavigator;

#[test]
fn empty_query_lists_all_tools_and_selects_first() {
    let nav = ToolNavigator::new(default_tools());
    assert_eq!(nav.filtered_len(), 4);
    assert_eq!(nav.selected_tool().unwrap().name, "ripgrep");
}

#[test]
fn exact_prefix_puts_match_first() {
    let mut nav = ToolNavigator::new(default_tools());
    nav.set_query("jq");
    assert!(nav.filtered_len() >= 1);
    assert_eq!(nav.selected_tool().unwrap().name, "jq");

    nav.set_query("fd");
    assert_eq!(nav.selected_tool().unwrap().name, "fd");
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
    assert_eq!(nav.list_state.selected(), Some(3));
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

    nav.list_state.select(Some(3));
    nav.page_down(100);
    assert_eq!(nav.list_state.selected(), Some(3));
}
