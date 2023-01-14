use crate::logic::logic_request::LogicRequest;
use crate::logic::storage_request::StorageRequest;
use async_channel::Sender;
use cooplan_amqp_api::api::input::input_element::InputElement;
use cooplan_amqp_api::api::input::request::Request;
use cooplan_amqp_api::error::{Error, ErrorKind};
use cooplan_amqp_api_shared::api::input::request_result::RequestResult;
use cooplan_lapin_wrapper::config::api::Api;
use std::sync::Arc;
use cooplan_amqp_api::api::input::input_element;

const ELEMENT_ID: &str = "example";

const ACTIONS: &[&str] = &["get"];

pub fn get(api: &Api) -> Result<InputElement<LogicRequest>, Error> {
    todo!();

    return Ok(input_element::extract_input(
        api,
        ELEMENT_ID,
        Arc::new(move |request, logic_request_sender| {
            Box::pin(request_handler(request, logic_request_sender))
        }),
        ACTIONS,
    )?);
}

async fn request_handler(
    request: Request,
    logic_request_sender: Sender<LogicRequest>,
) -> RequestResult {
    todo!();
}
