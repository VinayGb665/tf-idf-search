use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SearchResponse {
    pub len: usize,
    pub results: HashMap<String, f32>,
}
