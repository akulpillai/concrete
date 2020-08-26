use anyhow::Result;
use std::env;

// use nix::sys::wait::WaitStatus;

mod debugger;

use debugger::Debugger;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("No Binary Provided");
        return Ok(());
    }

    let mut dbg = Debugger::launch(&args[1], &args[2..])?;

    dbg.set_breakpoint(0x0000000000400c56)?;
    dbg.unpause()?;
    dbg.resume()?;

    // let mut counter: usize = 0;
    // loop {
    //     counter += 1;
    //     match dbg.step() {
    //         Ok(WaitStatus::Exited(pid, _)) => {
    //             println!("Process Exited {}", pid);
    //             break;
    //         }
    //         Ok(WaitStatus::Stopped(_, _)) => {
    //             continue;
    //         }
    //         Ok(status) => {
    //             eprintln!("Interuppted by unexpected status: {:?}", status);
    //             break;
    //         }
    //         Err(e) => {
    //             eprintln!("Error while stepping: {}", e);
    //         }
    //     }
    // }

    // println!("Inscount {}", counter);
    Ok(())
}
