use std::env;

// Debugger Stuff
use nix::unistd::{fork, ForkResult};
use nix::sys::ptrace::{traceme, step};
use nix::sys::wait::{wait, WaitStatus};
use nix::sys::signal::Signal;
use std::process::Command;
use std::os::unix::process::CommandExt;
// use std::ffi::{CString, CStr};

// ignoring program args for now
fn run_target(prog: &String, _progargs: &Vec<String>) {
    println!("Executing {}", prog);

    traceme().expect("Ptrace Failed");

    // Don't care for the output
    // Benchmarking right now
    Command::new(prog).exec();

    // CStr stuff for nix execv
    // prog.push('\x00');
    // let prog = &prog.into_bytes();
    // let progname = CStr::from_bytes_with_nul(prog)
    //     .expect("Could not Construct CStr");
}

fn run_debugger() {
    let mut wait_status = wait();
    let mut counter: usize = 0;

    loop {
        counter += 1;
        match wait_status {
            Ok(WaitStatus::Stopped(pid, _))=> {
                if let Err(e) = step(pid, Signal::SIGCONT){
                    println!("Step Error {}", e);
                    break;
                };
            },
            Ok(status) => {
                println!("{:?}", status);
                break;
            },
            Err(e) => {
                println!("Wait Error {}", e)
            }
        }
        wait_status = wait();
    }
    println!("Inscount: {}", counter);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("No Binary Provided");
        return
    }

    match fork() {
        Ok(ForkResult::Parent { child, .. }) => {
            println!("[*] Executing Debugger, Child PID: {}", child);
            run_debugger();
        }
        Ok(ForkResult::Child) => {
            run_target(&args[1].clone(), &args[2..].to_vec());
        },
        Err(_) => println!("Fork failed"),
    }
}
