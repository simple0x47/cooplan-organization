use std::str::FromStr;

pub enum Permission {
    CreateOrganization,
    JoinOrganization,
    ReadOrganization,
    UpdateOrganization,
    DeleteOrganization,
    RequestPermissionForOrganization,
    InviteUser,
    UpdateUser,
    DeleteUser,
}

impl FromStr for Permission {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "create:org" => Ok(Permission::CreateOrganization),
            "join:org" => Ok(Permission::JoinOrganization),
            "read:org" => Ok(Permission::ReadOrganization),
            "update:org" => Ok(Permission::UpdateOrganization),
            "delete:org" => Ok(Permission::DeleteOrganization),
            "request_permission:org" => Ok(Permission::RequestPermissionForOrganization),
            "invite:user" => Ok(Permission::InviteUser),
            "update:user" => Ok(Permission::UpdateUser),
            "delete:user" => Ok(Permission::DeleteUser),
            _ => Err(format!("unknown permission '{}'", value)),
        }
    }
}

impl ToString for Permission {
    fn to_string(&self) -> String {
        match self {
            Permission::CreateOrganization => "create:org".to_string(),
            Permission::JoinOrganization => "join:org".to_string(),
            Permission::ReadOrganization => "read:org".to_string(),
            Permission::UpdateOrganization => "update:org".to_string(),
            Permission::DeleteOrganization => "delete:org".to_string(),
            Permission::RequestPermissionForOrganization => "request_permission:org".to_string(),
            Permission::InviteUser => "invite:user".to_string(),
            Permission::UpdateUser => "update:user".to_string(),
            Permission::DeleteUser => "delete:user".to_string(),
        }
    }
}

pub fn organization_creator_permissions() -> Vec<String> {
    vec![
        Permission::ReadOrganization.to_string(),
        Permission::UpdateOrganization.to_string(),
        Permission::DeleteOrganization.to_string(),
        Permission::RequestPermissionForOrganization.to_string(),
        Permission::InviteUser.to_string(),
        Permission::UpdateUser.to_string(),
        Permission::DeleteUser.to_string(),
    ]
}
