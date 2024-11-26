//在单处理器环境中提供了一种方式来安全地共享和修改数据
pub struct UPSafeCell<T> {
  //内部数据
  inner: RefCell<T>,
}
//为 UPSafeCell 实现了 Sync trait。Sync 表示类型可以安全地在多个线程之间共享，没有数据竞争的风险。
//这里的 unsafe 表示这个实现是有条件的，即只有当 T 的类型保证在单处理器环境中使用时，这个 Sync 实现才是安全的。
unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T> {
  pub unsafe fn new(value: T) -> Self{
    Self { inner: RefCell::new(value) }
  }
  //得到它包裹的数据的独占访问权，如果数据已经被借走，则panic
  //访问数据时（无论读还是写），需要首先调用 exclusive_access 获得数据的可变借用标记
  pub fn exclusive_access(&self) -> RefMut<'_, T> {
    self.inner.borrow_mut()
  }
}