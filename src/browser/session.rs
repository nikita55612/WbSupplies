use chromiumoxide::{
    Browser, BrowserConfig, Page,
    browser::HeadlessMode,
    cdp::browser_protocol::{
        //emulation::SetGeolocationOverrideParams,
        network::CookieParam,
    },
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::{
    task::JoinHandle,
    time::{sleep, timeout},
};
use tokio_stream::StreamExt;

pub use super::error::BrowserError;
use super::extension;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MyIP {
    pub ip: String,
    pub country: String,
    pub cc: String,
}

pub static DEFAULT_ARGS: [&str; 7] = [
    "--disable-default-apps",
    "--no-first-run",
    "--disable-sync",
    "--lang=en_US",
    "--no-default-browser-check",
    "--disable-smooth-scrolling",
    "--disable-features=TranslateUI",
];

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct BrowserTimings {
    pub launch_sleep: u64,
    pub set_proxy_sleep: u64,
    pub action_sleep: u64,
    pub page_goto_timeout: u64,
}

impl Default for BrowserTimings {
    fn default() -> Self {
        Self {
            launch_sleep: 280,
            set_proxy_sleep: 180,
            action_sleep: 80,
            page_goto_timeout: 1400,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PageParams<'a> {
    pub proxy: Option<&'a str>,
    pub wait_for_el: Option<(&'a str, u64)>,
    pub wait_for_el_until: Option<(&'a str, &'a str, u64)>,
    pub user_agent: Option<&'a str>,
    pub cookies: Vec<CookieParam>,
    //pub geolocation: Option<(f64, f64)>,
    pub wait_open_on_page: Option<u64>,
    pub wait_for_navigation: Option<u64>,
    pub duration: u64,
}

impl<'a> Default for PageParams<'a> {
    fn default() -> Self {
        Self {
            proxy: None,
            wait_for_el: None,
            wait_for_el_until: None,
            user_agent: None,
            cookies: Vec::default(),
            //geolocation: None,
            wait_open_on_page: None,
            wait_for_navigation: None,
            duration: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BrowserSessionConfig {
    pub executable: Option<String>,
    pub user_data_dir: Option<String>,
    pub args: Vec<String>,
    pub headless: HeadlessMode,
    pub sandbox: bool,
    pub extensions: Vec<String>,
    pub incognito: bool,
    pub port: u16,
    pub launch_timeout: u64,
    pub request_timeout: u64,
    pub cache_enabled: bool,
    pub timings: BrowserTimings,
}

impl Default for BrowserSessionConfig {
    fn default() -> Self {
        Self {
            executable: None,
            user_data_dir: None,
            args: DEFAULT_ARGS.into_iter().map(|v| v.into()).collect(),
            headless: HeadlessMode::False,
            sandbox: false,
            extensions: Vec::new(),
            incognito: false,
            port: 0,
            launch_timeout: 1500,
            request_timeout: 2000,
            cache_enabled: true,
            timings: BrowserTimings::default(),
        }
    }
}

trait ToBrowserConfig {
    fn to_config(&self) -> Result<BrowserConfig, BrowserError>;
}

impl ToBrowserConfig for BrowserSessionConfig {
    fn to_config(&self) -> Result<BrowserConfig, BrowserError> {
        let mut extensions = Vec::new();
        extensions.push(extension::PATH.clone());
        extensions.extend_from_slice(self.extensions.as_slice());
        let mut builder = BrowserConfig::builder()
            .disable_default_args()
            .headless_mode(self.headless)
            .args(&self.args)
            .extensions(extensions)
            .viewport(None)
            .port(self.port)
            .launch_timeout(Duration::from_millis(self.launch_timeout))
            .request_timeout(Duration::from_millis(self.request_timeout));

        if self.incognito {
            builder = builder.incognito();
        }
        if !self.sandbox {
            builder = builder.no_sandbox();
        }
        if self.cache_enabled {
            builder = builder.enable_cache();
        }
        if let Some(user_data_dir) = &self.user_data_dir {
            builder = builder.user_data_dir(user_data_dir);
        }
        if let Some(executable) = &self.executable {
            builder = builder.chrome_executable(executable);
        }

        builder
            .build()
            .map_err(|_| BrowserError::BuildBrowserConfigError)
    }
}

pub struct BrowserSession {
    pub browser: Browser,
    pub handle: JoinHandle<()>,
    pub timings: BrowserTimings,
}

#[allow(dead_code)]
impl BrowserSession {
    pub async fn launch(bsc: &BrowserSessionConfig) -> Result<Self, BrowserError> {
        let timings = bsc.timings.clone();
        let (browser, mut handler) = Browser::launch(bsc.to_config()?).await?;
        let handle = tokio::task::spawn(async move { while handler.next().await.is_some() {} });
        sleep(Duration::from_millis(timings.launch_sleep)).await;

        Ok(Self {
            browser,
            handle,
            timings,
        })
    }

    pub async fn launch_with_default_config() -> Result<Self, BrowserError> {
        let config = BrowserSessionConfig::default();
        Self::launch(&config).await
    }

    pub async fn set_timings(&mut self, timings: BrowserTimings) {
        self.timings = timings;
    }

    pub async fn close(&mut self) {
        if self.browser.close().await.is_err() {
            self.browser.kill().await;
        }
        if self.browser.wait().await.is_err() {
            let mut attempts = 0;
            while self.browser.try_wait().is_err() && attempts < 4 {
                attempts += 1;
            }
        }
        self.handle.abort();
    }

    pub async fn new_page(&self) -> Result<Page, BrowserError> {
        let new_page = self.browser.new_page("about:blank").await?;
        Ok(new_page)
    }

    pub async fn open_on_page<'a>(&self, url: &str, page: &'a Page) -> Result<(), BrowserError> {
        //page.goto(url).await?;
        let _ = timeout(
            Duration::from_millis(self.timings.page_goto_timeout),
            page.goto(url),
        )
        .await;

        Ok(())
    }

    pub async fn open(&self, url: &str) -> Result<Page, BrowserError> {
        let page = self.new_page().await?;
        self.open_on_page(url, &page).await?;

        Ok(page)
    }

    pub async fn open_with_duration(&self, url: &str, duration: u64) -> Result<Page, BrowserError> {
        let page = self.new_page().await?;
        self.open_on_page(url, &page).await?;
        sleep(Duration::from_millis(duration)).await;

        Ok(page)
    }

    pub async fn open_with_params<'a>(
        &self,
        url: &str,
        params: &PageParams<'a>,
    ) -> Result<Page, BrowserError> {
        if let Some(proxy) = params.proxy {
            self.set_proxy(proxy).await?;
        }
        let page = self.new_page().await?;
        if let Some(user_agent) = params.user_agent {
            page.set_user_agent(user_agent).await?;
        }
        if !params.cookies.is_empty() {
            page.set_cookies(params.cookies.clone()).await?;
        }
        /*
        if let Some(geolocation) = param.geolocation {
            page.emulate_geolocation(
                SetGeolocationOverrideParams {
                    latitude: Some(geolocation.0),
                    longitude: Some(geolocation.1),
                    accuracy: None
                }
            ).await?;
        }
         */
        if params.wait_open_on_page.is_some() {
            let _ = timeout(
                Duration::from_millis(params.wait_open_on_page.unwrap()),
                self.open_on_page(url, &page),
            )
            .await;
        } else {
            self.open_on_page(url, &page).await?;
        }
        if let Some(wait_timeout) = params.wait_for_navigation {
            let _ = timeout(
                Duration::from_millis(wait_timeout),
                page.wait_for_navigation(),
            )
            .await;
        }
        sleep(Duration::from_millis(params.duration)).await;
        if let Some((selector, timeout)) = params.wait_for_el {
            let _ = page.wait_for_el_with_timeout(selector, timeout).await;
        }
        if let Some((selector, until_selector, timeout)) = params.wait_for_el_until {
            let _ = page
                .wait_for_el_until_with_timeout(selector, until_selector, timeout)
                .await;
        }

        Ok(page)
    }

    pub async fn set_proxy(&self, proxy: &str) -> Result<(), BrowserError> {
        if let Err(e) = self
            .browser
            .new_page(format!("chrome://set_proxy/{proxy}"))
            .await
        {
            let error = BrowserError::from(e);
            match error {
                BrowserError::NetworkIO => {}
                _ => {
                    return Err(error);
                }
            }
        }
        sleep(Duration::from_millis(self.timings.set_proxy_sleep)).await;
        Ok(())
    }

    pub async fn reset_proxy(&self) -> Result<(), BrowserError> {
        if let Err(e) = self.browser.new_page("chrome://reset_proxy").await {
            let error = BrowserError::from(e);
            match error {
                BrowserError::NetworkIO => {}
                _ => {
                    return Err(error);
                }
            }
        }
        sleep(Duration::from_millis(self.timings.action_sleep)).await;
        Ok(())
    }

    pub async fn close_tabs(&self) -> Result<(), BrowserError> {
        if let Err(e) = self.browser.new_page("chrome://close_tabs").await {
            let error = BrowserError::from(e);
            match error {
                BrowserError::NetworkIO => {}
                _ => {
                    return Err(error);
                }
            }
        }
        sleep(Duration::from_millis(self.timings.action_sleep)).await;
        Ok(())
    }

    pub async fn clear_data(&self) -> Result<(), BrowserError> {
        if let Err(e) = self.browser.new_page("chrome://clear_data").await {
            let error = BrowserError::from(e);
            match error {
                BrowserError::NetworkIO => {}
                _ => {
                    return Err(error);
                }
            }
        }
        sleep(Duration::from_millis(self.timings.action_sleep)).await;
        Ok(())
    }

    pub async fn myip(&self) -> Result<MyIP, BrowserError> {
        let page = self.open("https://api.myip.com/").await?;
        let myip = page
            .find_element("body")
            .await?
            .inner_text()
            .await?
            .ok_or(BrowserError::Serialization)
            .map(|s| serde_json::from_str(&s).map_err(|_| BrowserError::Serialization))?;
        let _ = page.close().await;
        myip
    }
}

pub trait Wait {
    const WAIT_SLEEP: u64 = 10;

    async fn wait_for_el(&self, selector: &str);

    async fn wait_for_el_until(&self, selector: &str, until_selector: &str);

    async fn wait_for_el_with_timeout(&self, selector: &str, t: u64) -> Result<(), BrowserError>;

    async fn wait_for_el_until_with_timeout(
        &self,
        selector: &str,
        until_selector: &str,
        t: u64,
    ) -> Result<(), BrowserError>;
}

impl Wait for Page {
    async fn wait_for_el(&self, selector: &str) {
        while self.find_element(selector).await.is_err() {
            sleep(Duration::from_millis(Self::WAIT_SLEEP)).await;
        }
    }

    async fn wait_for_el_until(&self, selector: &str, until_selector: &str) {
        while self.find_element(selector).await.is_err() {
            sleep(Duration::from_millis(Self::WAIT_SLEEP)).await;
            if self.find_element(until_selector).await.is_ok() {
                break;
            }
            sleep(Duration::from_millis(Self::WAIT_SLEEP)).await;
        }
    }

    async fn wait_for_el_with_timeout(&self, selector: &str, t: u64) -> Result<(), BrowserError> {
        timeout(Duration::from_millis(t), self.wait_for_el(selector)).await?;

        Ok(())
    }

    async fn wait_for_el_until_with_timeout(
        &self,
        selector: &str,
        until_selector: &str,
        t: u64,
    ) -> Result<(), BrowserError> {
        timeout(
            Duration::from_millis(t),
            self.wait_for_el_until(selector, until_selector),
        )
        .await?;

        Ok(())
    }
}

// static USER_AGENT_LIST: [&str; 20] = [
//     "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
//     "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
//     "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
//     "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Edge/117.0.2045.60 Safari/537.36",
//     "Mozilla/5.0 (Windows NT 10.0; WOW64; rv:102.0) Gecko/20100101 Firefox/102.0",
//     "Mozilla/5.0 (Macintosh; Intel Mac OS X 12.6; rv:116.0) Gecko/20100101 Firefox/116.0",
//     "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:118.0) Gecko/20100101 Firefox/118.0",
//     "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_6) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.0 Safari/605.1.15",
//     "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
//     "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/115.0",
//     "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36",
//     "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
//     "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
//     "Mozilla/5.0 (Windows NT 11.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36",
//     "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_0_1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36",
//     "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
//     "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.5 Safari/605.1.15",
//     "Mozilla/5.0 (Windows NT 10.0; rv:110.0) Gecko/20100101 Firefox/110.0",
//     "Mozilla/5.0 (X11; Linux x86_64; rv:91.0) Gecko/20100101 Firefox/91.0",
//     "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_14_6) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/13.1.2 Safari/605.1.15",
// ];

// pub fn random_user_agent() -> &'static str {
//     let mut rng = rand::rng();
//     let index = rng.random_range(0..USER_AGENT_LIST.len());
//     USER_AGENT_LIST[index]
// }
