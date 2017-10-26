use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::fmt;
use std::collections::BTreeSet;
use std::iter;
use std::cmp::Ordering;

//方向
#[derive(Copy,Clone,Debug,PartialEq)]
pub enum Direct {
    Left,
    Up,
    Right,
    Down
}

impl Default for Direct {
    fn default() -> Direct { Direct::Left }
}

//2d坐标
#[derive(Default,Ord,PartialOrd,Copy,Clone,Debug,PartialEq,Eq)]
pub struct Point {
    x: usize,
    y: usize
}

//地图
#[derive(Eq,Debug,PartialEq,Clone,Ord,PartialOrd)]
pub struct PushBoxMap {
    map: [[u8;16];16],          //16x16  0 - 不可通行 >0 可通行的连通域
    boxx: BTreeSet<Point>,      //箱子(3个)
    target: BTreeSet<Point>,    //目标(3个)
    player: u8,                 //玩家所在的连通域
}

//单元格、交叉点
#[derive(Copy,Clone,Debug,PartialEq)]
pub enum Cross {
    None,       //地图之外
    Stone,      //不可通行
    Domain(u8), //可通行的连通域
    Boxx        //箱子
}

impl Default for Cross {
    fn default() -> Cross { Cross::None }
}

impl Cross {
    fn is_none(&self) -> bool { *self == Cross::None}
    fn is_stone(&self) -> bool { *self == Cross::Stone}
    fn is_domain(&self) -> bool {
        match *self {
            Cross::Domain(v) => true,
            _ => false
        }
    }

    fn is_boxx(&self) ->bool { *self == Cross::Boxx}
}

//移动操作 (坐标点,方向)
#[derive(Default,Debug,PartialEq)]
pub struct MoveOP (Point,Direct);

//
impl PushBoxMap {
    pub fn new() -> PushBoxMap {
            PushBoxMap {
                map: [[0;16];16],
                boxx: BTreeSet::<Point>::new(),
                target: BTreeSet::<Point>::new(),
                player: 0,
            }
        }

