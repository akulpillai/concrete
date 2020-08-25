use derive_more::Display;
use log::error;
use std::error;
// Debugger Stuff
use linux_personality::{personality, ADDR_NO_RANDOMIZE};
use nix::sys::ptrace;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{fork, ForkResult, Pid};
use std::collections::HashMap;
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
    breakpoints: HashMap<u64, u64>,
}

impl Debugger {
    pub fn launch(prog: &String, args: &[String]) -> Result<Debugger, DebugError> {
        match fork() {
            Ok(ForkResult::Parent { child, .. }) => {
                let wait_status = waitpid(child, Some(WaitPidFlag::WSTOPPED))
                    .map_err(|_| DebugError::WaitError)?;
                let pid = wait_status.pid().unwrap(); //TODO: remove unwrap()
                return Ok(Debugger {
                    pid,
                    breakpoints: HashMap::new(),
                });
            }
            Ok(ForkResult::Child) => return Err(Debugger::run_target(prog, args)),
            Err(_) => {
                eprintln!("Fork failed");
                return Err(DebugError::LaunchError);
            }
        }
    }

    fn pid(&self) -> &Pid {
        &self.pid
    }

    fn run_target(prog: &String, progargs: &[String]) -> DebugError {
        eprintln!("Executing {}", prog);

        ptrace::traceme().expect("Ptrace Failed");

        // Disable ASLR
        personality(ADDR_NO_RANDOMIZE).expect("Could not disable ASLR");

        // Don't care for the output
        let err = Command::new(prog)
            .args(progargs)
            // .stdout(Stdio::null())
            // .stderr(Stdio::null())
            .exec();

        // Function will only return if there is an error
        eprintln!("Execution Failed: {}", err);
        DebugError::LaunchError
    }

    pub fn step(&self) -> Result<WaitStatus, DebugError> {
        ptrace::step(*self.pid(), None).map_err(|_| DebugError::PtraceError)?;
        let status = waitpid(*self.pid(), None).map_err(|_| DebugError::WaitError)?;
        Ok(status)
    }

    // To be only the first time after attaching,
    // if breakpoints are hit call resume()
    pub fn unpause(&self) -> Result<WaitStatus, DebugError> {
        ptrace::cont(*self.pid(), None).map_err(|_| DebugError::PtraceError)?;
        let status = waitpid(*self.pid(), None).map_err(|_| DebugError::WaitError)?;
        Ok(status)
    }

    pub fn resume(&mut self) -> Result<WaitStatus, DebugError> {
        // Subtract RIP and continue
        let mut regs = ptrace::getregs(*self.pid()).map_err(|_| DebugError::PtraceError)?;
        regs.rip -= 1;
        self.disable_breakpoint(&regs.rip)?;
        ptrace::setregs(*self.pid(), regs).map_err(|_| DebugError::PtraceError)?;
        Ok(self.unpause()?)
    }

    pub fn read(&self, addr: &u64) -> Result<u64, DebugError> {
        let value = ptrace::read(*self.pid(), *addr as *mut std::ffi::c_void)
            .map_err(|_| DebugError::PtraceError)?;
        Ok(value as u64)
    }

    pub fn write(&self, addr: &u64, value: u64) -> Result<(), DebugError> {
        unsafe {
            ptrace::write(
                *self.pid(),
                *addr as *mut std::ffi::c_void,
                value as *mut std::ffi::c_void,
            )
            .map_err(|_| DebugError::PtraceError)?;
        }
        Ok(())
    }
    // Returns Breakpoint
    pub fn set_breakpoint(&mut self, addr: u64) -> Result<(), DebugError> {
        let original_value = self.read(&addr)?;
        self.breakpoints.insert(addr, original_value);
        self.write(&addr, original_value & 0xFFFFFFFFFFFFFF00 | 0xCC)?;
        Ok(())
    }

    pub fn disable_breakpoint(&mut self, addr: &u64) -> Result<(), DebugError> {
        let original_value = self.breakpoints.get(&addr);
        match original_value {
            Some(val) => {
                self.write(&addr, *val)?;
                self.breakpoints.remove(&addr);
            }
            None => error!("Breakpoint not set"),
        }
        Ok(())
    }
}
