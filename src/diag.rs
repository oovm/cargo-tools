//! CLI 追踪与诊断（`tracing` + `miette`）。

use std::fmt;

use miette::Diagnostic;

use crate::errors::CargoError;

/// 从 `RUST_LOG` 初始化 `tracing-subscriber`（默认过滤级别：`warn`）。
pub fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));

    tracing_subscriber::registry().with(fmt::layer().with_writer(std::io::stderr).with_target(true)).with(filter).init();
}

#[derive(Debug, Diagnostic)]
#[diagnostic(code(cargo_tools::error))]
struct CliReport {
    message: String,
}

impl fmt::Display for CliReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for CliReport {}

/// 用 miette 将错误写入 stderr。
pub fn report_error(err: CargoError) {
    eprintln!("{:?}", miette::Report::new(CliReport { message: err.to_string() }));
}

/// 运行异步 CLI 入口：初始化 tracing，成功 exit `0`，失败经 miette 报告后 exit `1`。
pub fn main<F, Fut>(run: F) -> !
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<(), CargoError>>,
{
    init_tracing();
    let runtime = tokio::runtime::Runtime::new().expect("create tokio runtime");
    match runtime.block_on(run()) {
        Ok(()) => std::process::exit(0),
        Err(err) => {
            report_error(err);
            std::process::exit(1);
        }
    }
}
