use std::{
    fmt,
    path::PathBuf,
};

use clap::Parser;
use common::types::{
    ConvexOrigin,
    ConvexSite,
};
use keybroker::{
    InstanceSecret,
    KeyBroker,
    DEV_INSTANCE_NAME,
    DEV_SECRET,
};
use metrics::SERVER_VERSION_STR;
use url::Url;

#[derive(Parser, Clone)]
#[clap(version = &**SERVER_VERSION_STR, author = "Convex, Inc. <no-reply@convex.dev>")]
pub struct LocalConfig {
    /// File path for SQLite
    #[clap(default_value = "convex_local_backend.sqlite3")]
    pub db_spec: String,

    /// Host interface to bind to
    #[clap(short, long, default_value = "0.0.0.0")]
    pub interface: ::std::net::Ipv4Addr,

    /// Host port daemon should bind to
    #[clap(short, long, default_value = "3210")]
    pub port: u16,

    /// Host port to bind for Convex HTTP Actions
    #[clap(long, default_value = "3211")]
    site_proxy_port: u16,

    /// Origin of the Convex server
    convex_origin: Option<ConvexOrigin>,

    /// Origin of the Convex HTTP Actions
    convex_site: Option<ConvexSite>,

    #[clap(long)]
    pub convex_http_proxy: Option<Url>,

    #[clap(long, requires = "instance_secret")]
    pub instance_name: Option<String>,

    #[clap(long, requires = "instance_name")]
    pub instance_secret: Option<String>,

    /// Identifier (like a user ID) to attach to any senty
    /// events generated by this backend.
    #[clap(long)]
    pub sentry_identifier: Option<String>,

    /// Which directory should local storage use
    #[clap(long, default_value = "convex_local_storage")]
    local_storage: String,
}

impl fmt::Debug for LocalConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Config")
            .field("convex_origin", &self.convex_origin)
            .field("convex_site", &self.convex_site)
            .field("instance_name", &self.instance_name)
            .finish()
    }
}

impl LocalConfig {
    pub fn http_bind_address(&self) -> ([u8; 4], u16) {
        (self.interface.octets(), self.port)
    }

    pub fn site_bind_address(&self) -> Option<([u8; 4], u16)> {
        Some((self.interface.octets(), self.site_proxy_port))
    }

    pub fn convex_origin_url(&self) -> ConvexOrigin {
        self.convex_origin
            .clone()
            .unwrap_or(format!("http://127.0.0.1:{}", self.port).into())
    }

    pub fn convex_site_url(&self) -> ConvexSite {
        self.convex_site
            .clone()
            .unwrap_or(format!("http://127.0.0.1:{}", self.site_proxy_port).into())
    }

    pub fn name(&self) -> String {
        self.instance_name
            .clone()
            .unwrap_or(DEV_INSTANCE_NAME.to_owned())
            .clone()
    }

    pub fn key_broker(&self) -> anyhow::Result<KeyBroker> {
        let name = self.name().clone();
        KeyBroker::new(&name, self.secret()?)
    }

    pub fn secret(&self) -> anyhow::Result<InstanceSecret> {
        InstanceSecret::try_from(
            self.instance_secret
                .clone()
                .unwrap_or(DEV_SECRET.to_owned())
                .as_str(),
        )
    }

    pub fn storage_dir(&self) -> PathBuf {
        self.local_storage.clone().into()
    }

    #[cfg(test)]
    pub fn new_for_test() -> anyhow::Result<Self> {
        use anyhow::Context;

        let tempdir_handle = tempfile::tempdir()?;
        let db_path = tempdir_handle.path().join("convex_local_backend.sqlite3");
        // Easiest way to get a config object with defaults is to parse from cmd line
        let config = Self::try_parse_from([
            "convex-local-backend",
            db_path.to_str().context("invalid db path")?,
            "--local-storage",
            tempdir_handle
                .path()
                .to_str()
                .context("invalid local storage path")?,
        ])?;
        Ok(config)
    }
}
