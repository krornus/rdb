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
- Iterator functions for breakpoints
- Add checks for continue function instead of returning Err(PTrace(ESRCH))
    - for continuing when not interrupted
- Modularize breakpoints, seperate from debugger?
- Change DebugError to enum
    - impl Error for DebugError { ... }
- Parallelize memory searching
- Fix Process struct genericism stuff
