# Metadata capacity evidence

EVID-141; [controls](../../specs/METADATA-CAPACITY.md). Producer review, not
independent/human approval. Implementation: additive SDK transition/lineage;
existing NodeStore stays unchanged. Concurrency/privacy lenses retain scoped SQL
fences, bounded path-free plans and explicit owner adoption; no removal authority.

Verification: 16 original metadata tests, 10 capacity entries, explicit old-writer
fixture and strict all-feature/all-target Clippy pass. Four actual exit boundaries
recover original operations. Windows/NTFS rustc/cargo1.98.0 qualification migrates
92,256 nodes and reads back every body; stale pinned writer/reopen both refuse.
Validation: synthetic owner adoption only; REEL profile CAS, production, hosted
exact-head and power-loss acceptance remain separate. Trace: controls -> capacity
module/tests/legacy fixture -> this packet.

SHA256 pins:

- SDK manifest: `25e4a2173488842fe61201a48149ac4eb5f91f736f832a1a82e9e3e0401fddb4`
- Legacy exe: `6eee5f2b1c0b9cc8b8a904c2b9adb3c34dfbad762c24ef0a81e065864d868023`
- Test exe: `6798332d871bfde9e509be422df2529178b57092d8e1db35a8bf5b952d507f89`

Legacy source0bb2b20 plus shipped fixture. Manifest: sorted root Cargo.toml/lock
and all crates files; entries path/bytes/sha256; ASCII compact JSON, sorted keys.
Full manifest, tool/source/lock/log hashes remain external qualification artifacts.
