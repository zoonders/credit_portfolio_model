# Credit Portfolio Model
A rust implementaiton of a credit portfolio model. 

[![Crates.io](https://img.shields.io/crates/v/credit_portfolio_model)](https://crates.io/crates/credit_portfolio_model)
[![Docs.rs](https://docs.rs/credit_portfolio_model/badge.svg)](https://docs.rs/credit_portfolio_model)
[![Build Status](https://github.com/zoonders/credit_portfolio_model/workflows/CI/badge.svg)](https://github.com/zoonders/credit_portfolio_model/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Installation
Install the package via

```sh
cargo install credit_portfolio_model
```

## Usage
The crate comes with a binary that reads csv-data with the portfolio information and outputs the loss-distribution.
Besides, Mean and quantile information of the loss distribution is provided on stdout.

```sh
credit_portfolio_model --input /path/to/read/input/csv/files/from --output /path/to/store/output/csv --num-trials NUMBERTRIALS --chunk-size TRIALSPERTHREAD
```

The input consists of six files.

## Documentation

Full API documentation is available on [docs.rs](https://docs.rs/credit_portfolio_model).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

