use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use tokio::{
    sync::{
        Mutex,
        watch::{Receiver, Sender},
    },
    task::JoinHandle,
};

use crate::{
    browser::{BrowserSession, BrowserSessionConfig},
    error::Result,
    wbseller::{
        Client,
        models::{Cost, Supply},
    },
};

/// Структура для хранения обновлений себестоимости поставок
#[derive(Debug, Default, Clone)]
pub struct SupplyUpdateAcceptanceCosts {
    pub supply: Supply,
    pub costs: Vec<Cost>,
}

/// Карта принятой себестоимости: preorder_id -> (дата -> Cost)
pub type AcceptanceCosts = HashMap<i64, HashMap<String, Cost>>;

/// Тип сообщения об обновлении
pub type UpdateMessage = std::result::Result<Option<HashMap<i64, SupplyUpdateAcceptanceCosts>>, ()>;

/// Основная структура для отслеживания поставок и себестоимости
#[allow(dead_code)]
pub struct TrackingSupplies {
    browser_session: Arc<Mutex<BrowserSession>>,
    acceptance_costs: Arc<Mutex<AcceptanceCosts>>,
    channel: Sender<UpdateMessage>,
    background_handle: JoinHandle<()>,
    is_closed: AtomicBool,
}

#[allow(dead_code)]
impl TrackingSupplies {
    pub async fn watch(bs_config: &BrowserSessionConfig) -> Result<Self> {
        let bs = Arc::new(Mutex::new(BrowserSession::launch(bs_config).await?));
        let (tx, _) = tokio::sync::watch::channel(UpdateMessage::Ok(None));
        let acceptance_costs = Arc::new(Mutex::new(HashMap::new()));

        let background_handle = {
            let bs = Arc::clone(&bs);
            let acceptance_costs = Arc::clone(&acceptance_costs);
            let tx = tx.clone();

            // Интервалы обновления
            let update_interval = std::time::Duration::from_secs(5);
            let refresh_credentials_interval = std::time::Duration::from_secs(60 * 60);
            let mut n: u32 = 0;

            // Инициализация клиента
            let mut client = {
                let session = bs.lock().await;
                Client::from_browser_session(&*session).await?
            };

            tokio::spawn(async move {
                loop {
                    // Обновление учётных данных по расписанию
                    if update_interval * n > refresh_credentials_interval {
                        let guard_bs = bs.lock().await;
                        if let Ok(cli) = Client::from_browser_session(&*guard_bs).await {
                            client = cli;
                            n = 0;
                        }
                    }
                    n += 1;

                    // Получение поставок
                    let supplies = match client.not_planned_list_supplies().await {
                        Ok(response) => response.result.data,
                        Err(_) => {
                            tokio::time::sleep(update_interval).await;
                            continue;
                        }
                    };

                    if supplies.is_empty() {
                        tokio::time::sleep(update_interval).await;
                        continue;
                    }

                    // Получение себестоимости по поставкам
                    let data = match client.acceptance_costs_from_supplies(14, &supplies).await {
                        Ok(d) => d,
                        Err(_) => {
                            tokio::time::sleep(update_interval).await;
                            continue;
                        }
                    };

                    if data.is_empty() {
                        tokio::time::sleep(update_interval).await;
                        continue;
                    }

                    // Карта поставок по preorder_id
                    let supplies_map: HashMap<_, _> = supplies
                        .into_iter()
                        .filter_map(|s| s.preorder_id.map(|id| (id, s)))
                        .collect();

                    let mut updated_acceptance_costs = HashMap::new();
                    let mut guard = acceptance_costs.lock().await;

                    // Удаление устаревших записей
                    let obsolete_keys: Vec<_> = guard
                        .keys()
                        .filter(|k| !supplies_map.contains_key(k))
                        .cloned()
                        .collect();

                    for key in obsolete_keys {
                        guard.remove(&key);
                    }

                    for (k, v) in data {
                        let new_costs_map = v
                            .into_iter()
                            .map(|c| (c.date.clone(), c))
                            .collect::<HashMap<_, _>>();

                        if !guard.contains_key(&k) {
                            guard.insert(k, new_costs_map);
                            continue;
                        }

                        // Обновление и выявление изменений в себестоимости
                        if let Some(old_costs) = guard.get(&k) {
                            for (date, old_cost) in old_costs {
                                if let Some(new_cost) = new_costs_map.get(date) {
                                    // Считаем, что обновление произошло, если коэффициент стал неотрицательным
                                    if old_cost.coefficient < 0. && new_cost.coefficient >= 0. {
                                        updated_acceptance_costs
                                            .entry(k)
                                            .or_insert_with(|| SupplyUpdateAcceptanceCosts {
                                                supply: supplies_map[&k].clone(),
                                                costs: Vec::new(),
                                            })
                                            .costs
                                            .push(new_cost.clone());
                                    }
                                }
                            }
                        }

                        guard.insert(k, new_costs_map);
                    }

                    drop(guard);

                    if tx.is_closed() {
                        break;
                    }

                    // Отправка обновлений
                    if !updated_acceptance_costs.is_empty() {
                        if tx.send(Ok(Some(updated_acceptance_costs))).is_err() {
                            break;
                        }
                    } else {
                        let _ = tx.send(Ok(None));
                    }

                    tokio::time::sleep(update_interval).await;
                }

                let _ = tx.send(Err(()));
            })
        };

        Ok(Self {
            browser_session: bs,
            acceptance_costs: acceptance_costs,
            channel: tx,
            background_handle: background_handle,
            is_closed: AtomicBool::new(false),
        })
    }

    /// Подписка на канал обновлений
    pub async fn subscribe_channel(&self) -> Receiver<UpdateMessage> {
        self.channel.subscribe()
    }

    /// Получение текущей карты себестоимостей
    pub async fn read_acceptance_costs(&self) -> AcceptanceCosts {
        self.acceptance_costs.lock().await.clone()
    }

    /// Закрытие и остановка фонового процесса
    pub async fn close(&self) {
        // Защита от повторного вызова
        if self
            .is_closed
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return;
        }

        self.browser_session.lock().await.close().await;
        self.background_handle.abort();
    }
}
