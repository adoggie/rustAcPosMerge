use crate::config;
use std::borrow::Borrow;
use std::collections::{HashMap,HashSet};
use std::ops::Deref;
use ndarray::{Array2, Array, Axis,Ix2};
use std::fs::File;
use std::io::{self, BufRead,Result};
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const SYMBOL_ID_ALL:usize = 0 ;
// type struct PositionInfo(st)
// ac 配置项
#[derive(Debug,Clone,Default)]
pub struct AcEntry{
    pub ac:String,
    pub fac:String,     // be followed
    pub ratio:f64,      //
    pub symbol:String,  // contract or product
}

type AcIndex = usize;
type AcEntryId = String ;

impl AcEntry{
    pub fn hash(&self)->AcEntryId{
        format!("{}_{}_{}",self.ac,self.fac,self.ratio)
    }

    fn from_str( text :&str)->Option<Self>{
        // format:  ac,fac,ratio,symbol
        let mut en = AcEntry::default();
        let split_strings: Vec<&str> = text.split(',').map(|s| s.trim()).collect();
        if split_strings.len() == 4{
            if split_strings[0][0] == '#'{
                return None
            }
            en.ac = split_strings[0].to_string();
            en.fac = split_strings[1].to_string();
            match split_strings[2].to_string().parse(){
                Ok(num) => en.ratio = num ,
                Err(err) => {
                    println!("Failed to parse: {:?}",text);
                    return None;
                }
            }
            en.symbol = split_strings[3].to_string();
            Some(en)
        }
        None
    }
}

type AcEntryPtr = Box<AcEntry>;

pub struct AcRatioTable{
    matrix : Array2<f64>,
}

/// AcRatioTable 策略配比表
impl Deref for AcRatioTable{
    type Target = Array2<f64>;

    fn deref(&self) -> &Self::Target {
        &self.matrix
    }
}

impl AcRatioTable{
    fn new( n:usize ,m:usize )->Self{
        let matrix: Array2<f64> = Array::from_elem((n, m), f64::NAN);
        AcRatioTable { matrix }
    }
    fn get_matrix_mut(&mut self)->&mut Array2<f64>{
        &mut self.matrix
    }

}

pub type Symbol = String ;
pub type SymbolId = usize;

#[derive(Default)]
pub struct AcRatioTableSet{
    tables : HashMap<SymbolId,AcRatioTable>,
    entries: HashMap< AcEntryId, AcEntry >,
    acIndexes: HashMap< String , usize>,
    acIndexReversed: HashMap< usize,String>,
    acInxDispatch : Vec<usize>,
    // facIndexes: HashMap<String, usize>,
    symbolIndexes: HashMap<String,usize>,
    symbolIndexreversed : HashMap<usize,String>,
    // aclist : Vec<String>,
    // faclist : Vec<String>,
    // acIndexes :
}

struct DBConn{

}

#[test]
fn test_acRatioTableSet(){
    /// a -> (b,c,d)
    /// b-> x1
    /// c-> (x1,b)
    /// d-> (c,x2)
    ///

}

impl AcRatioTableSet{
    pub fn from_file(file_name :&str)->Result<Self>{
        let mut ts = AcRatioTableSet::default();
        for line in io::BufReader::new(File::open(file_name)?).lines() {
            if let Some(en) = AcEntry::from_str(line.unwrap()?){
                ts.add_entry(en);
            }
        }
        Ok(ts)
    }

    fn from_db( conn:&DBConn)->Result<Self>{
        let ts = AcRatioTableSet::default();
        Ok(ts)
    }


    pub fn new()->Self{
        let mut ts = AcRatioTableSet::default();
        // let cols =  0usize;
        // let rows = 0usize;
        // for id in &SYMTABLE.all_ids(){
        //     ts.tables.insert( id.to_owned() , AcRatioTable::new(cols,rows));
        // }
        ts
    }

    // 加载 pfollow 配置项
    pub fn add_entry(&mut self,en:AcEntry)->&Self{
        if self.entries.contains_key(&en.hash()){
            // ac+fac+symbol 合并条目
            let  mut  ac = self.entries.get_mut(&en.hash()).unwrap();
            ac.ratio += en.ratio;
        }else{
            self.acIndexes.insert( en.ac.clone(),0 );
            self.acIndexes.insert( en.fac.clone(),0);
            self.entries.insert(en.hash(),en);
        }
        self
    }

    pub fn get_dim(&self) -> (usize,usize,usize){
        let cols = self.acIndexes.len();
        let rows = self.acIndexes.len();
        (SYMTABLE.all_ids().len(),cols,rows)
        // symbols , rows ,cols
    }

