use super::Downloadable;

impl Downloadable {
    /// Check if just the version is differient from other
    pub fn is_same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Hangar { id: a, .. }, Self::Hangar { id: b, .. }) if a == b => true,
            (Self::CurseRinth { id: a, .. }, Self::CurseRinth { id: b, .. }) if a == b => true,
            (Self::CurseForge { id: a, .. }, Self::CurseForge { id: b, .. }) if a == b => true,
            (Self::Modrinth { id: a, .. }, Self::Modrinth { id: b, .. }) if a == b => true,
            (Self::Spigot { id: a, .. }, Self::Spigot { id: b, .. }) if a == b => true,
            (Self::Url { url: a, .. }, Self::Url { url: b, .. }) if a == b => true,
            _ => self == other,
        }
    }
}
