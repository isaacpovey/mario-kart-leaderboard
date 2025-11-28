use crate::graphql::context::GraphQLContext;
use crate::graphql::matches::types::Match;
use crate::models;
use crate::models::TournamentStatType as ModelStatType;
use async_graphql::*;
use chrono::NaiveDate;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Copy, Enum, PartialEq, Eq)]
pub enum TournamentStatType {
    BestTeammate,
    WorstTeammate,
    BestRace,
    WorstRace,
    BiggestSwing,
    MostHelped,
    MostHurt,
    BestMatch,
    WorstMatch,
}

impl From<ModelStatType> for TournamentStatType {
    fn from(model: ModelStatType) -> Self {
        match model {
            ModelStatType::BestTeammate => Self::BestTeammate,
            ModelStatType::WorstTeammate => Self::WorstTeammate,
            ModelStatType::BestRace => Self::BestRace,
            ModelStatType::WorstRace => Self::WorstRace,
            ModelStatType::BiggestSwing => Self::BiggestSwing,
            ModelStatType::MostHelped => Self::MostHelped,
            ModelStatType::MostHurt => Self::MostHurt,
            ModelStatType::BestMatch => Self::BestMatch,
            ModelStatType::WorstMatch => Self::WorstMatch,
        }
    }
}

#[derive(Clone)]
pub struct TournamentStat {
    pub id: Uuid,
    pub tournament_id: Uuid,
    pub stat_type: TournamentStatType,
    pub player_id: Uuid,
    pub value: i32,
    pub extra_data: Option<serde_json::Value>,
}

impl From<models::TournamentStat> for TournamentStat {
    fn from(model: models::TournamentStat) -> Self {
        Self {
            id: model.id,
            tournament_id: model.tournament_id,
            stat_type: model.stat_type.into(),
            player_id: model.player_id,
            value: model.value,
            extra_data: model.extra_data,
        }
    }
}

#[Object]
impl TournamentStat {
    async fn id(&self) -> ID {
        ID(self.id.to_string())
    }

    async fn stat_type(&self) -> TournamentStatType {
        self.stat_type
    }

    async fn player_id(&self) -> ID {
        ID(self.player_id.to_string())
    }

    async fn value(&self) -> i32 {
        self.value
    }

    async fn extra_data(&self) -> Option<String> {
        self.extra_data.as_ref().map(|v| v.to_string())
    }
}

#[derive(Clone)]
pub struct Tournament {
    pub id: Uuid,
    pub group_id: Uuid,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub winner: Option<Uuid>,
}

impl From<crate::models::Tournament> for Tournament {
    fn from(model: crate::models::Tournament) -> Self {
        Self {
            id: model.id,
            group_id: model.group_id,
            start_date: model.start_date,
            end_date: model.end_date,
            winner: model.winner,
        }
    }
}

#[Object]
impl Tournament {
    async fn id(&self) -> ID {
        ID(self.id.to_string())
    }

    async fn start_date(&self) -> Option<String> {
        self.start_date.map(|d| d.to_string())
    }

    async fn end_date(&self) -> Option<String> {
        self.end_date.map(|d| d.to_string())
    }

    async fn winner_id(&self) -> Option<ID> {
        self.winner.map(|id| ID(id.to_string()))
    }

    async fn matches(&self, ctx: &Context<'_>) -> Result<Vec<Match>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let matches = gql_ctx
            .matches_by_tournament_loader
            .load_one(self.id)
            .await?
            .unwrap_or_default();

        Ok(matches.into_iter().map(Match::from).collect())
    }

    async fn leaderboard(&self, ctx: &Context<'_>) -> Result<Vec<LeaderboardEntry>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let entries =
            models::PlayerTournamentScore::get_tournament_leaderboard(&gql_ctx.pool, self.id)
                .await?;

        Ok(entries
            .into_iter()
            .map(|(player_id, player_name, elo_rating, all_time_elo, avatar_filename)| LeaderboardEntry {
                player_id,
                player_name,
                elo_rating,
                all_time_elo,
                avatar_filename,
            })
            .collect())
    }

    async fn stats(&self, ctx: &Context<'_>) -> Result<Vec<TournamentStat>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let stats = models::TournamentStat::find_by_tournament_id(&gql_ctx.pool, self.id).await?;

        Ok(stats.into_iter().map(TournamentStat::from).collect())
    }
}

#[derive(Clone)]
pub struct LeaderboardEntry {
    pub player_id: Uuid,
    pub player_name: String,
    pub elo_rating: i32,
    pub all_time_elo: i32,
    pub avatar_filename: Option<String>,
}

#[Object]
impl LeaderboardEntry {
    async fn player_id(&self) -> ID {
        ID(self.player_id.to_string())
    }

    async fn player_name(&self) -> &str {
        &self.player_name
    }

    #[graphql(name = "allTimeEloRating")]
    async fn elo_rating(&self) -> i32 {
        self.elo_rating
    }

    async fn all_time_elo(&self) -> i32 {
        self.all_time_elo
    }

    async fn avatar_filename(&self) -> Option<&str> {
        self.avatar_filename.as_deref()
    }
}

