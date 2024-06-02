#[allow(unused_imports)]
use std::{env, fs, io::Write, path::Path, path::PathBuf};
#[cfg(feature = "java_wrapper")]
use bindgen::RustTarget;
#[cfg(feature = "java_wrapper")]
use std::fmt;

fn main() {
    //hack to always run the build script
    println!("cargo:rerun-if-changed=None");
    
    #[cfg(any(feature = "python_wrapper"))]
    {
        let crate_root = env::var("CARGO_MANIFEST_DIR")
        .expect("no CARGO_MANIFEST_DIR, but cargo should provide it");
        let crate_root_path = PathBuf::from(crate_root);
        let target = std::env::var("TARGET").unwrap();
        let store_filepath = crate_root_path.join("target/target.txt");
        let mut file = fs::File::create(&store_filepath)
        .or_else(|_| {
            fs::remove_file(&store_filepath)?;
            fs::File::create(&store_filepath)
        })
        .expect("Failed to create target.txt file");
            writeln!(file, "{}", target)
        .expect("Failed to write to target.txt file");
    }
    
    #[cfg(any(feature = "java_wrapper", feature = "cpp_wrapper"))]
    {
        let crate_root = env::var("CARGO_MANIFEST_DIR")
            .expect("no CARGO_MANIFEST_DIR, but cargo should provide it");

        let crate_root_path = PathBuf::from(crate_root);
        let ffi_source_folder = crate_root_path.join("src").join("ffi");
        let interface_filepath = ffi_source_folder.join("interface.rs");
        let out_dir = env::var("OUT_DIR").expect("no OUT_DIR, but cargo should provide it");

        //write executable_path to a file so it can be read by the cargo-make task after the build process was done
        //this is a stupid way to do it, but I couldn't find a better way
        let out_path = PathBuf::from(&out_dir);
        let store_filepath = crate_root_path.join("target/out_path.txt");
        let mut file = fs::File::create(&store_filepath)
            .or_else(|_| {
                fs::remove_file(&store_filepath)?;
                fs::File::create(&store_filepath)
            })
            .expect("Failed to create executable_path.txt file");
        writeln!(file, "{}", out_path.to_string_lossy())
            .expect("Failed to write to executable_path.txt file");

        #[cfg(feature = "cpp_wrapper")]
        {
            use flapigen::{CppConfig, CppOptional, CppStrView, CppVariant, LanguageConfig};
            let cpp_output_dir = crate_root_path.join("wrappers").join("cpp");
            if !cpp_output_dir.exists() {
                std::fs::create_dir_all(&cpp_output_dir)
                    .expect("Failed to create output directory");
            }

            let cpp_cfg = CppConfig::new(cpp_output_dir, "swordfish_com".into())
                .cpp_optional(CppOptional::Std17)
                .cpp_variant(CppVariant::Std17)
                .cpp_str_view(CppStrView::Std17);
            let swig_gen = flapigen::Generator::new(LanguageConfig::CppConfig(cpp_cfg))
                .rustfmt_bindings(true)
                .remove_not_generated_files_from_output_directory(true);
            swig_gen.expand(
                "swordfish_com_cpp",
                &interface_filepath,
                &Path::new(&out_dir).join("glue_cpp.rs"),
            );
        }

        #[cfg(feature = "java_wrapper")]
        {
            use flapigen::{JavaConfig, JavaReachabilityFence, LanguageConfig};
            let java_output_dir = crate_root_path.join("wrappers").join("java");
            if !java_output_dir.exists() {
                std::fs::create_dir_all(&java_output_dir)
                    .expect("Failed to create output directory");
            }

            let jni_c_headers_rs;
            let target = std::env::var("TARGET").unwrap();
            if [
                "i686-linux-android",
                "x86_64-linux-android",
                "arm-linux-androideabi",
            ]
            .contains(&target.as_str()) {
                eprintln!("specified android target not supported by this crate");
                eprintln!("supported targets are: aarch64-linux-android, arm-linux-androideabi");
                std::process::exit(1);
            }
            
            if [
                "aarch64-linux-android",
                "armv7-linux-androideabi",
            ]
            .contains(&target.as_str())
            { //android
                let ndk_home = env::var("ANDROID_NDK_HOME").expect("NDK_HOME env variable not set");
                let jni_h_path = search_file_in_directory_recursive(&[ndk_home], "jni.h")
                    .expect("Can not find jni.h");
                let include_dirs = jni_h_path.parent().unwrap().to_path_buf();
                jni_c_headers_rs = Path::new(&out_path).join("jni_c_header.rs");
                gen_java_binding(&target, &[include_dirs], &[jni_h_path], &jni_c_headers_rs).unwrap();
            } else { // host system is the target
                let java_home = env::var("JAVA_HOME").expect("JAVA_HOME env variable not set");
                let java_include_dir = Path::new(&java_home).join("include");
                let java_sys_include_dir = java_include_dir.join(if target.contains("windows") {
                    "win32"
                } else if target.contains("darwin") {
                    "darwin"
                } else {
                    "linux"
                });
                let include_dirs = [java_include_dir, java_sys_include_dir];
                println!("jni include dirs {:?}", include_dirs);
                let jni_h_path = search_file_in_directory_recursive(&include_dirs[..], "jni.h")
                    .expect("Can not find jni.h");
                jni_c_headers_rs = Path::new(&out_path).join("jni_c_header.rs");
                gen_java_binding(&target, &include_dirs, &[jni_h_path], &jni_c_headers_rs).unwrap();
            }
            let have_java_9 = fs::read_to_string(jni_c_headers_rs)
            .unwrap()
            .contains("JNI_VERSION_9");
            let java_cfg = JavaConfig::new(java_output_dir, "com.swordfish".into())
                // .use_optional_package(optional_package::JavaOptionalPackage::Guava)
                // .use_null_annotation_from_package("android.support.annotation".into())
                .use_reachability_fence(if have_java_9 {
                    JavaReachabilityFence::Std
                } else {
                    JavaReachabilityFence::GenerateFence(8)
                });

            let swig_gen = flapigen::Generator::new(LanguageConfig::JavaConfig(java_cfg))
                .rustfmt_bindings(true)
                .remove_not_generated_files_from_output_directory(true)
                .merge_type_map("java_type_map", include_str!("src/ffi/java_type_map.rs")) //this needs to be typed in for some reason
                .register_class_attribute_callback("PartialEq", |code, class_name| {
                    let needle = format!("class {} {{", class_name);
                    let class_pos = code
                        .windows(needle.len())
                        .position(|window| window == needle.as_bytes())
                        .expect("Can not find begin of class");
                    let insert_pos = class_pos + needle.len();
                    code.splice(
                        insert_pos..insert_pos,
                        format!(
                            r#"
                        public boolean equals(Object obj) {{
                            boolean equal = false;
                            if (obj instanceof {class})
                            equal = (({class})obj).rustEq(this);
                            return equal;
                        }}
                        public int hashCode() {{
                            return (int)mNativeObj;
                        }}
                        "#,
                            class = class_name
                        )
                        .as_bytes()
                        .iter()
                        .copied(),
                    );
                });
            swig_gen.expand(
                "swordfish_com_java",
                &interface_filepath,
                &Path::new(&out_path).join("glue_java.rs"),
            )
        }
    }
}

