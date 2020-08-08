use std::env;

use nix::sys::wait::WaitStatus;

mod debugger;
use debugger::{DebugError, Debugger};

fn main() -> Result<(), DebugError> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("No Binary Provided");
        return Ok(());
    }

    let dbg = Debugger::launch(&args[1], &args[2..])?;
    let mut counter: usize = 0;

    loop {
        counter += 1;
        match dbg.step() {
            Ok(WaitStatus::Exited(pid, _)) => {
                println!("Process Exited {}", pid);
                break;
            }
            Ok(WaitStatus::Stopped(_, _)) => {
                continue;
            }
            Ok(status) => {
                eprintln!("Interuppted by unexpected status: {:?}", status);
                break;
            }
            Err(e) => {
                eprintln!("Error while stepping: {}", e);
            }
        }
    }

    println!("Inscount {}", counter);
    Ok(())
}
