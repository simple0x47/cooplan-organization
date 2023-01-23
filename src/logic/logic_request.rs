use crate::logic::actions::organization_logic_action::OrganizationLogicAction;
use crate::logic::actions::organization_root_logic_action::OrganizationRootLogicAction;
use crate::logic::actions::user_logic_action::UserLogicAction;

pub enum LogicRequest {
    Organization(OrganizationLogicAction),
    OrganizationRoot(OrganizationRootLogicAction),
    User(UserLogicAction),
}
