# Combustion R Library

A library to convert Halo PC maps to maps that are compatible with Halo CE.

Note: This is not my work.  This project was taken from [https://opencarnage.net/index.php?/topic/4680-combustion-203/](https://opencarnage.net/index.php?/topic/4680-combustion-203/).

This project is courtesy of [https://github.com/Halogen002](002), who was kind enough to not only post the source code, but also use the MIT license.  I am only hosting this here because this project is currently not on github, and I would like to be able to use it in a command line tool.

To use Combustion in your project, I recommend either cloning/forking it and specifying the path in your project's cargo.toml file, or if you want to use this repository directly, add something like this to your cargo.toml file:

```toml
[dependencies]
tritium = { git = "https://github.com/brainthinks/combustion", tag = "2.0.3" }
```

If you are looking to use Combustion as a command line tool on Linux, check out [https://github.com/brainthinks/combustion_cli](https://github.com/brainthinks/combustion_cli).
