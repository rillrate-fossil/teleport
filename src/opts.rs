use clap::Clap;

#[derive(Clap)]
pub struct Opts {
    #[clap(long)]
    pub name: Option<String>,
}