    pub fn complete(&mut self) ->&Self{
        let cols = self.acIndexes.len();
        let rows = self.acIndexes.len();

        for id in SYMTABLE.all_ids(){
            let table = AcRatioTable::new(rows,cols);
            self.tables[id] = table;
        }

        for en in self.entries.values(){
            // list all symbol table
            let mut symbols = Vec::new();
            if en.symbol == SYMBOL_ID_ALL{
                symbols = SYMTABLE.all_ids().clone();
            }else{
                symbols.push( SYMTABLE.get_id(en.symbol.into()));
            }
            for sym in symbols{
                let table  = self.tables.get(&sym).as_mut().unwrap();
                (*table)[( self.get_ac_index( &en.ac ),
                        self.get_ac_index(&en.fac))] = en.ratio;
            }
        }
        self
    }

    fn reset(&mut self) ->&Self{
        return self;
    }

    pub fn build_index(&mut self)->&Self{
        let mut temp_indexes = HashMap::new();
        let mut temp_index_reversed = HashMap::new();
        for (n,v ) in self.acIndexes.keys().enumerate(){
            temp_indexes.insert(v.to_string(), n as usize);
            temp_index_reversed.insert(n as usize,v.to_string());
        }
        self.acIndexes = temp_indexes;
        self.acIndexReversed = temp_index_reversed;

        // let mut temp_indexes = HashMap::new();
        // for (n,v ) in self.facIndexes.keys().enumerate(){
        //     temp_indexes.insert(v.to_string(), n as usize);
        // }
        // self.facIndexes = temp_indexes;
        self
    }

    // ac 的访问索引
    fn get_ac_index( &self,ac:&str)->AcIndex{
        self.acIndexes.get(ac).unwrap_or_else({println!("ac index not found:{} ",ac)}).to_owned() as AcIndex
    }

    fn get_ac_name(&self,index :usize)->String{
        "".to_string()
    }

    fn get_symbol_index(&self,symbol:&str) ->usize{
        0
    }

    fn get_symbol_name(&self,index:usize) ->String{
        "".to_string()
    }

}
impl Deref for AcRatioTableSet{
    type Target = HashMap<SymbolId,AcRatioTable>;

    fn deref(&self) -> &Self::Target {
        &self.tables
    }
}

pub fn test_actable(){
    // let act = AcRatioTable::from_file("abc");
}

// #[macro_use]
// extern crate lazy_static;

use lazy_static::lazy_static;
use crate::config::Config;
use crate::message::MessagePosition;

lazy_static! {
    static ref SYMTABLE : SymbolTable = SymbolTable(HashMap::new());
    // AcRatioTableSet
    static ref ACRATIO_TABSET : AcRatioTableSet = AcRatioTableSet::new();
}


pub struct SymbolTable( HashMap<String,SymbolId>);

impl SymbolTable{
    fn get_id(&self, name:&str)->SymbolId {
        let id = self.0.get(name);
        id.unwrap().clone()
    }

    fn add_one(&mut self, name :&str, id :usize){
        self.0.insert(name.to_string(),id);
    }

    fn all_ids(&self)-> Vec<SymbolId>{
        // let ids = self.0.iter().map(|(k,v)| v).collect::<Vec<u32>>();
        // ids
        // let array = [1, 2, 3, 2, 4, 3, 5, 1];
        // let unique_values: Vec<_> = self.0.iter().cloned().collect::<HashSet<SymbolId>>().into_iter().collect();
        // unique_values
        // self.0.values().cloned().collect::<Vec<SymbolId>>()
        let res = self.0.values().cloned().collect();
        res
    }

    fn name_by_index(&self,id :SymbolId)->String{
        "".to_string()
    }

}


// 合约仓位表 M2
// x - fac
// y - symbol

//      a   b   c  x1  x2
//  A   0   0   0  2.1  1.5
//  M
//  P
//
pub struct AcPosTable{
    matrix : Array2<f64>,
}

impl AcPosTable{
    fn new(n:usize,m:usize) -> Self{
        // AcPosTable
        let matrix: Array2<f64> = Array::from_elem((n, m), f64::NAN);
        AcPosTable { matrix }
    }

    fn put(&mut self,symbol:SymbolId,ac:AcIndex,ps:f64)->&Self{
        let r = symbol as usize;
        let c = ac as usize;
        self.matrix[(r,c)] = ps ;
        self
    }

    fn get_matrix_mut(&mut self)->&mut Array2<f64>{
        &mut self.matrix
    }
}