//-----------------------------functions copied from flapigen example-----------------------------------
#[cfg(feature = "java_wrapper")]
fn search_file_in_directory_recursive<P: AsRef<Path>>(dirs: &[P], file: &str) -> Result<PathBuf, ()> {
    for dir in dirs {
        let dir = dir.as_ref().to_path_buf();
        let file_path = dir.join(file);
        if file_path.exists() && file_path.is_file() {
            return Ok(file_path);
        }
        for entry in fs::read_dir(&dir)
            .expect(&format!("Can not read directory: {}", &dir.display())) {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                let file_path = search_file_in_directory_recursive(&[path], file);
                if file_path.is_ok() {
                    return file_path;
                }
            }
        }
    }
    Err(())
}

#[cfg(feature = "java_wrapper")]
fn gen_java_binding<P1, P2>(
    target: &str,
    include_dirs: &[P1],
    c_headers: &[P2],
    output_rust: &Path,
) -> Result<(), String>
where
    P1: AsRef<Path> + fmt::Debug,
    P2: AsRef<Path> + fmt::Debug,
{
    assert!(!c_headers.is_empty());
    let c_file_path = &c_headers[0];

    let mut bindings: bindgen::Builder =
        bindgen::builder().header(c_file_path.as_ref().to_str().unwrap());
    bindings = include_dirs.iter().fold(bindings, |acc, x| {
        acc.clang_arg("-I".to_string() + x.as_ref().to_str().unwrap())
    });
    println!("Generate binding for {:?}", c_headers);
    bindings = bindings
        .rust_target(RustTarget::Stable_1_73)
        //long double not supported yet, see https://github.com/servo/rust-bindgen/issues/550
        .blocklist_type("max_align_t");
    bindings = if target.contains("windows") {
        //see https://github.com/servo/rust-bindgen/issues/578
        bindings.trust_clang_mangling(false)
    } else {
        bindings
    };
    bindings = c_headers[1..].iter().fold(
        Ok(bindings),
        |acc: Result<bindgen::Builder, String>, header| {
            let c_file_path = header;
            let c_file_str = c_file_path
                .as_ref()
                .to_str()
                .ok_or_else(|| format!("Invalid unicode in path to {:?}", c_file_path.as_ref()))?;
            Ok(acc.unwrap().clang_arg("-include").clang_arg(c_file_str))
        },
    )?;

    let generated_bindings = bindings
        //        .clang_arg(format!("-target {}", target))
        .generate()
        .map_err(|_| "Failed to generate bindings".to_string())?;
    generated_bindings
        .write_to_file(output_rust)
        .map_err(|err| err.to_string())?;

    Ok(())
}
