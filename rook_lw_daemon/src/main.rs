use rook_lw_daemon::app;
use rook_lw_daemon::RookLWResult;

fn main() -> RookLWResult<()> {
    app::init_tracing();
    let app = app::create_app()?;
    app.run()
}
