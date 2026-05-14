//! Persistent state abstraction (stub).

use std::error::Error;

pub trait StateStore: Send + Sync {
    fn save_json(
        &mut self,
        _key: &str,
        _value: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;

    fn load_json(
        &mut self,
        _key: &str,
    ) -> Result<Option<String>, Box<dyn Error + Send + Sync>>;
}

pub struct NullStateStore;

impl StateStore for NullStateStore {
    fn save_json(
        &mut self,
        _key: &str,
        _value: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }

    fn load_json(
        &mut self,
        _key: &str,
    ) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        Ok(None)
    }
}
