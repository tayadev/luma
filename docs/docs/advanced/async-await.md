---
sidebar_position: 1
---

# Async and Await

Luma has first-class support for asynchronous programming.

## Async Functions

Functions that contain `await` and return a `Promise` are automatically treated as async:

```luma
let fetchData = fn(url: String): Result(String, Error) do
  let res = await http.get(url)
  return res
end
```

When called, this function returns `Promise(Result(String, Error))`.

## The await Keyword

`await` suspends execution until a Promise resolves:

```luma
let fetchDog = fn(id: String): Result(Dog, Error) do
  let data = await http.get("https://dogs.api/" + id)
  return cast(Dog, data)
end

-- Calling without await returns a Promise
let promise = fetchDog("123")  -- Promise(Result(Dog, Error))

-- Using await resolves the Promise
let result = await fetchDog("123")  -- Result(Dog, Error)
```

## Promise Type

`await` transforms `Promise(T)` → `T`:

```luma
let promise: Promise(Number) = asyncOperation()
let value: Number = await promise
```

## Error Handling with Async

Combine async/await with Result types for robust error handling:

```luma
let processData = fn(url: String): Result(Data, Error) do
  let response = await http.get(url)
  
  match response do
    ok do
      return Result.ok(parseData(response.ok))
    end
    err do
      return Result.err(response.err)
    end
  end
end
```

## Parallel Execution

Execute multiple async operations in parallel:

```luma
let fetchUser = fn(id: String): Promise(User) do
  await http.get("/users/" + id)
end

let fetchPosts = fn(userId: String): Promise(Array(Post)) do
  await http.get("/posts?user=" + userId)
end

-- Wait for all promises
let [user, posts] = await Promise.all([
  fetchUser("123"),
  fetchPosts("123")
])
```

## Async Inference

Async behavior is inferred from:
1. Presence of `await` in function body
2. `Promise` return type annotation

```luma
-- Explicitly async
let getData = fn(): Promise(String) do
  await fetch("data.txt")
end

-- Implicitly async (inferred from await)
let getData = fn(): String do
  await fetch("data.txt")  -- return type becomes Promise(String)
end
```
