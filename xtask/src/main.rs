use std::{collections::HashMap, env, io, process::Command, sync::OnceLock};

static BUILD_INFO: OnceLock<HashMap<String, &str>> = OnceLock::new();

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

fn build(){
    #[cfg(target_os = "windows")]
    {
        let path = std::env::current_dir().unwrap();
        let crdir = path.to_string_lossy();
        env::set_var("LUA_INC", format!("{crdir}\\release\\build\\LuaJIT-2.1.0-beta3\\src"));
        env::set_var("LUA_LIB", format!("{crdir}\\release\\build\\LuaJIT-2.1.0-beta3\\src"));
        env::set_var("LUA_LIB_NAME", "lua51");
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

fn test(){
    build();
    build_zig_tests();
    r#move();
    gen_cmd(&["nvim","-u","NONE","-l","./default_cfg/testing.lua"])
        .expect("Failed nvim test command")
}

fn r#move(){
    #[cfg(target_os = "linux")]
    {
        use std::fs;

        fs::create_dir_all("lua").unwrap();
        gen_cmd(&["cp","./target/debug/libwasm_nvim.so",
            "./lua/wasm_nvim.so"])
            .expect("Failed to move ./target/release/libwasm_nvim.so to ./lua/wasm_nvim.so");
    }
}

fn main() {
    fill_build_info();
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        show_information();
        return;
    }

    match BUILD_INFO.get().unwrap().contains_key(args.get(1).unwrap()) {
        true => match args.get(1){
            Some(x) if *x == "build" => build(),
            Some(x) if *x == "test" => test(),
            Some(x) if *x == "build_zig_test" => build_zig_tests(),
            _ => show_information()
        }
        _ => show_information()
    }
}
