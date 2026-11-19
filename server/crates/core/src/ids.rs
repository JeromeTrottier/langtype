//! Typed identifiers. All rows use UUIDv7 (time-ordered), see DOMAIN_MODEL §15.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            /// A new time-ordered identifier.
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Uuid::from_str(s).map(Self)
            }
        }
    };
}

id_type!(UserId);
id_type!(WorkspaceId);
id_type!(KbId);
id_type!(EntityId);
id_type!(RelationId);
id_type!(EventId);
id_type!(ActivityRunId);
id_type!(ConnectionId);
id_type!(ChangesetId);
id_type!(SessionId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_time_ordered() {
        let a = EntityId::new();
        let b = EntityId::new();
        assert!(a < b, "UUIDv7 ids must sort by creation time");
    }

    #[test]
    fn ids_round_trip_through_strings() {
        let id = KbId::new();
        let parsed: KbId = id.to_string().parse().unwrap();
        assert_eq!(id, parsed);
    }
}
