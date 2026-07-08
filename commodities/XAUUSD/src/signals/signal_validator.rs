// Signal Validator — Validate signal before execution

use super::signal_types::Signal;

pub fn validate(signal: &Signal) -> bool {
    signal.is_valid()
}
