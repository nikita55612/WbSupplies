mod browser;
mod config;
mod error;
mod telebot;
mod tracking;
mod util;
mod wbseller;

use browser::*;
use error::Result;

use crate::{config::Config, tracking::TrackingSupplies};

const LOGO: &str = r#"

 __      _____.     _________                   .__  .__
/  \    /  \_ |__  /   _____/__ ________ ______ |  | |__| ____   ______
\   \/\/   /| __ \ \_____  \|  |  \____ \\____ \|  | |  |/ __ \ /  ___/
 \        / | \_\ \/        \  |  /  |_> >  |_> >  |_|  \  ___/ \___ \
  \__/\  /  |___  /_______  /____/|   __/|   __/|____/__|\___  >____  >
       \/       \/        \/      |__|   |__|                \/     \/
"#;

/// Обработка сигнала завершения (Ctrl+C или SIGINT/SIGTERM)
async fn shutdown_signal() {
    #[cfg(windows)]
    {
        let _ = tokio::signal::ctrl_c().await;
    }

    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut sigint = signal(SignalKind::interrupt()).expect("Не удалось слушать SIGINT");
        let mut sigterm = signal(SignalKind::terminate()).expect("Не удалось слушать SIGTERM");

        tokio::select! {
            _ = sigint.recv() => {},
            _ = sigterm.recv() => {},
        }
    }

    println!("Завершение по сигналу...");
}

/// Инициализация конфигурации и авторизация пользователя
async fn startup() -> Result<&'static Config> {
    // Если конфигурация не инициализирована — инициализируем
    if config::init_if_not().expect("Ошибка инициализации конфигурации")
    {
        let full_config_path = std::env::current_dir()?.join(config::CONFIG_PATH);
        println!(
            "Файл конфигурации инициализирован по пути: {:?}",
            full_config_path
        );
        let _ = open::that_in_background(full_config_path);
    }

    let cfg = config::get();

    // Первая загрузка: открываем браузер для авторизации
    if cfg.launch_options.first_run {
        let mut browser_config = cfg.browser.clone();
        browser_config.headless = false;
        let bs_config = browser_config.to_browser_session_config();
        let mut bs = BrowserSession::launch(&bs_config).await?;
        let page = bs.open("https://seller.wildberries.ru").await?;

        // Показываем alert для входа
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let _ = page
                .evaluate(r#"alert("Выполните вход в личный кабинет")"#)
                .await;
            let _ = page.close().await;
        });

        println!("Ожидание входа в личный кабинет. Не закрывайте окно браузера");
        println!("Нажмите Enter чтобы продолжить:");
        let _ = std::io::stdin().read_line(&mut String::new());

        // Проверяем успешность авторизации
        if wbseller::Client::from_browser_session(&bs).await.is_ok() {
            println!("Авторизация прошла успешно!");
        } else {
            panic!(
                "Не удалось получить параметры авторизации личного кабинета seller.wildberries.ru"
            );
        }

        bs.close().await;
    }

    // Предупреждение о некорректной настройке телеграм-бота
    if cfg.launch_options.telegram_notifications
        && (cfg.telegram_bot.token.is_empty() || cfg.telegram_bot.allow_users.is_empty())
    {
        println!("Предупреждение: не указан токен телеграм-бота или пуст список пользователей");
    }

    Ok(cfg)
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("\x1b[95m{}\x1b[0m\n", LOGO);
    let cfg = startup().await?;

    // Инициализация телеграм-бота (если включён)
    let bot = if cfg.launch_options.telegram_notifications
        && !cfg.telegram_bot.token.is_empty()
        && !cfg.telegram_bot.allow_users.is_empty()
    {
        let token = cfg.telegram_bot.token.clone();
        let allow_users = cfg
            .telegram_bot
            .allow_users
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        Some(
            telebot::BotBuilder::new(token)
                .add_chat_ids(allow_users)
                .parse_mode("HTML")
                .build(),
        )
    } else {
        None
    };

    // Запуск браузерной сессии и слежение за поставками
    let bs_config = cfg.browser.to_browser_session_config();
    let tracking_supplies = TrackingSupplies::watch(&bs_config).await?;
    let mut rx = tracking_supplies.subscribe_channel().await;

    let shutdown_handle = tokio::spawn({
        async move {
            shutdown_signal().await;
            tracking_supplies.close().await;
        }
    });

    println!("Процесс отслеживания поставок запущен");

    while rx.changed().await.is_ok() {
        let data = rx.borrow_and_update();
        if data.is_err() {
            break;
        }
        if data.as_ref().unwrap().is_none() {
            continue;
        }
        let data = data.clone().unwrap().unwrap();

        if cfg.launch_options.verbose {
            println!("Обновление поставок: {:#?}", data);
        }

        let urls: Vec<_> = data.keys().map(|k| util::preorder_id_to_url(*k)).collect();

        // Открытие ссылок в браузере, если включено
        if cfg.launch_options.open {
            for url in &urls {
                open::with_in_background(url, "chrome");
            }
        }

        // Отправка уведомления через телеграм-бота
        if let Some(ref b) = bot {
            let mut message = String::from("🔊 <b><i>Обновление поставок</b></i>\n\n");
            let mut reply_markup = Vec::with_capacity(urls.len());

            for ((url, v), warehouse) in urls
                .iter()
                .zip(data.values())
                .zip(data.values().map(|v| &v.supply.warehouse_name))
            {
                message.push_str(&format!("▫️ <b>{}</b>\n", warehouse));

                let costs_info = v
                    .costs
                    .iter()
                    .map(|c| {
                        let short_date = c.date.split_once('T').map_or(&c.date[..], |(d, _)| d);
                        format!(
                            "Коэффициент: <b>{}</b>\nСтоимость: <b>{}</b>\nДата: <b>{}</b>",
                            c.coefficient, c.cost, short_date
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                message.push_str(&costs_info);
                message.push_str("\n\n");

                reply_markup.push(vec![telebot::types::InlineKeyboardMarkup {
                    text: warehouse.clone(),
                    url: url.to_string(),
                }]);
            }

            let _ = b.write(&message, Some(&reply_markup));
        }
    }

    let _ = shutdown_handle.await;
    Ok(())
}
