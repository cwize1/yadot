# Yadot: A YAML template tool

Yadot is a tool for building YAML documents using a template file. The template files
are themselves a valid YAML files.

Links:

- [Manual](./docs/Manual.md)

## Quick start guide

1. Create a template file called `mytemplate.yaml`:

   ```yaml
   greetings: Hello, ${{ .name }}
   ```

2. Create a config file called `myconfig.yaml`:

   ```yaml
   name: World
   ```

3. Run `yadot`:

   ```bash
   yadot --config myconfig.yaml mytemplate.yaml
   ```

   The output should look like this:

   ```yaml
   greetings: Hello, World
   ```

## Building Yadot

Yadot is a fairly bog standard Rust tool. To build it, all you need to run is
`cargo build`.
