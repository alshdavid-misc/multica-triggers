use std::sync::Arc;

use super::StartCommand;
use crate::context::AppContext;

#[derive(Clone)]
pub struct HandlerContext {
  pub ctx: AppContext,
  pub args: Arc<StartCommand>,
}
