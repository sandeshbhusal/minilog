use std::collections::HashMap;

use clap::{Arg, Command};
use config::read_configuration;
use module::{FileIn, FileOut, Module};

use crate::module::Event;

mod config;
mod errors;
mod module;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    log::info!("Welcome to minilog.");
    let args = Command::new("minilog")
        .args(&[Arg::new("config_file").short('c').long("config")])
        .get_matches();

    log::info!("Reading configuration file.");
    let configuration_file_path = args
        .get_one::<String>("config_file")
        .expect("No configuration file passed");
    let configuration_path = std::path::Path::new(configuration_file_path);
    let conf = read_configuration(configuration_path)?;

    let mut configured_modules: HashMap<String, Box<dyn Module>> = HashMap::new();

    log::info!("Building modules.");
    for module in conf.modules.iter() {
        let (name, conf) = module;
        let module: Box<dyn Module> = match conf.uses.as_ref() {
            #[cfg(feature = "module_file_in")]
            "im_file" => FileIn::initialize(conf).await?,

            #[cfg(feature = "module_file_out")]
            "om_file" => FileOut::initialize(conf).await?,

            _ => {
                anyhow::bail!("No such usable module exists {}", conf.uses)
            }
        };

        configured_modules.insert(name.to_owned(), module);
    }

    // All configured modules get wired up.
    log::info!("Wiring up modules.");
    for route in conf.routes {
        let (routename, conf) = route;
        log::debug!("Setting up route {}", routename);

        let start = conf.from;
        let end = conf.to;

        // All output endpoints are collected, and all inputs are configured after.
        let mut outgoing_addrs = Vec::new();
        for module_identifier in end {
            let module = configured_modules
                .get_mut(&module_identifier)
                .expect(format!("No such module: {}", module_identifier).as_str());
            let addr = module.mailbox_address().await?;
            outgoing_addrs.push(addr);
        }

        // Tell all input modules about where they should send their data.
        for module_identifier in start {
            let module = configured_modules.get_mut(&module_identifier).unwrap();
            for addr in outgoing_addrs.iter() {
                module.register_output(addr.clone()).await?;
            }
        }
    }

    // Fire up all modules.
    for module in configured_modules.iter_mut() {
        module.1.handle_event(Event::Start).await?;
    }

    tokio::signal::ctrl_c().await.unwrap();
    Ok(())
}
