use crate::api::input::elements;
use crate::logic::logic_request::LogicRequest;
use cooplan_amqp_api::api::input::input_element::InputElement;
use cooplan_amqp_api::error::Error;
use cooplan_lapin_wrapper::config::api::Api;

pub fn register(api: &Api) -> Result<Vec<InputElement<LogicRequest>>, Error> {
    let elements: Vec<InputElement<LogicRequest>> =
        vec![elements::organization::get(api)?, elements::user::get(api)?];

    Ok(elements)
}
