+++
title = "Code a Snake AI"
date = 2023-12-04
type = "post"
description = "Battlesnake"
in_search_index = true
[taxonomies]
tags = ["Talks"]
+++

[Battlesnake](https://play.battlesnake.com/) is a competitive multiplayer game where your code is the controller.
To quote the docs:

> Each Battlesnake is controlled by a live web server and the code you write.

To get everyone started fast, I'm going to quickly walk you through setting up your local environment for development and testing, writing the first bit of logic for your snake, and leaving you with the source for my snake to start.

## Testing

You'll need a copy of the Battlesnake CLI to test the snake you write.
It is a go program and is easy to download and install on your computer [following these instructions](https://github.com/BattlesnakeOfficial/rules/tree/main/cli).

## Hosting

Here is [the wiki on choosing a hosting solution](https://docs.battlesnake.com/guides/tips/hosting-suggestions).
For mine, I decided to use Digital Ocean as I have had a good experience hosting there.
Jonathan used [Replit](https://docs.battlesnake.com/quickstart) which also seems like it has a really good option.

I used the DigitalOcean App Platrom with the least expensive option ($5/mo) for my snake, Lethal Lora.
After pointing Digital Ocean at my GitHub repo, which has this Dockerfile in the root, it rebuilds and deploys whenever I push to the main branch.
No CI config needed.

```Dockerfile
FROM rust:1.71

COPY . /usr/app
WORKDIR /usr/app

RUN cargo install --path .

CMD ["lethal-lora"]
```

I'd suggest using either Replit or Digital Ocean if you don't have any prior experience with hosting a webserver as they can let you quickly get started working on the logic of your snake.
My Dockerfile above was copied from the [Rust battlesnake starter project](https://github.com/BattlesnakeOfficial/starter-snake-rust) with one minor modification to update the version of Rust.

## Starter Projects

There are [official starter projects](https://docs.battlesnake.com/starter-projects#official-templates) for battlesnake for Python, Go, Rust, TypeScript, and JavaScript.
There are also starter projects from the community for many other languages.
These projects include many of the fiddly details of getting the webserver setup so you can jump right into programming your snake's ai.

## 4 Possible Moves

What your AI is trying to decide each turn is one of four possible moves: Up, Down, Left, or Right.
These are in the global frame.
The origin (0,0) is the lower left corner of the map.
+y is up, +x is right.

For my Rust snake I defined an Enum for these 4 possible moves:

```rust
#[derive(Debug, Eq, PartialEq)]
enum Move {
    Left,
    Right,
    Up,
    Down,
}
```

Then, using the `Coord` type from the example project I defined functions to convert from my `Move` enum and the `Coord` type.
I also wrote a function to make a vector of all of the 4 moves.

```rust
impl Move {
    fn to_coord(&self, you: &Battlesnake) -> Option<Coord> {
        let head = &you.body[0];
        match (self, head.x, head.y) {
            (Move::Down, _, 1..) => Some(Coord {
                x: head.x,
                y: head.y - 1,
            }),
            (Move::Up, _, _) => Some(Coord {
                x: head.x,
                y: head.y + 1,
            }),
            (Move::Right, _, _) => Some(Coord {
                x: head.x + 1,
                y: head.y,
            }),
            (Move::Left, 1.., _) => Some(Coord {
                x: head.x - 1,
                y: head.y,
            }),
            _ => None,
        }
    }

    fn from_coord(you: &Battlesnake, coord: &Coord) -> Option<Self> {
        let head = &you.body[0];
        if head.x == coord.x {
            if head.y + 1 == coord.y {
                Some(Move::Up)
            } else if coord.y + 1 == head.y {
                Some(Move::Down)
            } else {
                None
            }
        } else if head.y == coord.y {
            if head.x + 1 == coord.x {
                Some(Move::Right)
            } else if coord.x + 1 == head.x {
                Some(Move::Left)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn all() -> Vec<Self> {
        vec![Self::Left, Self::Right, Self::Up, Self::Down]
    }
}
```

You will notice that I return an Option type from both of these as there are are invalid inputs and in those cases I just return `None`.
This enables me to use functions like `flat_map` to convert a vector of moves to coordinates and skip the ones that are invalid.

## 🔗The world is finite; don't leave it

In the standard game the board is 11 by 11 in size.
We need to be careful that we do not drive our snake off the end of the world.
To avoid this problem I created a function to use as a filter:

```rust
fn in_bounds(board: &Board, coord: &Coord) -> bool {
    coord.x < board.width && coord.y < board.height
}
```

## Niave Route Planning

My snake is rather dumb, so it only considers the next possible moves for now.
In-order to do that I created these three functions for selecting my next move:

```rust
fn manhattan_distance(a: &Coord, b: &Coord) -> u32 {
    a.x.abs_diff(b.x) + a.y.abs_diff(b.y)
}

fn select_toward<'a>(coords: &'a [Coord], target: &Coord) -> &'a Coord {
    coords
        .iter()
        .map(|c| (c, manhattan_distance(&c, target)))
        .min_by(|(_, ad), (_, bd)| ad.cmp(bd))
        .map(|(m, _)| m)
        .unwrap()
}

fn select_away<'a>(coords: &'a [Coord], target: &Coord) -> &'a Coord {
    coords
        .iter()
        .map(|c| (c, manhattan_distance(&c, target)))
        .max_by(|(_, ad), (_, bd)| ad.cmp(bd))
        .map(|(m, _)| m)
        .unwrap()
}
```

## Making my Lora unique

Here is how I make my snake green with the specific head and tail I chose:

```rust
pub fn info() -> Value {
    info!("INFO");

    return json!({
        "apiversion": "1",
        "author": "Lethal Lora",
        "color": "#8ceb34",
        "head": "fang",
        "tail": "pixel",
    });
}
```

## Choosing a Move

See [the full source of my logic.rs here](./logic.rs).
