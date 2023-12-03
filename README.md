# rusty_calc

This is a Rust project that implements a basic calculator with support for variables. The calculator supports basic arithmetic operations (addition, subtraction, multiplication, and division) and parentheses. It also allows you to declare variables using the `let` keyword.

## Features

- Basic arithmetic operations: `+`, `-`, `*`, `/`
- Parentheses for grouping: `(`, `)`
- Variable declaration and usage with the `let` keyword


## Prerequisites

You need to have Rust and Cargo installed on your machine. You can install them using [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then follow the on-screen instructions to complete the installation.

## Running

After installing Rust and Cargo, you can run the project using the following command:

```bash
cargo run
```

## Usage

After running the project, you can start typing expressions into the console. Here are some examples:


- Basic arithmetic:
    ```
    > 5 + 3
    // Prints:
    =8
    ```
- Using parentheses: 
    ```
    > 5 + 3 * (2 + 1)
    // Prints:
    =14
    ```
- Declaring variables:
    ```
    > let x = 5
    // Prints:
    =5
    ```
- Using variables:
    ```
    > x + 3
    // Prints:
    =8
    ```
- Evaluating multiple expressions in one go:
    ```
    > let x = 5; let y = 3; x + y
    // Prints:
    =5
    =3
    =8
    ```

## Running Tests

You can run the test suite using the following command:

```bash
cargo test
```

## Contributing

Contributions are what make the open-source community such an amazing place to learn, inspire, and create. Any contributions you make are greatly appreciated.

1. Fork the Project
2. Create your Feature Branch ({code}git checkout -b feature/AmazingFeature{code})
3. Commit your Changes ({code}git commit -m 'Add some AmazingFeature'{code})
4. Push to the Branch ({code}git push origin feature/AmazingFeature{code})
5. Open a Pull Request

## License

Distributed under the MIT License. See `LICENSE` for more information.

## Contact

Alexandre Pedroza - pedrozaalexandre@gmail.com

Project Link: https://github.com/pedrozaalex/rusty_calc/