use anyhow::Result;
use std::fs::File;
use std::io::prelude::*;
use std::env;
// use nix::sys::wait::WaitStatus;
//

mod coverage;
mod debugger;

use coverage::Coverage;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("No Binary Provided");
        return Ok(());
    }

    let mut cov = Coverage::new(&args[1], &args[2..])?;

    cov.set_marks(&args[1])?;
    let coverage = cov.get_coverage()?;
    println!("Hit {} Marks", coverage.len());
    let output_filename = args[1].clone() + ".cov";
    let mut output_file = File::create(&output_filename)?;
    for i in coverage.iter() {
        output_file.write(format!("{:#x}\n", i).as_bytes())?;
    }
    println!("Output written to {}", output_filename);
    Ok(())
}
