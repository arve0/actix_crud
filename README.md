# actix_crud
An implementation of create, read, update and delete over HTTP with [actix-web](https://actix.rs) and [SQLite](https://www.sqlite.org).

## Running
```sh
# start server
cargo run --release
# in another terminal
curl localhost:8080
```

Static files are served from `./static`. See [test.sh](test.sh) for how to:

- register users
- login
- CRUD-ing documents


## Development
```sh
# install cargo-watch if not present
cargo install cargo watch
# start server, compile and restart on changes to files
cargo watch -x run
```


## Testing
```sh
# start server
cargo run
# in another terminal
./test.sh
```
