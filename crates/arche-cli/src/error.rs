use std::process::ExitCode;

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("{0}")]
    Generic(String),

    #[error("Invalid arguments: {0}")]
    InvalidArgs(String),

    #[error("API error: {0}")]
    ApiError(String),
}

impl CliError {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            CliError::Generic(_) => ExitCode::from(1),
            CliError::InvalidArgs(_) => ExitCode::from(2),
            CliError::ApiError(_) => ExitCode::from(3),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_error_exit_code() {
        let err = CliError::Generic("something went wrong".into());
        assert_eq!(err.exit_code(), ExitCode::from(1));
    }

    #[test]
    fn test_invalid_args_exit_code() {
        let err = CliError::InvalidArgs("missing --api-key".into());
        assert_eq!(err.exit_code(), ExitCode::from(2));
    }

    #[test]
    fn test_api_error_exit_code() {
        let err = CliError::ApiError("rate limited".into());
        assert_eq!(err.exit_code(), ExitCode::from(3));
    }

    #[test]
    fn test_error_display() {
        let err = CliError::Generic("msg".into());
        assert_eq!(err.to_string(), "msg");

        let err = CliError::InvalidArgs("bad arg".into());
        assert_eq!(err.to_string(), "Invalid arguments: bad arg");

        let err = CliError::ApiError("not found".into());
        assert_eq!(err.to_string(), "API error: not found");
    }
}
