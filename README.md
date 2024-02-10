# brainfsck

Yet another ~bad~ implementation of the [brainfuck](https://esolangs.org/wiki/Brainfuck) programming language \
Interpreter and compiler


## Building
`cargo build`

## Usage
| Command                                     | Action                                                  |
| :------------------------------------------ | :------------------------------------------------------ |
| `brainfsck interpret <file>`                | Interprets a brainfuck file                             |
| `brainfsck compile <file> <out_path>`       | Compiles a brainfuck file (requires `cc`, linux only)   |
