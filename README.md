# Rusty Maze

Console Maze game written in Rust. 

> Disclaimer: This project is a learning exercise in rust and GitHub actions/packages

## Ideas

- ~~Visited path tracker toggle~~
- Draw options: large/small
- Difficulty: Hard/Normal 
- Timer
- Save/Share/Replay maze

## Usage

```shell
# build and run this project locally
cargo run
```

```shell
docker run --rm -it -e COLUMNS="`tput cols`" -e LINES="`tput lines`" ghcr.io/cronik/rusty-maze 
```