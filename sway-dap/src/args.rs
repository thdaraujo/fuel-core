use clap::arg_enum;
use structopt::StructOpt;
use tracing::Level;

use std::{env, io, net, path};

arg_enum! {
    #[derive(Debug)]
    pub enum Log {
        Error,
        Warn,
        Info,
        Debug,
        Trace,
    }
}

impl Into<Level> for Log {
    fn into(self) -> Level {
        match self {
            Log::Error => Level::ERROR,
            Log::Warn => Level::WARN,
            Log::Info => Level::INFO,
            Log::Debug => Level::DEBUG,
            Log::Trace => Level::TRACE,
        }
    }
}

#[derive(StructOpt, Debug)]
pub struct Opt {
    #[structopt(long = "log-level", default_value = "info")]
    pub log_level: Log,

    #[structopt(long = "log", parse(from_os_str))]
    pub log: Option<path::PathBuf>,

    #[structopt(long = "ip", default_value = "127.0.0.1", parse(try_from_str))]
    pub ip: net::IpAddr,

    #[structopt(long = "port", default_value = "4000")]
    pub port: u16,
}

impl Opt {
    pub fn exec(self) -> io::Result<net::SocketAddr> {
        let Opt {
            log,
            log_level,
            ip,
            port,
        } = self;
        let log_level: tracing::Level = log_level.into();

        match log {
            Some(log) => {
                let path = log
                    .as_path()
                    .parent()
                    .map(|l| Ok(l.to_path_buf()))
                    .unwrap_or_else(|| env::current_dir())?;

                let file = log.as_path().file_name().ok_or(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Invalid log path provided!",
                ))?;

                let appender = tracing_appender::rolling::never(path, file);
                let (non_blocking_appender, _guard) = tracing_appender::non_blocking(appender);

                let subscriber = tracing_subscriber::fmt::Subscriber::builder()
                    .with_max_level(log_level)
                    .with_writer(non_blocking_appender)
                    .finish();

                tracing::subscriber::set_global_default(subscriber)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            }

            None => {
                let subscriber = tracing_subscriber::fmt::Subscriber::builder()
                    .with_max_level(log_level)
                    .finish();

                tracing::subscriber::set_global_default(subscriber)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            }
        }

        let addr = net::SocketAddr::new(ip, port);

        Ok(addr)
    }
}
