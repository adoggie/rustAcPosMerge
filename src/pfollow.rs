#![allow(dead_code)]
#![allow(unused_variables)]

use crate::config;
use ndarray::s;
use ndarray::{Array, Array2, Axis, Ix2};
use std::borrow::{Borrow, BorrowMut};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead, Result};
use std::ops::Deref;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use serde_json::{Value,from_reader};
use serde::Deserialize;
use log::{error, warn, info, debug, trace};

const SYMBOL_ID_ALL: usize = 0;
// type struct PositionInfo(st)
// ac 配置项
#[derive(Debug, Clone, Default)]
pub struct AcEntry {
    pub ac: String,
    pub fac: String,      // be followed
    pub ratio: f64,       //
    pub symbol: SymbolId, // contract or product
}

type AcIndex = usize;
type AcEntryId = String;    // entry hash id

impl AcEntry {
    ///
    pub fn hash(&self) -> AcEntryId {
        format!("{}_{}_{}", self.ac, self.fac, self.ratio)
    }

    fn from_str(text: &str) -> Option<Self> {
        // format:  ac,fac,ratio,symbol
        let mut en = AcEntry::default();
        let split_strings: Vec<&str> = text.split(',').map(|s| s.trim()).collect();
        if split_strings.len() == 4 {
            let c = split_strings[0usize].chars().nth(0).unwrap();
            if c == '#' {
                return None;
            }
            en.ac = split_strings[0].to_string();
            en.fac = split_strings[1].to_string();
            match split_strings[2].to_string().parse() {
                Ok(num) => en.ratio = num,
                Err(err) => {
                    println!("Failed to parse: {:?}", text);
                    return None;
                }
            }
            // let s = split_strings[3]
            en.symbol = split_strings[3].parse::<usize>().unwrap();
            return Some(en);
        }
        None
    }
}

type AcEntryPtr = Box<AcEntry>;


/// AcRatioTable
///     M1
///   A    B    M    w    t
/// A 0    0    0    0    0
/// B 0.1  0    0    0    0
/// M 1.0  0    0    0    0
/// w 0.5  1.0  0    0    0
/// t 1.0  0    0.5  0    0
/// 
/// x - ac , y - fac  , w,t - primitive ac
/// 
/// A->B(0.1*1.0) + M(1*.05) +  w(0.5) + t(1)
/// 
/// 
#[derive(Debug, Default)]
pub struct AcRatioTable {
    matrix: Array2<f64>,
}

/// AcRatioTable 策略配比表
impl Deref for AcRatioTable {
    type Target = Array2<f64>;

    fn deref(&self) -> &Self::Target {
        &self.matrix
    }
}

impl AcRatioTable {
    fn new(n: usize, m: usize) -> Self {
        let matrix: Array2<f64> = Array::from_elem((n, m), f64::NAN);
        AcRatioTable { matrix }
    }
    fn get_matrix_mut(&mut self) -> &mut Array2<f64> {
        &mut self.matrix
    }

    fn get_matrix(&self) -> &Array2<f64> {
        &self.matrix
    }
}

pub type Symbol = String;
pub type SymbolId = usize;

/// AcRatioTableSet 策略配比表集合
/// 一个品种对应一个配比表
/// 
#[derive(Default, Debug)]
pub struct AcRatioTableSet {
    tables: HashMap<SymbolId, AcRatioTable>,    // 每个品种有独立的配置表
    entries: HashMap<AcEntryId, AcEntry>,       // 配置项 与ac名称
    ac_indexes: HashMap<String, usize>,         // ac 名称与索引
    ac_index_reversed: HashMap<usize, String>,  // ac 索引与名称
    ac_index_dispatch: Vec<usize>,
    // facIndexes: HashMap<String, usize>,
    symbol_indexes: HashMap<String, usize>,     // 品种名称与索引
    symbol_index_reversed: HashMap<usize, String>,  // 品种索引与名称
    // aclist : Vec<String>,
    // faclist : Vec<String>,
    // acIndexes :
}

struct DBConn {}

#[test]
fn test_acRatioTableSet() {
    // a -> (b,c,d)
    // b-> x1
    // c-> (x1,b)
    // d-> (c,x2)
    //
}


impl AcRatioTableSet {

    /// 从文件加载
    pub fn from_file(file_name: &str) -> Result<Self> {
        let mut ts = AcRatioTableSet::default();
        for line in io::BufReader::new(File::open(file_name)?).lines() {
            if let Some(en) = AcEntry::from_str(line.unwrap().as_str()) {
                ts.add_entry(en);
            }
        }
        Ok(ts)
    }

    /// 从数据库加载
    fn from_db(conn: &DBConn) -> Result<Self> {
        let ts = AcRatioTableSet::default();
        Ok(ts)
    }

