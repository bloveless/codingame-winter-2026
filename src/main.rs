use std::{
    collections::{HashMap, HashSet, vec_deque::VecDeque},
    io,
};

use itertools::Itertools;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Pos {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone)]
struct PowerSource {
    pos: Pos,
    above_platform: i32,
}

#[derive(Clone, Debug)]
struct Snake {
    id: i32,
    mine: bool,
    pos: Pos,
    body: Vec<Pos>,
}

#[derive(Debug)]
struct Game {
    my_id: i32,
    my_snake_ids: Vec<i32>,
    opp_snake_ids: Vec<i32>,
    width: i32,
    height: i32,
    platforms: HashSet<Pos>,
    proximity: Vec<Vec<u32>>,
}

fn get_dir(start: Pos, end: Pos) -> String {
    let d = ((end.x - start.x), (end.y - start.y));
    match d {
        (1, 0) => String::from("RIGHT"),
        (-1, 0) => String::from("LEFT"),
        (0, -1) => String::from("UP"),
        (0, 1) => String::from("DOWN"),
        _ => panic!("get dir received unreachable direction"),
    }
}

impl Game {
    fn tick(&self, snakes: Vec<Snake>, power_sources: Vec<PowerSource>) {
        for snake in &snakes {
            if snake.mine {
                let Some(close_power_source) = find_closest_power_source(&power_sources, &snake)
                else {
                    // no power source was found snake will continue in the direction it was going
                    continue;
                };
                if let Some(path) = self.shortest_path(snake.pos, close_power_source.pos, &snakes) {
                    eprintln!(
                        "snake: {} ({:?}) to ({:?})", //  path: {:?}",
                        snake.id,
                        snake.pos,
                        close_power_source //, path
                    );
                    print!(
                        "MARK {} {};",
                        close_power_source.pos.x, close_power_source.pos.y
                    );
                    let dir = get_dir(snake.pos, path[0]);
                    print!("{} {} IM {};", snake.id, dir, snake.id);
                } else {
                    let neighbors = self.get_neighbors(&snakes, snake.pos.clone());
                    if neighbors.len() > 0 {
                        let dir = get_dir(snake.pos, neighbors[0]);
                        print!("{} {} WAIT {};", snake.id, dir, snake.id,)
                    } else {
                        // Just go up I guess.
                        print!("{} {} WAIT {};", snake.id, "UP", snake.id,)
                    }
                }
            }
        }

        println!("");
    }

    fn get_neighbors(&self, snakes: &Vec<Snake>, p: Pos) -> Vec<Pos> {
        let mut neighbors = Vec::new();
        // snakes cant move diagonally
        // check left and right first to try and get around the going up forever problem
        for (dx, dy) in vec![(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let cur_pos = Pos {
                x: p.x + dx,
                y: p.y + dy,
            };
            let is_platform = self.platforms.get(&cur_pos).is_some();
            let is_snake = snakes
                .iter()
                .find(|s| s.pos == cur_pos || s.body.contains(&cur_pos))
                .is_some();
            if !is_platform && !is_snake {
                neighbors.push(Pos {
                    x: p.x + dx,
                    y: p.y + dy,
                })
            }
        }
        neighbors
    }

    fn shortest_path(&self, start: Pos, end: Pos, snakes: &Vec<Snake>) -> Option<Vec<Pos>> {
        const PROXIMITY_WEIGHT: f32 = 2.0;

        #[derive(Clone, PartialEq)]
        struct Node {
            f: f32,
            g: f32,
            pos: Pos,
        }
        impl Eq for Node {}
        impl Ord for Node {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                other
                    .f
                    .partial_cmp(&self.f)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        }
        impl PartialOrd for Node {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        let h = |p: Pos| -> f32 { ((p.x - end.x).abs() + (p.y - end.y).abs()) as f32 };

        let prox_cost = |p: Pos| -> f32 {
            let d = self
                .proximity
                .get(p.y as usize)
                .and_then(|row| row.get(p.x as usize))
                .copied()
                .unwrap_or(u32::MAX);
            if d == u32::MAX {
                PROXIMITY_WEIGHT * 10.0
            } else {
                PROXIMITY_WEIGHT * d as f32
            }
        };

        let mut open = std::collections::BinaryHeap::new();
        let mut g_score: HashMap<Pos, f32> = HashMap::new();
        let mut parents: HashMap<Pos, Pos> = HashMap::new();

        g_score.insert(start, 0.0);
        open.push(Node {
            f: h(start),
            g: 0.0,
            pos: start,
        });

        while let Some(Node { pos, g, .. }) = open.pop() {
            if pos == end {
                let mut path = Vec::new();
                let mut cur = end;
                while cur != start {
                    path.push(cur);
                    cur = parents[&cur];
                }
                path.reverse();
                return Some(path);
            }

            if g > *g_score.get(&pos).unwrap_or(&f32::INFINITY) {
                continue;
            }

            for nb in self.get_neighbors(snakes, pos) {
                let tentative_g = g + 1.0 + prox_cost(nb);
                if tentative_g < *g_score.get(&nb).unwrap_or(&f32::INFINITY) {
                    g_score.insert(nb, tentative_g);
                    parents.insert(nb, pos);
                    open.push(Node {
                        f: tentative_g + h(nb),
                        g: tentative_g,
                        pos: nb,
                    });
                }
            }
        }

        None
    }

    // fn shortest_path(&self, start: Pos, end: Pos, snakes: &Vec<Snake>) -> Option<Vec<Pos>> {
    //     let mut visited = Vec::new();
    //     let mut queue = VecDeque::new();
    //     let mut parents: HashMap<Pos, Pos> = HashMap::new();

    //     queue.push_back(start);

    //     while let Some(node) = queue.pop_front() {
    //         if node == end {
    //             let mut path = Vec::new();
    //             let mut cur = end;
    //             while cur != start {
    //                 path.push(cur.clone());
    //                 cur = parents[&cur];
    //             }
    //             path.reverse();
    //             return Some(path);
    //         }
    //         if !visited.contains(&node) {
    //             visited.push(node.clone());
    //             for neighbor in self.get_neighbors(snakes, node) {
    //                 if !visited.contains(&neighbor) {
    //                     queue.push_back(neighbor.clone());
    //                     parents.insert(neighbor, node.clone());
    //                 }
    //             }
    //         }
    //     }

    //     None
    // }

    fn process_frame(&self) -> (Vec<Snake>, Vec<PowerSource>) {
        let mut snakes = Vec::new();
        let mut power_sources = Vec::new();

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let power_source_count = parse_input!(input_line, i32);
        for _ in 0..power_source_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let x = parse_input!(inputs[0], i32);
            let y = parse_input!(inputs[1], i32);
            let mut next = 0;
            while let None = self.platforms.get(&Pos { x: x, y: y + next }) {
                next += 1;
            }
            power_sources.push(PowerSource {
                pos: Pos { x, y },
                above_platform: next,
            });
        }

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let snake_count = parse_input!(input_line, i32);
        for _ in 0..snake_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let snake_id = parse_input!(inputs[0], i32);

            let coords: Vec<(i32, i32)> = inputs[1]
                .trim()
                .split(&[',', ':'])
                .map(|x| parse_input!(x, i32))
                .tuples()
                .collect();

            let mut body = Vec::new();
            for (x, y) in &coords[1..] {
                body.push(Pos { x: *x, y: *y });
            }
            snakes.push(Snake {
                id: snake_id,
                pos: Pos {
                    x: coords[0].0,
                    y: coords[0].1,
                },
                mine: self.my_snake_ids.contains(&snake_id),
                body,
            })
        }

        (snakes, power_sources)
    }
}

