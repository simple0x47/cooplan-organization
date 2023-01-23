use crate::logic::actions::organization_logic_action::OrganizationLogicAction;
use crate::logic::actions::organization_root_logic_action::OrganizationRootLogicAction;
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

const ACTIONS: &[&str] = &[
    "create",
    "join",
    "read",
    "update",
    "delete",
    "request_permission",
];

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
        "join" => join(authorized_token, data, logic_request_sender).await,
        "read" => read(authorized_token, data, logic_request_sender).await,
        _ => {
            return RequestResult::Err(RequestResultError::new(
                RequestResultErrorKind::MalformedRequest,
                format!("unknown action '{}'", action),
            ))
        }
    }
}

/// Expected parameters:
/// - **name**: String
/// - **country**: String
/// - **address**: String
/// - **telephone**: String
async fn create(
    authorized_token: Token,
    data: Map<String, Value>,
    logic_request_sender: Sender<LogicRequest>,
) -> RequestResult {
    let user_id = match extract_user_id_from_token(&authorized_token) {
        Ok(user_id) => user_id,
        Err(request_result) => return request_result,
    };

    let (name, country, address, telephone) = match extract_create_expected_parameters(data) {
        Ok((name, country, address, telephone)) => (name, country, address, telephone),
        Err(error) => return error,
    };

    let (replier, receiver) = tokio::sync::oneshot::channel();

    let action = OrganizationLogicAction::Create {
        user_id,
        name,
        country,
        address,
        telephone,
        replier,
    };

    match logic_request_sender
        .send(LogicRequest::OrganizationRequest(action))
        .await
    {
        Ok(_) => (),
        Err(error) => {
            return RequestResult::Err(RequestResultError::new(
                RequestResultErrorKind::InternalFailure,
                format!(
                    "failed to send organization create request to logic: {}",
                    error
                ),
            ))
        }
    }

    let result = match receiver.await {
        Ok(result) => match result {
            Ok(organization) => match serde_json::to_value(organization) {
                Ok(value) => RequestResult::Ok(value),
                Err(error) => RequestResult::Err(RequestResultError::new(
                    RequestResultErrorKind::InternalFailure,
                    format!("failed to serialize organization: {}", error),
                )),
            },
            Err(error) => RequestResult::Err(RequestResultError::new(
                RequestResultErrorKind::MalformedRequest,
                format!("failed to create organization: {}", error),
            )),
        },
        Err(error) => RequestResult::Err(RequestResultError::new(
            RequestResultErrorKind::InternalFailure,
            format!("failed to receive result from logic: {}", error),
        )),
    };

    result
}

fn extract_user_id_from_token(authorized_token: &Token) -> Result<String, RequestResult> {
    let user_id = match authorized_token.get("sub") {
        Some(user_id) => match user_id.as_str() {
            Some(user_id) => user_id.to_string(),
            None => {
                return Err(RequestResult::Err(RequestResultError::new(
                    RequestResultErrorKind::MalformedRequest,
                    "failed to read user id from token",
                )))
            }
        },
        None => {
            return Err(RequestResult::Err(RequestResultError::new(
                RequestResultErrorKind::MalformedRequest,
                "failed to read user id from token",
            )))
        }
    };

    Ok(user_id)
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

/// Expected parameters:
/// - **invitation_code**: String
async fn join(
    authorized_token: Token,
    data: Map<String, Value>,
    logic_request_sender: Sender<LogicRequest>,
) -> RequestResult {
    let user_id = match extract_user_id_from_token(&authorized_token) {
        Ok(user_id) => user_id,
        Err(request_result) => return request_result,
    };

    const ORGANIZATION_INVITATION_CODE_KEY: &str = "invitation_code";

    let invitation_code = match extract_parameter_from_request_data::<String>(
        &data,
        ORGANIZATION_INVITATION_CODE_KEY,
    ) {
        Ok(invitation_code) => invitation_code,
        Err(error) => return error,
    };

    let (replier, receiver) = tokio::sync::oneshot::channel();

    let action = OrganizationLogicAction::Join {
        user_id,
        invitation_code,
        replier,
    };

    match logic_request_sender
        .send(LogicRequest::OrganizationRequest(action))
        .await
    {
        Ok(_) => (),
        Err(error) => {
            return RequestResult::Err(RequestResultError::new(
                RequestResultErrorKind::InternalFailure,
                format!(
                    "failed to send organization join request to logic: {}",
                    error
                ),
            ))
        }
    }

    let result = match receiver.await {
        Ok(result) => match result {
            Ok(organization) => match serde_json::to_value(organization) {
                Ok(value) => RequestResult::Ok(value),
                Err(error) => RequestResult::Err(RequestResultError::new(
                    RequestResultErrorKind::InternalFailure,
                    format!("failed to serialize organization: {}", error),
                )),
            },
            Err(error) => RequestResult::Err(RequestResultError::new(
                RequestResultErrorKind::MalformedRequest,
                format!("failed to join organization: {}", error),
            )),
        },
        Err(error) => RequestResult::Err(RequestResultError::new(
            RequestResultErrorKind::InternalFailure,
            format!("failed to receive result from logic: {}", error),
        )),
    };

    result
}

/// Expected parameters:
/// - **organization_id**: String
async fn read(
    authorized_token: Token,
    data: Map<String, Value>,
    logic_request_sender: Sender<LogicRequest>,
) -> RequestResult {
    const READ_ORGANIZATION_ID_KEY: &str = "organization_id";

    let organization_id =
        match extract_parameter_from_request_data::<String>(&data, READ_ORGANIZATION_ID_KEY) {
            Ok(invitation_code) => invitation_code,
            Err(error) => return error,
        };

    let (replier, receiver) = tokio::sync::oneshot::channel();

    let action = OrganizationRootLogicAction::Read {
        organization_id,
        replier,
    };

    match logic_request_sender
        .send(LogicRequest::OrganizationRootRequest(action))
        .await
    {
        Ok(_) => (),
        Err(error) => {
            return RequestResult::Err(RequestResultError::new(
                RequestResultErrorKind::InternalFailure,
                format!(
                    "failed to send organization root read request to logic: {}",
                    error
                ),
            ))
        }
    }

    let result = match receiver.await {
        Ok(result) => match result {
            Ok(organization) => match serde_json::to_value(organization) {
                Ok(value) => RequestResult::Ok(value),
                Err(error) => RequestResult::Err(RequestResultError::new(
                    RequestResultErrorKind::InternalFailure,
                    format!("failed to serialize organization root: {}", error),
                )),
            },
            Err(error) => RequestResult::Err(RequestResultError::new(
                RequestResultErrorKind::MalformedRequest,
                format!("failed to read organization root: {}", error),
            )),
        },
        Err(error) => {
            return RequestResult::Err(RequestResultError::new(
                RequestResultErrorKind::InternalFailure,
                format!(
                    "failed to receive organization root read response from logic: {}",
                    error
                ),
            ))
        }
    };

    result
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
