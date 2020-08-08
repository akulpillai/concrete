use derive_more::Display;
use std::error;
// Debugger Stuff
use linux_personality::{personality, ADDR_NO_RANDOMIZE};
use nix::sys::ptrace;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{fork, ForkResult, Pid};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
// use std::ffi::{CString, CStr};

#[derive(Display, Debug)]
pub enum DebugError {
    #[display(fmt = "Failed during Ptrace")]
    PtraceError,
    #[display(fmt = "Failed during Wait")]
    WaitError,
    #[display(fmt = "Failed to launch application")]
    LaunchError,
}

impl error::Error for DebugError {}

pub struct Debugger {
    pid: Pid,
}

impl Debugger {
    pub fn launch(prog: &String, args: &[String]) -> Result<Debugger, DebugError> {
        match fork() {
            Ok(ForkResult::Parent { child, .. }) => {
                let wait_status = waitpid(child, Some(WaitPidFlag::WSTOPPED))
                    .map_err(|_| DebugError::WaitError)?;
                let pid = wait_status.pid().unwrap(); //TODO: remove unwrap()
                return Ok(Debugger { pid });
            }
            Ok(ForkResult::Child) => return Err(Debugger::run_target(prog, args)),
            Err(_) => {
                eprintln!("Fork failed");
                return Err(DebugError::LaunchError);
            }
        }
    }

    fn pid(&self) -> Pid {
        self.pid
    }

    fn run_target(prog: &String, progargs: &[String]) -> DebugError {
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
        DebugError::LaunchError
    }

    pub fn step(&self) -> Result<WaitStatus, DebugError> {
        ptrace::step(self.pid(), None).map_err(|_| DebugError::PtraceError)?;
        let status = waitpid(self.pid(), None).map_err(|_| DebugError::WaitError)?;
        Ok(status)
    }

    // continue execution
    pub fn unpause(&self) -> Result<WaitStatus, DebugError> {
        ptrace::cont(self.pid(), None).map_err(|_| DebugError::PtraceError)?;
        let status = waitpid(self.pid(), None).map_err(|_| DebugError::WaitError)?;
        Ok(status)
    }
}