#[derive(Debug,Default)]
pub struct Runner{
    config :Config ,
    // posTables: HashMap<SymbolId, AcPosTable>,   // 仓位表
    posTable : Arc<Mutex<AcPosTable>> ,
    // ratioTable : AcRatioTable,
    ratioTs: AcRatioTableSet,
    ch_pos_tx: mpsc::Sender< crate::message::MessagePosition>,
    ch_pos_rx: mpsc::Receiver<crate::message::MessagePosition>,
}

impl Runner {
    // 加载最近的仓位
    fn load_pos(){

    }

    fn init_ratioTable(&mut self)->Result<()>{
        let file_name = "samples/acfollow.txt";
        self.ratioTs.reset();
        for line in io::BufReader::new(File::open(file_name)?).lines() {
            if let Some(en) = AcEntry::from_str(line.unwrap()?){
                ts.add_entry(en);
            }
        }
        self.ratioTs.build_index();
        self.ratioTs.complete();
        Ok(())
    }

    fn init_posTable(&mut self) ->Result<()>{

        let (symbols,rows,cols) = self.ratioTs.get_dim();
        // self.posTable = AcPosTable::new(symbols,cols);
        let matrix: Array2<f64> = Array::from_elem((symbols, cols), f64::NAN);
        self.posTable = Arc::new(Mutex::new(AcPosTable{matrix:matrix}));

        Ok(())
    }

    fn new()->Self{
        Runner::default()
        // let ts = AcRatioTableSet::new();
        // let runner = Runner{config:Config::default(),}
    }

    pub fn get_ratioTs(&mut self)->&mut AcRatioTableSet{
        &mut self.ratioTs
    }

    pub fn get_posTable(&mut self)->&mut Arc<Mutex<AcPosTable>>{
        &mut self.posTable
    }

    fn init_mx(&mut self)->Result<()>{

        Ok(())
    }

    // 仓位信号到达
    pub fn put_pos(&mut self,ps:crate::message::MessagePosition)->Result<&mut Self>{
        self.ch_pos_tx.send(ps)?;
        Ok(&mut self)
    }

    pub fn run(&mut self)->Result<()>{
        let (sender, receiver) = mpsc::channel();
        self.ch_pos_tx = sender;
        self.ch_pos_rx = receiver;

        self.init_ratioTable()?;
        self.init_posTable()?;

        self.init_mx()?;

        let mtxpos = Arc::clone(&self.posTable);
        let thr = thread::spawn(move||{
            loop{
                let m:MessagePosition = self.ch_pos_rx.recv().unwrap();
                let mut postab = mtxpos.lock().unwrap();
                postab.put( SYMTABLE.get_id(m.symbol.as_str()),
                                            self.ratioTs.get_ac_index(&m.ac),
                            m.ps
                );
            }
        });

        loop{
            thread::sleep(Duration::from_secs(self.config.merge_interval.to_owned() as u64));
            self.posMergeDispatch();
        }
        Ok(())
    }

    //  max_follow_depth( posTable * ratioTable => resultTable )
    //  resultTable -> delivery zmq
    fn posMergeDispatch(&mut self)->&Self{
        // 仓位 x 配比 ， 循环最大配比深度次数，最后得到最新的仓位
        let  matrix : Array2<f64>;
        {
            let mtxpos = Arc::clone(&self.posTable);
            let mut postab = mtxpos.lock().unwrap();
            matrix = postab.get_matrix_mut().clone();
        }
        for _ in 0..self.config.max_follow_depth{
            // *self.ratioTable.get_matrix_mut() = self.posTable.get_matrix_mut().dot( self.ratioTable.get_matrix_mut())
            matrix = matrix.dot( self.ratioTable.get_matrix_mut())
        }
        // 开始分发
        // 仅仅分发 ac 项目
        for acidx in self.ratioTs.acInxDispatch{

        }
        self
    }

    // 分发仓位  acTable
    fn dispatch_pos(&mut self,matrix :Array2<f64> ) -> &Self{

        for (index, row) in matrix.axis_iter(Axis(0)).enumerate() {
            let symbol :String  = SYMTABLE.name_by_index(index);
            for index in self.ratioTs.acInxDispatch{
                let &cell = row[index];
                println!("{}", cell);
                let ac = self.get_acname_by_index(index.to_owned());
                self.zmq_send( &ac,&symbol,ps);
            }
        }
        self
    }

    fn zmq_send(&self, ac:&str,symbol:&str,ps:f64)->&Self{

        self
    }

    fn data_persist(&self,ac:&str,symbol:&str,ps:f64)->&Self{

        self
    }

    fn get_acname_by_index(&self,index:usize)->String{
        "".to_string()
    }


// static  symbol_table:SymbolTable = SymbolTable;
}
#[test]
fn test_one(){
    let st = SymboTable(HashMap::new());
    println!("-- {:?}",st.all_ids());
}