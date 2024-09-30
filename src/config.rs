use std::collections::HashMap;

use serde::Deserialize;

pub type Config = HashMap<String, Replacement>;

#[derive(Debug, Deserialize)]
pub struct Replacement {
    pub replace: String,
    pub with: String,
}
