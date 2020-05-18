use std::convert::TryFrom;
use std::path::PathBuf;
use structopt::StructOpt;

use sc_cli::{RunCmd, Subcommand, SubstrateCli};
use sp_core::crypto::{set_default_ss58_version, Ss58AddressFormat};

use crate::chain_spec;
use crate::service;

#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[structopt(flatten)]
    pub run: RunCmd,
}

pub fn run(cli: Cli) -> sc_cli::Result<()> {
    set_address_type(&cli)?;

    match &cli.subcommand {
        None => {
            let runner = cli.create_runner(&cli.run)?;
            runner.run_node(service::new_light, service::new_full, runtime::VERSION)
        }
        Some(subcommand) => {
            let runner = cli.create_runner(subcommand)?;
            runner.run_subcommand(subcommand, |config| Ok(new_full_start!(config).0))
        }
    }
}

fn set_address_type(opt: &Cli) -> sc_cli::Result<()> {
    let chain_key = match opt.run.shared_params.chain {
        Some(ref chain) => chain.clone(),
        None => {
            if opt.run.shared_params.dev {
                "dev".into()
            } else {
                "".into()
            }
        }
    };
    let alternative = match chain_spec::Alternative::from(&chain_key) {
        Some(spec) => spec,
        None => {
            let spec = chain_spec::ChainSpec::from_json_file(PathBuf::from(&chain_key))?;
            let id = spec.id();
            chain_spec::get_alternative_from_id(id)?
        }
    };

    let info = chain_spec::CHAIN_TYPE
        .get(&alternative)
        .ok_or("Alternative must exist")?;
    let p = info.properties.as_ref().ok_or(format!(
        "SpecInfo.properties for this Alternative:{:?} must not be None",
        alternative
    ))?;
    let v = p
        .get("ss58Format")
        .ok_or("`ss58Format` must exist in properties")?;
    let v = v.as_u64().ok_or("`ss58Format` must be a number")?;
    let v = Ss58AddressFormat::try_from(v.to_string().as_str()).expect("must use u8");
    Ok(set_default_ss58_version(v))
}

// fn load_spec(id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
//
// }

// fn run_until_exit<T, E>(mut runtime: Runtime, service: T, e: E) -> sc_cli::Result<()>
// where
//     T: AbstractService,
//     E: IntoExit,
// {
//     let (exit_send, exit) = oneshot::channel();
//
//     let informant = informant::build(&service);
//
//     let future = select(exit, informant).map(|_| Ok(())).compat();
//
//     runtime.executor().spawn(future);
//
//     // we eagerly drop the service so that the internal exit future is fired,
//     // but we need to keep holding a reference to the global telemetry guard
//     let _telemetry = service.telemetry();
//
//     let service_res = {
//         let exit = e.into_exit();
//         let service = service.map_err(|err| error::Error::Service(err)).compat();
//         let select = select(service, exit).map(|_| Ok(())).compat();
//         runtime.block_on(select)
//     };
//
//     let _ = exit_send.send(());
//
//     // TODO [andre]: timeout this future #1318
//
//     use futures01::Future;
//
//     let _ = runtime.shutdown_on_idle().wait();
//
//     service_res
// }
//
// // handles ctrl-c
// pub struct Exit;
// impl IntoExit for Exit {
//     type Exit = Map<oneshot::Receiver<()>, fn(Result<(), oneshot::Canceled>) -> ()>;
//     fn into_exit(self) -> Self::Exit {
//         // can't use signal directly here because CtrlC takes only `Fn`.
//         let (exit_send, exit) = oneshot::channel();
//
//         let exit_send_cell = RefCell::new(Some(exit_send));
//         ctrlc::set_handler(move || {
//             let exit_send = exit_send_cell
//                 .try_borrow_mut()
//                 .expect("signal handler not reentrant; qed")
//                 .take();
//             if let Some(exit_send) = exit_send {
//                 exit_send.send(()).expect("Error sending exit notification");
//             }
//         })
//         .expect("Error setting Ctrl-C handler");
//
//         exit.map(drop)
//     }
// }
