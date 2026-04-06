use std::sync::{Arc, Mutex};

use crate::{scripts::Script, *};

mod script_system_tests;

#[derive(Debug, Clone)]
pub struct SpyScript {
    run_count: Arc<Mutex<u32>>,
}
impl SpyScript {
    pub fn new() -> Self {
        Self {
            run_count: Arc::new(Mutex::new(0)),
        }
    }
}
impl Script for SpyScript {
    fn run(&self, _: &[&Components], _: &Events) -> Vec<Components> {
        let mut count = self.run_count.lock().unwrap();
        *count += 1;
        vec![]
    }
}
