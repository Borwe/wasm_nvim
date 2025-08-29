#[cfg(windows)]
use std::{str::FromStr,fs,path::PathBuf, error::Error};
#[cfg(windows)]
use zip;

use std::{collections::HashMap, env, io, process::Command, sync::OnceLock};

static BUILD_INFO: OnceLock<HashMap<String, &str>> = OnceLock::new();


#[cfg(windows)]
const LUAJIT_2_1_LINK: &'static str = "https://github.com/luajit/luajit/archive/51d4c26ec7805d77bfc3470fdf99b73c4ef2faec.zip";

#[cfg(windows)]
const OUT_DIR: &'static str = ".\\target\\debug\\build\\lua_setup";

#[cfg(windows)]
const LUAJIT_DIR: &'static str = "LuaJIT-51d4c26ec7805d77bfc3470fdf99b73c4ef2faec";

#[cfg(windows)]
async fn get_luajit_source()-> Result<(), Box<dyn Error>>{

    let resp = match reqwest::get(LUAJIT_2_1_LINK)
        .await {
        Ok(res) if res.status() == 200 => res.bytes().await.expect("Couldn't get bytes"),
        Ok(res) => panic!("Got response code {}",res.status()),
        Err(e) => panic!("Error downloading: {}", e),
    };
    let target_dir = PathBuf::from_str(OUT_DIR)
        .unwrap().join("luaj.zip");
    std::fs::create_dir_all(PathBuf::from_str(OUT_DIR).unwrap())
        .expect("Couldn't create dir");


    //write bytes to target/dls
    std::fs::write(&target_dir, resp)
        .expect("couldn't write zip to file");


    //excract the file there
    let luaj = fs::File::open(&target_dir)
        .expect("can't open zip file");

    let mut zipped = zip::ZipArchive::new(luaj)
        .expect("can't reference zip from file");

    for i in 0..zipped.len(){
        let mut file = zipped.by_index(i)
            .expect("can't get zip by index");
        let mut path: PathBuf = target_dir.parent()
            .unwrap().to_path_buf();
        path.push(file.name());
        if !file.name().ends_with("/") {
            let mut dest = std::fs::File::create(path)
                .expect("can't create file to extract to");
            std::io::copy(&mut file, &mut dest)?;
        }else {
            fs::create_dir_all(path).expect("can't create folder");
        }
    }

    //compile luajit
    let cl = cc::windows_registry::find_tool(&env::var("TARGET").unwrap(),"cl.exe").expect("failed to find cl.exe");

    let build_cmd = target_dir.parent().unwrap()
        .join(LUAJIT_DIR).join("src").join("msvcbuild.bat");



    let loc = build_cmd.parent().unwrap();
    let mut cmd = Command::new(build_cmd.to_str().unwrap());
    let cmd = cmd.current_dir(&loc);

    for (k,v) in cl.env(){
        cmd.env(k, v);
    }

    eprintln!("CMD: {}", build_cmd.to_str().unwrap());

    let status = cmd.status()?;
    if !status.success() {
        panic!("Command for building didn't run to completion");
    }


    let src_dir = build_cmd.parent().unwrap();

    println!("cargo:rustc-env=LUA_INC={:?}",
             src_dir);
    println!("cargo:rustc-env=LUA_LIB={:?}",
             src_dir);
    println!("cargo:rustc-env=LUA_LIB_NAME=lua51");
    println!("cargo:rustc-env=LUA_LINK=dylib");

    Ok(())
}

fn gen_cmd(args: &[&str])-> io::Result<()> {
    let mut cmd = Command::new(args[0]);
    let extra =&args[1..];
    cmd.args(extra)
        .stdout(io::stdout())
        .stderr(io::stderr())
        .output()?;
    Ok(())
}


fn fill_build_info(){
    BUILD_INFO.get_or_init(||{
        let mut cmds = HashMap::new();
        cmds.insert("build".to_string(), "Build the wasm_nvim library");
        cmds.insert("test".to_string(), "Test wasm_nvim library");
        cmds.insert("build_zig_test".to_string(), "Build zig wasm module library for testing");
        cmds
    });
}

fn show_information(){
    println!("Use the following options bellow with cargo xtask =>");
    for (k,v) in BUILD_INFO.get().unwrap().into_iter(){
        println!("\t{k}: {v}")
    }
}

#[cfg(target_os = "windows")]
async fn build_lua_for_windows(){
    get_luajit_source().await.expect("failed building luajit");

    //setup environments
    {
        let path = std::env::current_dir().unwrap();
        let crdir = path.to_string_lossy();
        env::set_var("LUA_INC", format!("{OUT_DIR}\\LuaJIT-51d4c26ec7805d77bfc3470fdf99b73c4ef2faec\\src"));
        env::set_var("LUA_LIB", format!("{OUT_DIR}\\LuaJIT-51d4c26ec7805d77bfc3470fdf99b73c4ef2faec\\src"));
        env::set_var("LUA_LIB_NAME", "lua51");
    }
}

async fn build(){

    #[cfg(target_os = "windows")] {
        build_lua_for_windows();
    }
    let mut cmd = Command::new("cargo");
    cmd.args(["build","--package","wasm_nvim", /*"-r"*/])
        .stdout(io::stdout()).stderr(io::stderr());
    cmd.output().expect("Failed to build wasm_nvim");
}

fn build_zig_tests(){
    let current_dir = env::current_dir().unwrap();

    env::set_current_dir("./wasm").expect("Couldn't change dir to './wasm'");
    gen_cmd(&["zig","build-exe","tests.zig","-target"
        ,"wasm32-wasi", "-fno-sanitize-c",
        "-fno-entry", "-dynamic", "-rdynamic" ]).expect("Failed to build zig wasm module for testing");

    env::set_current_dir(current_dir).unwrap();
}

async fn test(){
    build().await;
    build_zig_tests();
    r#move();
    gen_cmd(&["nvim","-u","NONE","-l","./default_cfg/testing.lua"])
        .expect("Failed nvim test command")
}

fn r#move(){
    use std::fs;

    fs::create_dir_all("lua").unwrap();
    #[cfg(target_os = "linux")]
    {
        gen_cmd(&["cp","./target/debug/libwasm_nvim.so",
            "./lua/wasm_nvim.so"])
            .expect("Failed to copy ./target/debug/libwasm_nvim.so to ./lua/wasm_nvim.so");
    }
    #[cfg(target_os = "macos")]
    {
        gen_cmd(&["cp","./target/debug/libwasm_nvim.dylib",
            "./lua/wasm_nvim.so"])
            .expect("Failed to copy ./target/debug/libwasm_nvim.so to ./lua/wasm_nvim.so");
    }

    #[cfg(target_os = "windows")]
    {
        gen_cmd(&["cp",".\\target\\debug\\wasm_nvim.dll",
            ".\\lua\\wasm_nvim.dll"])
            .expect("Failed to copy .\\target\\debug\\wasm_nvim.dll to .\\lua\\wasm_nvim.dll");
    }
}

#[tokio::main]
async fn main() {
    fill_build_info();
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        show_information();
        return;
    }

    match BUILD_INFO.get().unwrap().contains_key(args.get(1).unwrap()) {
        true => match args.get(1){
            Some(x) if *x == "build" => build().await,
            Some(x) if *x == "test" => test().await,
            Some(x) if *x == "build_zig_test" => build_zig_tests(),
            _ => show_information()
        }
        _ => show_information()
    }
}
