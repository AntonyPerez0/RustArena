# semver-lite

Parse, compare and match [semantic versions](https://semver.org) in Rust.

```rust
use semver_lite::{Parser, VersionReq};

let published = Parser::new().parse_all(["1.0.0", "1.4.2", "2.0.0"]).unwrap();
let req: VersionReq = "^1.2".parse().unwrap();
assert_eq!(req.max_satisfying(&published).unwrap().to_string(), "1.4.2");
```

- `Version`: parse, order and bump versions like `1.4.2` and `2.0.0-rc.1`.
- `VersionReq`: Cargo-style requirements: `^1.2`, `~1.2.3`, `>=1.0.0, <2.0.0`, `*`.
- `Parser`: accept `v1.2.3` (Git tags) and partial versions like `1.2` when you choose to.
- Typed errors you can `match` on.

## Features

- `serde`: `Serialize`/`Deserialize` for `Version`, as a string.

## License

MIT OR Apache-2.0
