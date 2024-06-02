# SwordFish Comm Example (computer side)

This is an EXAMPLE repo that only contains parts of our internal serial over USB communication protocol.
The main purposes of this repo:
* display the protocol
* showcase how we made our rust code available in python/c/cpp/java with [flapigen](https://github.com/Dushistov/flapigen-rs) and [pyo3](https://github.com/PyO3/pyo3) to support multiple products.

## system dependencies
assumes you have rust [installed](https://www.rust-lang.org/tools/install)
```
#for build process
cargo install --force cargo-make

#for serialport-rs
sudo apt install libudev-dev pkg-config

#for python wrapper
pip install maturin patchelf

#for android targets
sudo apt install clang
rustup target add arm-linux-androideabi
rustup target add aarch64-linux-android
```

## Setup

### JVM for local java_wrapper (no cross compilaton)
set JAVA_HOME to the jvm directory. easiest to install the jvm through the jdk
```
sudo apt install openjdk-17-jdk
ll /usr/lib/jvm/java-17-openjdk-amd64/ #test that it was installed properlly
JAVA_HOME="/usr/lib/jvm/java-17-openjdk-amd64/"
```
_(recommended to export in ~/.bashrc)_

### NDK for compiling to android
set ANDROID_NDK_HOME to the ndk directory which you can install [through this link](https://developer.android.com/ndk/downloads)
```
ANDROID_NDK_HOME="<DIRECTORY_PATH>"
```
_(recommended to export in ~/.bashrc)_

#### linkers
inside `.cargo/config`, change the linkers' path such that:
```
[target.aarch64-linux-android]
linker = "<LINKER_BIN_PATH>/aarch64-linux-android21-clang++"

[target.armv7-linux-androideabi]
linker = "<LINKER_BIN_PATH>/armv7a-linux-androideabi21-clang++"
```

where <LINKER_BIN_PATH> is usually located in $NDK/toolchains/llvm/prebuilt/linux-x86_64/bin

## rust only:
```
cargo build
cargo test
```

## cargo make for wrappers
```
cargo make (displays help infomration)
cargo make cpp_wrapper
cargo make java_wrapper
cargo make python_wrapper
cargo make all_wrappers
```

# Examples
the cpp and java_desktop examples are hard-coded to run on the default host target, but can easily be configured to run on other desktop targets with simple modifications to paths variables within their respective sources.

## java example:
```
cargo make java_wrapper
cd java_example
javac Main.java
java Main
```


## cpp example
```
cargo make cpp_wrapper
cd cpp_example
mkdir build
cd build
cmake ..
make 
./main
```

## python example
```
cargo make python_wrapper
cd python_example
pip install <wheel_filename> --force-reinstall
python3 main.py
```

## to make Java wrapper for android:
```
cargo make java_wrapper --target aarch64-linux-android

or

cargo make java_wrapper --target armv7-linux-androideabi
```

note, it is also possible to use cargo-ndk
```
cargo ndk -t aarch64-linux-android -o ./jniLibs build --release
```