    pub fn new() -> Self {
        let mut ts = AcRatioTableSet::default();
        // let cols =  0usize;
        // let rows = 0usize;
        // for id in &SYMTABLE.all_ids(){
        //     ts.tables.insert( id.to_owned() , AcRatioTable::new(cols,rows));
        // }
        ts
    }

    // 加载 pfollow 配置项
    pub fn add_entry(&mut self, en: AcEntry) -> &Self {
        if let Some(ac) = self.entries.get_mut(&en.hash()) {
            ac.ratio += en.ratio;
            return self;
        }else {            
            self.ac_indexes.insert(en.ac.clone(), 0); 
            // en will be moved into self.entries , so must use .clone() to copy
            self.ac_indexes.insert(en.fac.clone(), 0);
            self.entries.insert(en.hash(), en);    
        }
        self
    }

    /// return: (symbols , rows ,cols)
    pub fn get_dim(&self) -> (usize, usize, usize) {
        let cols = self.ac_indexes.len();
        let rows = self.ac_indexes.len();
        (SYMTABLE.lock().unwrap().all_ids().len(), cols, rows)
        // symbols , rows ,cols
    }

    pub fn complete(&mut self) -> &Self {
        let cols = self.ac_indexes.len();
        let rows = self.ac_indexes.len();
        
        // allocate ratio tables
        for id in SYMTABLE.lock().unwrap().all_ids() {
            let table = AcRatioTable::new(rows, cols);
            self.tables.insert(id, table);
        }

        // fill ac ratios throughout all  tables
        for en in self.entries.values() {
            // list all symbol table
            let mut symbols = Vec::new(); // all symbols
            if en.symbol == SYMBOL_ID_ALL {
                symbols = SYMTABLE.lock().unwrap().all_ids().clone();
            } else {                
                symbols.push(en.symbol);
            }
            
            // for each symbol , fill ac ratios in tables[symbol]
            for sym in symbols {
                if let Some(row) = self.get_ac_index(&en.ac) {
                    if let Some(col) = self.get_ac_index(&en.fac) {                                                        
                        if let Some(table)  = self.tables.get_mut(&sym){                                
                            table.matrix[(row,col)] = en.ratio.to_owned();
                        }
                    }
                }
            }               
        }
        self
    }

    fn reset(&mut self) -> &Self {
        return self;
    }

    pub fn build_index(&mut self) -> &mut Self {
        let mut temp_indexes = HashMap::new();
        let mut temp_index_reversed = HashMap::new();
        for (n, v) in self.ac_indexes.keys().enumerate() {
            temp_indexes.insert(v.to_string(), n as usize);
            temp_index_reversed.insert(n as usize, v.to_string());
        }
        self.ac_indexes = temp_indexes;
        self.ac_index_reversed = temp_index_reversed;

        // let mut temp_indexes = HashMap::new();
        // for (n,v ) in self.facIndexes.keys().enumerate(){
        //     temp_indexes.insert(v.to_string(), n as usize);
        // }
        // self.facIndexes = temp_indexes;
        self
    }

    // ac 的访问索引
    fn get_ac_index(&self, ac: &str) -> Option<AcIndex> {
        self.ac_indexes.get(ac).copied()
    }

    fn get_ac_name(&self, index: usize) -> String {
        "".to_string()
    }

    fn get_symbol_index(&self, symbol: &str) -> usize {
        0
    }

    fn get_symbol_name(&self, index: usize) -> String {
        "".to_string()
    }
}
impl Deref for AcRatioTableSet {
    type Target = HashMap<SymbolId, AcRatioTable>;

    fn deref(&self) -> &Self::Target {
        &self.tables
    }
}

pub fn test_actable() {
    // let act = AcRatioTable::from_file("abc");
}

// #[macro_use]
// extern crate lazy_static;

use crate::config::Config;
use crate::message::MessagePosition;
use lazy_static::lazy_static;

lazy_static! {
    // static ref SYMTABLE : SymbolTable = SymbolTable(HashMap::new());
    static  ref SYMTABLE:Arc<Mutex<SymbolTable>> = Arc::new(Mutex::new(SymbolTable(HashMap::new())));
    // AcRatioTableSet
    static ref ACRATIO_TABSET : AcRatioTableSet = AcRatioTableSet::new();
    // static ref Instance : Box<Runner> = Runner::new();
}
#[derive(Debug, Default)]
pub struct SymbolTable(HashMap<String, SymbolId>);

#[test]
fn test_symtable(){
    match SymbolTable::init_from_file("samples/symbols.json") {
        Ok(st) => {
            println!("{:?}",SYMTABLE.lock().unwrap());
            // println!("{}",SYMTABLE.lock().unwrap().get_id("P"));
            assert_eq!(SYMTABLE.lock().unwrap().get_id("P"),2);
        },
        Err(err) => {
            println!("{:?}",err);
        }
    }
}

