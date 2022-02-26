use std::io;
use std::io::{Read, Write};
use std::process::exit;
use structopt::StructOpt;
use tagged_base64::TaggedBase64;

#[derive(StructOpt)]
#[structopt(
    name = "Tagged Base64",
    about = "Converts raw data to and from TaggedBase64 notation"
)]
pub struct MainOpt {
    /// Convert TaggedBase64 to raw data.
    ///
    /// To write the raw data to a file,
    ///    `tagged_base64 -d ADVENTURE~WFlaWllD > adventure.bin`
    #[structopt(long = "decode", short = "d")]
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
    #[structopt(long = "tag")]
    pub tag: Option<String>,
}

fn main() {
    let tb64 = MainOpt::from_args().tb64_str;
    let tag = MainOpt::from_args().tag;
    if tb64.is_some() == tag.is_some() {
        println!(
            "tagged_base64: one argument required\n\
             Try 'tagged_base64 --help' for more information."
        );
        exit(2);
    } else if MainOpt::from_args().tb64_str.is_some() {
        let s: String = MainOpt::from_args().tb64_str.unwrap();
        match TaggedBase64::parse(&s) {
            Ok(v) => {
                io::stdout().write(&v.value()).unwrap();
                exit(0);
            }
            Err(e) => {
                print!("Error: {}", e);
                exit(1);
            }
        };
    } else if MainOpt::from_args().tag.is_some() {
        let mut v = Vec::new();
        io::stdin().read_to_end(&mut v).unwrap();
        println!(
            "{}",
            TaggedBase64::new(&MainOpt::from_args().tag.unwrap(), &v)
                .unwrap()
                .to_string()
        );
        exit(0);
    }
}
