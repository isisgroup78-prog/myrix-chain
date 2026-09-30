#[derive(Clone, Debug, Default)]
pub struct SdkClient {
    pub endpoint: String,
}

impl SdkClient {
    pub fn new(endpoint: String) -> Self {
        Self { endpoint }
    }

    pub fn is_ready(&self) -> bool {
        !self.endpoint.is_empty()
    }
}
