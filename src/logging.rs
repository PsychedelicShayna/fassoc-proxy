use std::io::Write;
use std::{env, fs};

pub struct MainLogger;
pub static MAIN_LOGGER: MainLogger = MainLogger;

impl log::Log for MainLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let current_exe_path = env::current_exe().unwrap();
            let exe_dir_path = current_exe_path.parent().unwrap();
            let log_file_path = exe_dir_path.join("fassoc-proxy.log");

            let mut log_file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file_path)
                .unwrap();

            let log_time = chrono::Local::now().format("%d-%m-%y %H:%M:%S");

            let outmsg = format!(
                "[{}] - {} - {}",
                log_time.to_string(),
                record.level(),
                record.args()
            );

            println!("{}", outmsg);

            match writeln!(log_file, "{}", outmsg) {
                Ok(val) => val,
                Err(_) => (),
            }
        }
    }

    fn flush(&self) {}
}
