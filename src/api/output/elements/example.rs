use cooplan_amqp_api::api::output::amqp_output_element::AmqpOutputElement;
use cooplan_amqp_api::error::{Error, ErrorKind};
use cooplan_lapin_wrapper::config::api::Api;
use cooplan_state_tracker::state_tracker_client::StateTrackerClient;

const ELEMENT_ID: &str = "example";

pub fn get(api: &Api, state_sender: StateTrackerClient) -> Result<AmqpOutputElement, Error> {
    let api_config = match api
        .output()
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

    Ok(AmqpOutputElement::new(
        ELEMENT_ID.to_string(),
        api_config.clone(),
        state_sender,
    ))
}
