# TL;DR

Using a Vec of borrowed arguments as parameters requires the parameter holder to live the time the parameters are in memory but it does guaranty only pointers are stored and copied during the operation which is optimal from a performances point of view. 

Decision has been made to keep that approach.

# WhereCondition parameter memory management
_Exported on 27/01/2026 at 21:19:36 UTC+1 from Cursor (2.4.21)_

---

**User**

The `WhereCondition` structure holds the query parameters indexed by condition order. 
For now, the parameters structure is a Vec<&'a T> where T: dyn ToSql + Sync
What does that imply from a memory management perspective to hold a borrow on parameters?

---

**Cursor**


Checking how `params!` and `WhereCondition` are used at call sites to complete the picture:



Here’s what holding `Vec<&'a (dyn ToSql + Sync)>` in `WhereCondition<'a>` implies for memory and lifetimes.

---

## 1. **No ownership of parameter data**

`WhereCondition` does **not** own the values. It only holds references. All parameter values must live somewhere else (local variables, struct fields, etc.) and that storage is unrelated to the `WhereCondition` instance.

So the **caller** is responsible for keeping the underlying `T` values alive. The struct is just a view over data that must outlive it.

---

## 2. **Lifetime `'a` ties the condition to the data**

The `'a` in `WhereCondition<'a>` means:

- Every `&'a (dyn ToSql + Sync)` in `parameters` is valid for at least `'a`.
- The `WhereCondition<'a>` as a whole must not be used after the borrowed data might have been dropped.

So you can’t:

- Store a `WhereCondition` and use it “later” after the original values are gone, or
- Send it to another thread or future that might run after those values are destroyed,

unless the types/lifetimes guarantee the data still lives (e.g. `'static` or a long-lived borrow).

---

## 3. **Typical use: build and use in one scope**

In your code, conditions are usually built and used in the same scope where the parameters live, which keeps borrows valid:

```101:102:tests/simple.rs
        let condition = WhereCondition::new("thing_id = $?", params![thing_id]);
        let entity = self.provider.fetch(condition).await?.pop();
```

`thing_id` outlives `condition`, and `condition` (and its references) are fully consumed by `fetch` in that same function. So the borrow of `thing_id` is valid for the whole use.

Same idea in the bike_station example: `position` and `search_radius` live for the whole call, and the condition is built and passed straight into `fetch`.

---

## 4. **`expand()` returns references, so they must stay valid**

`expand()` gives the parameters back as `Vec<&'a (dyn ToSql + Sync)>`:

```72:86:src/core/condition.rs
    pub fn expand(self) -> (String, Vec<&'a (dyn ToSql + Sync)>) {
        let mut expression = self.condition.expand();
        let parameters = self.parameters;
        // ...
        (expression, parameters)
    }
```

Whoever calls `expand()` and then uses that `Vec` (e.g. to run the query) must do it while the original parameter values are still alive. If you stored the condition or the expanded params and used them after the values were dropped, you’d have use-after-free.

---

## 5. **Implications for API design**

- **Reuse**: You can’t safely keep a “prebuilt” `WhereCondition` with non-`'static` params in a struct or cache and reuse it indefinitely; the borrows are only valid while the underlying data lives.
- **Async**: `fetch(&self, condition: WhereCondition<'_>)` is fine as long as the condition and its params are created in the same async context and don’t outlive the parameters. You can’t hold references across `.await` points unless the data is known to live that long (e.g. `'static`).
- **Composing**: In `and_where` / `or_where`, both sides use the same `'a`, so all parameter values must share one lifetime. You can’t mix “short-lived” and “long-lived” params in one condition in a type-safe way; everything is constrained by the same `'a`.

---

## 6. **Literals and the `params!` macro**

With `params![0_i32]` or `params!["foo"]`, the macro builds a block that creates temporaries and pushes references to them. Whether that’s valid depends on Rust’s temporary lifetime rules:

- Temporaries in a `let` initializer can be extended to the end of the surrounding block, so in:

  `let c = WhereCondition::new("...", params![0_i32]);`

  the literal’s temporary may live until the end of the block. Using `c` only in that same block can be valid.

- If the condition or the `Vec` from `expand()` were stored or returned and used later, those references could dangle.

So with **literal** arguments, the safe pattern is: build the condition, call `expand()` and run the query in the same scope, and don’t hold on to the condition or the parameter slice.

---

**Summary:** Holding `Vec<&'a (dyn ToSql + Sync)>` means `WhereCondition` never owns the parameter data; it only borrows it. Lifetime `'a` forces the condition to be used only while that borrowed data is still valid, and the caller must keep the real values alive. That keeps builders and queries flexible and allocation-light, but limits reuse and storage of conditions to cases where the parameter data outlives every use (commonly by building and using the condition in one scope).

---

**User**

wouldn't owning the parameters resut in fewer memory operations ?

---

**Cursor**


Owning the parameters would usually **increase** memory work, not reduce it. Here’s the comparison.

## Borrowing (current design)

- **Allocations:** Only the `Vec<&'a (dyn ToSql + Sync)>` (and any reallocs as it grows). One buffer of pointer-sized slots.
- **Copies:** No copies of parameter values. You only copy/append pointers (e.g. 8 bytes each on 64-bit) into that Vec.
- **Parameter data:** Stays where the caller put it (stack, fields, etc.). The condition just holds references.

So you get a small number of allocations and almost no copying of the actual param data.

## Owning the parameters

To own heterogeneous `ToSql` values you’d need something like:

- `Vec<Box<dyn ToSql + Sync>>`, or
- A type that can store/embed different param types.

Then:

- **Allocations:** One vec allocation **plus** one allocation per parameter (e.g. one `Box` per value), so more allocations overall.
- **Copies/moves:** Each value is moved or copied into that owned storage. So you do at least one full copy/move of every parameter’s bytes.

So you add both extra allocations and extra copying compared to the borrowed design.

## When owning could “feel” like fewer operations

It could look like “fewer” only in specific scenarios:

1. **Caller already collects into a `Vec`**  
   If the caller today does `let v = vec![a, b, c]; WhereCondition::new("...", v.iter().map(|x| x as &dyn ToSql).collect())`, then yes, they’re already allocating and you could fold that into a single “owned” vec built inside `WhereCondition`. You’d be consolidating work the caller already does, not improving on the zero-copy borrowed case.

2. **One big buffer instead of N boxes**  
   If you design an owned representation as “one buffer, all params serialized into it,” you can get down to one allocation and one contiguous region. But then you’re doing a lot of copying and encoding into that buffer. You’ve traded “N small allocations + N moves” for “1 allocation + many bytes copied.” Total memory operations (bytes copied, cache lines touched) usually go up.

3. **Easier reuse / storage**  
   Owning can simplify APIs (no lifetime, condition is self-contained and storable). The benefit is ergonomics and flexibility, not “fewer memory operations” in the narrow sense.

So in the normal case—caller has `thing_id`, `name`, etc. and passes `params![thing_id, name]`—borrowing does **fewer** memory operations: one (or a few) vec allocations and no copying of parameter data. Owning adds per-parameter allocations and/or copies of that data, so it’s more work from a memory-allocation and -copying perspective.

