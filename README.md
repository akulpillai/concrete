# concrete

Get coverage for compiled x86_64 ELFs. 

Right now, concrete only works on Linux and only outputs coverage information as basic block
addresses in a `.cov` file.
The idea is to add snapshotting capabilities allowing us to write an in memory fuzzer with it.

```
USAGE: 
    concrete <binary> <binary_arg1> ...
```
