use anyhow::Result;
use std::env;
#[cfg(windows)]
use std::str::FromStr;
#[cfg(windows)]
use std::fs;
#[cfg(windows)]
use zip;
#[cfg(windows)]
use std::path::PathBuf;
#[cfg(windows)]
use std::process::Command;

#[cfg(windows)]
const LUAJIT_2_1_LINK: &'static str = "https://github.com/luajit/luajit/archive/51d4c26ec7805d77bfc3470fdf99b73c4ef2faec.zip";

#[cfg(windows)]
const LUAJIT_DIR: &'static str = "LuaJIT-51d4c26ec7805d77bfc3470fdf99b73c4ef2faec";

#[cfg(windows)]
async fn get_luajit_source()-> Result<()>{

    let resp = match reqwest::get(LUAJIT_2_1_LINK)
        .await {
        Ok(res) if res.status() == 200 => res.bytes().await.expect("Couldn't get bytes"),
        Ok(res) => panic!("Got response code {}",res.status()),
        Err(e) => panic!("Error downloading: {}", e),
    };
    let target_dir = PathBuf::from_str(
        &env::var("OUT_DIR").expect("Cargo out_dir not found"))?
        .parent().unwrap().parent().unwrap().join("luaj.zip");


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
    println!("cargo:rustc-env=LUA_LIB_NAME=lua");

    Ok(())
}


#[cfg(unix)]
async fn get_luajit_source()-> Result<()>{
    Ok(())
}

#[tokio::main]
async fn main()-> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    get_luajit_source().await?;
    Ok(())
}
