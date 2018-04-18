# RDB: Rust programatic DeBugger

RDB is a debugger written in rust used to debug binary applications.
Currently supports x86_64 ELF files.

# Install
```cargo build --release```

# Usage
Example usages are found in the examples directory
Run examples using ```cargo run --example <example name>```
Do not include ```.rs``` in the example name

# TODO
- Benchmarking for hashed and unhashed memory
- Iterator functions for breakpoints
- Add checks for continue function instead of returning Err(PTrace(ESRCH))
    - for continuing when not interrupted
- Create seperate breakpoint manager for debugger
    - place in RefCell<> etc... for interior mutability
- Create /proc/ interface for reading memory
    - reading from /proc/pid/mem directly does not work
    - dereferencing unmapped sections gives io error
- Parallelize HashMemory
- Change DebugError to enum
    - impl Error for DebugError { ... }
