#![allow(unused)]
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path, time::Duration};
use tokio::task::JoinHandle;

use crate::config::ModuleConfiguration;

#[derive(Debug)]
pub enum Event {
    Start,
    Stop,
    Data(String),
}

#[async_trait]
pub trait Module {
    async fn initialize(config: &ModuleConfiguration) -> anyhow::Result<Box<Self>>
    where
        Self: Sized;

    async fn handle_event(&mut self, event: Event) -> anyhow::Result<()>;
    async fn mailbox_address(&mut self) -> anyhow::Result<crossbeam_channel::Sender<Event>>;

    async fn register_output(
        &mut self,
        external_route: crossbeam_channel::Sender<Event>,
    ) -> anyhow::Result<()>;
}

// struct ModuleImpl<T: Module> {
//     module: T,
//     outgoing: Vec<Sender<Event>>,
//     incoming: Receiver<Event>,
// }

// impl<T: Module> ModuleImpl<T> {
//     fn new(module: T) -> Self {
//         Self { module }
//     }
// }

#[derive(Debug, Default)]
pub struct FileIn {
    file_path: String,
    watch_interval: usize,
    outputs: Vec<Sender<Event>>,
    background_process: Option<JoinHandle<()>>,
}

#[async_trait]
impl Module for FileIn {
    async fn initialize(config: &ModuleConfiguration) -> anyhow::Result<Box<Self>> {
        let file_path = config.get_mandatory("file_path")?.try_into()?;
        let watch_interval = config.get_mandatory("watch_interval")?.try_into()?;

        Ok(Box::new(Self {
            file_path,
            watch_interval,
            outputs: Vec::new(),
            background_process: None,
        }))
    }

    async fn handle_event(&mut self, event: Event) -> anyhow::Result<()> {
        println!("Handling event: {:?}", event);

        match event {
            Event::Start => {
                if let Some(background) = self.background_process.take() {
                    log::warn!("Module already started. Dropping old execution context.");
                }

                let outputs = self.outputs.clone();
                self.background_process = Some(tokio::spawn(async move {
                    // Do stuff here

                    // Test: every 0.1 seconds, send one event to all outputs.
                    loop {
                        let outputs = outputs.clone();

                        tokio::time::sleep(Duration::from_millis(10000));
                        for output in outputs {
                            output
                                .send(Event::Data("Hello world!".into()))
                                .expect("Couldn't send data.")
                        }
                    }
                }));
            }
            Event::Stop => {}
            Event::Data(_) => {
                // Not necessary for input modules.
            }
        }

        Ok(())
    }

    async fn mailbox_address(&mut self) -> anyhow::Result<crossbeam_channel::Sender<Event>> {
        anyhow::bail!("Input modules should not have a mailbox address.");
    }

    async fn register_output(
        &mut self,
        external_route: crossbeam_channel::Sender<Event>,
    ) -> anyhow::Result<()> {
        self.outputs.push(external_route);
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct FileOut {
    file_path: String,
    plumbing: Option<(Sender<Event>, Receiver<Event>)>,
}

#[async_trait]
impl Module for FileOut {
    async fn initialize(config: &ModuleConfiguration) -> anyhow::Result<Box<Self>> {
        let file_path = config.get_mandatory("file_path")?.try_into()?;
        let plumbing = crossbeam_channel::unbounded();

        Ok(Box::new(Self {
            file_path,
            plumbing: Some(plumbing),
        }))
    }

    async fn handle_event(&mut self, event: Event) -> anyhow::Result<()> {
        println!("Handling event: {:?}", event);
        match event {
            Event::Start => {
                // BUG: when module gets dropped, receiver needs to be regenerated.
                if let Some((sender, receiver)) = self.plumbing.take() {
                    tokio::spawn(async move {
                        while let Ok(msg) = receiver.recv() {
                            log::warn!("Received {:?}", msg);
                        }
                    })
                } else {
                    anyhow::bail!("Output module has no plumbing")
                };
            }
            Event::Stop => todo!(),
            Event::Data(received) => log::info!("Received {}", received),
        }

        Ok(())
    }

    async fn mailbox_address(&mut self) -> anyhow::Result<crossbeam_channel::Sender<Event>> {
        Ok(self
            .plumbing
            .as_mut()
            .expect("Module not ready yet. This should never happen")
            .0
            .clone())
    }

    async fn register_output(
        &mut self,
        external_route: crossbeam_channel::Sender<Event>,
    ) -> anyhow::Result<()> {
        todo!()
    }
}
