use std::io::Write;
use std::path::PathBuf;
use std::sync::LazyLock;

pub static PATH: LazyLock<String> = LazyLock::new(|| {
    let extension_path = match init() {
        Ok(pb) => match pb.to_str() {
            Some(s) => s.to_owned(),
            None => panic!("InitExtensionPathError"),
        },
        Err(_) => panic!("InitExtensionPathError"),
    };
    extension_path
});

fn write_to_file(path: impl AsRef<std::path::Path>, content: &str) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    file.flush()
}

fn init() -> std::io::Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    let extension_path = current_dir.join("extension");
    if extension_path.exists() {
        return Ok(extension_path);
    }
    std::fs::create_dir(&extension_path)?;
    let background_path = extension_path.join("background.js");
    let manifest_path = extension_path.join("manifest.json");
    write_to_file(background_path, BACKGROUND_JS)?;
    write_to_file(manifest_path, MANIFEST_JSON)?;
    Ok(extension_path)
}

const MANIFEST_JSON: &'static str = r##"
{
    "version": "1.0.0",
    "manifest_version": 3,
    "name": "ChromeProxy",
    "permissions": [
      "proxy",
      "tabs",
      "webRequest",
      "webRequestAuthProvider",
      "browsingData"
    ],
    "host_permissions": [
      "<all_urls>"
    ],
    "incognito": "spanning",
    "background": {
      "service_worker": "background.js"
    }
}
"##;

const BACKGROUND_JS: &'static str = r##"
// Configuration constants
const CHROME_URLS = {
    SET_PROXY: 'chrome://set_proxy/',
    RESET_PROXY: 'chrome://reset_proxy',
    CLEAR_DATA: 'chrome://clear_data',
    CLEAR_COOKIES: 'chrome://clear_cookies',
    INIT_EXTENSION: 'chrome://init_extension',
    CLOSE_TABS: 'chrome://close_tabs',
};

// Proxy configuration handler
class ProxyManager {
    constructor() {
        this.currentAuthHandler = null;
    }

    /**
     * Parse proxy configuration from URL
     * @param {URL} parsedUrl - Parsed URL object
     * @returns {Object|null} Parsed proxy configuration
     */
    parseProxyConfig(parsedUrl) {
        const searchParams = parsedUrl.searchParams;

        // Extract parameters from query string
        const queryParams = {
            host: searchParams.get('host'),
            port: searchParams.get('port') ? parseInt(searchParams.get('port'), 10) : null,
            username: searchParams.get('username'),
            password: searchParams.get('password')
        };

        // Updated regex for parsing path
        const proxyRegex = /^\/([^:]+):([^@]+)@([^:]+):(\d+)\/?$/;
        const pathMatch = parsedUrl.pathname.match(proxyRegex);

        console.log('pathnameUrl:', parsedUrl.pathname);
        console.log('pathMatch:', pathMatch);

        // Extract data from path if match exists
        const pathParams = pathMatch ? {
            username: pathMatch[1] || null,
            password: pathMatch[2] || null,
            host: pathMatch[3],
            port: parseInt(pathMatch[4], 10)
        } : {};

        // Priority to query parameters
        const config = {
            host: queryParams.host || pathParams.host,
            port: queryParams.port || pathParams.port,
            username: queryParams.username || pathParams.username,
            password: queryParams.password || pathParams.password
        };

        // Validate mandatory fields
        return config.host && config.port && !isNaN(config.port) ? config : null;
    }

    /**
     * Create authentication credentials callback
     * @param {Object} proxyConfig - Proxy configuration
     * @returns {Object} Authentication credentials
     */
    createAuthCredentials(proxyConfig) {
        return {
            authCredentials: {
                username: proxyConfig.username,
                password: proxyConfig.password
            }
        };
    }

    /**
     * Set proxy configuration
     * @param {Object} proxyConfig - Proxy configuration
     */
    setProxy(proxyConfig) {
        try {
            console.log('Setting proxy:', proxyConfig);

            // Remove existing auth listeners
            this.removeAuthListener();

            // Create new auth handler
            const authHandler = this.createAuthHandler(proxyConfig);
            this.currentAuthHandler = authHandler;

            const proxySettings = {
                mode: 'fixed_servers',
                rules: {
                    singleProxy: {
                        scheme: 'http',
                        host: proxyConfig.host,
                        port: proxyConfig.port,
                    },
                    bypassList: ["localhost"]
                }
            };

            console.log('Proxy settings:', proxySettings);

            chrome.proxy.settings.set(
                { value: proxySettings, scope: 'regular' },
                this.handleProxySetup(authHandler)
            );
        } catch (error) {
            console.error('Proxy setup error:', error);
        }
    }

    /**
     * Create authentication handler with proxy host validation
     * @param {Object} proxyConfig - Proxy configuration
     * @returns {Function} Authentication handler
     */
    createAuthHandler(proxyConfig) {
        if (!proxyConfig.username || !proxyConfig.password) {
            return (details) => {};
        }
        return (details) => {
            console.log('Authentication request details:', details);

            if (details.challenger && details.challenger.host !== proxyConfig.host) {
                console.warn('Detected authentication request for a different proxy host');
                this.resetProxy();
                this.setProxy(proxyConfig);

                return { cancel: true };
            }

            return this.createAuthCredentials(proxyConfig);
        };
    }

