use crate::error::Error;
use crate::logic::actions::user_logic_action::UserLogicAction;
use crate::logic::logic_request::LogicRequest;
use async_channel::Sender;
use cooplan_amqp_api::api::input::input_element;
use cooplan_amqp_api::api::input::input_element::InputElement;
use cooplan_amqp_api::api::input::request::{extract_parameter_from_request_data, Request};
use cooplan_amqp_api_shared::api::input::request_result::RequestResult;
use cooplan_amqp_api_shared::api::input::request_result_error::{
    RequestResultError, RequestResultErrorKind,
};
use cooplan_lapin_wrapper::config::api::Api;
use serde_json::{Map, Value};
use std::sync::Arc;

const ACTIONS: &[&str] = &["get_organization_and_permissions"];

pub fn get(api: &Api) -> Result<InputElement<LogicRequest>, Error> {
    const ELEMENT_ID: &str = "user";

    let input_api = input_element::extract_input(
        api,
        ELEMENT_ID,
        Arc::new(move |request, logic_request_sender| {
            Box::pin(request_handler(request, logic_request_sender))
        }),
        ACTIONS,
    )?;

    Ok(input_api)
}

async fn request_handler(
    request: Request,
    logic_request_sender: Sender<LogicRequest>,
) -> RequestResult {
    let action = match request.try_get_header() {
        Ok(header) => {
            if !ACTIONS.contains(&header.action()) {
                return RequestResult::Err(RequestResultError::new(
                    RequestResultErrorKind::MalformedRequest,
                    format!("unknown action '{}'", header.action()),
                ));
            }

            header.action().to_string()
        }
        Err(error) => return RequestResult::Err(error.into()),
    };

    let data = request.data;

    match action.as_str() {
        "read" => read(data, logic_request_sender).await,
        _ => {
            return RequestResult::Err(RequestResultError::new(
                RequestResultErrorKind::MalformedRequest,
                format!("unknown action '{}'", action),
            ))
        }
    }
}

/// Expected parameters:
/// - **user_id**: String
async fn read(
    data: Map<String, Value>,
    logic_request_sender: Sender<LogicRequest>,
) -> RequestResult {
    let user_id = match extract_parameter_from_request_data::<String>(&data, "user_id") {
        Ok(user_id) => user_id,
        Err(error) => return error,
    };

    let (replier, receiver) = tokio::sync::oneshot::channel();

    let action = UserLogicAction::Read { user_id, replier };

    match logic_request_sender.send(LogicRequest::User(action)).await {
        Ok(_) => (),
        Err(error) => {
            return RequestResult::Err(RequestResultError::new(
                RequestResultErrorKind::InternalFailure,
                format!("failed to send logic request: {}", error),
            ))
        }
    }

    let result = match receiver.await {
        Ok(result) => match result {
            Ok(user) => match serde_json::to_value(user) {
                Ok(value) => RequestResult::Ok(value),
                Err(error) => RequestResult::Err(RequestResultError::new(
                    RequestResultErrorKind::InternalFailure,
                    format!("failed to serialize user: {}", error),
                )),
            },
            Err(error) => RequestResult::Err(RequestResultError::new(
                RequestResultErrorKind::MalformedRequest,
                format!("failed to read user: {}", error),
            )),
        },
        Err(error) => RequestResult::Err(RequestResultError::new(
            RequestResultErrorKind::InternalFailure,
            format!("failed to receive user from logic: {}", error),
        )),
    };

    result
}
