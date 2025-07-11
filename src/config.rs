use crate::{browser::BrowserSessionConfig, error::Result};
use serde::{Deserialize, Serialize};
use std::{fs, sync::OnceLock};

pub const CONFIG_PATH: &str = "Config.toml";

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn init_if_not() -> Result<bool> {
    if fs::metadata(CONFIG_PATH).is_ok() {
        return Ok(false);
    }
    fs::write(CONFIG_PATH, DEFAULT_CONFIG_STR.as_bytes())?;
    Ok(true)
}

pub fn get() -> &'static Config {
    CONFIG.get_or_init(|| {
        let buf = fs::read(CONFIG_PATH).expect("Ошибка чтения конфигурации");
        toml::from_slice::<Config>(&buf).expect("Формат конфигурации не распознан")
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub launch_options: LaunchOptions,
    pub telegram_bot: TelegramBot,
    pub tracking_supplies: TrackingSupplies,
    pub browser: Browser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchOptions {
    pub first_run: bool,
    pub telegram_notifications: bool,
    pub open: bool,
    pub verbose: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramBot {
    pub token: String,
    pub allow_users: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingSupplies {
    pub days: u32,
    pub interval_millis: u64,
    pub sync_credentials_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Browser {
    pub port: u16,
    pub user_data_dir: String,
    pub headless: bool,
}

impl Browser {
    pub fn to_browser_session_config(&self) -> BrowserSessionConfig {
        let user_data_dir = std::env::current_dir().unwrap().join(&self.user_data_dir);
        BrowserSessionConfig {
            port: self.port,
            user_data_dir: user_data_dir.to_str().map(|v| v.to_string()),
            headless: if self.headless {
                chromiumoxide::browser::HeadlessMode::True
            } else {
                chromiumoxide::browser::HeadlessMode::False
            },
            ..Default::default()
        }
    }
}

const DEFAULT_CONFIG_STR: &str = r##"
# Параметры запуска
[launch_options]
first_run = true # Первый запуск. Используется для авторизации клиента в seller.wildberries.ru (true/false)
telegram_notifications = true # Получение уведомлений в телеграм (true/false)
open = false # Поставка, которая стала доступна в результате отслеживания откроется в браузере (true/false)
verbose = true # Вывод информации об обновлении поставок в консоль (true/false)

# Параметры телеграм бота
[telegram_bot]
token = "" # Токен бота
allow_users = [] # ID пользователей для отправки уведомлений ([1234, 4321])

# Параметры отслеживания поставок
[tracking_supplies]
days = 14 # За какой период в днях отслеживать поставки
interval_millis = 5000 # Интервал обновления поставок в миллисекундах
sync_credentials_interval_secs = 5400 # Интервал синхронизации cookie и authorizev3 в секундах

# Параметры браузера
[browser]
port = 8889 # Порт браузера
user_data_dir = "user_data" # Относительный путь хранения данных пользователя
headless = false # Скрытый режим работы (true/false)
"##;
