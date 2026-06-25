# Guppty Syntax

Keywords are configurable in `src/syntax.rs`. Examples below use the defaults.

## Hello World

```
out("Hello World")
```

## Variables (numbers, strings, booleans, chars, floats)

```
x = 5
bool = true
y = 'h'
z = "hello"
f = 1.25
```

## Math

```
out(2 + 3)
out(10 - 3)
out(6 * 5)
out(15 / 5)
```

## Comparisons and logic

```
out(5 == 5)
out(3 < 10)
out(true and false)
out(not false)
```

## If / else

```
score = 85
if score >= 80
    out("You passed!")
else
    out("Keep studying!")
```

## While loop

```
count = 3
while count > 0
    out(count)
    count = count - 1
```

## For loop

```
for i in range(1 through 6)
    out(i)
```

## Functions with parameters and return

```
add(a, b)
    return a + b

out(add(2, 3))
```

## Closures

```
makeAdder(x)
    addIt(y)
        return x + y
    return addIt

adder = makeAdder(5)
out(adder(3))
```

## Comments

```
// double slash starts a comment
out(42) // inline comments work too
```

## Function definition (indented block)

```
math()
    x = 5
    y = 6
    out(x + y)

main()
    math()
```
