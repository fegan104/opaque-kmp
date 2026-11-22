use std::sync::Mutex;

uniffi::setup_scaffolding!();

#[derive(uniffi::Object)]
pub struct FibonacciGenerator {
    seed: (i32, i32),
    state: Mutex<(i32, i32)>,
}

#[uniffi::export]
impl FibonacciGenerator {
    #[uniffi::constructor]
    pub fn new(a: i32, b: i32) -> Self {
        Self {
            seed: (a, b),
            state: Mutex::new((a, b)),
        }
    }

    pub fn next(&self) -> Option<i32> {
        let mut guard = self.state.lock().ok()?;
        let (current, next_val) = *guard;
        let next_sum = current.checked_add(next_val)?;
        *guard = (next_val, next_sum);
        Some(current)
    }

    pub fn reset(&self) {
        if let Ok(mut guard) = self.state.lock() {
            *guard = self.seed;
        }
    }
}