# Tarbetu's Lox

## What is Lox
Lox is a programming language described and crafted in the book, [Crafting Interpreters](https://craftinginterpreters.com/). It's dynamically typed, object-oriented and has a C-like syntax. An example Lox program:

```
fun some_function(x) {
  if (x / 2 == 0) {
    return x * 5;
  } else if (x / 5 == 0) {
    return x * 10;
  } else {
    return x * 2;
  }
}

for (var i = 0; i <= 100; i = i + 1) {
  print some_function(i);
}
```

Some differences from JavaScript are:
- Lox has stronger typing than JS and has only `==` for equality checking
- Lox has a built-in `print` statement
- Lox is class-based while JavaScript is prototype-based.
- Lox don't have some interesting features like modules, bitwise operators, arrays, for-each loop, modules etc.

You can get further introduction here:
[The Lox Language](https://craftinginterpreters.com/the-lox-language.html)

## What is Tarbetu's Lox

"Tarbetu's Lox" or "Tarlox" is my toy implementation of Lox. It has some features which Lox don't have in its original implementation.

### No numeric overflows

Tarbetu's Lox has 256-bit precision floats for number type. You can go beyond as you can.

```
var x = 100000000000000000000000000000000000000000000000000000000000000000000 * 9999999999999999;
```

Note that the printing would format the number as the scientific notation if the number is too big.

```
print x; // 9.999999999999999000000000000000000000000000000000000000000000000000000000000000e83
```

### Values of variables are calculated parallelly

In Tarbetu's Lox, variable declarations are processed in parallel threads; it won't block the main thread until it's accessed.

```
var x = a_function_that_took_seconds();
print "The variable x is calculating, as the main thread goes on.";
var y = an_another_function_that_took_seconds();
print "The variable y is calculating like x. Now, two threads are running besides the main thread";
var z = an_another_function_with_parameters_took_seconds(y);
print "The main thread still isn't blocked, but the variable z will be calculated after variable z is ready.
var r = 666 * 333;
print "Variable r is calculated parallelly"
print x;
// Main thread is blocked until the variable x is ready. At this moment, y and z are still calculating in another thread.
```

To avoid this behaviour, you can declare your variables with `await_var`.
```
await_var x = a_function_that_took_seconds();  
// Thread is blocked until x is ready.
print x;
// No blocking, because x is calculated.
```

### Function Return Value Memoization

If a function is called with the same arguments, the function will quickly return the value which is cached before unless executing the main thread.

```
fun power_of_two(x) {
  print "Calculating power of " + x;
  return x * x;
}

print power_of_two(2); // Prints the text from the function body before returning a value;
print power_of_two(2);
// Returns the value, but does not execute it so the print statement from the function body doesn't work.
print power_of_two(4); // Prints the text from the function body before returning a value;
print power_of_two(4);
// Quickly returns the value so the print statement from the function body doesn't work.
```

Always the cache doesn't work and the function call will execute the body in this situation:
- If callable returns nil
- If callable don't take any arguments
- If callable is a method or native function (Implemented in the host language, Rust)

You can return functions in Lox, but the cached values will be cleared.

### Tail Call Optimization

Tail Call Optimization is an optimization technique which eliminates additional calls in recursion if the return statement only consists of function calls.

```
fun factorial(x) {
	fun factorial(acc, n) {
		if (n < 2) {
			return acc;
		}
		return factorial(acc * n, n - 1);
	}

	return factorial(1, x);
}

print factorial(666);
```

(This example also demonstrates that Lox allows inner functions)

### Lambda

You can declare lambdas like this:

```
var power_of_two = lambda(x) { return x * x };
print power_or_two(2);
```

Like other functions, memoization will work in the same scope.

## Issues and Caveats

### About Paralelism

Sometimes the program goes into a deadlock while playing around with variables. The behaviour is not consistent, so I can't reproduce and it's not common, and the interpreter may execute the same code without no issue.

### Direct call on Lambda

Currently, you can't do that:
```
lambda(x) { print x * 2; }(4);
```

It's odd, I know.

### Tail Call Optimization is only triggered when the return statement is a call statement.

This won't be counted as a tail call:

```
fun factorial(acc, n) {
	if (n < 2) {
		return acc;
	}
  var x = factorial(acc * n, n - 1);
	return x;
}
```

### Memoization causes memory leaks in the global scope

How will the cached values cleared? If you leave them in a scope, they will be cleared.

However, what happens when you implement them in a global scope like dozens of functions? Dozens of memory leaks.

Check this:

```
fun factorial(x) {
	fun factorial(acc, n) {
		if (n < 2) {
			return acc;
		}
		return factorial(acc * n, n - 1);
	}

	return factorial(1, x);
}

print factorial(10000000);
```

All results of function calls in recursion were stored. It is pure waste!

You can avoid wasting your memory like this:

```
class Factorial {
	init(x) {
		fun factorial(acc, n) {
			if (n < 2) {
				return acc;
			}
			return factorial(acc * n, n - 1);
		}

		this.result = factorial(1, x);
	}
}

print Factorial(42).result;
```

Since methods don't do memoization while nested functions do, there will be no memory leak after this point.

## Wishlist

To be honest, I don't have lots of free time to implement this but I would like to do them in my free time. If you implement this for me, you will get a place in my heart.

- Direct call on Lambda and environment capturing
- Changing `await_var x = 0;` as `await var x = 0;`.
- Importing mechanism
- A good standard library
- Arrays and Hashmap
- And iterator protocol and for-each syntax
- Implementing module and bitwise operators
- Better approach for storing variables to avoid deadlocks
- A decent GC for preventing memory leaks of memoization

## How to Run

First of all, you will need Rust tools to execute the interpreter. The best way to install them is to install [Rustup](https://rustup.rs/) on your machine.

To get the REPL:
```
cargo run
```

To run a file:

```
cargo run -- ~/Code/Lox/my_script.lox
```

To run an example:

```
cargo run -- example/tail_call.lox
```

You can install "Tarbetu's Lox" to your system like this:

```
cargo install --path.
```

The executable will be available in your `$PATH`, you can call the interpreter like this:

```
tarlox my_script.lox
```

To remove:

```
cargo unistall tarlox
```
