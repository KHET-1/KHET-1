//! Helper mode state: disables agent dispatch, enables navigation-only operation on NixOS.

/// Configuration for runtime behavior (helper vs. full agent mode).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HelperModeConfig {
    /// When true, agent commands are disabled; navigation and preview only.
    pub helper_only: bool,
}

impl HelperModeConfig {
    #[must_use]
    pub const fn new(helper_only: bool) -> Self {
        Self { helper_only }
    }

    #[must_use]
    pub const fn default_mode() -> Self {
        Self { helper_only: false }
    }

    #[must_use]
    pub const fn helper_only_mode() -> Self {
        Self { helper_only: true }
    }
}

impl Default for HelperModeConfig {
    fn default() -> Self {
        Self::default_mode()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_only_disables_agent() {
        let config = HelperModeConfig::helper_only_mode();
        assert!(config.helper_only);
    }

    #[test]
    fn default_mode_enables_agent() {
        let config = HelperModeConfig::default_mode();
        assert!(!config.helper_only);
    }
}
