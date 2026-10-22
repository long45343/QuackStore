//! Device-backed RKP provisioning. Vendor ABI access, TA framing, provisioning and
//! artifact ownership stay inside this feature; RKP protocol primitives come from quack-rkp.

mod artifacts;
mod cli;
mod error;
mod issuance;
#[cfg(all(target_os = "android", target_arch = "aarch64"))]
#[allow(unsafe_code)]
mod keymint;
mod process;
mod protocol;
mod service;
mod soc;
mod status;
mod v1;
mod worker;

pub use cli::FEATURE;
pub use issuance::Protocol;
pub use service::{Google, Options as ProvisionOptions, ProvisionData};
pub use status::{DeviceStatus, detect as status_detect};

pub async fn execute_provision(
    ctx: &quack_core::Context,
    acknowledge_limitations: bool,
    options: service::Options,
    server_url: Option<String>,
) -> anyhow::Result<service::ProvisionData> {
    let server = service::Google::new(server_url)?;
    service::provision(
        ctx,
        acknowledge_limitations,
        options,
        &service::RandomKeys,
        &service::Device {
            paths: &ctx.paths,
            sysroot: &ctx.sysroot,
        },
        &server,
        &service::DeviceEnvironment,
    )
    .await
}
