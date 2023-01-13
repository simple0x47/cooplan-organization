use crate::logic::logic_request::LogicRequest;
use crate::logic::storage_request::StorageRequest;
use async_channel::Sender;
use cooplan_amqp_api::api::input::input_element::InputElement;
use cooplan_amqp_api::api::input::request::Request;
use cooplan_amqp_api::error::{Error, ErrorKind};
use cooplan_amqp_api_shared::api::input::request_result::RequestResult;
use cooplan_lapin_wrapper::config::api::Api;
use std::sync::Arc;

const ELEMENT_ID: &str = "example";

const ACTIONS: &[&str] = &["get"];

pub fn get(api: &Api) -> Result<InputElement<LogicRequest>, Error> {
    todo!();

    let api_config = match api
        .input()
        .iter()
        .find(|api_config| api_config.id() == ELEMENT_ID)
    {
        Some(api_config) => api_config,
        None => {
            return Err(Error::new(
                ErrorKind::AutoConfigFailure,
                format!("failed to find output api with id '{}'", ELEMENT_ID),
            ))
        }
    };

    return Ok(InputElement::new(
        ELEMENT_ID.to_string(),
        Arc::new(move |request, logic_request_sender| {
            Box::pin(request_handler(request, logic_request_sender))
        }),
        ACTIONS,
        api_config.clone(),
    ));
}

async fn request_handler(
    request: Request,
    logic_request_sender: Sender<LogicRequest>,
) -> RequestResult {
    todo!();
}
