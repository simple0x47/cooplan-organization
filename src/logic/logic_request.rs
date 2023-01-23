use crate::logic::actions::organization_logic_action::OrganizationLogicAction;
use crate::logic::actions::organization_root_logic_action::OrganizationRootLogicAction;

pub enum LogicRequest {
    OrganizationRequest(OrganizationLogicAction),
    OrganizationRootRequest(OrganizationRootLogicAction)
}
