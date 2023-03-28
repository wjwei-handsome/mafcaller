use std::{
    fs::File,
    io::{self, BufReader, Read},
    path::Path,
};

use log::{error, LevelFilter};
use log4rs::{
    append::console::{ConsoleAppender, Target},
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
use mafcaller::{get_maf_item, MAFItem};

fn main() {
    let log_level = LevelFilter::Info;
    // Build a stderr logger.
    let log_stderr = ConsoleAppender::builder()
        .target(Target::Stderr)
        .encoder(Box::new(PatternEncoder::new("{d} {h({l})} {m}{n}")))
        .build();
    let log_config = Config::builder()
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(log_level)))
                .build("stderr", Box::new(log_stderr)),
        )
        .build(Root::builder().appender("stderr").build(log_level))
        .unwrap();
    // init logger using config
    log4rs::init_config(log_config).unwrap();
    let test_file_path = "test/test.maf";
    let mut buf_reader = BufReader::new(File::open(Path::new(test_file_path)).unwrap());
    let mut stored_block = Vec::new();

    while let Ok(item) = get_maf_item(&mut buf_reader) {
        if let MAFItem::Block(block) = item {
            stored_block.push(block);
        }
    }

    println!("{:?}", stored_block);
}
