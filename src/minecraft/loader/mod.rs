#![allow(async_fn_in_trait)]
use crate::json::version::meta::vanilla::VersionMeta;

use super::{emitter::Emitter, config::Config};

pub mod fabric;
pub mod quilt;
pub mod forge;

pub trait Loader { 
    async fn merge<T: Loader>(&self, config: &Config<T>, meta: VersionMeta, emitter: Option<&Emitter>) -> crate::Result<VersionMeta>;
    fn get_version(&self) -> String;
}

impl Loader for () {
    async fn merge<T: Loader>(&self, _: &Config<T>, meta: VersionMeta, _: Option<&Emitter>) -> crate::Result<VersionMeta> {
        Ok(meta)
    }

    fn get_version(&self) -> String {
        "".to_string()
    }
}
