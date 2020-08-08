use std::env;
use std::error;

// Debugger Stuff
use nix::unistd::{fork, ForkResult, Pid};
use nix::sys::ptrace;
use nix::sys::wait::{waitpid, WaitStatus, WaitPidFlag};
use nix::sys::signal::Signal;
use std::process::{Command, Stdio};
use std::os::unix::process::CommandExt;
use linux_personality::{personality, ADDR_NO_RANDOMIZE};
// use std::ffi::{CString, CStr};

// TODO: change result to WaitStatus
struct Debugger {
    pid:  Pid,
}

impl Debugger {
    pub fn launch(prog: &String, args: &[String])
                  -> Result<Debugger, Box<dyn error::Error>> {
        match fork() {
            Ok(ForkResult::Parent { child, .. }) => {
                let wait_status = waitpid(child, Some(WaitPidFlag::WSTOPPED))?;
                let pid = wait_status.pid().unwrap();
                return Ok(
                    Debugger { pid }
                )
            }
            Ok(ForkResult::Child) => {
                return Err(Box::new(Debugger::run_target(prog, args)))
            },
            Err(e) => {
                eprintln!("Fork failed");
                return Err(Box::new(e))
            },
        }
    }

    fn pid(&self) -> Pid { self.pid }

    fn run_target(prog: &String, progargs: &[String]) -> std::io::Error {
        eprintln!("Executing {}", prog);

        ptrace::traceme().expect("Ptrace Failed");

        // Disable ASLR
        personality(ADDR_NO_RANDOMIZE).expect("Could not disable ASLR");

        // Don't care for the output
        let err = Command::new(prog)
            .args(progargs)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .exec();

        // Function will only return if there is an error
        eprintln!("Execution Failed: {}", err);
        err
    }

    fn step(&self) -> Result<WaitStatus, Box<dyn std::error::Error>> {
        ptrace::step(self.pid(), None)?;
        let status = waitpid(self.pid(), None)?;
        Ok(status)
    }
}


fn main() -> Result<(), Box<dyn error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("No Binary Provided");
        return Ok(())
    }

    let dbg = Debugger::launch(&args[1], &args[2..])?;
    let mut counter: usize = 0;

    loop{
         counter += 1;
         match dbg.step() {
             Ok(WaitStatus::Exited(pid, _)) => {
                 println!("Process Exited {}", pid);
                 break
             }
             _ => {},
         }
     }

    println!("Inscount {}", counter);
    Ok(())
}
