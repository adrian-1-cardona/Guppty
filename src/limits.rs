// === limits.rs ===
// Runtime resource limits for execution budgets, call stacks, ranges, and outputs.

/// Configurable limits to prevent unbounded resource consumption.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceLimits {
    /// Maximum nested call frames allowed.
    pub max_call_depth: usize,
    /// Maximum values allowed on the VM operand/value stack.
    pub max_stack_size: usize,
    /// Maximum number of items in a range or iterable allocation.
    pub max_range_size: usize,
    /// Maximum execution steps (instructions in VM, statements in interpreter).
    pub max_steps: Option<usize>,
    /// Maximum cumulative bytes produced by `out(...)`.
    pub max_output_bytes: Option<usize>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_call_depth: 1_000,
            max_stack_size: 10_000,
            max_range_size: 1_000_000,
            max_steps: None,
            max_output_bytes: None,
        }
    }
}

#[allow(dead_code)]
impl ResourceLimits {
    /// Conservative limits suitable for running untrusted code or fast sandboxed verification.
    pub fn conservative() -> Self {
        Self {
            max_call_depth: 256,
            max_stack_size: 1_024,
            max_range_size: 10_000,
            max_steps: Some(100_000),
            max_output_bytes: Some(1_000_000),
        }
    }

    /// Unlimited limits for unconstrained research runs or long-running computations.
    pub fn unlimited() -> Self {
        Self {
            max_call_depth: usize::MAX,
            max_stack_size: usize::MAX,
            max_range_size: usize::MAX,
            max_steps: None,
            max_output_bytes: None,
        }
    }

    /// Load limits from environment variables with defaults.
    pub fn from_env() -> Self {
        let mut limits = if std::env::var("GUPPTY_CONSERVATIVE")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
        {
            Self::conservative()
        } else {
            Self::default()
        };

        if let Ok(val) = std::env::var("GUPPTY_MAX_CALL_DEPTH") {
            if let Ok(parsed) = val.parse::<usize>() {
                limits.max_call_depth = parsed;
            }
        }
        if let Ok(val) = std::env::var("GUPPTY_MAX_STACK_SIZE") {
            if let Ok(parsed) = val.parse::<usize>() {
                limits.max_stack_size = parsed;
            }
        }
        if let Ok(val) = std::env::var("GUPPTY_MAX_RANGE_SIZE") {
            if let Ok(parsed) = val.parse::<usize>() {
                limits.max_range_size = parsed;
            }
        }
        if let Ok(val) = std::env::var("GUPPTY_MAX_STEPS") {
            if let Ok(parsed) = val.parse::<usize>() {
                limits.max_steps = if parsed == 0 { None } else { Some(parsed) };
            }
        }
        if let Ok(val) = std::env::var("GUPPTY_MAX_OUTPUT_BYTES") {
            if let Ok(parsed) = val.parse::<usize>() {
                limits.max_output_bytes = if parsed == 0 { None } else { Some(parsed) };
            }
        }

        limits
    }
}
