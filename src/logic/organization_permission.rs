use std::str::FromStr;

/// Permissions of a user within an organization.
/// Although it is similar to the API level permissions, this has a great level of abstraction.
pub enum OrganizationPermission {
    ReadOrganization,
    UpdateOrganization,
    DeleteOrganization,
    RequestPermissionForOrganization,
    InviteUser,
    UpdateUser,
    DeleteUser,
}

impl FromStr for OrganizationPermission {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "read:org" => Ok(OrganizationPermission::ReadOrganization),
            "update:org" => Ok(OrganizationPermission::UpdateOrganization),
            "delete:org" => Ok(OrganizationPermission::DeleteOrganization),
            "request_permission:org" => {
                Ok(OrganizationPermission::RequestPermissionForOrganization)
            }
            "invite:user" => Ok(OrganizationPermission::InviteUser),
            "update:user" => Ok(OrganizationPermission::UpdateUser),
            "delete:user" => Ok(OrganizationPermission::DeleteUser),
            _ => Err(format!("unknown permission '{}'", value)),
        }
    }
}

impl ToString for OrganizationPermission {
    fn to_string(&self) -> String {
        match self {
            OrganizationPermission::ReadOrganization => "read:org".to_string(),
            OrganizationPermission::UpdateOrganization => "update:org".to_string(),
            OrganizationPermission::DeleteOrganization => "delete:org".to_string(),
            OrganizationPermission::RequestPermissionForOrganization => {
                "request_permission:org".to_string()
            }
            OrganizationPermission::InviteUser => "invite:user".to_string(),
            OrganizationPermission::UpdateUser => "update:user".to_string(),
            OrganizationPermission::DeleteUser => "delete:user".to_string(),
        }
    }
}

pub fn organization_creator_permissions() -> Vec<String> {
    vec![
        OrganizationPermission::ReadOrganization.to_string(),
        OrganizationPermission::UpdateOrganization.to_string(),
        OrganizationPermission::DeleteOrganization.to_string(),
        OrganizationPermission::RequestPermissionForOrganization.to_string(),
        OrganizationPermission::InviteUser.to_string(),
        OrganizationPermission::UpdateUser.to_string(),
        OrganizationPermission::DeleteUser.to_string(),
    ]
}
