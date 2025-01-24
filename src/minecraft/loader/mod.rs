#![allow(async_fn_in_trait)]
use fabric::Fabric;
use quilt::Quilt;
use serde::{Deserialize, Serialize};

use crate::json::version::meta::vanilla::VersionMeta;

use super::{config::Config, emitter::Emitter};

pub mod fabric;
pub mod forge;
pub mod quilt;

pub trait Loader {
    async fn merge<T: Loader>(
        &self,
        config: &Config<T>,
        meta: VersionMeta,
        emitter: Option<&Emitter>,
    ) -> crate::Result<VersionMeta>;
    fn get_version(&self) -> String;
}

impl Loader for () {
    async fn merge<T: Loader>(
        &self,
        _: &Config<T>,
        meta: VersionMeta,
        _: Option<&Emitter>,
    ) -> crate::Result<VersionMeta> {
        Ok(meta)
    }

    fn get_version(&self) -> String {
        "".to_string()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Loaders {
    Fabric(String),
    Quilt(String),
    Forge(String)
}