    //读取文件
    pub fn load<'a>(&mut self, file_name: &'a str) -> bool {
        let f = File::open(&file_name).expect("file open failed");

        let mut reader = BufReader::new(f);

        //
        for i in 0..16 {
            let mut line_buf = String::new();
            reader.read_line(&mut line_buf).expect("read line from file faild");
        
            let bytes = line_buf.into_bytes();
            for j in 0..16 {
                self.map[j][i] = bytes[j] - 48;
            }
        }
        
        //box=
        {
            let mut line_buf = String::new();
            reader.read_line(&mut line_buf).expect("read line from file faild (box)");

            let s_value = parse_key_value(&line_buf,"box").expect("parse box= failed,(parse_key_value)");

            if !parse_3_points(&mut self.boxx,s_value) {
                panic!("parse box= failed,(parse_3_points)");
            }
        }

        //target=
        {
            let mut line_buf = String::new();
            reader.read_line(&mut line_buf).expect("read line from file faild (target)");

            let s_value = parse_key_value(&line_buf,"target").expect("parse target= failed,(parse_key_value)");

            if !parse_3_points(&mut self.target,s_value) {
                panic!("parse target= failed,(parse_3_points)");
            }
        }

        //player
        let mut player_point : Point = Point {x:0,y:0};
        {
            let mut line_buf = String::new();
            reader.read_line(&mut line_buf).expect("read line from file faild (player)");

            let s_value = parse_key_value(&line_buf,"player").expect("parse player= failed,(parse_key_value)");

            let mut iter2 = s_value.split(',');
            match (iter2.next(),iter2.next()) {
                (Some(x),Some(y)) => {
                    player_point.x = x.parse().expect("parse peopl= failed,(x.parse)");
                    player_point.y = y.parse().expect("parse peopl= failed,(y.parse)");
                },
                _ => panic!("parse player= failed,(x,ys)"),
            }
        }

        //校验数据合理性
        let result = self.check_map_valid(&player_point);
        match result {
            Some(x) => panic!(x),
            None    => {},
        }

        //
        self.calc_domain(player_point);
        true
    }

    //获得坐标的cross属性
    fn get_pos(&self, _x: isize, _y:isize) -> Cross {
        match (_x,_y) {
            (0...16,0...16) => {
                let v = self.map[_x as usize][_y as usize];
                if v == 0 {
                    Cross::Stone
                } else {
                    let mut rtn = Cross::Domain(v);
                    for p in self.boxx.iter().into_iter() {
                        if p.x == _x as usize && p.y == _y as usize{
                            rtn = Cross::Boxx;
                            break;
                        }
                    }
                    rtn
                }
            },

            _ => Cross::None,
        }
    }

    //计算可连通域
    //    T
    //   LP
    fn calc_domain(&mut self,player_point: Point) {
        let mut gen_ids : Vec<u8> = vec![9,8,7,6,5,4,3,2,1];
        //遍历
        for i in 0..16isize {
            for j in  0..16isize {
                if self.get_pos(j,i).is_domain() {
                    //p左边的点
                    let p_left = self.get_pos(j-1,i);

                    //p上边的点
                    let p_top  = self.get_pos(j,i-1);

                    match (p_left,p_top) {
                        (Cross::Domain(v1),Cross::Domain(v2)) => {
                            if v1 == v2 {
                                self.map[j as usize][i as usize] = v1;
                            } else {
                                self.map[j as usize][i as usize] = self.connect_domain(v1,v2,j as usize-1,i as usize,&mut gen_ids);
                            }
                        },
                        (Cross::Domain(v),_) | (_,Cross::Domain(v)) => self.map[j as usize][i as usize] = v,
                        _ => {
                            let id = gen_ids.pop();
                            self.map[j as usize][i as usize] = id.unwrap();
                        },
                    };
                }
            }
        }

        self.player = self.map[player_point.x][player_point.y];
    }

    //连接两个domain
    //比如连通域2和连通域3合并，则保留小值2, 大值3收回到可用队列gen_ids
    fn connect_domain(&mut self, v1: u8, v2: u8, x: usize, y: usize, gen_ids: &mut Vec<u8>) -> u8 {
        let (min,max) = if v1 < v2 {(v1,v2)} else {(v2,v1)};

        for i in 0..y+1 {
            for j in  0..16 {
                if i == y && j > x {
                    break;
                }

                if self.map[j][i] == max {
                    self.map[j][i] = min;
                }
            }
        }

        gen_ids.push(max);
        min
    }

    //获得可移动操作列表
    pub fn find_move_op(&self, move_ops: &mut Vec<MoveOP>) {
        move_ops.clear();

        for boxx_pos in  self.boxx.iter().into_iter() {
            //考察横向
            let c1 = self.get_pos(boxx_pos.x as isize -1,boxx_pos.y as isize);
            let c2 = self.get_pos(boxx_pos.x as isize +1,boxx_pos.y as isize);
            match (c1, c2) {
                (Cross::Domain(v1),Cross::Domain(v2)) => {
                    if self.player == v1 {
                        move_ops.push( MoveOP (*boxx_pos,Direct::Right));
                    }

                    if self.player == v2 {
                        move_ops.push( MoveOP (*boxx_pos,Direct::Left));
                    }
                },
                _ => {},
            }

            //考察竖向
            let c1 = self.get_pos(boxx_pos.x as isize,boxx_pos.y as isize - 1);
            let c2 = self.get_pos(boxx_pos.x as isize,boxx_pos.y as isize + 1);
            match (c1, c2) {
                (Cross::Domain(v1),Cross::Domain(v2)) => {
                    if self.player == v1 {
                        move_ops.push( MoveOP (*boxx_pos,Direct::Down));
                    }

                    if self.player == v2 {
                        move_ops.push( MoveOP (*boxx_pos,Direct::Up));
                    }
                },
                _ => {},
            }
        }
    }

    //移动箱子
    pub fn move_boxx(&mut self,move_op: &MoveOP) {
        self.boxx.take(&move_op.0);
        let mut boxx_point = move_op.0;
        match move_op.1 {
            Direct::Left  => boxx_point.x -= 1,
            Direct::Up    => boxx_point.y -= 1,
            Direct::Right => boxx_point.x += 1,
            Direct::Down  => boxx_point.y += 1,
        }

        self.boxx.insert(boxx_point);
        self.calc_domain(move_op.0);
    }

    //
    //校验数据合理性
    fn check_map_valid(&self, player_pos: &Point) -> Option<String> {

        //0...16
        for v in self.boxx.iter().into_iter() {
            match (v.x, v.y) {
                (0...16,0...16) => {},
                _     => return Some("check vaild boxx failed".to_string()),
            
            }
        }

        for v in self.target.iter().into_iter() {
            match (v.x, v.y) {
                (0...16,0...16) => {},
                _     => return Some("check vaild boxx failed".to_string()),
            
            }
        }

        match (player_pos.x, player_pos.y) {
            (0...16,0...16) => {},
            _     => return Some("check vaild player failed".to_string()),
        }

        //箱子之间，人的位置不能重合
        if self.boxx.len() < 3 {
            return Some("check diff point(boxx) failed".to_string());
        }

        if self.boxx.contains(player_pos) {
            return Some("check diff point(player) failed".to_string());
        }

        //目标之间的位置不能重合
        if self.target.len() < 3 {
            return Some("check diff point(target) failed".to_string());
        }

        //箱子、人、目标的坐标点，必须在可通行的路上
        let check_pass = | is_pass : Option<String>, p : Point | {
            if is_pass == None && self.map[p.x][p.y] == 0 {
                let s = format!("check point in pass failed ({},{},{})",p.x,p.y,self.map[p.x][p.y]);
                Some(s)
            } else {
                is_pass
            }
        };
        
        let mut is_pass : Option<String> = None;
        for v in self.boxx.iter().into_iter() {
            is_pass = check_pass(is_pass,*v);
        }
        for v in self.target.iter().into_iter() {
            is_pass = check_pass(is_pass,*v);
        }
        is_pass = check_pass(is_pass,*player_pos);

        is_pass
    }

    //检查是否完成目标
    pub fn check_map_goal(&self) -> bool {
        self.target == self.boxx
    }

    //显示详细信息
    pub fn show_detail(&self) -> String {
        let mut f = String::new();

        f += &format!("box={:?}\n",self.boxx);
        f += &format!("target={:?}\n",self.target);
        f += &format!("player={}\n",self.player);

        for i in 0..16 {
            for j in 0..16 {
                f += &format!("{}", (self.map[j][i]+ 48) as char );
            }

            f += &format!("\n");
        }


        f
    }

    //默认显示，较直观
    pub fn show(&self) -> String {
        let mut f = String::new();

        f = format!("\nplayer={}\n",self.player);
        for i in 0..16 {
            for j in 0..16 {

                let mut show_char  = (self.map[j][i]+ 48) as char;
                if show_char == '0' {
                    show_char = ' ';
                }
                let get_show_char = | sc:&mut char,p:Point, c | {
                    if j == p.x && i == p.y {
                        *sc = c;                
                    }
                };

                //target的显示优先级最小，所以要放在前面
                for v in self.target.iter().into_iter() {
                    get_show_char(&mut show_char,*v,'T');
                }
                for v in self.boxx.iter().into_iter() {
                    get_show_char(&mut show_char,*v,'B');
                }

                f += &show_char.to_string();
            }

            f += "\n";
        }

        f
    }

    //带移动信息的显示
    pub fn show_move(&self,move_op: &MoveOP) -> String {
        let mut f = String::new();

        f = format!("\nplayer={}\n",self.player);
        for i in 0..16 {
            for j in 0..16 {

                let mut show_char  = (self.map[j][i]+ 48) as char;
                if show_char == '0' {
                    show_char = ' ';
                }
                let get_show_char = | sc:&mut char,p:Point, c | {
                    if j == p.x && i == p.y {
                        *sc = c;                
                    }
                };

                //target的显示优先级最小，所以要放在前面
                for v in self.target.iter().into_iter() {
                    get_show_char(&mut show_char,*v,'T');
                }
                for v in self.boxx.iter().into_iter() {
                    get_show_char(&mut show_char,*v,'B');
                }

                let move_char = match move_op.1 {
                    Direct::Left => '<',
                    Direct::Up   => '^',
                    Direct::Right=> '>',
                    Direct::Down => 'v',
                };

                get_show_char(&mut show_char,move_op.0,move_char);

                f += &show_char.to_string();
            }

            f += "\n";
        }

        f
    }
}

impl fmt::Display for PushBoxMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.show())
    }
}

//解析3个坐标
fn parse_3_points<'a,'b>(three_point: &'b mut BTreeSet<Point>, text: &'a str) -> bool{
    let v : Vec<_> = text.split(';').collect();
    if v.len() < 3 {
        return  false;
    }

    for i in 0..3 {
        let mut iter2 = v[i].split(',');
        match (iter2.next(),iter2.next()) {
            (Some(x),Some(y)) => {
                let mut p = Point {x:0,y:0};
                p.x = match x.parse() {
                    Ok(_x) => _x,
                    Err(_) => return false,
                };
                p.y = match y.parse() {
                    Ok(_y) => _y,
                    Err(_) => return false,
                };

                three_point.insert(p);
            },
            _ => return false,
        }
    }

    true
}

//解析key=value
fn parse_key_value<'a>(text: &'a str, key: &'a str) -> Option<&'a str> {
    let mut iter = text.split("=");
    match (iter.next(),iter.next()) {
        (Some(_k), Some(_v))  => {

            let rtn : Option<&'a str>;
            if _k.trim() == key {
                rtn = Some(_v.trim());
            } else {
                rtn = None;
            }

            rtn
        },
        _ => None

    }
}
