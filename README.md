strangl is a string-centered esoteric programming language developed for the PLTDI discord's Octopler 2026 event.

The only control flow constructs are `eval ...` and `eval ... if ...`, which fits the string theme.

# Features
The core feature of strangl is that all memory is a single global string, and all values are lazily evaluated spans into that string. This leads to behaviour like this:
```
x = "foo" // Appends "foo" to the memory, and stores it's span in `x`.
y = [-1, -3] // `y` refers to the last 3 characters of the memory, backwards.
print y // Prints "oof".
z = "bar"
print y // prints "rab".
```

strangl is highly imperative, and a program is structured as a series of *statements*. No seperators are required between statements, but newlines are conventional. Some statements contain *expressions*, and some statements and expressions contain *values*. 

Four kinds of statement exist:
- Assignment, of the form `ident = expr`
- Evaluation, of the form `eval value`. This may optionally be followed by a condition of the form `if expr`
- Deletion, of the form `delete value`. This deletes the contents of the span that the value evaluates to, shifting everything after it to the left. This is the only way to clean up memory in strangl, but will have significant effects on other variables in the program
- Printing, of the form `print value`

Four kinds of value exist: 
- Variable accesses, of the form `ident`. This evaluates to the span stored by that variable, or the empty span if it's undefined
- Globs, of the form `*`. This evalutes to the entire current span of the memory
- Literals strings, 
- Explicit spans, of the form 



Arithmetic operations are performed using double-precision floating point numbers, which are parsed immediately before the operation and converted back to strings immediately after. Strings containing only a decimal number with an optional sign and exponent are parsed as the number they contain. The decimal point is allowed to be leading or trailing (e.g. `.5` and `5.` instead of `0.5` and `5.0`). `infinity`, `inf`, and `nan` (case-insensitive) are parsed as infinity or NaN values. Any other string is parsed as `0.0`.

Logical operations parse only their left-hand operand as a boolean and return either their left- or right-hand operand as appropriate, without any boolean normalisation or other changes. The empty string and the string `"false"` are treated as being false. Any other strings are treated as being `true`.

There are no runtime errors. Reading an out-of-bounds span results in reading from a random span of the same length instead, and `eval`-ing code with a syntax error silently does nothing.