use tairitsu_packager::cli;

fn main() {
    if let Some(result) = cli::handle_sync_daemon() {
        match result {
            Ok(()) => {}
            Err(e) => {
                tairitsu_packager::log_fail!("{}", e);
                std::process::exit(1);
            }
        }
        std::process::exit(0);
    }

    cli::run_tokio();
}