impl SymbolTable {
    pub fn init_from_file(file_name: &str) -> Result<()> {
        // let mut st = SymbolTable(HashMap::new());
        let file = File::open(file_name)?;
        let data :HashMap<String, SymbolId>= serde_json::from_reader(&file)?;
        SYMTABLE.lock().unwrap().0 = data;        
        Ok(())
    }

    fn get_id(&self, name: &str) -> SymbolId {
        let id = self.0.get(name);
        id.unwrap().clone()
    }

    fn add_one(&mut self, name: &str, id: usize) {
        self.0.insert(name.to_string(), id);
    }

    /// 获得所有产品 ids 
    fn all_ids(&self) -> Vec<SymbolId> {
        // let ids = self.0.iter().map(|(k,v)| v).collect::<Vec<u32>>();
        // ids
        // let array = [1, 2, 3, 2, 4, 3, 5, 1];
        // let unique_values: Vec<_> = self.0.iter().cloned().collect::<HashSet<SymbolId>>().into_iter().collect();
        // unique_values
        // self.0.values().cloned().collect::<Vec<SymbolId>>()
        let res = self.0.values().cloned().collect();
        res
    }

    fn name_by_index(&self, id: SymbolId) -> String {
        "".to_string()
    }
}

// 合约仓位表 M2
// x - fac
// y - symbol

/////////////////////////
//  a , b, c  策略或账号
//      a   b   c  x1  x2
//  A   0   0   0  2.1  1.5
//  M
//  P
// =================
//     A   M   P
//  a  0   0   0
//  b  0   0   0
//  c  0   0   0
//  x1 0   0   0
//  x2 0   0   0
// 

#[derive(Debug, Default)]
pub struct AcPosTable {
    matrix: Array2<f64>,
}

impl AcPosTable {
    fn new(n: usize, m: usize) -> Self {
        // AcPosTable
        let matrix: Array2<f64> = Array::from_elem((n, m), f64::NAN);
        AcPosTable { matrix }
    }

    fn put(&mut self, symbol: SymbolId, ac: AcIndex, ps: f64) -> &Self {
        let r = symbol as usize;
        let c = ac as usize;
        self.matrix[(r, c)] = ps;
        self
    }

    fn get_matrix_mut(&mut self) -> &mut Array2<f64> {
        &mut self.matrix
    }
}

#[derive(Debug, Default)]
pub struct Runner {
    config: Config,
    // posTables: HashMap<SymbolId, AcPosTable>,   // 仓位表
    posTable: Arc<Mutex<AcPosTable>>,
    // ratioTable : AcRatioTable,
    ratioTs: Arc<Mutex<AcRatioTableSet>>,
    // ratioTs: AcRatioTableSet,
    // ch_pos_tx: Option<mpsc::Sender<crate::message::MessagePosition>>,
    ch_pos_tx: Option<mpsc::Sender<crate::message::MessagePosition>>,
    ch_pos_rx: Option< Arc<Mutex<mpsc::Receiver<crate::message::MessagePosition>>> >,
}

impl Runner {

    // 加载最近的仓位
    fn load_pos() {}

    pub fn get_config(&self) -> &Config {
        &self.config
    }

    fn init_ratio_table(&mut self) -> Result<()> {
        
        if !self.config.ac_file.trim().is_empty(){            
            self.ratioTs.lock().unwrap().reset();
            // l.as_ref().unwrap().trim().is_empty() == false  因为 l 是 ref ,
            // 不能直接unwrap内容，会破坏原来数据，转换成 ref 后，可以使用 as_ref() 转换成 &str
            for line in io::BufReader::new(File::open(&self.config.ac_file)?).lines()
                .filter(|l| l.as_ref().unwrap().trim().is_empty() == false) {
                if let Some(en) = AcEntry::from_str(line.unwrap().as_str()) {
                    self.ratioTs.lock().unwrap().add_entry(en);
                }
            }
        }

        if !self.config.db_uri.trim().is_empty() {
            self.ratioTs.lock().unwrap().reset();
            // let conn = DBConn{};
            // self.ratioTs = AcRatioTableSet::from_db(&conn)?;
        } 
        self.ratioTs.lock().unwrap().build_index().complete(); //         
        Ok(())
    }

    fn init_pos_table(&mut self) -> Result<()> {
        let (symbols, rows, cols) = self.ratioTs.lock().unwrap().get_dim();
        // self.posTable = AcPosTable::new(symbols,cols);
        let matrix: Array2<f64> = Array::from_elem(( rows,symbols), f64::NAN);
        self.posTable = Arc::new(Mutex::new(AcPosTable { matrix: matrix }));

        Ok(())
    }

