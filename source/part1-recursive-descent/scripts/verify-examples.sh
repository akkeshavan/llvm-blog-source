#!/usr/bin/env bash
# Verify the three blog examples (Hello World, FizzBuzz, array-of-records) build and run.
# Run from source/part1-recursive-descent (or set LUM and run from project root).

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PART1="$(cd "$SCRIPT_DIR/.." && pwd)"
LUM="${LUM:-$PART1/09-lum-cli/target/release/lum}"

if [[ ! -x "$LUM" ]]; then
  echo "Building lum in $PART1/09-lum-cli..."
  (cd "$PART1/09-lum-cli" && cargo build --release)
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
cd "$TMP"

echo "=== 1. Hello World ==="
"$LUM" init .
echo 'println("Hello, World!");' > src/main.lum
out=$("$LUM" run 2>&1)
echo "$out" | grep -q "Hello, World!" || { echo "Expected 'Hello, World!' in output. Got:"; echo "$out"; exit 1; }
echo "OK"

echo "=== 2. FizzBuzz ==="
cat > src/main.lum << 'LUM'
fn fizzbuzz(n: i64) -> unit {
    if n % 15 == 0 then println("FizzBuzz")
    else if n % 3 == 0 then println("Fizz")
    else if n % 5 == 0 then println("Buzz")
    else println(n);
    return;
}

for i in 1..=20 {
    fizzbuzz(i);
}
LUM
out=$("$LUM" run 2>&1)
echo "$out" | grep -q "FizzBuzz" || { echo "Expected 'FizzBuzz' in output."; exit 1; }
echo "$out" | grep -q "Buzz" || { echo "Expected 'Buzz' in output."; exit 1; }
echo "OK"

echo "=== 3. Array of records (filter_adults) ==="
cat > src/main.lum << 'LUM'
type User = { name: str, age: i64, address: str };

fn filter_adults(users: Array<User>) -> Array<User> {
    let result: Array<User> = [];
    for i in 0..ArrayLen(users) {
        let u = get(users, i);
        if u.age > 18 then append(result, u) else unit;
    }
    return result;
}

let users: Array<User> = [
    { name: "Alice", age: 16, address: "1 Main St" },
    { name: "Bob", age: 22, address: "2 Oak Ave" },
    { name: "Carol", age: 14, address: "3 Elm Rd" },
    { name: "Dave", age: 25, address: "4 Pine Ln" }
];
let adults = filter_adults(users);
println(ArrayLen(adults));
LUM
out=$("$LUM" run 2>&1)
echo "$out" | grep -q "^2$" || { echo "Expected '2' as line of output. Got:"; echo "$out"; exit 1; }
echo "OK"

echo "All three examples verified."
