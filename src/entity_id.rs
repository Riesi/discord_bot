use serde::{Deserialize, Serialize};
use serenity::model::id::{RoleId, UserId};

#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct EntityId(pub u64);

impl PartialEq<UserId> for EntityId {
    fn eq(&self, other: &UserId) -> bool {
        self.0 == other.0
    }
}
impl PartialEq<EntityId> for UserId {
    fn eq(&self, other: &EntityId) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<RoleId> for EntityId {
    fn eq(&self, other: &RoleId) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<EntityId> for RoleId {
    fn eq(&self, other: &EntityId) -> bool {
        self.0 == other.0
    }
}

impl From<EntityId> for UserId {
    fn from(item: EntityId) -> Self {
        UserId { 0: item.0 }
    }
}
impl From<UserId> for EntityId {
    fn from(item: UserId) -> Self {
        EntityId { 0: item.0 }
    }
}

impl From<EntityId> for RoleId {
    fn from(item: EntityId) -> Self {
        RoleId { 0: item.0 }
    }
}
impl From<RoleId> for EntityId {
    fn from(item: RoleId) -> Self {
        EntityId { 0: item.0 }
    }
}

impl From<EntityId> for u64 {
    fn from(item: EntityId) -> Self {
        item.0
    }
}

impl From<u64> for EntityId {
    fn from(item: u64) -> Self {
        EntityId { 0: item }
    }
}