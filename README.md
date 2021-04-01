# Rusty Maze

Console Maze game written in Rust.

## Usage

```shell
# build and run this project locally
cargo run -- --help
```

```shell
docker run --rm -it -e COLUMNS="`tput cols`" -e LINES="`tput lines`" ghcr.io/cronik/rusty-maze 
```

## Ideas

- [x] Visited path tracker toggle
- [x] Difficulty: Hard/Normal
- [x] Save/Share/Replay maze
- [ ] Draw options: large/small
- [ ] Timer
