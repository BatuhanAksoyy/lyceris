use crate::json::version::meta::vanilla::VersionMeta;

use super::{config::Config, emitter::Emitter};

pub mod fabric;
pub mod forge;
pub mod quilt;
pub mod neoforge;

use std::future::Future;
use std::pin::Pin;

pub trait Loader {
    fn merge<'a>(
        &'a self,
        config: &'a Config<()>,
        meta: VersionMeta,
        emitter: Option<&'a Emitter>,
    ) -> Pin<Box<dyn Future<Output = crate::Result<VersionMeta>> + Send + 'a>>;

    fn get_version(&self) -> String;
}

impl Loader for () {
    fn merge<'a>(
        &'a self,
        _config: &'a Config<()>,
        meta: VersionMeta,
        _emitter: Option<&'a Emitter>,
    ) -> Pin<Box<dyn Future<Output = crate::Result<VersionMeta>> + Send + 'a>> {
        // Use Box::pin to return a boxed async block
        Box::pin(async move { Ok(meta) })
    }

    fn get_version(&self) -> String {
        "".to_string()
    }
}

impl Loader for Box<dyn Loader> {
    fn merge<'a>(
        &'a self,
        config: &'a Config<()>,
        meta: VersionMeta,
        emitter: Option<&'a Emitter>,
    ) -> Pin<Box<dyn Future<Output = crate::Result<VersionMeta>> + Send + 'a>> {
        self.as_ref().merge(config, meta, emitter)
    }

    fn get_version(&self) -> String {
        self.as_ref().get_version()
    }
}
