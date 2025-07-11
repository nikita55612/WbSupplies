use reqwest::Client;
use serde_json::json;
use std::time::Duration;

use crate::telebot::types;

const BASE_URL: &str = "https://api.telegram.org";

pub struct Bot {
    token: String,
    client: Client,
    timeout: Duration,
    chat_ids: Vec<String>,
    parse_mode: Option<String>,
}

pub struct BotBuilder {
    token: String,
    timeout: Duration,
    chat_ids: Vec<String>,
    parse_mode: Option<String>,
}

#[allow(dead_code)]
impl BotBuilder {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            timeout: Duration::from_secs(5),
            chat_ids: vec![],
            parse_mode: Some("HTML".to_string()),
        }
    }

    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = duration;
        self
    }

    pub fn parse_mode(mut self, mode: impl Into<String>) -> Self {
        self.parse_mode = Some(mode.into());
        self
    }

    pub fn add_chat_id(mut self, id: impl Into<String>) -> Self {
        self.chat_ids.push(id.into());
        self
    }

    pub fn add_chat_ids<I: IntoIterator<Item = String>>(mut self, ids: I) -> Self {
        self.chat_ids.extend(ids);
        self
    }

    pub fn build(self) -> Bot {
        Bot {
            token: self.token,
            client: Client::new(),
            timeout: self.timeout,
            chat_ids: self.chat_ids,
            parse_mode: self.parse_mode,
        }
    }
}

impl Bot {
    pub async fn send_message(
        &self,
        chat_id: &str,
        text: &str,
        reply_markup: Option<&Vec<Vec<types::InlineKeyboardMarkup>>>,
    ) -> Result<(), String> {
        let url = format!("{}/bot{}/sendMessage", BASE_URL, self.token);
        let mut payload = json!({
            "chat_id": chat_id,
            "text": text
        });

        if let Some(mode) = &self.parse_mode {
            payload["parse_mode"] = json!(mode);
        }

        if let Some(rm) = reply_markup {
            payload["reply_markup"] = json!( {"inline_keyboard": rm} );
        }

        let response = self
            .client
            .post(&url)
            .timeout(self.timeout)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Ошибка при запросе: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("Ошибка: статус {}", response.status()));
        }

        Ok(())
    }

    pub async fn write(
        &self,
        message: impl Into<String>,
        reply_markup: Option<&Vec<Vec<types::InlineKeyboardMarkup>>>,
    ) -> Result<(), String> {
        let msg = message.into();

        for id in &self.chat_ids {
            self.send_message(id, &msg, reply_markup.clone()).await?;
        }

        Ok(())
    }
}
