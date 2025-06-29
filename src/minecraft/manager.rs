use std::{sync::Arc, time::Duration};

use tokio::{process::Child, sync::Mutex};

use crate::{minecraft::emitter::Emit, Config};

use super::{emitter::Emitter, loader::Loader};

#[derive(Default)]
pub struct Manager {
    instances: Vec<Instance<Box<dyn Loader>>>,
}

impl Manager {
    pub fn create_instance(
        &mut self,
        config: Config<Box<dyn Loader>>,
        emitter: Option<&Emitter>,
    ) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let name = config
            .profile
            .as_ref()
            .map_or_else(|| config.get_version_name(), |profile| profile.name.clone());

        self.instances.push(Instance {
            id: id.clone(),
            name,
            emitter: emitter.cloned(),
            process: Arc::new(Mutex::new(None)),
            config,
        });

        id
    }

    pub async fn start_instance(&mut self, id: &str) -> crate::Result<()> {
        if let Some(instance) = self.get_instance_mut(id) {
            instance.install().await?;
            instance.launch().await?;
        } else {
            return Err(crate::Error::InstanceNotFound(id.to_string()));
        }

        Ok(())
    }

    pub async fn stop_instance(&mut self, id: &str) -> crate::Result<()> {
        if let Some(instance) = self.get_instance_mut(id) {
            instance.abort().await?;
        } else {
            return Err(crate::Error::InstanceNotFound(id.to_string()));
        }

        Ok(())
    }

    pub fn get_instance(&self, id: &str) -> Option<&Instance<Box<dyn Loader>>> {
        self.instances.iter().find(|instance| instance.id == id)
    }

    pub fn get_instance_mut(&mut self, id: &str) -> Option<&mut Instance<Box<dyn Loader>>> {
        self.instances.iter_mut().find(|instance| instance.id == id)
    }

    pub fn get_instances(&self) -> &Vec<Instance<Box<dyn Loader>>> {
        &self.instances
    }

    pub fn get_instances_mut(&mut self) -> &mut Vec<Instance<Box<dyn Loader>>> {
        &mut self.instances
    }

    pub fn remove_instance(&mut self, id: &str) -> Option<Instance<Box<dyn Loader>>> {
        if let Some(pos) = self.instances.iter().position(|instance| instance.id == id) {
            Some(self.instances.remove(pos))
        } else {
            None
        }
    }

    pub fn remove_instance_by_name(&mut self, name: &str) -> Option<Instance<Box<dyn Loader>>> {
        if let Some(pos) = self
            .instances
            .iter()
            .position(|instance| instance.name == name)
        {
            Some(self.instances.remove(pos))
        } else {
            None
        }
    }

    pub async fn stop_all_instances(&mut self) -> crate::Result<()> {
        for instance in &mut self.instances {
            instance.abort().await?;
        }
        Ok(())
    }

    pub fn clear_instances(&mut self) {
        self.instances.clear();
    }
}

pub struct Instance<R: Loader> {
    id: String,
    name: String,
    emitter: Option<Emitter>,
    process: Arc<Mutex<Option<Child>>>,
    config: Config<R>,
}

impl<R: Loader> Instance<R> {
    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_emitter(&self) -> Option<&Emitter> {
        self.emitter.as_ref()
    }

    pub fn get_process(&self) -> &Arc<Mutex<Option<Child>>> {
        &self.process
    }

    pub async fn launch(&mut self) -> crate::Result<()> {
        let mut process = self.process.lock().await;
        if process.is_none() {
            let child = crate::launch(&self.config, self.emitter.as_ref()).await?;
            *process = Some(child);

            let child_arc = self.process.clone();
            let process_clone = self.process.clone();
            let emitter_clone = self.emitter.clone();
            tokio::spawn(async move {
                if let Err(e) = monitor_process(child_arc, emitter_clone).await {
                    eprintln!("Error monitoring process: {}", e);
                }
                let mut process = process_clone.lock().await;
                *process = None;
            });
        } else {
            self.emitter
                .emit(crate::minecraft::emitter::Event::AlreadyRunning, ())
                .await;
        }

        Ok(())
    }

    pub async fn install(&self) -> crate::Result<()> {
        let process = self.process.lock().await;
        if process.is_some() {
            self.emitter
                .emit(crate::minecraft::emitter::Event::AlreadyRunning, ())
                .await;
            return Ok(());
        }

        crate::install(&self.config, self.emitter.as_ref()).await
    }

    pub async fn abort(&mut self) -> crate::Result<()> {
        let mut process = self.process.lock().await;
        if let Some(child) = process.as_mut() {
            child.kill().await?;
        }
        Ok(())
    }
}

async fn monitor_process(
    child: Arc<Mutex<Option<Child>>>,
    emitter: Option<Emitter>,
) -> crate::Result<()> {
    loop {
        let mut child = child.lock().await;
        if let Some(ref mut child) = *child {
            if child.try_wait()?.is_some() {
                emitter
                    .emit(crate::minecraft::emitter::Event::Exit, ())
                    .await;
                break;
            }
        } else {
            break;
        }
        drop(child);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Ok(())
}
