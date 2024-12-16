use lazy_static::*;
use alloc::vec::Vec;

lazy_static!{ 
  //APP_NAMES按照顺序将所有应用的名字保存在内存
  static ref APP_NAMES: Vec<&'static str> = {
    let num_app = get_num_app();
    extern "C" {fn _app_names();}
    let mut start = _app_names as usize as *const u8;
    let mut v = Vec::new();
    unsafe {
      for _ in 0..num_app {
        let mut end = start;
        while end.read_volatile() != '\0' as u8 {
          end = end.add(1);
        }
        let slice = core::slice::from_raw_parts(start, end as usize - start as usize);
        let str = core::str::from_utf8(slice).unwrap();
        v.push(str);
        start = end.add(1);
      }
    }
    v
  };
}
///根据传入的应用编号取出对应应用的 ELF 格式可执行文件数据
pub fn get_app_data(app_id: usize) -> &'static [u8] {
  extern "C" {
     fn _num_app();
  }
  let num_app_ptr = _num_app as usize as *const usize;
  let num_app = get_num_app();
  //存储每个app的起始地址
  let app_start = unsafe{
    core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1)
  };
  assert!(app_id < num_app);
  unsafe {
    core::slice::from_raw_parts(
      app_start[app_id] as *const u8, 
      app_start[app_id + 1] - app_start[app_id]
    )
  }
}

///按照应用的名字来查找获得应用的 ELF 数据
pub fn get_app_data_by_name(name: &str) -> Option<&'static [u8]> {
  let num_app = get_num_app();
  (0..num_app)
    .find(|&i| APP_NAMES[i] == name)
    .map(|i| get_app_data(i))
}

///得到应用的个数
pub fn get_num_app() -> usize {
  extern "C" {
    fn _num_app();
  }
  unsafe{ (_num_app as usize as *const usize).read_volatile()}
}

///打印出所有可用的应用的名字
pub fn list_apps() {
  println!("/**** APPS ****");
  for app in APP_NAMES.iter() {
    println!("{}", app);
  }
  println!("**************/");
}