#[derive(Clone)]
pub struct ActiveTournamentWithLeaderboard {
    pub tournament: Tournament,
    pub leaderboard: Vec<LeaderboardEntry>,
}

#[Object]
impl ActiveTournamentWithLeaderboard {
    async fn id(&self) -> ID {
        ID(self.tournament.id.to_string())
    }

    async fn start_date(&self) -> Option<String> {
        self.tournament.start_date.map(|d| d.to_string())
    }

    async fn end_date(&self) -> Option<String> {
        self.tournament.end_date.map(|d| d.to_string())
    }

    async fn winner_id(&self) -> Option<ID> {
        self.tournament.winner.map(|id| ID(id.to_string()))
    }

    async fn leaderboard(&self) -> &[LeaderboardEntry] {
        &self.leaderboard
    }

    async fn matches(&self, ctx: &Context<'_>) -> Result<Vec<Match>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let matches = gql_ctx
            .matches_by_tournament_loader
            .load_one(self.tournament.id)
            .await?
            .unwrap_or_default();

        Ok(matches.into_iter().map(Match::from).collect())
    }
}

#[derive(Clone, SimpleObject)]
pub struct TournamentSummary {
    #[graphql(name = "id")]
    pub id_str: ID,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub winner_id: Option<ID>,
}

impl From<models::Tournament> for TournamentSummary {
    fn from(model: models::Tournament) -> Self {
        Self {
            id_str: ID(model.id.to_string()),
            start_date: model.start_date.map(|d| d.to_string()),
            end_date: model.end_date.map(|d| d.to_string()),
            winner_id: model.winner.map(|id| ID(id.to_string())),
        }
    }
}

#[derive(Clone, SimpleObject)]
pub struct EloDataPoint {
    pub timestamp: String,
    pub elo: i32,
}

#[derive(Clone, SimpleObject)]
pub struct PlayerEloHistory {
    pub player_id: ID,
    pub player_name: String,
    pub data_points: Vec<EloDataPoint>,
}

#[derive(Clone)]
pub struct TournamentDetail {
    pub id: Uuid,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub winner: Option<Uuid>,
    pub leaderboard: Vec<LeaderboardEntry>,
    pub stats: Vec<TournamentStat>,
    pub player_elo_history: Vec<PlayerEloHistory>,
}

#[Object]
impl TournamentDetail {
    async fn id(&self) -> ID {
        ID(self.id.to_string())
    }

    async fn start_date(&self) -> Option<String> {
        self.start_date.map(|d| d.to_string())
    }

    async fn end_date(&self) -> Option<String> {
        self.end_date.map(|d| d.to_string())
    }

    async fn winner_id(&self) -> Option<ID> {
        self.winner.map(|id| ID(id.to_string()))
    }

    async fn leaderboard(&self) -> &[LeaderboardEntry] {
        &self.leaderboard
    }

    async fn stats(&self) -> &[TournamentStat] {
        &self.stats
    }

    async fn player_elo_history(&self) -> &[PlayerEloHistory] {
        &self.player_elo_history
    }
}

pub fn build_player_elo_history(
    raw_data: Vec<(Uuid, String, i32, String)>,
    all_players: &[(Uuid, String)],
) -> Vec<PlayerEloHistory> {
    let all_timestamps: Vec<String> = {
        let mut timestamps: Vec<String> = raw_data.iter().map(|(_, _, _, ts)| ts.clone()).collect();
        timestamps.sort();
        timestamps.dedup();
        timestamps
    };

    if all_timestamps.is_empty() {
        return all_players
            .iter()
            .map(|(player_id, player_name)| PlayerEloHistory {
                player_id: ID(player_id.to_string()),
                player_name: player_name.clone(),
                data_points: Vec::new(),
            })
            .collect();
    }

    let mut player_elo_at_timestamp: HashMap<Uuid, HashMap<String, i32>> = HashMap::new();
    let mut player_names: HashMap<Uuid, String> = HashMap::new();

    raw_data.into_iter().for_each(|(player_id, name, elo, timestamp)| {
        player_names.insert(player_id, name);
        player_elo_at_timestamp
            .entry(player_id)
            .or_default()
            .insert(timestamp, elo);
    });

    for (player_id, player_name) in all_players {
        player_names.entry(*player_id).or_insert_with(|| player_name.clone());
    }

    player_names
        .into_iter()
        .map(|(player_id, player_name)| {
            let player_timestamps = player_elo_at_timestamp.get(&player_id);
            let mut current_elo = 1200;

            let data_points: Vec<EloDataPoint> = all_timestamps
                .iter()
                .map(|timestamp| {
                    if let Some(timestamps_map) = player_timestamps {
                        if let Some(&elo) = timestamps_map.get(timestamp) {
                            current_elo = elo;
                        }
                    }
                    EloDataPoint {
                        timestamp: timestamp.clone(),
                        elo: current_elo,
                    }
                })
                .collect();

            PlayerEloHistory {
                player_id: ID(player_id.to_string()),
                player_name,
                data_points,
            }
        })
        .collect()
}
