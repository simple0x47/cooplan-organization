use crate::logic::logic_request::LogicRequest;
use async_channel::Sender;
use cooplan_amqp_api::api::input::input_element;
use cooplan_amqp_api::api::input::input_element::InputElement;
use cooplan_amqp_api::api::input::request::{extract_parameter_from_request_data, Request};
use cooplan_amqp_api::api::input::token::Token;
use cooplan_amqp_api::error::{Error, ErrorKind};
use cooplan_amqp_api_shared::api::input::request_result::RequestResult;
use cooplan_amqp_api_shared::api::input::request_result_error::{
    RequestResultError, RequestResultErrorKind,
};
use cooplan_lapin_wrapper::config::api::Api;
use serde_json::{Map, Value};
use std::sync::Arc;

const ACTIONS: &[&str] = &["create", "read", "update", "delete", "request_access"];

pub fn get(api: &Api) -> Result<InputElement<LogicRequest>, Error> {
    const ELEMENT_ID: &str = "organization";

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

    let authorized_token = match request.authorized_token {
        Some(authorized_token) => authorized_token,
        None => {
            return RequestResult::Err(RequestResultError::new(
                RequestResultErrorKind::InternalFailure,
                "missing authorized token after authorization",
            ))
        }
    };

    let data = request.data;

    match action.as_str() {
        "create" => create(authorized_token, data, logic_request_sender).await,
        _ => {
            return RequestResult::Err(RequestResultError::new(
                RequestResultErrorKind::MalformedRequest,
                format!("unknown action '{}'", action),
            ))
        }
    }
}

async fn create(
    authorized_token: Token,
    data: Map<String, Value>,
    logic_request_sender: Sender<LogicRequest>,
) -> RequestResult {
    let (name, country, address, telephone) = match extract_create_expected_parameters(data) {
        Ok((name, country, address, telephone)) => (name, country, address, telephone),
        Err(error) => return error,
    };

    RequestResult::Ok(Value::Null)
}

fn extract_create_expected_parameters(
    data: Map<String, Value>,
) -> Result<(String, String, String, String), RequestResult> {
    const ORGANIZATION_NAME_KEY: &str = "name";
    const ORGANIZATION_COUNTRY_KEY: &str = "country";
    const ORGANIZATION_ADDRESS_KEY: &str = "address";
    const ORGANIZATION_TELEPHONE_KEY: &str = "telephone";

    let name = match extract_parameter_from_request_data::<String>(&data, ORGANIZATION_NAME_KEY) {
        Ok(name) => name,
        Err(error) => return Err(error),
    };

    let country =
        match extract_parameter_from_request_data::<String>(&data, ORGANIZATION_COUNTRY_KEY) {
            Ok(country) => country,
            Err(error) => return Err(error),
        };

    let address =
        match extract_parameter_from_request_data::<String>(&data, ORGANIZATION_ADDRESS_KEY) {
            Ok(address) => address,
            Err(error) => return Err(error),
        };

    let telephone =
        match extract_parameter_from_request_data::<String>(&data, ORGANIZATION_TELEPHONE_KEY) {
            Ok(telephone) => telephone,
            Err(error) => return Err(error),
        };

    Ok((name, country, address, telephone))
}

#[cfg(test)]
async fn setup(logic_request_channel_boundary: usize) -> (Request, Sender<LogicRequest>) {
    let request = Request::new(Map::new());
    let (sender, receiver) = async_channel::bounded(logic_request_channel_boundary);

    (request, sender)
}

#[tokio::test]
async fn error_if_action_is_unknown() {
    let request_header_json =
        "{ \"token\": \"abcd\", \"element\": \"organization\", \"action\": \"fail\" }";
    let mut map = Map::new();
    map.insert(
        "header".to_string(),
        serde_json::from_str::<Value>(request_header_json).unwrap(),
    );

    let request = Request::new(map);

    let (sender, receiver) = async_channel::bounded(1usize);

    let result = request_handler(request, sender).await;

    match result {
        RequestResult::Ok(_) => panic!("expected RequestResult::Err"),
        RequestResult::Err(error) => {
            assert_eq!(RequestResultErrorKind::MalformedRequest, error.kind());
            assert_eq!("unknown action 'fail'", error.message());
        }
    }
}

#[tokio::test]
async fn error_if_missing_authorized_token() {
    let request_header_json =
        "{ \"token\": \"abcd\", \"element\": \"organization\", \"action\": \"create\" }";
    let mut map = Map::new();
    map.insert(
        "header".to_string(),
        serde_json::from_str::<Value>(request_header_json).unwrap(),
    );

    let request = Request::new(map);

    let (sender, receiver) = async_channel::bounded(1usize);

    let result = request_handler(request, sender).await;

    match result {
        RequestResult::Ok(_) => panic!("expected RequestResult::Err"),
        RequestResult::Err(error) => {
            assert_eq!(RequestResultErrorKind::InternalFailure, error.kind());
            assert_eq!(
                "missing authorized token after authorization",
                error.message()
            );
        }
    }
}
