<div align="center">

  # Spring boot CLI writen in Rust

  This is a command-line tool built in [Rust](https://www.rust-lang.org/) that allows developers to quickly create new Spring Boot projects in the terminal.
  this tool uses [Spring Initializr API](https://start.spring.io/) by default to generate a Spring Boot project.
</div>

> Dependencies <br>
>> reqwest,
>> inquire,
>> clap,
>> serde,
>> serde_json,
>> anyhow
>> resolve-path 



## Install

  1. ``` cargo install spring-boot-cli ```  
  2. ``` spring-boot-cli -h ```


## ⬇️ Locally project installation


```sh
git clone https://github.com/khopland/spring-cli-rust.git

cd spring-cli-rust

cargo build --release
```
