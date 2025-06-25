A high performance CRDT library.

**Note**: This project is under active development. **NOT READY FOR PRODUCTION USE**

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
- [x] move
- [x] delete
- [x] diff
- [x] merge
- [x] sync docs
- [ ] sync move changes
- [ ] undo/redo manager

### TODO

- [ ] use queue store for pending items instead of IdStore<T>
- [ ] merge adjacent string with same marks
- [ ] add tests with 100 users and docs
- [ ] add tests for props update
