use crate::api::input::elements;
use crate::logic::logic_request::LogicRequest;
use cooplan_amqp_api::api::input::input_element::InputElement;
use cooplan_amqp_api::error::Error;
use cooplan_lapin_wrapper::config::api::Api;

pub fn register(api: &Api) -> Result<Vec<InputElement<LogicRequest>>, Error> {
    todo!();

    let elements: Vec<InputElement<LogicRequest>> = vec![elements::example::get(api)?];

    Ok(elements)
}
