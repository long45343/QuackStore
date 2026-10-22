//! Native Rust scheduler daemon and sync executor for QuackStore.

use std::{fs, path::PathBuf, time::Duration};

use anyhow::{Context as _, Result};
use quack_core::{AppPaths, Context};
use serde::{Deserialize, Serialize};

use crate::rsa_2100::inject_rsa_if_missing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 1=Mon .. 7=Sun, default 4 (Thursday)
    #[serde(default = "default_dow")]
    pub day_of_week: u8,
    /// 0..23, default 16 (16:00)
    #[serde(default = "default_hour")]
    pub hour: u8,
    /// 0..59, default 0
    #[serde(default)]
    pub minute: u8,
    /// Interval in seconds for daemon loop polling (default 300s = 5m)
    #[serde(default = "default_check_interval")]
    pub check_interval: u64,
    /// Retry delay in seconds after a failure (default 900s = 15m)
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatConfig {
    #[serde(default = "default_true")]
    pub inject_legacy_rsa: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// "auto" | "google" | "xiaomi"
    #[serde(default = "default_server_strategy")]
    pub server_strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuackConfig {
    #[serde(default)]
    pub scheduler: SchedulerConfig,
    #[serde(default)]
    pub compat: CompatConfig,
    #[serde(default)]
    pub network: NetworkConfig,
}

fn default_true() -> bool { true }
fn default_dow() -> u8 { 4 }
fn default_hour() -> u8 { 16 }
fn default_check_interval() -> u64 { 300 }
fn default_retry_delay() -> u64 { 900 }
fn default_server_strategy() -> String { "auto".into() }

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            day_of_week: default_dow(),
            hour: default_hour(),
            minute: 0,
            check_interval: default_check_interval(),
            retry_delay: default_retry_delay(),
        }
    }
}

impl Default for CompatConfig {
    fn default() -> Self {
        Self {
            inject_legacy_rsa: default_true(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            server_strategy: default_server_strategy(),
        }
    }
}

impl QuackConfig {
    pub fn load_or_init(paths: &AppPaths) -> Self {
        let config_path = paths.data_root.join("config.toml");
        if config_path.is_file() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(config) = toml::from_str::<QuackConfig>(&content) {
                    return config;
                }
            }
        }
        let default_config = QuackConfig::default();
        if let Ok(content) = toml::to_string_pretty(&default_config) {
            let _ = fs::create_dir_all(&paths.data_root);
            let _ = fs::write(&config_path, content);
        }
        default_config
    }
}

pub fn log_message(paths: &AppPaths, msg: &str) {
    let now = time::OffsetDateTime::now_utc();
    let line = format!(
        "[{:04}-{:02}-{:02} {:02}:{:02}:{:02}] {}\n",
        now.year(),
        u8::from(now.month()),
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
        msg
    );
    let log_file = paths.data_root.join("sync.log");
    use std::io::Write;
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(log_file) {
        let _ = f.write_all(line.as_bytes());
    }
    eprint!("{line}");
}

