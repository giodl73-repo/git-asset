# Resume cache eviction preview

This lane implements issue #78 on top of PR72. See the
[upstream checkpoint](https://github.com/giodl73-repo/git-asset/blob/94865c07928275bb130cf3e002dc48ac82d9cfce/context/RESUME.md)
for the original P1-P6 status and P2 verification evidence.

The cache preview starts from observed catalogue inventory and holds unknown
coverage. Add operator controls, pinned paging tests, and bounded keeper
verification. Mixed legacy execution and successful reclaim/restore depend
on #77. Use synthetic fixtures without opening production caches.