fn main() {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let my_id = parse_input!(input_line, i32);

    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let width = parse_input!(input_line, i32);

    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let height = parse_input!(input_line, i32);

    let mut platforms = HashSet::new();
    for y in 0..height as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        for (x, cell) in input_line.chars().enumerate() {
            if cell == '#' {
                platforms.insert(Pos {
                    x: x as i32,
                    y: y as i32,
                });
            }
        }
    }
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let snakes_per_player = parse_input!(input_line, i32);

    let mut my_snake_ids = Vec::new();
    for _ in 0..snakes_per_player as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        my_snake_ids.push(parse_input!(input_line, i32));
    }

    let mut opp_snake_ids = Vec::new();
    for _ in 0..snakes_per_player as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        opp_snake_ids.push(parse_input!(input_line, i32));
    }

    let proximity = build_proximity_map(width as usize, height as usize, &platforms);

    let game = Game {
        my_id: my_id,
        my_snake_ids,
        opp_snake_ids,
        width,
        height,
        platforms,
        proximity,
    };

    loop {
        let (snakes, power_sources) = game.process_frame();
        eprintln!("snakes: {:?}", snakes);
        game.tick(snakes, power_sources);
    }
}

fn build_proximity_map(width: usize, height: usize, platforms: &HashSet<Pos>) -> Vec<Vec<u32>> {
    let mut dist = vec![vec![u32::MAX; width]; height];

    for p in platforms {
        let pp = Pos { x: p.x, y: p.y };
        let mut d = 0;
        dist[p.y as usize][p.x as usize] = d;
        for ny in (0..pp.y).rev() {
            d += 1;
            if platforms.contains(&Pos { x: p.x, y: ny }) {
                break;
            }
            dist[ny as usize][p.x as usize] = d;
        }
    }

    dist
}

fn find_closest_power_source(
    power_sources: &Vec<PowerSource>,
    snake: &Snake,
) -> Option<PowerSource> {
    let max_reach = snake.body.len() as i32 + 1;

    power_sources
        .iter()
        .filter(|p| p.above_platform <= max_reach)
        .min_by_key(|p| (p.pos.x - snake.pos.x).abs() + (p.pos.y - snake.pos.y).abs())
        .cloned()
}

#[cfg(test)]
mod test {
    use super::*;

    fn board_from_ascii(input: &str) -> HashSet<Pos> {
        let mut platforms = HashSet::new();
        for (y, row) in input.lines().map(|x| x.trim()).enumerate() {
            for (x, cell) in row.chars().enumerate() {
                if cell == '#' {
                    platforms.insert(Pos {
                        x: x as i32,
                        y: y as i32,
                    });
                }
            }
        }
        platforms
    }

    #[test]
    fn get_tiles() {
        let g = Game {
            my_id: 0,
            my_snake_ids: vec![1, 2, 3],
            opp_snake_ids: vec![4, 5, 6],
            width: 5,
            height: 5,
            platforms: board_from_ascii(
                ".....
                 #...#
                 .....
                 #...#
                 .###.",
            ),
            proximity: Vec::new(),
        };
        println!("Tiles: {:?}", g.platforms);
        assert_eq!(g.platforms.len(), 7);
        assert!(g.platforms.get(&Pos { x: 0, y: 1 }).is_some());

        let path = g.shortest_path(Pos { x: 0, y: 0 }, Pos { x: 3, y: 2 }, &Vec::new());
        assert_eq!(path.unwrap().len(), 5);
    }

    #[test]
    fn test_build_proximity_map() {
        let platforms = board_from_ascii(
            ".......
             .#...#.
             .......
             .#...#.
             .......
             #.....#
             ..###..",
        );
        let proximities = build_proximity_map(7, 7, &platforms);
        for row in proximities {
            for col in row {
                print!("{}, ", col)
            }
            println!();
        }
    }
}
