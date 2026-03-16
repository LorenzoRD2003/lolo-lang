# Bolt's Journal - lolo-lang optimizations

## 2025-05-15 - Optimized name resolution and fixed redeclaration bug
**Learning:** The `declared_in_scope` implementation in `ScopeArena` was performing two `resolve` calls, each being O(depth) of the scope hierarchy. This was not only inefficient but also buggy, as it failed to detect redeclarations when they shadowed a variable from an outer scope. Direct O(1) lookup in the current scope's symbol map is both faster and correct for this check.
**Action:** Always prefer direct map lookups for scope-local checks instead of hierarchical resolution when performing declaration-site analysis.
