use oci_spec::runtime::Spec;

pub mod default;

pub static EMPTY: Vec<String> = Vec::new();

#[derive(Debug, thiserror::Error)]
pub enum ExecutorError {
    #[error("invalid argument")]
    InvalidArg,
    #[error("failed to execute workload")]
    Execution(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("{0}")]
    Other(String),
}

pub trait Executor {
    /// Executes the workload
    fn exec(&self, spec: &Spec) -> Result<(), ExecutorError>;

    /// Checks if the handler is able to handle the workload
    fn can_handle(&self, spec: &Spec) -> bool;

    /// The name of the handler
    fn name(&self) -> &'static str;
}

#[derive(Debug, thiserror::Error)]
pub enum ExecutorManagerError {
    #[error("failed executor {name}")]
    ExecutionFailed {
        source: Box<dyn std::error::Error + Send + Sync>,
        name: String,
    },
    #[error("failed to find an executor that satisfies all requirements")]
    NoExecutorFound,
}

/// Manage the functions that actually run on the container
pub struct ExecutorManager {
    pub executors: Vec<Box<dyn Executor>>,
}

impl ExecutorManager {
    pub fn exec(&self, spec: &Spec) -> Result<(), ExecutorManagerError> {
        if self.executors.is_empty() {
            return Err(ExecutorManagerError::NoExecutorFound);
        };

        for executor in self.executors.iter() {
            if executor.can_handle(spec) {
                return executor.exec(spec).map_err(|e| {
                    tracing::error!(err = ?e, name = ?executor.name(), "failed to execute workload");
                    ExecutorManagerError::ExecutionFailed {
                        source: e.into(),
                        name: executor.name().to_string(),
                    }
                });
            }
        }
        Err(ExecutorManagerError::NoExecutorFound)
    }
}
