# Yadot Manual

## Command line

### --out \<out>

Write the built YAML file to the specified file path instead of `stdout`.

### --config \<config>

The file path to a YAML file to use as template's configuration file.

Example:

- `template.yaml`:

  ```yaml
  greetings: Hello, ${{ .name }}
  ```

- `config.yaml`:

  ```yaml
  name: World
  ```

- Run:

  ```bash
  yadot --config config.yaml template.yaml
  ```

- Output:

  ```yaml
  greetings: Hello, World
  ```

### --arg \<name> \<value>

Assigns a string value to a variable.

Example:

- `template.yaml`:

  ```yaml
  greetings: Hello, ${{ $name }}
  ```

- Run:

  ```bash
  yadot --arg name World template.yaml
  ```

- Output:

  ```yaml
  greetings: Hello, World
  ```

### --argyaml \<name> \<value>

Assigns a YAML string to a variable.

Note: Since all JSON is also valid YAML, the value may also be a JSON string.

Example:

- `template.yaml`:

  ```yaml
  greetings: Hello, ${{ $person.name }}
  ```

- Run:

  ```bash
  yadot --argyaml person '{"name":"World"}' template.yaml
  ```

- Output:

  ```yaml
  greetings: Hello, World
  ```

## Template expressions

Within the template YAML file, a template expression starts with `${{` and ends with
``}}`.

## Strings

You can write a string literal using double quotes. String literals use JSON's character
escaping rules.

Example:

- `template.yaml`:

  ```yaml
  greetings: Hello, ${{ "World" }}
  ```

- Run:

  ```bash
  yadot template.yaml
  ```

- Output:

  ```yaml
  greetings: Hello, World
  ```

## Query

A query expression is used to retrieve values from the config file or from variables set
on the command line.

To refer to the config file, start the query expression with a period: `.`.

To refer to a variable, use a `$` sign followed by the name. For example, `$person`.

To refer to fields within an object, add a period and the name of the field. For
example, `$person.name`.

When referring to a field in the root of the config file, the additional period can be
omitted. For example, `.name`.

If the field name has special characters in it, then you can use the array indexing
syntax to refer to the field. For example, `.["complex.name"]`.

The syntax of queries borrows heavily from [jq](https://jqlang.github.io/jq/)'s syntax.
Though, most of jq's features have not been implemented (yet).

Example:

- `template.yaml`:

  ```yaml
  animals:
  - name: ${{ $name }}
    eats: ${{ .food }}
    drinks: ${{ .["liquid"] }}
  ```

- `config.yaml`:

  ```yaml
  food: meat
  liquid: water
  ```

- Run:

  ```bash
  yadot --config config.yaml --arg name cat template.yaml
  ```

- Output:

  ```yaml
  animals:
  - name: cat
    eats: meat
    drinks: water
  ```

## If statements

If statements can be used to conditionally include content.

Example:

- `template.yaml`:

  ```yaml
  animals:
  - ${{ if .includeCat }}:
    - name: cat
      eats: meat
  ```

- `config.yaml`:

  ```yaml
  includeCat: true
  ```

- Run:

  ```bash
  yadot --config config.yaml template.yaml
  ```

- Output:

  ```yaml
  animals:
  - name: cat
    eats: meat
  ```

## Comparison operators

The following binary comparison operators are available:

- `==`
- `!=`

Example:

- `template.yaml`:

  ```yaml
  animals:
  - ${{ if .name == "cat" }}:
    - name: ${{ .name }}
      eats: ${{ .eats }}
  ```

- `config.yaml`:

  ```yaml
  name: cat
  eats: meat
  ```

- Run:

  ```bash
  yadot --config config.yaml template.yaml
  ```

- Output:

  ```yaml
  animals:
  - name: cat
    eats: meat
  ```

## For loops

For loops can be used to produce a multiple items from a list. For loops can also
iterate over the key-value pairs of a map.

Example:

- `template.yaml`:

  ```yaml
  animals:
  - ${{ for $name in .animals }}:
    - name: ${{ $name }}
  ```

- `config.yaml`:

  ```yaml
  animals:
  - cat
  - dog
  - elephant
  ```

- Run:

  ```bash
  yadot --config config.yaml template.yaml
  ```

- Output:

  ```yaml
  animals:
  - name: cat
  - name: dog
  - name: elephant
  ```

Example:

- `template.yaml`:

  ```yaml
  animals:
  - ${{ for $animal, $movement in . }}:
    - name: ${{ $animal }}
      movement: ${{ $movement }}
  ```

- `config.yaml`:

  ```yaml
  cat: pounce
  dog: run
  elephant: march
  ```

- Run:

  ```bash
  yadot --config config.yaml template.yaml
  ```

- Output:

  ```yaml
  animals:
  - name: cat
    movement: pounce
  - name: dog
    movement: run
  - name: elephant
    movement: march
  ```

## inline

The `inline` expression is used to take child content and move it inline with the parent
object.

An `inline` expression isn't particularly useful itself. It is mainly an implementation
detail of the `if` statement. In particular, when the if statement's conditional
expression is `true`, then the child contents are "inlined".

## drop

The `drop` expression is used to omit all child content.

This is mainly used as an implementation detail of the `if` statement. In particular,
when the if statement's conditional expression is `false`, then the child contents are
"dropped".

In addition, `drop` could potentially be used to "comment out" items in the template
file.
