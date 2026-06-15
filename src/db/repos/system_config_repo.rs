use sea_orm::*;
use serde::{Serialize, de::DeserializeOwned};

use crate::AppError;

pub trait SystemConfigSection: Serialize + DeserializeOwned {
    const SCOPE: &'static str;
    const SCOPE_ID: &'static str;
    fn default_value() -> Self;
}

pub struct SystemConfigRepo;

impl SystemConfigRepo {
    pub fn get<T: SystemConfigSection>(_db: &impl ConnectionTrait) -> Result<T, AppError> {
        Ok(T::default_value())
    }

    pub fn set<T: SystemConfigSection>(_db: &impl ConnectionTrait, _settings: &T) -> Result<(), AppError> {
        Ok(())
    }
}
