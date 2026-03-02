use crate::frontend::{config::FrontendConfig, frontend_result::FrontendResult};

#[derive(Debug, Clone)]
pub struct FrontendPipeline;

impl FrontendPipeline {
  pub fn run(source: &str, config: &FrontendConfig) -> FrontendResult {
    todo!()
  }
}
