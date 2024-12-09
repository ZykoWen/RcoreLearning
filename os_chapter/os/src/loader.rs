
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
    core::slice::from_raw_parts(app_start[app_id] as *const u8, app_start[app_id + 1] - app_start[app_id]
    )
  }
}


///得到应用的个数
pub fn get_num_app() -> usize {
  extern "C" {
    fn _num_app();
  }
  unsafe{ (_num_app as usize as *const usize).read_volatile()}
}
