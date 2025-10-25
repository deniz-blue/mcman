use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SideSupport {
    Required,
    Optional,
    Unsupported,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    Approved,
    Archived,
    Rejected,
    Draft,
    Unlisted,
    Processing,
    Withheld,
    Scheduled,
    Private,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ProjectType {
    Mod,
    ModPack,
    ResourcePack,
    Shader,
}

// TODO: `gallery`
// TODO: `license`
// TODO: `moderator_message`
// TODO: `monetization_status`
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModrinthProject {
    /// The ID of the project, encoded as a base62 string
    pub id: String,
    /// The ID of the team that has ownership of this project
    pub team: String,
    /// The date the project was published
    pub published: String,
    /// The date the project was last updated
    pub updated: String,
    /// The total number of users following the project
    pub followers: u64,
    /// The total number of downloads of the project
    pub downloads: u64,
    /// The date the project’s status was set to an approved status
    pub approved: Option<String>,
    /// The date the project’s status was submitted to moderators for review
    pub queued: Option<String>,
    /// The slug of a project, used for vanity URLs.
    pub slug: String,
    /// The title or name of the project
    pub title: String,    
    /// A short description of the project
    pub description: String,    
    /// A list of the categories that the project has
    pub categories: Vec<String>,
    /// The client side support of the project
    pub client_side: SideSupport,
    /// The server side support of the project
    pub server_side: SideSupport,
    /// A long form description of the project
    pub body: String,
    /// The status of the project
    pub status: ProjectStatus,
    /// The requested status when submitting for review or scheduling the project for release
    pub requested_status: Option<ProjectStatus>,
    /// A list of categories which are searchable but non-primary
    pub additional_categories: Vec<String>,
    /// An optional link to where to submit bugs or issues with the project
    pub issues_url: Option<String>,
    /// An optional link to the source code of the project
    pub source_url: Option<String>,
    /// An optional link to the project’s wiki page or other relevant information
    pub wiki_url: Option<String>,
    /// An optional invite link to the project’s discord
    pub discord_url: Option<String>,
    /// A list of donation links for the project
    pub donation_urls: Vec<DonationLink>,
    /// The project type of the project
    pub project_type: ProjectType,
    /// The URL of the project’s icon
    pub icon_url: Option<String>,
    /// The RGB color of the project, automatically generated from the project icon
    pub color: Option<u64>,
    /// The ID of the moderation thread associated with this project
    pub thread_id: Option<String>,
    /// A list of the version IDs of the project (will never be empty unless draft status)
    pub versions: Vec<String>,
    /// A list of all of the game versions supported by the project
    pub game_versions: Vec<String>,
    /// A list of all of the loaders supported by the project
    pub loaders: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DonationLink {
    /// The ID of the donation platform
    pub id: String,
    /// The donation platform this link is to
    pub platform: String,
    /// The URL of the donation platform and user
    pub url: String,
}
