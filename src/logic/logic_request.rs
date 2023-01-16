use crate::logic::actions::organization_logic_action::OrganizationLogicAction;

pub enum LogicRequest {
    OrganizationRequest(OrganizationLogicAction),
}
