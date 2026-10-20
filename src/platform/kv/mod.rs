use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub struct KvStore {}

impl KvStore {
  pub fn new() -> Self {
    todo!()
  }
  pub fn get<D: DeserializeOwned>(
    &self,
    key: &str,
  ) -> anyhow::Result<D> {
    todo!()
  }
  pub fn set<S: Serialize>(
    &self,
    key: &str,
    value: &S,
  ) -> anyhow::Result<bool> {
    todo!()
  }
  pub fn remove(
    &self,
    key: &str,
  ) -> anyhow::Result<bool> {
    todo!()
  }
}
