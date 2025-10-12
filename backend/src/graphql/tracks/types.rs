use async_graphql::*;
use uuid::Uuid;

#[derive(Clone)]
pub struct Track {
    pub id: Uuid,
    pub name: String,
}

impl From<crate::models::Track> for Track {
    fn from(model: crate::models::Track) -> Self {
        Self {
            id: model.id,
            name: model.name,
        }
    }
}

#[Object]
impl Track {
    async fn id(&self) -> ID {
        ID(self.id.to_string())
    }

    async fn name(&self) -> &str {
        &self.name
    }
}
