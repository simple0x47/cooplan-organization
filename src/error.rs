use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ErrorKind {
    ApiFailure,
    ApiRequestFailure,
    AutoConfigFailure,
    InternalFailure,
    StorageFailure,
    InvalidArgument,
    InvalidCountry,
    InvalidTelephone,
    NameAlreadyTaken,
    TelephoneAlreadyInUse,
    UserCannotCreateOrganization,
    UserCannotJoinAnyOrganization,
    InvitationNotFound,
    InvitationHasExpired,
    OrganizationNotFound,
    ProcessReversion,
}

impl From<cooplan_amqp_api::error::ErrorKind> for ErrorKind {
    fn from(error_kind: cooplan_amqp_api::error::ErrorKind) -> Self {
        match error_kind {
            cooplan_amqp_api::error::ErrorKind::TokenDecodingFailure => {
                ErrorKind::ApiRequestFailure
            }
            cooplan_amqp_api::error::ErrorKind::MalformedToken => ErrorKind::ApiRequestFailure,
            cooplan_amqp_api::error::ErrorKind::InvalidToken => ErrorKind::ApiRequestFailure,
            cooplan_amqp_api::error::ErrorKind::PermissionNotFound => ErrorKind::ApiRequestFailure,
            cooplan_amqp_api::error::ErrorKind::ApiRunnerAutoConfigFailure => ErrorKind::ApiFailure,
            cooplan_amqp_api::error::ErrorKind::ApiConnectionFailure => ErrorKind::ApiFailure,
            cooplan_amqp_api::error::ErrorKind::ApiExecutionFailure => ErrorKind::ApiFailure,
            cooplan_amqp_api::error::ErrorKind::AutoConfigFailure => ErrorKind::AutoConfigFailure,
            cooplan_amqp_api::error::ErrorKind::InternalFailure => ErrorKind::InternalFailure,
            cooplan_amqp_api::error::ErrorKind::SanitizationFailure => ErrorKind::ApiRequestFailure,
            cooplan_amqp_api::error::ErrorKind::AuthorizationFailure => {
                ErrorKind::ApiRequestFailure
            }
            cooplan_amqp_api::error::ErrorKind::MalformedRequest => ErrorKind::ApiRequestFailure,
            cooplan_amqp_api::error::ErrorKind::ApiNotFound => ErrorKind::ApiRequestFailure,
            cooplan_amqp_api::error::ErrorKind::ApiRouterFailure => ErrorKind::ApiFailure,
            cooplan_amqp_api::error::ErrorKind::AmqpFailure => ErrorKind::ApiFailure,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}

impl Error {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Error {
        Error {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<cooplan_amqp_api::error::Error> for Error {
    fn from(error: cooplan_amqp_api::error::Error) -> Self {
        Error::new(error.kind().into(), error.message)
    }
}
