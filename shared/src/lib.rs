use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AnnealParam {
    pub grid: Vec<Vec<i32>>,
    pub target: Vec<i32>,
    pub t0: f64,
    pub t_min: f64,
    pub alpha: f64,
    pub max_iters: usize,
}
impl Default for AnnealParam {
    fn default() -> Self {
        Self {
            grid: vec![],
            target: vec![],
            t0: 100.0,
            t_min: 0.1,
            alpha: 0.99,
            max_iters: 10000,
        }
    }
}

pub type Grid = Vec<Vec<i32>>;
