use crate::deep_lynx::APIError::MissingFields;
use jwt::{Claims, Header, Token, Unverified};
use multipart::client::lazy::Multipart;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::io::{BufReader, Read};
use std::num::ParseIntError;
use std::path::PathBuf;
use thiserror::Error;
use ureq::serde_json::Value;

#[derive(Error, Debug)]
pub enum APIError {
    #[error("unknown API error")]
    Unknown,
    #[error("deeplynx API returned an error")]
    DeepLynx(String),
    #[error("missing required fields")]
    MissingFields(Option<Vec<String>>),
    #[error("jwt parse")]
    JWTParsingError(#[from] jwt::error::Error),
    #[error("missing iat claim")]
    MissingIATError,
    #[error("parse int error")]
    ParseIntError(#[from] ParseIntError),
    #[error("io error")]
    IOError(#[from] io::Error),
    #[error("ureq error")]
    UreqError(#[from] ureq::Error),
    #[error("json error")]
    JSONParsing(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct DeepLynxAPI {
    client: ureq::Agent,
    server: String,
    bearer_token: Option<String>,
    api_key: Option<String>,
    api_secret: Option<String>,
    secured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResult {
    value: Option<Value>,
    error: Option<ErrorResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    message: Option<String>,
    code: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitiateDataSourceDownloadResponse {
    // there are more fields but we only care about these
    pub id: String,                     // comes back as strong from json due to bigint
    pub container_id: String,           // comes back as string from json due to bigint
    pub data_source_id: Option<String>, // comes back as string from json due to bigint
    pub file_name: String,
    pub file_size: f64,
    pub md5hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitiateDataSourceDownloadQuery {
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub secondary_index_name: Option<String>,
    pub secondary_index_start_value: Option<u64>,
}

impl DeepLynxAPI {
    pub fn new(
        server: String,
        api_key: Option<String>,
        api_secret: Option<String>,
    ) -> Result<DeepLynxAPI, APIError> {
        let client = ureq::AgentBuilder::new().build();

        Ok(DeepLynxAPI {
            client,
            server,
            secured: api_key.is_some() && api_secret.is_some(),
            api_key,
            api_secret,
            bearer_token: None,
        })
    }

    // get token should be called by the root GET/POST/PUT etc. and only if the JWT has expired
    fn get_token(&mut self) -> Result<(), APIError> {
        let api_key = match &self.api_key {
            None => return Err(APIError::MissingFields(Some(vec!["api_key".to_string()]))),
            Some(k) => k,
        };

        let api_secret = match &self.api_secret {
            None => {
                return Err(APIError::MissingFields(Some(
                    vec!["api_secret".to_string()],
                )))
            }
            Some(k) => k,
        };

        let server = &self.server; // do this because format! is kinda bad at accessing fields
        let token = self
            .client
            .get(format!("{server}/oauth/token").as_str())
            .set("x-api-key", api_key)
            .set("x-api-secret", api_secret)
            .set("expiry", "12h")
            .call()?
            .into_string()?;

        // remove quotes from token text()
        let token = &token[1..token.len() - 1];
        self.bearer_token = Some(token.to_string());

        Ok(())
    }

    fn token_expired(&self) -> Result<bool, APIError> {
        match &self.bearer_token {
            None => Ok(false), // return false because it might be we're calling unsecured here
            Some(token) => {
                let parsed: Token<Header, Claims, Unverified> =
                    jwt::Token::parse_unverified(token)?;

                let claims = parsed.claims();

                let exp = match claims.registered.expiration {
                    None => return Err(APIError::MissingIATError),
                    Some(id) => id,
                };

                let current_time = chrono::Utc::now().timestamp();

                Ok(exp <= current_time as u64)
            }
        }
    }

    pub fn import(
        &mut self,
        container_id: u64,
        data_source_id: u64,
        file: Option<PathBuf>,
        data: Option<Vec<u8>>,
    ) -> Result<(), APIError> {
        if (self.secured && self.bearer_token.is_none()) || (self.token_expired()? && self.secured)
        {
            self.get_token()?;
        }

        let server = &self.server;
        let route = format!(
            "{server}/containers/{container_id}/import/datasources/{data_source_id}/imports?fastLoad=true"
        );
        let mut agent = self.client.post(route.as_str());

        match &self.bearer_token {
            None => {} // no bearer token = no attachment but also no error
            Some(t) => agent = agent.set("Authorization", format!("Bearer {t}").as_str()),
        }

        match file {
            None => {}
            Some(f) => {
                let guess = match new_mime_guess::from_path(f.clone()).first() {
                    None => return Err(APIError::Unknown),
                    Some(m) => m,
                };

                let opened = BufReader::new(File::open(f.clone())?);

                let mut m = Multipart::new();
                m.add_text("key", "value");
                m.add_stream("data", opened, f.to_str(), Some(guess));

                let mdata = m.prepare().unwrap();
                agent
                    .set(
                        "Content-Type",
                        &format!("multipart/form-data; boundary={}", mdata.boundary()),
                    )
                    .send(mdata)?;

                return Ok(());
            }
        }

        match data {
            None => {}
            Some(d) => {
                agent.send_json(d)?;
                return Ok(());
            }
        }

        Err(MissingFields(None))
    }

    pub fn initiate_data_source_download(
        &mut self,
        container_id: u64,
        data_source_id: u64,
        query_options: InitiateDataSourceDownloadQuery,
    ) -> Result<InitiateDataSourceDownloadResponse, APIError> {
        if (self.secured && self.bearer_token.is_none()) || (self.token_expired()? && self.secured)
        {
            self.get_token()?;
        }

        let server = &self.server;
        let route = format!(
            "{server}/containers/{container_id}/import/datasources/{data_source_id}/download?startTime={}&endTime={}&secondaryIndexName={}&secondaryIndexStartValue={}",
            query_options.start_time.unwrap_or("".to_string()),
            query_options.end_time.unwrap_or("".to_string()),
            query_options.secondary_index_name.unwrap_or("".to_string()),
            query_options.secondary_index_start_value.unwrap_or(0),

        );
        let mut agent = self.client.get(route.as_str());

        match &self.bearer_token {
            None => {} // no bearer token = no attachment but also no error
            Some(t) => agent = agent.set("Authorization", format!("Bearer {t}").as_str()),
        }

        let response = agent.call()?;
        let response: ApiResult = response.into_json()?;

        match response.error {
            None => {}
            Some(e) => {
                return Err(APIError::DeepLynx(format!(
                    "{}-{}",
                    e.code.unwrap_or(500),
                    e.message
                        .unwrap_or(String::from("DeepLynx responded with an error"))
                )))
            }
        }

        match response.value {
            None => Err(APIError::DeepLynx(String::from(
                "unable to parse response body from DeepLynx",
            ))),
            Some(v) => Ok(serde_json::from_value(v)?),
        }
    }

    pub fn download_file(
        &mut self,
        container_id: u64,
        file_id: u64,
        delete_after: bool,
    ) -> Result<Box<dyn Read + Send + Sync>, APIError> {
        if (self.secured && self.bearer_token.is_none()) || (self.token_expired()? && self.secured)
        {
            self.get_token()?;
        }

        let server = &self.server;
        let route = format!("{server}/containers/{container_id}/files/{file_id}/download?deleteAfter={delete_after}");
        let mut agent = self.client.get(route.as_str());

        match &self.bearer_token {
            None => {} // no bearer token = no attachment but also no error
            Some(t) => agent = agent.set("Authorization", format!("Bearer {t}").as_str()),
        }

        let response = agent.call()?;
        Ok(response.into_reader())
    }
}