    pub fn instance() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Runner::default()))
    }

    pub fn new( filename :&str) ->Result< Self> {
        let mut runner =  Runner::default();
        // let config = Config::default();
        let file = File::open(filename)?;
        // let data :HashMap<String, SymbolId>= serde_json::from_reader(&file)?;
        // let mut de  = serde_json::Deserializer::from_reader(file);
        // let cfg = Config::deserialize(&mut de)?;
        let cfg:Config = serde_json::from_reader(file)?;
        
        runner.config = cfg;
        Ok(runner)
        // let ts = AcRatioTableSet::new();
        // let runner = Runner{config:Config::default(),}
    }

    pub fn get_ratioTs(&mut self) -> Arc<Mutex<AcRatioTableSet>> {
        //  let mut data = self.ratioTs.lock().unwrap();
        //  &mut data.borrow_mut()
        self.ratioTs.clone()

    }

    pub fn get_posTable(&mut self) -> &mut Arc<Mutex<AcPosTable>> {
        &mut self.posTable
    }

    fn init_mx(&mut self) -> Result<()> {
        Ok(())
    }

    // 仓位信号到达
    pub fn put_pos(&mut self, ps: crate::message::MessagePosition) -> &Self {
        // let mut postab =  mtxpos.lock().unwrap();
        let _ = self.ch_pos_tx.as_mut().unwrap().send(ps);
        self
    }

    // pub fn run(&mut self, self_arc: Arc<Mutex<Self>>) -> Result<()> {
    pub fn run(&mut self) -> Result<()> {
        let (sender, receiver) = mpsc::channel();
        self.ch_pos_tx = Some(sender);
        // self.ch_pos_rx = Some(receiver);
        self.init_ratio_table()?;
        self.init_pos_table()?;
        self.init_mx()?;
        
        let mtxpos = Arc::clone(&self.posTable);
        let mtx_ratiots = Arc::clone(&self.ratioTs);
        let ch_pos_rx = receiver;

        // let self_ref = Arc::clone(&self_arc);
        let thr = thread::spawn(move || {
            for m in ch_pos_rx.iter() {
                debug!("MessagePos: {} {} {}", m.ac, m.symbol, m.ps);
                let mut postab = mtxpos.lock().unwrap();
                let x = mtx_ratiots.lock().unwrap().get_ac_index(&m.ac).unwrap().to_owned();
                postab.put(SYMTABLE.lock().unwrap().get_id(m.symbol.as_str()), 0, m.ps);
            }
        });

        loop {
            thread::sleep(Duration::from_secs( self.config.merge_interval.to_owned() as u64
            ));
            let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            debug!("pos_merge sleep..");
            self.pos_merge_and_dispatch();
        }

        // thr.join().unwrap();
        Ok(())
    }


    //  max_follow_depth( posTable * ratioTable => resultTable )
    //  resultTable -> delivery zmq
    fn pos_merge_and_dispatch(&mut self) -> &Self {
        // 仓位 x 配比 ， 循环最大配比深度次数，最后得到最新的仓位
        let mut m1: Array2<f64>; // 临时的仓位表 
        {
            let mtxpos = Arc::clone(&self.posTable);
            let mut postab = mtxpos.lock().unwrap();
            m1 = postab.get_matrix_mut().clone();
        }
        for (symbol,table) in self.ratioTs.lock().unwrap().tables.iter(){
            let   m2 = table.get_matrix(); //.clone();
            let mut m3 = m2.dot(&m1);
            for _ in 0..self.config.max_follow_depth {
                m3 = m3.dot(&m1);
            }
            self.dispatch_pos(m3);
        }
      
        // 开始分发
        // 仅仅分发 ac 项目
        for acidx in self.ratioTs.lock().unwrap().ac_index_dispatch.iter() {}
        self
    }

    // 分发仓位  acTable
    fn dispatch_pos(& self, matrix: Array2<f64>) -> &Self {
        for (index, mut row) in matrix.axis_iter(Axis(0)).enumerate() {
            let symbol: String = SYMTABLE.lock().unwrap().name_by_index(index);
            for index in self.ratioTs.lock().unwrap().ac_index_dispatch.iter() {
                // let ps = row[index];
                let ps = row.get(index.to_owned()).unwrap().to_owned();
                println!("{}", ps);
                let ac = self.get_acname_by_index(index.to_owned());

                self.zmq_send(&ac, &symbol, ps);
            }
        }
        self
    }

    fn zmq_send(&self, ac: &str, symbol: &str, ps: f64) -> &Self {
        self
    }

    fn data_persist(&self, ac: &str, symbol: &str, ps: f64) -> &Self {
        self
    }

    fn get_acname_by_index(&self, index: usize) -> String {
        "".to_string()
    }

    // static  symbol_table:SymbolTable = SymbolTable;
}
#[test]
fn test_one() {
    // let st = SymboTable{HashMap::new()};
    // println!("-- {:?}",st.all_ids());
}
