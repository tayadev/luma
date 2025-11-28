---
sidebar_position: 1
---

# Async and Await

:::caution Work in Progress
Async/await is planned for implementation. This documentation describes the target design.
:::

Luma has **first-class support for asynchronous programming** with native async/await syntax. Asynchronous operations are represented as `Promise` types.

## Promises

A `Promise(T)` represents a value that will eventually become available:

```luma
fn fetchData(url: String): Promise(String) do
  -- returns a promise that resolves to a String
end
```

Promises allow **non-blocking operations** — your program continues running while waiting for results.

## The await Keyword

`await` suspends execution until a promise resolves, then returns its value:

```luma
fn fetchUser(id: String): Result(User, Error) do
  let response = await http.get("/users/${id}")
  parseUser(response)
end
```

**Type transformation:**
- `await Promise(T)` → `T`
- The promise is unwrapped to get the actual value

## Async Functions

Functions that use `await` are implicitly async and return `Promise`:

```luma
fn fetchAndProcess(id: String): Promise(Result(Data, Error)) do
  let raw = await fetchData(id)
  process(raw)
end
```

When called, this function returns `Promise(Result(Data, Error))` immediately, not the result itself.

## Sequential vs Concurrent Execution

### Sequential Execution

Operations wait for each other (slower):

```luma
let user = await fetchUser("/users/123")    -- waits
let posts = await fetchPosts(user.id)       -- then waits
-- Total time: time1 + time2
```

### Concurrent Execution

Start operations in parallel (faster):

```luma
let userPromise = fetchUser("/users/123")   -- starts now
let postsPromise = fetchPosts("123")        -- starts now (doesn't wait)
let user = await userPromise                -- wait for first
let posts = await postsPromise              -- wait for second
-- Total time: max(time1, time2)
```

### Parallel Execution with all()

```luma
let results = await all([
  fetchUser("/users/1"),
  fetchUser("/users/2"),
  fetchUser("/users/3")
])
-- Results waits for ALL promises to complete
```

### Race Conditions with race()

```luma
let first = await race([
  fetchFromServer1(data),
  fetchFromServer2(data),
  fetchFromServer3(data)
])
-- Returns the FIRST promise to complete
```

## Error Handling with Async

Combine `Result` types with async for robust error handling:

```luma
fn fetchUser(id: String): Promise(Result(User, Error)) do
  let response = await http.get("/users/${id}")
  
  if response.err != null do
    return { ok = null, err = response.err }
  end
  
  parseUser(response.ok)
end

-- Usage
let result = await fetchUser("123")
match result do
  ok do
    print("User: ${result.ok.name}")
  end
  err do
    print("Error: ${result.err}")
  end
end
```

## Async Loops

Loop through async operations:

```luma
let userIds = ["1", "2", "3"]
let users = []

for id in userIds do
  let user = await fetchUser(id)
  if user.err == null do
    users.push(user.ok)
  end
end
```

### Parallel Loops with Promise.all()

```luma
let userIds = ["1", "2", "3"]
let promises = []

for id in userIds do
  promises.push(fetchUser(id))
end

let results = await all(promises)
```

## Cancellation

Cancel a promise before it completes (planned):

```luma
let task = fetchData(url)

if userCancelled do
  task.cancel()
end
```

## Timeouts

Set a timeout for a promise:

```luma
let result = await timeout(fetchData(url), 5000)
-- Rejects if not completed within 5 seconds
```

## Example: Data Fetching

```luma
let fetchUserWithPosts = fn(userId: String) do
  -- Fetch user and posts in parallel
  let userPromise = http.get("/users/${userId}")
  let postsPromise = http.get("/posts?user=${userId}")
  
  -- Wait for both
  let user = await userPromise
  let posts = await postsPromise
  
  -- Check for errors
  if user.err != null do
    return { ok = null, err = user.err }
  end
  
  if posts.err != null do
    return { ok = null, err = posts.err }
  end
  
  -- Return combined result
  {
    ok = {
      user = parseUser(user.ok),
      posts = parsePosts(posts.ok)
    },
    err = null
  }
end

-- Usage
let result = await fetchUserWithPosts("user-123")
match result do
  ok do
    print("User: ${result.ok.user.name}")
    print("Posts: ${result.ok.posts.length()}")
  end
  err do
    print("Error: ${result.err}")
  end
end
```

## Example: Retry Logic

```luma
let retryAsync = fn(operation: fn(): Promise(Any), maxAttempts: Number) do
  var attempt = 0
  var lastError = null
  
  while attempt < maxAttempts do
    let result = await operation()
    
    if result.err == null do
      return result
    end
    
    lastError = result.err
    attempt = attempt + 1
  end
  
  { ok = null, err = "Max attempts reached: ${lastError}" }
end

-- Usage
let result = await retryAsync(fn() do
  fetchData(url)
end, 3)
```

## Async Best Practices

1. **Start parallel operations early:**
   ```luma
   let p1 = operation1()       -- Start now
   let p2 = operation2()       -- Start now
   let r1 = await p1           -- Wait for results
   let r2 = await p2
   ```

2. **Always handle errors:**
   ```luma
   let result = await operation()
   if result.err != null do
     -- Handle error
   end
   ```

3. **Use Result types with async:**
   ```luma
   fn asyncOp(): Promise(Result(T, E)) do
     -- Combine async with explicit error handling
   end
   ```

4. **Avoid callback hell — use await:**
   ```luma
   -- Instead of nested callbacks, use await
   let user = await fetchUser(id)
   let posts = await fetchPosts(user.id)
   let comments = await fetchComments(posts[0].id)
   ```

5. **Consider timeouts for slow operations:**
   ```luma
   let result = await timeout(slowOperation(), 30000)
   ```

## Runtime Considerations

:::info
The async runtime is currently under development. When available, Luma will use:
- **Event loop** for managing promises
- **Microtasks** for await continuations
- **Work stealing** for efficient concurrency
- **Cancellation tokens** for cleanup
:::

## Related Documentation

- [Error Handling](./error-handling.md) — Combine with Result types
- [Functions](../basics/functions.md) — Function syntax and types
- [Control Flow](../basics/control-flow.md) — Loops with await
let posts = await postsPromise
```
