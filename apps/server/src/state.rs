#[derive(Debug, Clone)]
pub struct AppState {
    pub servvice_name: &'static str,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            servvice_name: "rust_ai_chat",
        }
    }
}
