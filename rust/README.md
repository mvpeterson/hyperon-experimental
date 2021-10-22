![Hyperon CI](https://github.com/trueagi-io/hyperon-experimental/workflows/ci.yml/badge.svg)

# Overview

This is reimplementation of the [C++ Hyperon prototype](https://github.com/trueagi-io/hyperon) from scratch in Rust programming language.
The goal of the project is to replace the previous prototype but work is in progress so some features are absent.
Please see [Python examples](https://github.com/trueagi-io/hyperon/tree/master/python/tests) of previous version to become familiar with Hyperon features.

# Prerequisites

Install Rust v1.55, see [Rust installation
page](https://www.rust-lang.org/tools/install)

# Build project and run tests

```
cargo build
cargo test
```

# Test C API

Prerequisites:
```
pip install conan
cargo install cbindgen
```

Setup build:
```
mkdir -p target/capi
cd target/capi
conan install ../../capi
cmake ../../capi
```

Build and run tests:
```
make
make test_verbose
```

# Setup IDE

See [Rust Language Server](https://github.com/rust-lang/rls) page.