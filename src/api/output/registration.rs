use crate::api::output::elements;
use cooplan_amqp_api::api::output::amqp_output_element::AmqpOutputElement;
use cooplan_amqp_api::error::Error;
use cooplan_lapin_wrapper::config::api::Api;
use cooplan_state_tracker::state_tracker_client::StateTrackerClient;

pub fn register(
    api: &Api,
    state_tracker: StateTrackerClient,
) -> Result<Vec<AmqpOutputElement>, Error> {
    let elements: Vec<AmqpOutputElement> = vec![];

    Ok(elements)
}
