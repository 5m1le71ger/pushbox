mod pushboxmap;
mod solve;

use std::env;
use pushboxmap::Point;
use pushboxmap::PushBoxMap;
use std::collections::BTreeSet;

use solve::solve_pushbox;
use solve::TraceNode;

fn main() {
    if env::args().len() < 2 {
        println!("Usage: pushbox.exe map_file_name");
        return;
    }

    let filename = env::args().nth(1).unwrap();

    //读取文件
    let mut x1 = PushBoxMap::new();
    x1.load(&filename);
    println!("load {},\n{}",filename,x1);

    let mut maps = BTreeSet::<PushBoxMap>::new();
    let mut trace = Vec::<TraceNode>::new();

    //开始解题
    let b = solve_pushbox(&x1,&mut maps,&mut trace);

    println!("solve_pushbox, b={}, maps size = {},trace size = {}", b, maps.len(),trace.len());
    println!("solve_pushbox, trace=");

    //trace为倒序
    for v in trace.iter().rev().into_iter() {
        match v.1 {
            Some(ref u) => {
                println!("{}",v.0.show_move(&u));
            },
            None    => println!("{}",v.0),
        }
    }

}