    /**
     * Handle proxy setup and listener registration
     * @param {Function} authHandler - Authentication handler
     * @returns {Function} Callback for proxy settings
     */
    handleProxySetup(authHandler) {
        return () => {
            if (chrome.runtime.lastError) {
                console.error('Proxy setup error:', chrome.runtime.lastError);
                return;
            }

            console.log('Proxy configured.');
            this.addAuthListener(authHandler);
        };
    }

    /**
     * Add authentication listener
     * @param {Function} authHandler - Authentication handler
     */
    addAuthListener(authHandler) {
        chrome.webRequest.onAuthRequired.addListener(
            authHandler,
            { urls: ["<all_urls>"] },
            ["blocking"]
        );
        console.log('Authentication listener added.');
    }

    /**
     * Remove existing authentication listener
     */
    removeAuthListener() {
        if (this.currentAuthHandler &&
            chrome.webRequest.onAuthRequired.hasListener(this.currentAuthHandler)) {
            chrome.webRequest.onAuthRequired.removeListener(this.currentAuthHandler);
            console.log('Existing authentication listener removed.');
        }
    }

    /**
     * Reset proxy to system settings
     */
    resetProxy() {
        this.removeAuthListener();
        chrome.proxy.settings.set(
            { value: { mode: 'system' }, scope: 'regular' },
            () => {
                if (chrome.runtime.lastError) {
                    console.error('Proxy reset error:', chrome.runtime.lastError);
                } else {
                    console.log('Proxy reset to system settings');
                }
            }
        );
    }
}

// Browser data management
class BrowserDataManager {
    /**
     * Remove all browsing data
     */
    static removeBrowsingData() {
        chrome.browsingData.remove(
            { since: 0 },
            {
                appcache: true,
                cache: true,
                cacheStorage: true,
                // cookies: true,
                downloads: true,
                fileSystems: true,
                formData: true,
                history: true,
                indexedDB: true,
                localStorage: true,
                passwords: true,
                serviceWorkers: true,
                webSQL: true
            },
            () => {}
        );
    }

    static clearBrowsingCookies() {
        chrome.browsingData.remove(
            { since: 0 },
            {
                cookies: true,
            },
            () => {}
        );
    }
}

// Close all tabs except first
function closeAllTabsExceptFirst() {
    chrome.tabs.query({}, function(tabs) {
        if (tabs.length <= 1) return;
        const sortedTabs = tabs.slice().sort((a, b) => a.index - b.index);
        for (let i = 1; i < sortedTabs.length; i++) {
            chrome.tabs.remove(sortedTabs[i].id, function() {});
        }
    });
}

// URL command handler
class CommandHandler {
    constructor(proxyManager) {
        this.proxyManager = proxyManager;
    }

    /**
     * Handle URL-based commands
     * @param {string} url - URL to parse
     * @returns {boolean} - Whether a command was processed
     */
    handleUrlCommand(url) {
        const parsedUrl = new URL(url);

        switch (true) {
            case url.startsWith(CHROME_URLS.SET_PROXY):
                const proxyConfig = this.proxyManager.parseProxyConfig(parsedUrl);
                return proxyConfig && (this.proxyManager.setProxy(proxyConfig), true);
            case url.startsWith(CHROME_URLS.RESET_PROXY):
                return (this.proxyManager.resetProxy(), true);
            case url.startsWith(CHROME_URLS.CLEAR_DATA):
                return (BrowserDataManager.removeBrowsingData(), true);
            case url.startsWith(CHROME_URLS.CLEAR_COOKIES):
                return (BrowserDataManager.clearBrowsingCookies(), true);
            case url.startsWith(CHROME_URLS.INITIALIZE):
                return (extension.init(), true);
            case url.startsWith(CHROME_URLS.CLOSE_TABS):
                return (closeAllTabsExceptFirst(), true);
            default:
                return false;
        }
    }
}

// Extension initialization
class ChromeExtension {
    constructor() {
        this.proxyManager = new ProxyManager();
        this.commandHandler = new CommandHandler(this.proxyManager);
        this.tabUpdateListener = null;
    }

    /**
     * Creates tab update listener
     * @returns {Function} Tab update event listener
     */
    createTabUpdateListener() {
        return (tabId, changeInfo, tab) => {
            if (changeInfo.status === 'complete') {
                if (this.commandHandler.handleUrlCommand(tab.url)) {
                    chrome.tabs.remove(tabId);
                }
            }
        };
    }

    /**
     * Setup tab update listener
     */
    setupTabListener() {
        if (this.tabUpdateListener) {
            chrome.tabs.onUpdated.removeListener(this.tabUpdateListener);
        }

        this.tabUpdateListener = this.createTabUpdateListener();
        chrome.tabs.onUpdated.addListener(this.tabUpdateListener);
    }

    /**
     * Initialize extension
     */
    init() {
        BrowserDataManager.removeBrowsingData();
        this.proxyManager.resetProxy();
        this.setupTabListener();
    }
}

// Initialize the extension
const extension = new ChromeExtension();
extension.init();
"##;
