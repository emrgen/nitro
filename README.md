A high performance CRDT library.

### Bench

```
sudo sysctl kernel.perf_event_paranoid=3
flamegraph -- cargo run --example huge_list
```

### Features

- [x] document
- [x] list
- [x] map
- [x] test
- [x] string
- [x] atom
- [x] mark
- [x] delete
- [x] diff
- [x] merge
- [x] sync docs
- [x] local move
- [ ] sync move changes
- [ ] proxy - same block visible in multiple places, change in one place updates all places

### TODO

- [ ] use queue store for pending items instead of IdStore<T>
- [ ] merge adjacent string with same marks
- [ ] add move feature
- [ ] add tests with 100 users and docs
- [ ] add tests for props update
