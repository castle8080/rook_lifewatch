# rook_lw_deamon

Runs video motion capture.

## Build dependencies

Dependin on the features in the build some extra dependencies may be required.

### Clang
You will need libclang.

### opencv

To use opencv, dependcies need to be installed to build.

```sudo apt update
sudo apt install libopencv-dev clang libclang-dev llvm-dev
```

### sqlite 3

The app needs sqlite3.

```
sudo apt install sqlite3 libsqlite3-dev
```

## Windows Support

There is some support for building and running on windows, but not well tested. It won't use libcamera, but opencv may work. You need to install clang and have it on the path and also need to download binaries for openc unpacked to c:\opencv.