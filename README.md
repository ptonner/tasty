# `tasty`: terminal-accessible shadertoy

Run [shadertoys](https://www.shadertoy.com/) from your terminal, and edit them
with your tools of choice.

## Installation

From rust tooling:
```sh
  cargo install
```

or from `nix`: include this project as a flake.

## Usage

### Watch
To interactively develop a toy, run `tasty watch <path/to/to>`. This will open
a window displaying the compiled toy definition. Changing definition files
(`image.glsl`, `toy.toml`) will automatically recompile and redisplay the new
defintion.

## Features

- [Texture channels with built-in data](./examples/aa-texture-sample/)

## References
- [Integrating shadertoy shaders into a larger pipeline](https://webgl2fundamentals.org/webgl/lessons/webgl-shadertoy.html)
- [Nathan Vaughn's Shader Toy Tutorial](https://inspirnathan.com/posts/47-shadertoy-tutorial-part-1)
- [glslViewer](https://github.com/patriciogonzalezvivo/glslViewer): A much more feature complete terminal-based shader tool
