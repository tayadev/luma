---
sidebar_position: 1
---

# Async and Await

Luma has first-class support for asynchronous programming with native async/await.

## Promises

Asynchronous operations return `Promise(T)`:

```luma
let fetchData = fn(url: String): Promise(String) do
  -- returns promise
end
```

## The await Keyword

`await` suspends execution until a promise resolves:

```luma
let fetchUser = fn(id: String): Result(User, Error) do
  let response = await http.get("/users/${id}")
  return parseUser(response)
end
```

**Type transformation:**
- `await Promise(T)` → `T`

## Async Function Inference

Functions are automatically async if they:
1. Contain `await` expressions
2. Return a `Promise` type

```luma
fn fetchAndProcess(id: String): Promise(Result(Data, Error)) do
  let raw = await fetchData(id)
  return process(raw)
end
```

When called, this function returns `Promise(Result(Data, Error))`.

## Sequential vs Concurrent Execution

### Sequential Execution

```luma
let data1 = await fetch("/api/data1")
let data2 = await fetch("/api/data2")
-- Total time: time1 + time2
```

### Concurrent Execution

```luma
let promise1 = fetch("/api/data1")
let promise2 = fetch("/api/data2")
let data1 = await promise1
let data2 = await promise2
-- Total time: max(time1, time2)
```

## Error Handling with Async

Combine async/await with Result types for robust error handling:

```luma
fn fetchUser(id: String): Promise(Result(User, Error)) do
  let response = await http.get("/users/${id}")
  if response.err != null do
    return { ok = null, err = response.err }
  end
  return parseUser(response.ok)
end
```

## Example: Parallel Execution

Execute multiple async operations concurrently:

```luma
let fetchUser = fn(id: String): Promise(User) do
  await http.get("/users/${id}")
end

let fetchPosts = fn(userId: String): Promise(List(Post)) do
  await http.get("/posts?user=${userId}")
end

-- Start both operations
let userPromise = fetchUser("123")
let postsPromise = fetchPosts("123")

-- Wait for both
let user = await userPromise
let posts = await postsPromise
```
