# MOTD Templater

A SSH Login Banner Generator for Linux

## Installation

### Building from source

Download the latest version of Rust compiler to compile it, and copy the executable to any folder within PATH.

```bash
$ git clone https://github.com/higersky/motd-templater
$ cd motd-templater
$ cargo build --release
$ sudo cp target/release/motd-templater /usr/local/bin
```

## Usage

The program parse a template file and generate the motd message for you. You can call it in motd scripts, e.g. `/etc/update-motd.d/01-sysinfo`

```bash
$ motd-templater
Usage: motd-templater <template file path>
$ motd-templater sample/sysinfo.motd-template
> 24 cores Server ubuntu with Sky @ Thu Jul  6 20:00:00 CST 2023
  System Load   : 3.64  Physical Memory : 42.8%
  Disk Usage    : 56%   Swap Usage      : 2.7
  Sessions      : 3     CUDA Version    : 12.1.1
```

The template follows a braces-based format string syntax. You can find a sample in the `sample` folder. 

At the beginning of the file, you can assign custom variables as the output of any external shell command.

- This part is optional.
- Declare one assignment per line.
- Use `<name> := <command>` to assign custom variables. Commands will be executed as `sh -c <command>`. The program passes `<command>` as a single argument, so you don't need to add extra quotation marks.
- Use `env <name> := <builtin>` to assign environment variables with builtin variables. It will apply to all of the following commands.

```
@{
   nickname := echo Sky
   date := date 
   users := echo $((`users | wc -w` + 1)) 
   
   env root_usage = $root_disk_usage
   env memory_usage = $memory_usage
   env swap_usage = $swap_usage
   env cpu_cores = $cpu_cores
   env load = $load1
   warn := bash ./test/warn.sh
}
```

After that, you can write the contents. The program will replace template expressions with its actual values, and write all characters to the standard output.  

- Syntax: `{identifier :optional_modifier1 :optional_modifier2 ...}`

```
> {$cpu_cores} cores Server {$hostname:underline} with {nickname:bold:underline} @ {date}
  System Load	: {$load5}	Physical Memory	: {$memory_usage:percent:warn_color}
  Disk Usage	: {$root_disk_usage:percent:warn_color}	Swap Usage	: {$swap_usage}
  Sessions	: {users}	CUDA Version	: {$cuda_version}
{warn}
```

Use any variables inside braces defiend at the beginning. You can also use builtin variables starting with $ (See `src/handlers.rs` for more information). Modifiers provide string transformations such as colorizing or formatting.
