#[derive(clap::Parser, Clone, Debug)]
pub struct Args {
    /// URL to the PDNS Update server
    #[clap(long)]
    pub url: String,

    /// Hostname to update, like "myhost.example.com"
    #[clap(long)]
    pub hostname: String,

    /// If set, overrides the auto-detected IP address for this machine
    #[clap(long)]
    pub ip_address: Option<String>,
}
