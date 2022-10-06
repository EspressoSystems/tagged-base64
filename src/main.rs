use clap::Parser;
use std::io;
use std::io::{Read, Write};
use std::process::exit;
use tagged_base64::TaggedBase64;

#[derive(Parser)]
#[command(
    name = "Tagged Base64",
    about = "Converts raw data to and from TaggedBase64 notation"
)]
pub struct MainOpt {
    /// Convert TaggedBase64 to raw data.
    ///
    /// To write the raw data to a file,
    ///    `tagged_base64 -d ADVENTURE~WFlaWllD > adventure.bin`
    #[arg(long = "decode", short = 'd')]
    pub tb64_str: Option<String>,

    /// Tag for raw data from stdin.
    ///
    /// To read from the terminal,
    ///    `echo -n asdf | tagged_base64 --tag FOO`
    ///
    /// To read a file,
    ///    `tagged_base64 --tag ADVENTURE < adventure.bin`
    /// or
    ///    `cat adventure.bin | tagged_base64 --tag ADVENTURE`
    #[arg(long = "tag")]
    pub tag: Option<String>,
}

fn main() {
    let parsed = MainOpt::parse();
    let tb64 = &parsed.tb64_str;
    let tag = &parsed.tag;
    if tb64.is_some() == tag.is_some() {
        println!(
            "tagged_base64: one argument required\n\
             Try 'tagged_base64 --help' for more information."
        );
        exit(2);
    } else if let Some(tb64_str) = &parsed.tb64_str {
        match TaggedBase64::parse(tb64_str) {
            Ok(v) => {
                io::stdout().write_all(&v.value()).unwrap();
                exit(0);
            }
            Err(e) => {
                print!("Error: {}", e);
                exit(1);
            }
        };
    } else if let Some(tag) = &parsed.tag {
        let mut v = Vec::new();
        io::stdin().read_to_end(&mut v).unwrap();
        println!("{}", TaggedBase64::new(tag, &v).unwrap());
        exit(0);
    }
}
