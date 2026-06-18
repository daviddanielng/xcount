use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProfileData {
    #[serde(rename = "mainEntity")]
    pub(crate) main_entity: MainEntity,
}
#[derive(Debug, Deserialize)]
pub struct MainEntity {
    #[serde(rename = "interactionStatistic")]
    pub(crate) interaction_statistic: Vec<InteractionCounter>,
}

#[derive(Debug, Deserialize)]
pub struct InteractionCounter {
    pub(crate) name: String,
    #[serde(rename = "userInteractionCount")]
    pub(crate) count: u64,
}
