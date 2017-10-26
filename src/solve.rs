use pushboxmap::Point;
use pushboxmap::MoveOP;
use pushboxmap::PushBoxMap;
use std::collections::BTreeSet;

pub struct TraceNode (pub PushBoxMap,pub Option<MoveOP>);

//解题(当前地图状态,状态表,跟踪表)
pub fn solve_pushbox(pbm: &PushBoxMap, maps: &mut BTreeSet<PushBoxMap>, trace: &mut Vec<TraceNode>) -> bool {
    //如果已经存在这个状态，则返回false
    if maps.contains(&pbm) {
        return false;
    }

    //加入该状态
    maps.insert(pbm.clone());

    //判断该状态是否完成目标，如果是则返回true
    if pbm.check_map_goal() {
        return true;
    }

    //获得该状态下的移动操作列表
    let mut move_ops : Vec<MoveOP> = Vec::new();
    pbm.find_move_op(&mut move_ops);

    //如果没有则返回false
    if move_ops.len() == 0 {
        return false;
    }

    //遍历执行移动操作
    for op in move_ops.into_iter() {
        //移动得到新状态
        let mut pbm_new = pbm.clone();
        pbm_new.move_boxx(&op);

        //递归解决该新状态        
        if solve_pushbox(&pbm_new,maps,trace) {
            //如果已经找到解答，则记录该状态和移动操作
            trace.push( TraceNode (pbm_new.clone(),Some(op)) );
            return true;
        }
    }

    false
}

#[test]
fn test_parse() {
    let mut line_buf = "box= aaaaa";

    // let v = parse_key_value("box= aaaaa","box");
    // assert_eq!(Some("aaaaa"), v);

    let v = parse_key_value("box= aaaaa","box");
    assert_eq!(Some("aaaaa"), v);
}

#[test]
fn test_stack() {
    let a = PushBoxMap::new();
    let b = PushBoxMap::new();
    let mut s = stack::Stack::<&PushBoxMap>::new();
    assert_eq!(s.pop(), None);
    s.push(&a);
    s.push(&b);
    println!("{:?}", s);
    assert_eq!(s.pop(), Some(&b));
    assert_eq!(s.pop(), Some(&a));
    assert_eq!(s.pop(), None);
}