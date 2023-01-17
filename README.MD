# Organization
[![Continuous integration](https://github.com/simple0x47/cooplan-organization/actions/workflows/ci.yml/badge.svg)](https://github.com/simple0x47/cooplan-organization/actions/workflows/ci.yml)

Microservice in charge of storing & managing organizations and their users.

## API

* `organization`:
  * `create #AUTH (*name, country, address, *telephone) -> Result<Organization, Error>` 
    * User must have no other organizations in order to be able to use this functionality.
  <br></br>
  * `read #AUTH (organization_id) -> Result<Organization, Error>`
  <br></br>
  * `update #AUTH (*name, country, address, *telephone) -> Result<(), Error>`
    * User must have the permission to update the organization.
  <br></br>
  * `delete #AUTH (organization_id) -> Result<(), Error>`
    * There must be no other users in the organization other than the one deleting the organization.
  <br></br>
  * `request_access #AUTH (organization_type, definition_category) -> Result<(), Error>`
    * `organization_type: ` `producer`, `modifier`, `service provider`, `endpoint`.
  <br></br>
* `user`:
  * `invite #AUTH (email, permissions) -> Result<(), Error>`: invites a user by their email,
  generating a specific **invite code**.
  <br></br>
  * `edit_permissions #AUTH (target_user, permissions) -> Result<(), Error>`: within the organization's permissions,
  select which ones the user has access to.
  <br></br>
  * `remove #AUTH (target_user) -> Result<(), Error>`: remove user from the organization.
  <br></br>
  * `get_user_permissions_and_organization #AUTH () -> Result<User, Error>`
