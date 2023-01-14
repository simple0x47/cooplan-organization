use crate::logic::actions::example_logic_action::ExampleLogicAction;

pub enum LogicRequest {
    OrganizationRequest(ExampleLogicAction),
}
