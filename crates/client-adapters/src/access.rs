//! Wire types of sign-in and rights, turned into the client core's types.

use veltrunox_client_app::{ManagedUser, Rights, SignedIn};
use veltrunox_core::UserId;
use veltrunox_protocol::{IdentityDto, SessionDto, UserDto, capability_from_dto, role_from_dto};

pub fn rights_from_dto(dto: &IdentityDto) -> Rights {
    Rights {
        user_id: dto.user_id.map(UserId::new),
        user_name: dto.user_name.clone(),
        role: role_from_dto(dto.role),
        capabilities: dto
            .capabilities
            .iter()
            .copied()
            .map(capability_from_dto)
            .collect(),
    }
}

pub fn signed_in_from_dto(dto: &SessionDto) -> SignedIn {
    SignedIn {
        token: dto.token.clone(),
        rights: rights_from_dto(&dto.identity),
    }
}

pub fn managed_user_from_dto(dto: &UserDto) -> ManagedUser {
    ManagedUser {
        id: UserId::new(dto.id),
        user_name: dto.user_name.clone(),
        role: role_from_dto(dto.role),
        active: dto.active,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use veltrunox_core::{Capability, Role};
    use veltrunox_protocol::{CapabilityDto, RoleDto};

    #[test]
    fn identities_become_rights() {
        let rights = rights_from_dto(&IdentityDto {
            user_id: None,
            user_name: None,
            role: RoleDto::Guest,
            capabilities: vec![CapabilityDto::PlayControl, CapabilityDto::Upload],
        });
        assert_eq!(rights.role, Role::Guest);
        assert_eq!(
            rights.capabilities,
            [Capability::PlayControl, Capability::Upload]
        );
    }
}
