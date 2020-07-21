# sudoku_solver

A console app that aims to find the solution to a sudoku, using the brute for method, as quickly as possible.

## Methodology

This application utilizes a brute force search with [backtracking](https://en.wikipedia.org/wiki/Sudoku_solving_algorithms#Backtracking).

The only remotely clever bits can be found in
* [worker.rs](src/worker.rs), which recursively attempts to find a solution while occasionally donating work, and
* [beggar_pool.rs](src/beggar_pool.rs), which is a simple beggar/donator pool for sharing work among workers.

The application attempts to spawn a thread for each machine core, and it attempts to bind each thread to a separate core, if possible.

## Usage

```bash
./sudoku_solver sudoku_file
```

Where `sudoku_file` takes the form

```
X 1 X X 6 X 3 X X
6 X X X X 2 X X X
X 3 4 X X 8 6 X 1
X 6 8 4 X X X X X
3 4 2 X X X 8 X 6
X X X X X X 5 X X
8 X X X X X X X X
X 2 X X X 1 4 X 3
4 7 X X X 6 X 1 X
```

## Tests

...tomorrow?

## License

```
The MIT License (MIT)

Copyright (c) 2020 Aaron Roney

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```