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

    /// Path to the private key file. If it cannot be read, a new key will be generated
    #[clap(long)]
    pub private_key: String,
}
