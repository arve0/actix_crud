# actix crud client

## Development

Install the dependencies:

```sh
npm install
```

Then start [Rollup](https://rollupjs.org):

```sh
npm run autobuild
```

...and the rust backend server:

```sh
cargo watch -i client -x run
```

Navigate to [localhost:8080](http://localhost:8080). You should see your app running. Edit a component file in `src`, save it, and reload the page to see your changes.
