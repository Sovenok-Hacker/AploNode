use std::{io, str::FromStr};

use tracing_subscriber::{filter::Directive, fmt, layer::SubscriberExt, EnvFilter};

use crate::models::config::LogConfig;
pub fn init_logger(log_config: &LogConfig) {
    let mut env_filter = EnvFilter::new(
        log_config
            .global_directive
            .as_ref()
            .unwrap_or(&String::from("")),
    );

    if let Some(directives) = &log_config.directives {
        for (name, value) in directives.iter() {
            env_filter = env_filter
                .add_directive(Directive::from_str(&format!("{}={}", name, value)).unwrap());
        }
    }

    let collector = tracing_subscriber::registry().with(env_filter).with(
        fmt::Layer::new()
            .with_writer(io::stdout)
            .with_thread_names(true),
    );
    let file_appender = tracing_appender::rolling::minutely(&log_config.logs_dir, "node_log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let collector = collector.with(
        fmt::Layer::new()
            .with_writer(non_blocking)
            .with_thread_names(true),
    );
    tracing::subscriber::set_global_default(collector).unwrap();
}
