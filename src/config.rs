
// to serde ref: https://serde.rs/derive.html
// https://serde.rs/attr-default.html
use serde::{Deserialize};


#[derive(Debug, Clone, Default,Deserialize)]
pub struct Config {
    pub max_follow_depth: u32, // 最大 ac 跟随深度
    pub merge_interval: u64,
    pub ac_file: String,

    #[serde(default)]
    pub db_uri: String,

    pub symbol_file : String,
    #[serde(rename = "zmq_pos_sub_addr")]
    pub zmq_pos_sub_addr: String,         // 仓位订阅
    pub zmq_pos_merged_pub_addr: String, // 聚合发布
}
