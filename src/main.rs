//! Substrate Node Template CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;

use std::ffi::OsString;

use sc_cli::SubstrateCli;
use sc_service::ChainSpec;
use structopt::StructOpt;

use cli::Cli;

impl SubstrateCli for Cli {
    fn impl_name() -> &'static str {
        "Rio Defi Chain Node"
    }

    fn impl_version() -> &'static str {
        env!("SUBSTRATE_CLI_IMPL_VERSION")
    }

    fn executable_name() -> &'static str {
        "riodefi"
    }

    fn description() -> &'static str {
        "rio defi blockchain"
    }

    fn author() -> &'static str {
        "Rio Defi Team"
    }

    fn support_url() -> &'static str {
        "support@riodefi.com"
    }

    fn copyright_start_year() -> i32 {
        2019
    }

    fn load_spec(&self, id: &str) -> Result<Box<dyn ChainSpec>, String> {
        Ok(match chain_spec::Alternative::from(id) {
            Some(spec) => Box::new(spec.load()?),
            None => Err("not support")?,
        })
    }

    fn from_args() -> Self
    where
        Self: StructOpt + Sized,
    {
        let mut args = std::env::args_os().collect::<Vec<OsString>>();
        let find = args.iter().find(|s| {
            let s = s.to_string_lossy();
            if s.contains("execution") {
                true
            } else {
                false
            }
        });
        if find.is_none() {
            args.push("--execution=NativeElseWasm".to_string().into())
        }
        <Self as SubstrateCli>::from_iter(args)
    }
}

fn main() -> sc_cli::Result<()> {
    let cli = <Cli as sc_cli::SubstrateCli>::from_args();
    cli::run(cli)
}
