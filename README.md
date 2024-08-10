# “高效”安全链表

尝试使用 StaticRc 和 GhostCell 实现的安全链表。

实现一个 Safe 版本的链表应该可以算是 rust 语言的 hello world？

然而 Rc 和 Cell 本身包含着一些运行时检查，例如 Rc 需要一个引用计数，Cell 需要保存目前的引用状态。
就算通过 Rc 和 Cell 在**不显式地使用 unsafe 代码块**的前提下实现了链表，也会损失部分性能。

而 StaticRc 和 GhostCell 则通过编译时检查保证了部分安全性。
之所以说“部分”，是因为 StaticRc<1,2> 如果没有被合并为 StaticRc<2,2> 拿到数据的所有权，就会导致 drop 时发生内存泄漏，这种错误只在运行时才能被检查出。与 Rc 的循环引用只能在运行时检查出基本一致。

实际上还是用 Arena 布局的链表比较好，既没有引用检查的约束，对缓存命中也友好许多。

## TODO
- [x] IntoIter
- [x] Iter
- [ ] IterMut
- [ ] Cursor
- [ ] CursorMut
- [ ] 话说 GhostCell 真的是这么用吗？

## 参考

- [1] https://frank-king.github.io/rustblog-zh/2021-09-ghost-cell/09-03-ghost-cell.html
- [2] https://docs.rs/ghost-cell/latest/ghost_cell/
- [3] https://docs.rs/static-rc/latest/static_rc/
