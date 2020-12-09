use reqwest::{Client as ReqwestClient, Error as ReqwestError};
use serde_json::Value;
use std::{collections::HashMap, result::Result as StdResult};

#[derive(Debug)]
pub enum RobloxError {
    Reqwest(ReqwestError),
    MissingField,
}

type Result<T> = StdResult<T, RobloxError>;

#[derive(Clone, Default)]
pub struct Client {
    client: ReqwestClient,
}

impl Client {
    pub async fn get_user_roles(&self, roblox_id: i64) -> Result<HashMap<i64, i64>> {
        let url = format!(
            "https://groups.roblox.com/v2/users/{}/groups/roles",
            roblox_id
        );
        let body: Value = self.client.get(&url).send().await?.json::<Value>().await?;

        if let Some(resp) = body["data"].as_array() {
            let mut ranks = HashMap::new();
            for rank in resp.iter() {
                ranks.insert(
                    rank["group"]["id"].as_i64().unwrap(),
                    rank["role"]["rank"].as_i64().unwrap(),
                );
            }
            return Ok(ranks);
        }

        Err(RobloxError::MissingField)
    }

    pub async fn get_username_from_id(&self, roblox_id: i64) -> Result<String> {
        let url = format!("https://api.roblox.com/users/{}", roblox_id);
        let body = self.client.get(&url).send().await?.json::<Value>().await?;

        body["Username"]
            .as_str()
            .map_or(Err(RobloxError::MissingField), |r| Ok(r.to_string()))
    }

    pub async fn get_id_from_username(&self, username: &str) -> Result<Option<i64>> {
        let url = format!(
            "https://api.roblox.com/users/get-by-username?username={}",
            username
        );
        let body = self.client.get(&url).send().await?.json::<Value>().await?;

        Ok(body["Id"].as_i64())
    }

    pub async fn has_asset(&self, roblox_id: i64, item: i64, asset_type: &str) -> Result<bool> {
        let url = format!(
            "https://inventory.roblox.com/v1/users/{}/items/{}/{}",
            roblox_id, asset_type, item
        );
        let body = self.client.get(&url).send().await?.json::<Value>().await?;
        if let Some(data) = body["data"].as_array() {
            return Ok(!data.is_empty());
        }
        Ok(false)
    }

    pub async fn check_code(&self, roblox_id: i64, code: &str) -> Result<bool> {
        let url = format!("https://www.roblox.com/users/{}/profile", roblox_id);
        let body = self.client.get(&url).send().await?.text().await?;

        Ok(body.contains(code))
    }

    pub async fn get_group_rank(&self, group_id: i64, rank_id: i64) -> Result<Option<Value>> {
        let url = format!("https://groups.roblox.com/v1/groups/{}/roles", group_id);
        let body = self.client.get(&url).send().await?.json::<Value>().await?;
        let ranks_array = match body["roles"].as_array() {
            Some(a) => a,
            None => return Ok(None),
        };
        let rank = match ranks_array
            .iter()
            .find(|r| r["rank"].as_i64().unwrap_or_default() == rank_id)
        {
            Some(r) => r,
            None => return Ok(None),
        };
        Ok(Some(rank.to_owned()))
    }

    pub async fn get_group_ranks(
        &self,
        group_id: i64,
        min_rank: i64,
        max_rank: i64,
    ) -> Result<Vec<Value>> {
        let url = format!("https://groups.roblox.com/v1/groups/{}/roles", group_id);
        let body = self.client.get(&url).send().await?.json::<Value>().await?;
        let ranks_array = match body["roles"].as_array() {
            Some(a) => a,
            None => return Ok(Vec::new()),
        };
        let ranks = ranks_array
            .iter()
            .filter_map(|r| {
                let rank = r["rank"].as_i64().unwrap();
                if rank >= min_rank && rank <= max_rank {
                    return Some(r.to_owned());
                }
                None
            })
            .collect::<Vec<Value>>();

        Ok(ranks)
    }
}

impl From<ReqwestError> for RobloxError {
    fn from(err: ReqwestError) -> Self {
        RobloxError::Reqwest(err)
    }
}
