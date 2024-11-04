#[derive(clap::Parser, Clone, Debug)]
pub struct Args {
    /// Root URL of the PowerDNS server (without trailing slash)
    #[clap(long)]
    pub root_url: String,
    /// API key to use
    #[clap(long)]
    pub api_key: String,

    /// PowerDNS zone name
    #[clap(long)]
    pub zone: String,
}