/// Runs a single RKP provision and sync roundtrip.
pub async fn run_sync_once(ctx: &Context) -> Result<quack_tricky_store::keybox::KeyboxInstallData> {
    let config = QuackConfig::load_or_init(&ctx.paths);
    log_message(&ctx.paths, "=== 开始执行 RKP 硬件签发与自动同步任务 ===");

    // 1. 检查硬件 TA 状态
    log_message(&ctx.paths, "检查硬件 TA 前置条件...");
    let dev_status = quack_rkp_device::status_detect(ctx);
    if !dev_status.prerequisites_met {
        anyhow::bail!("硬件 TA 前置条件未满足: {:?}", dev_status.limitations);
    }

    // 2. 签发 RKP Keybox (通过内部 API 或 quackctl 调用)
    log_message(&ctx.paths, "正在请求高通硬件 TA 签名并获取 Google RKP 证书...");
    let options = quack_rkp_device::ProvisionOptions {
        protocol: quack_rkp_device::Protocol::Auto,
        fingerprint: None,
        output: None,
    };

    // 网络代理策略
    let server_url = match config.network.server_strategy.as_str() {
        "xiaomi" => Some(quack_rkp::http::XIAOMI_PROXY_URL.to_owned()),
        "google" => Some(quack_rkp::http::RKP_SERVER_URL.to_owned()),
        _ => None, // auto: 优先直连，超时自动回退至备用高可用通道
    };

    let result = quack_rkp_device::execute_provision(ctx, true, options, server_url).await?;
    let mut keybox_content = result.keybox_xml;
    log_message(&ctx.paths, &format!("RKP 签发成功，原始 XML 大小: {} 字节", keybox_content.len()));

    // 3. 2100年 RSA 兼容注入
    if config.compat.inject_legacy_rsa {
        keybox_content = inject_rsa_if_missing(&keybox_content);
        log_message(&ctx.paths, &format!("已成功并入 2100年 AOSP RSA 占位块，合成 XML 大小: {} 字节", keybox_content.len()));
    }

    // 4. 同步安装到当前活跃的 Keystore 模块
    log_message(&ctx.paths, "正在自动安装到当前激活的 Keystore 模块...");
    let install_data = quack_tricky_store::keybox::install(ctx, &keybox_content, "quack-sync")?;
    log_message(&ctx.paths, &format!(
        "安装成功！后端: [{:?}], 目标路径: {}",
        install_data.backend, install_data.target_path
    ));

    // 5. 更新 last_run 记录
    let now = time::OffsetDateTime::now_utc();
    let today = format!("{:04}-{:02}-{:02}", now.year(), u8::from(now.month()), now.day());
    let _ = fs::write(ctx.paths.data_root.join("last_run"), today);

    log_message(&ctx.paths, "=== 任务全部圆满完成 ===");
    Ok(install_data)
}

/// The continuous daemon loop.
pub async fn run_daemon(ctx: Context) -> Result<()> {
    let pid = std::process::id();
    let pid_file = ctx.paths.data_root.join("daemon.pid");
    let _ = fs::create_dir_all(&ctx.paths.data_root);
    let _ = fs::write(&pid_file, pid.to_string());

    log_message(&ctx.paths, &format!("QuackStore 调度守护进程启动 (PID: {pid})"));

    loop {
        let config = QuackConfig::load_or_init(&ctx.paths);
        if !config.scheduler.enabled {
            tokio::time::sleep(Duration::from_secs(config.scheduler.check_interval)).await;
            continue;
        }

        let now = time::OffsetDateTime::now_utc();
        let current_dow = now.weekday().number_from_monday(); // 1=Mon .. 7=Sun
        let current_hour = now.hour();
        let today_str = format!("{:04}-{:02}-{:02}", now.year(), u8::from(now.month()), now.day());

        let last_run = fs::read_to_string(ctx.paths.data_root.join("last_run"))
            .unwrap_or_default()
            .trim()
            .to_owned();

        // 检查触发条件
        if current_dow == config.scheduler.day_of_week && current_hour >= config.scheduler.hour {
            if last_run != today_str {
                log_message(&ctx.paths, "到达计划任务触发窗口，开始执行自动签发与同步...");
                match run_sync_once(&ctx).await {
                    Ok(_) => {
                        log_message(&ctx.paths, "本次计划任务执行成功！今日已完成。");
                        tokio::time::sleep(Duration::from_secs(config.scheduler.check_interval)).await;
                    }
                    Err(e) => {
                        log_message(&ctx.paths, &format!("计划任务执行失败: {e:#}，将在 15 分钟后重试..."));
                        tokio::time::sleep(Duration::from_secs(config.scheduler.retry_delay)).await;
                    }
                }
                continue;
            }
        }

        tokio::time::sleep(Duration::from_secs(config.scheduler.check_interval)).await;
    }
}
