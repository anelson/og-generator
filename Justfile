default:
    just --list

clean: 
    cargo clean && rm -rf pkg

install_tools:
    cargo install wasm-pack

# Set up LLVM environment on macOS
#
# See https://github.com/rust-lang/libz-sys/issues/103#issuecomment-1339702214 to understand
# why such stupidity is even necessary
set-llvm-env:
    #!/usr/bin/env sh
    if command -v brew >/dev/null 2>&1 && brew --prefix llvm >/dev/null 2>&1; then
        LLVM_PATH=$(brew --prefix llvm)
        echo "PATH=\"${LLVM_PATH}/bin:$PATH\""
        echo "LDFLAGS=\"-L${LLVM_PATH}/lib -L${LLVM_PATH}/lib/c++ -Wl,-rpath,${LLVM_PATH}/lib\""
        echo "CPPFLAGS=\"-I${LLVM_PATH}/include\""
        echo "CC=\"${LLVM_PATH}/bin/clang\""
        echo "CXX=\"${LLVM_PATH}/bin/clang++\""
    fi

# Build the wasm module for development
# You typically want this to be `release` even when developing.  you won't get a stack trace
# either way, and the debug build is SLOW!!
wasm-dev: 
    #!/usr/bin/env sh
    # Use the `set-llvm-env` script to print out the env vars that should be set to use the Brew LLVM
    # if this is macoS. On Linux this will have no effect
    set -a
    eval "`just set-llvm-env`"
    set +a
    which clang
    clang --version
    wasm-pack build --release

# Build the wasm module for release on Cloudflare Pages
# Don't try to run this locally
wasm-cfp:
    wasm-pack build --release
