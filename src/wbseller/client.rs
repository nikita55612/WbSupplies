use reqwest::{
    RequestBuilder,
    header::{HeaderMap, HeaderValue},
};
use serde::de::DeserializeOwned;
use serde_json::json;
use std::collections::HashMap;

use crate::{
    browser::{BrowserSession, PageParams},
    wbseller::models::{AcceptanceCostsResponse, Cost, ListSuppliesResponse, Supply},
};

use super::error::{Result, WbSellerError};

/// HTTP-клиент для взаимодействия с Wildberries Seller
#[derive(Debug, Default)]
pub struct Client {
    pub headers: HeaderMap,
}

impl Client {
    /// Создание клиента с авторизацией и куками
    pub fn new(authorizev3: &str, cookies: &HashMap<String, String>) -> Self {
        let mut headers = HeaderMap::new();

        headers.insert(
            "origin",
            HeaderValue::from_static("https://seller.wildberries.ru"),
        );
        headers.insert(
            "referer",
            HeaderValue::from_static("https://seller.wildberries.ru/"),
        );
        headers.insert("user-agent", HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36"
        ));

        // Формирование строки cookie
        let cookie_str = cookies
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("; ");

        headers.insert("authorizev3", HeaderValue::from_str(authorizev3).unwrap());
        headers.insert("cookie", HeaderValue::from_str(&cookie_str).unwrap());

        Self { headers }
    }

    /// Создание клиента из активной сессии браузера
    pub async fn from_browser_session(bs: &BrowserSession) -> Result<Self> {
        let params = PageParams {
            wait_for_navigation: Some(2000),
            wait_open_on_page: Some(3000),
            wait_for_el: Some(("#root", 5000)),
            duration: 1500,
            ..Default::default()
        };

        let page = bs
            .open_with_params("https://seller.wildberries.ru/", &params)
            .await?;

        let browser_cookies = bs
            .browser
            .get_cookies()
            .await
            .map_err(|e| WbSellerError::Browser(e.into()))?;

        let cookies = browser_cookies
            .into_iter()
            .map(|c| (c.name, c.value))
            .collect::<HashMap<_, _>>();

        // Получение токена авторизации из localStorage
        let result = page
            .evaluate("localStorage.getItem('wb-eu-passport-v2.access-token')")
            .await
            .map_err(|e| WbSellerError::Browser(e.into()))?;

        let authorizev3 = result
            .value()
            .and_then(|v| v.as_str().map(str::to_string))
            .ok_or_else(|| WbSellerError::Custom("parse authorizev3 value".into()))?;

        let _ = page.close().await;

        Ok(Self::new(&authorizev3, &cookies))
    }

    /// Отправка HTTP-запроса и десериализация ответа
    async fn send_request<T: DeserializeOwned>(&self, builder: RequestBuilder) -> Result<T> {
        let response = builder.headers(self.headers.clone()).send().await?;

        let parsed = response.json::<T>().await?;

        Ok(parsed)
    }

    /// Получение списка поставок по статусу
    pub async fn list_supplies(&self, status_id: i8) -> Result<ListSuppliesResponse> {
        let payload = json!({
            "params": {
                "pageNumber": 1,
                "pageSize": 100,
                "sortBy": "createDate",
                "sortDirection": "desc",
                "statusId": status_id,
                "searchById": null
            },
            "jsonrpc": "2.0",
            "id": "json-rpc_33"
        });

        let url = "https://seller-supply.wildberries.ru/ns/sm-supply/supply-manager/api/v1/supply/listSupplies";
        let builder = reqwest::Client::new().post(url).json(&payload);

        self.send_request(builder).await
    }

    /// Получение списка неподтверждённых поставок
    pub async fn not_planned_list_supplies(&self) -> Result<ListSuppliesResponse> {
        self.list_supplies(-1).await
    }

    // /// Получение всех поставок
    // pub async fn all_list_supplies(&self) -> Result<ListSuppliesResponse> {
    //     self.list_supplies(-2).await
    // }

    /// Получение информации о себестоимости по preorder_id за заданное число дней
    pub async fn get_acceptance_costs(
        &self,
        preorder_id: i64,
        days: u8,
    ) -> Result<AcceptanceCostsResponse> {
        let now = chrono::Utc::now();
        let date_from = now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let date_to = (now + chrono::Duration::days((days.max(1) - 1).into()))
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

        let payload = json!({
            "params": {
                "dateFrom": date_from,
                "dateTo": date_to,
                "preorderID": preorder_id,
                "supplyId": null
            },
            "jsonrpc": "2.0",
            "id": "json-rpc_39"
        });

        let url = "https://seller-supply.wildberries.ru/ns/sm-supply/supply-manager/api/v1/supply/getAcceptanceCosts";
        let builder = reqwest::Client::new().post(url).json(&payload);

        self.send_request(builder).await
    }

    /// Получение себестоимости по списку поставок
    pub async fn acceptance_costs_from_supplies(
        &self,
        days: u8,
        supplies: &[Supply],
    ) -> Result<HashMap<i64, Vec<Cost>>> {
        let mut result = HashMap::with_capacity(supplies.len());

        for supply in supplies {
            if let Some(preorder_id) = supply.preorder_id {
                let response = self
                    .get_acceptance_costs(preorder_id, days)
                    .await
                    .unwrap_or_else(|_| AcceptanceCostsResponse::default());

                result.insert(preorder_id, response.result.costs);
            }
        }

        Ok(result)
    }
}
