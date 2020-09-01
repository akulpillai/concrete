use crate::debugger::Debugger;
use capstone::arch::x86::X86OperandType;
use capstone::arch::ArchOperand;
use capstone::prelude::*;
use capstone::InsnGroupType::*;
use nix::sys::wait::WaitStatus;
use std::path::PathBuf;

use anyhow::{anyhow, Result};

struct TextSection {
    addr: u64,
    // size: usize,
    data: Vec<u8>,
}

fn get_text_section(fname: &str) -> Result<TextSection> {
    let path = PathBuf::from(fname);
    let file = elf::File::open_path(&path).map_err(|_| anyhow!("Failed to open ELF"))?;

    let section = match file.get_section(".text") {
        Some(sec) => sec,
        None => return Err(anyhow!("Failed to get .text section")),
    };
    Ok(TextSection {
        addr: section.shdr.addr as u64,
        // size: section.shdr.size as usize,
        data: section.data.to_vec(),
    })
}

fn init_capstone() -> CsResult<Capstone> {
    let mut cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .build()?;
    cs.set_detail(true)?;

    Ok(cs)
}

fn is_cflow_group(g: u32) -> bool {
    g == CS_GRP_JUMP || g == CS_GRP_CALL || g == CS_GRP_RET || g == CS_GRP_IRET
}

fn is_cflow_ins(detail: &InsnDetail) -> bool {
    for i in detail.groups() {
        if is_cflow_group(i.0 as u32) {
            return true;
        }
    }
    false
}
pub struct Coverage {
    dbg: Debugger,
}

//TODO: refactor to get multiple coverages
impl Coverage {
    pub fn new(prog: &str, args: &[String]) -> Result<Coverage> {
        Ok(Coverage {
            dbg: Debugger::launch(&prog, &args)?,
        })
    }

    pub fn set_marks(&mut self, prog: &str) -> Result<()> {
        let text_section = get_text_section(&prog)?;

        let cs = init_capstone().map_err(|_| anyhow!("Failed to init Capstone"))?;

        let insns = cs
            .disasm_all(&text_section.data, text_section.addr)
            .map_err(|_| anyhow!("Failed to disassemble"))?;
        println!("Found {} instructions", insns.len());
        let mut push_next = true;
        let mut bb_no: usize = 0;
        for i in insns.iter() {
            if push_next {
                //TODO: handle result
                if self.dbg.set_breakpoint(i.address()).is_ok() {
                    bb_no += 1;
                }
            }
            let detail: InsnDetail = cs
                .insn_detail(&i)
                .map_err(|_| anyhow!("Failed to get insn detail"))?;
            let arch_detail: ArchDetail = detail.arch_detail();
            let ops = arch_detail.operands();
            if is_cflow_ins(&detail) {
                push_next = true;
                for op in ops {
                    if let ArchOperand::X86Operand(op) = op {
                        if let X86OperandType::Imm(addr) = op.op_type {
                            if addr != 0 {
                                //TODO: handle result
                                if self.dbg.set_breakpoint(addr as u64).is_ok() {
                                    bb_no += 1
                                }
                            }
                        }
                    }
                }
            } else {
                push_next = false;
            }
        }

        println!("{} Marks Set", bb_no);

        // dbg.set_breakpoint(0x0000000000400c56)?;
        // dbg.unpause()?;
        // dbg.resume()?;
        Ok(())
    }

    pub fn get_coverage(&mut self) -> Result<Vec<u64>> {
        let mut cov = Vec::new();
        self.dbg.unpause()?;
        loop {
            match self.dbg.resume() {
                Ok((WaitStatus::Exited(pid, _), _)) => {
                    println!("Process Exited {}", pid);
                    break;
                }
                Ok((WaitStatus::Stopped(_, _), rip)) => {
                    cov.push(rip);
                    continue;
                }
                Ok((status, _)) => {
                    return Err(anyhow!("Interuppted by unexpected status: {:?}", status))
                }
                Err(e) => return Err(anyhow!("Error while continuing: {}", e)),
            }
        }
        Ok(cov)
    }
}
