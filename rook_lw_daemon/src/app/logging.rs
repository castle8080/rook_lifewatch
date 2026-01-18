use tracing_subscriber::EnvFilter;

pub fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stdout)
        .with_ansi(false)
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
    			"%Y-%m-%dT%H:%M:%S%.3f%:z".to_owned(),
    		))
        .with_thread_ids(true)
        .compact()
        .init();
}
