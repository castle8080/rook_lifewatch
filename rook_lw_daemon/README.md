# rook_lw_deamon

Runs video motion capture.

## Build dependencies

Dependin on the features in the build some extra dependencies may be required.

### opencv

To use opencv, dependcies need to be installed to build.

```sudo apt update
sudo apt install libopencv-dev clang libclang-dev llvm-dev
```

### Building on windows

To build on windows you need to

1. Install rust
2. Install clang/llvm (needed to build opencv)
   https://github.com/llvm/llvm-project/releases/tag/llvmorg-18.1.8
3. Install vcpkg
