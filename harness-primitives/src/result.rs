use candid::CandidType;

/// A result type from a call to the harness canister. It contains the error message if any on error and
/// the data returned from the call if successful.
///
/// The data is generic depending on the return type of the operation called, we know the type
/// must implement the CandidType trait.
#[derive(CandidType)]
pub struct HarnessResult<T>
where
    T: CandidType,
{
    pub error: String,
    pub success: bool,
    pub data: Option<T>,
}

impl<T: CandidType> HarnessResult<T> {
    /// Wraps an error into a HarnessResult, setting data to None.
    pub fn wrap_error(err: crate::error::Error) -> Self {
        Self {
            error: err.to_string(),
            success: false,
            data: None,
        }
    }

    /// Wraps an error string into a HarnessResult, setting data to None.
    pub fn wrap_error_str(err: &str) -> Self {
        Self {
            error: err.to_string(),
            success: false,
            data: None,
        }
    }

    /// Creates a successful HarnessResult with the provided data.
    pub fn wrap_success(data: T) -> Self {
        Self {
            error: String::new(),
            success: true,
            data: Some(data),
        }
    }
}
