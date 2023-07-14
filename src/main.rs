extern crate core;

use crate::config::config::Config;
use crate::logic::logic_request::LogicRequest;
use crate::logic::storage_request::StorageRequest;
use cooplan_amqp_api::api::initialization_package::InitializationPackage;
use cooplan_lapin_wrapper::config::api::Api;
use serde_json::Value;
use std::io::{Error, ErrorKind};
use std::time::Duration;

mod api;
pub mod config;
mod error;
mod logic;
mod storage;

#[tokio::main]
async fn main() -> Result<(), Error> {
    match simple_logger::init() {
        Ok(_) => (),
        Err(error) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("failed to initialize logger: {}", error),
            ));
        }
    }

    let api = get_api().await?;

    let api_config = get_api_config().await?;

    let config = get_config().await?;

    let (logic_request_sender, logic_request_receiver) =
        async_channel::bounded::<LogicRequest>(config.logic_requests_boundary);

    let (output_sender, output_receiver) =
        tokio::sync::mpsc::channel::<(String, Value)>(config.output_channel_boundary);

    let state_tracker_client = cooplan_state_tracker::state_tracker_client::build(
        config.state_tracking_config,
        config.state_tracking_channel_boundary,
    )
    .await;

    let api_package = InitializationPackage::new(
        logic_request_sender,
        Box::new(api::input::registration::register),
        output_receiver,
        Box::new(api::output::registration::register),
        api,
        api_config,
        state_tracker_client,
    );

    match cooplan_amqp_api::api::init::initialize(api_package).await {
        Ok(()) => (),
        Err(error) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("failed to initialize api: {}", error),
            ));
        }
    }

    let (storage_request_sender, storage_request_receiver) =
        async_channel::bounded::<StorageRequest>(config.storage_requests_boundary);

    match logic::init::initialize(
        config.logic_request_dispatch_instances,
        logic_request_receiver,
        storage_request_sender,
    )
    .await
    {
        Ok(()) => (),
        Err(error) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("failed to initialize logic: {}", error),
            ));
        }
    }

    match storage::init::initialize(
        config.storage_request_dispatch_instances,
        storage_request_receiver,
        config.mongodb_uri
    )
    .await
    {
        Ok(()) => (),
        Err(error) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("failed to initialize storage: {}", error),
            ));
        }
    }

    std::thread::sleep(Duration::MAX);

    Ok(())
}

async fn get_api() -> Result<Api, Error> {
    let api_file = match std::env::args().nth(1) {
        Some(api_file) => api_file,
        None => {
            return Err(Error::new(ErrorKind::InvalidInput, "no api file provided"));
        }
    };

    let api = match cooplan_lapin_wrapper::config::api::try_get(api_file.as_str()).await {
        Ok(api) => api,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("failed to read api: {}", error),
            ))
        }
    };

    Ok(api)
}

async fn get_api_config() -> Result<cooplan_amqp_api::config::config::Config, Error> {
    let api_config_file = match std::env::args().nth(2) {
        Some(config_file) => config_file,
        None => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "no config file provided",
            ))
        }
    };

    let api_config = match cooplan_amqp_api::config::config::try_read_config(&api_config_file).await
    {
        Ok(api_config) => api_config,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("failed to read config: {}", error),
            ));
        }
    };

    Ok(api_config)
}

async fn get_config() -> Result<Config, Error> {
    let config_file = match std::env::args().nth(2) {
        Some(config_file) => config_file,
        None => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "no config file provided",
            ))
        }
    };

    let config = match config::config::try_read_config(&config_file).await {
        Ok(config) => config,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("failed to read config: {}", error),
            ));
        }
    };

    Ok(config)
}
