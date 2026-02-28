use crate::lexer::token::Token;

// Por ahora necesitamos dos tokens como cache. en el futuro podrian ser mas
pub(crate) const CACHE_LEN: usize = 2;

#[derive(Debug, Clone)]
pub struct TokenCache([Option<Token>; CACHE_LEN]);

impl TokenCache {
  pub fn empty() -> Self {
    debug_assert!(CACHE_LEN >= 1);
    let cache: [Option<Token>; CACHE_LEN] = (0..CACHE_LEN)
      .map(|_| None)
      .collect::<Vec<Option<Token>>>()
      .try_into()
      .unwrap();
    TokenCache(cache)
  }

  pub fn is_empty(&self) -> bool {
    self.0[0].is_none()
  }

  pub fn get_at(&self, index: usize) -> Option<&Token> {
    debug_assert!(index < CACHE_LEN);
    self.0[index].as_ref()
  }

  pub fn update_at(&mut self, index: usize, token: Option<Token>) {
    debug_assert!(index < CACHE_LEN);
    // dbg!(index, &token);
    self.0[index] = token;
  }
}
