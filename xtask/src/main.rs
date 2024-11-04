use std::{collections::HashMap, env, io, process::Command, sync::OnceLock};

static BUILD_INFO: OnceLock<HashMap<String, &str>> = OnceLock::new();

fn fill_build_info(){
    BUILD_INFO.get_or_init(||{
        let mut cmds = HashMap::new();
        cmds.insert("build".to_string(), "Build the wasm_nvim library");
        cmds.insert("test".to_string(), "Test wasm_nvim library");
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
    cmd.args(["build","--package","wasm_nvim", "-r"])
        .stdout(io::stdout()).stderr(io::stderr());
    cmd.output().unwrap();
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
            Some(x) => println!("GOT: {x}"),
            None => show_information()
        }
        _ => show_information()
    }
}
