use riscv::register::time;
use crate::config::CLOCK_FREQ;
use crate::sbi::set_timer;

const TICKS_PER_SEC: usize = 100;
const MICRO_PER_SEC: usize = 1_000_000;

pub fn get_time() -> usize {
  time::read()
}

///下一次中断的时间--10ms 之后一个 S 特权级时钟中断就会被触发
pub fn set_next_trigger() {
  set_timer(get_time()+CLOCK_FREQ / TICKS_PER_SEC);
}

///统计一个应用的运行时长--以微秒为单位返回当前计数器的值
pub fn get_time_ms() -> usize {
  time::read() /(CLOCK_FREQ / MICRO_PER_SEC)
}