
#[derive(Debug,Clone,Default)]
pub struct Config{
    pub max_follow_depth :u32 , // 最大 ac 跟随深度
    pub merge_interval : u64,
}