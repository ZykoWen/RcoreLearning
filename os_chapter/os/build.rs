//! 生成 link_app.S 将应用作为一个数据段链接到内核

use std::fs::{read_dir, File};
use std::io::{Result, Write};

fn main() {
  //告诉 Cargo 当指定目录下的文件发生变化时，需要重新运行构建脚本。
    println!("cargo:rerun-if-changed=../user/src/");
    println!("cargo:rerun-if-changed={}", TARGET_PATH);
    insert_app_data().unwrap();
}

///静态字符串，指向目标应用程序的路径
static TARGET_PATH: &str = "../user/target/riscv64gc-unknown-none-elf/release/";

///将一系列的应用程序的elf数据插入到一个汇编文件中
fn insert_app_data() -> Result<()> {
    //尝试创建一个名为 link_app.S 的新文件，用于写入汇编代码。
    let mut f = File::create("src/link_app.S").unwrap();
    //读取 ../user/src/bin 目录下的所有文件，并创建一个 apps 向量来存储这些文件名
    let mut apps: Vec<_> = read_dir("../user/src/bin")
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();
    //对 apps 向量进行排序
    apps.sort();
    //向文件 f 写入汇编代码，定义了一个名为 _num_app 的全局变量，其值为 apps 向量的长度。
    writeln!(
        f,
        r#"
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad {}"#,
        apps.len()
    )?;
    //循环遍历 apps 向量，定义每个应用程序的起始和结束标签。
    for i in 0..apps.len() {
        writeln!(f, r#"    .quad app_{}_start"#, i)?;
    }
    writeln!(f, r#"    .quad app_{}_end"#, apps.len() - 1)?;
    //再次循环遍历 apps 向量，这次是为了将每个应用程序的elf文件嵌入到汇编文件中。
    for (idx, app) in apps.iter().enumerate() {
        println!("app_{}: {}", idx, app);
        writeln!(
            f,
            r#"
    .section .data
    .global app_{0}_start
    .global app_{0}_end
app_{0}_start:
    .incbin "{2}{1}"
app_{0}_end:"#,
            idx, app, TARGET_PATH
        )?;
    }
    Ok(())
}