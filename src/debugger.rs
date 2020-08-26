use anyhow::{anyhow, Context, Result};
use linux_personality::{personality, ADDR_NO_RANDOMIZE};
use nix::sys::ptrace;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{fork, ForkResult, Pid};
use std::collections::HashMap;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
// use std::ffi::{CString, CStr};

pub struct Debugger {
    pid: Pid,
    breakpoints: HashMap<u64, u64>,
}

impl Debugger {
    pub fn launch(prog: &str, args: &[String]) -> Result<Debugger> {
        match fork() {
            Ok(ForkResult::Parent { child, .. }) => {
                let wait_status = waitpid(child, Some(WaitPidFlag::WSTOPPED))?;
                let pid = wait_status.pid().unwrap(); //TODO: Fix this
                Ok(Debugger {
                    pid,
                    breakpoints: HashMap::new(),
                })
            }
            Ok(ForkResult::Child) => Err(Debugger::run_target(prog, args)),
            Err(_) => Err(anyhow!("Fork failed")),
        }
    }

    fn pid(&self) -> &Pid {
        &self.pid
    }

    fn run_target(prog: &str, progargs: &[String]) -> anyhow::Error {
        if ptrace::traceme().is_err() {
            return anyhow!("Failed to start ptrace");
        }

        // Disable ASLR
        if personality(ADDR_NO_RANDOMIZE).is_err() {
            return anyhow!("Failed to disable ASLR");
        }

        // TODO: Don't care for the output
        let err = Command::new(prog)
            .args(progargs)
            // .stdout(Stdio::null())
            // .stderr(Stdio::null())
            .exec();

        // Function will only return if there is an error
        anyhow!("Execution Failed: {}", err)
    }

    pub fn step(&self) -> Result<WaitStatus> {
        ptrace::step(*self.pid(), None)?;
        let status = waitpid(*self.pid(), None)?;
        Ok(status)
    }

    // To be only used the first time after attaching,
    // if breakpoints are hit call resume()
    pub fn unpause(&self) -> Result<WaitStatus> {
        ptrace::cont(*self.pid(), None)?;
        let status = waitpid(*self.pid(), None)?;
        Ok(status)
    }

    pub fn resume(&mut self) -> Result<WaitStatus> {
        // Subtract RIP and unpause
        let mut regs = ptrace::getregs(*self.pid())?;
        regs.rip -= 1;
        self.disable_breakpoint(&regs.rip)?;
        ptrace::setregs(*self.pid(), regs)?;
        Ok(self.unpause()?)
    }

    pub fn read(&self, addr: &u64) -> Result<u64> {
        let value = ptrace::read(*self.pid(), *addr as *mut std::ffi::c_void)?;
        Ok(value as u64)
    }

    pub fn write(&self, addr: &u64, value: u64) -> Result<()> {
        unsafe {
            ptrace::write(
                *self.pid(),
                *addr as *mut std::ffi::c_void,
                value as *mut std::ffi::c_void,
            )?;
        }
        Ok(())
    }

    pub fn set_breakpoint(&mut self, addr: u64) -> Result<()> {
        let original_value = self
            .read(&addr)
            .context("Failed to read at breakpoint address")?;
        self.breakpoints.insert(addr, original_value);
        self.write(&addr, original_value & 0xFFFFFFFFFFFFFF00 | 0xCC)?;
        Ok(())
    }

    pub fn disable_breakpoint(&mut self, addr: &u64) -> Result<()> {
        let original_value = self.breakpoints.get(&addr);
        match original_value {
            Some(val) => {
                self.write(&addr, *val)?;
                self.breakpoints.remove(&addr);
            }
            None => return Err(anyhow!("Breakpoint not set at 0x{:x}", addr)),
        }
        Ok(())
    }
}
