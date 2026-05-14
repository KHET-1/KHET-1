use agentic_terminal::helper_mode::HelperModeConfig;
use agentic_terminal::state::SessionState;

#[test]
fn helper_mode_config_creation() {
    let helper = HelperModeConfig::helper_only_mode();
    assert!(helper.helper_only);

    let normal = HelperModeConfig::default_mode();
    assert!(!normal.helper_only);
}

#[test]
fn helper_mode_config_equality() {
    let a = HelperModeConfig::new(true);
    let b = HelperModeConfig::helper_only_mode();
    assert_eq!(a, b);

    let c = HelperModeConfig::new(false);
    let d = HelperModeConfig::default_mode();
    assert_eq!(c, d);
}

#[test]
fn session_state_recent_commands_fifo() {
    let mut state = SessionState::new();
    state.add_recent("cmd1".into());
    state.add_recent("cmd2".into());
    state.add_recent("cmd3".into());

    assert_eq!(state.recent_commands.len(), 3);
    assert_eq!(state.recent_commands[0], "cmd3");
    assert_eq!(state.recent_commands[1], "cmd2");
    assert_eq!(state.recent_commands[2], "cmd1");
}

#[test]
fn session_state_pinned_commands() {
    let mut state = SessionState::new();
    state.toggle_pin("nix develop".into());
    state.toggle_pin("nix build".into());

    assert!(state.is_pinned("nix develop"));
    assert!(state.is_pinned("nix build"));

    state.toggle_pin("nix develop".into());
    assert!(!state.is_pinned("nix develop"));
}

#[test]
fn session_state_recent_max_limit() {
    let mut state = SessionState::new();
    for i in 0..30 {
        state.add_recent(format!("cmd{}", i));
    }
    assert_eq!(state.recent_commands.len(), 20);
    // Most recent command should be at index 0
    assert_eq!(state.recent_commands[0], "cmd29");
}

#[test]
fn session_state_pinned_max_limit() {
    let mut state = SessionState::new();
    for i in 0..15 {
        state.toggle_pin(format!("cmd{}", i));
    }
    assert_eq!(state.pinned_commands.len(), 10);
}

#[test]
fn session_state_empty_command_not_added() {
    let mut state = SessionState::new();
    state.add_recent("".into());
    assert_eq!(state.recent_commands.len(), 0);
}
